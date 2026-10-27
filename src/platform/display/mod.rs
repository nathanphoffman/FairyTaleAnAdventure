//! The window. Converts the planar bitmaps the game draws into (view -> viewports -> pixels) an
//! RGB frame and shows it with SDL2. Also paces frames at 50 Hz and handles window events.
//!
//! The original screen is built from viewports: the play field (lores, 5 bitplanes, 32 colours)
//! on top and the text panel (hires, 4 bitplanes) below. Output is 640x200; lores pixels are
//! doubled horizontally.

mod compose;

mod events;

mod hooks;

mod scripting;

use crate::amiga::*;
use sdl2::EventPump;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// Counts calls to wait_frame/present; the test watchdog uses it to see if the game is stuck.
static BEAT: AtomicU32 = AtomicU32::new(0);

/// Set before init(): press Space for the player early on, which skips the intro.
static SKIP_INTRO: AtomicBool = AtomicBool::new(false);

pub fn set_skip_intro(skip: bool) {
    SKIP_INTRO.store(skip, Ordering::Relaxed);
}

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 200;
const TICK: Duration = Duration::from_millis(20); // 50 Hz
/// If the game falls this many ticks behind, pacing restarts instead of catching up.
const MAX_LAG_TICKS: u32 = 4;
const WINDOW_TITLE: &str = "The Faery Tale Adventure";
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 800;

/// 640x200 pixels are shown on a 4:3 screen, so each row is drawn two pixels tall.
const ROW_SCALE: u32 = 2;
const BYTES_PER_PIXEL: usize = 4;

/// Scripted Space press that skips the intro: (press tick, release tick).
const SKIP_INTRO_TICKS: (u32, u32) = (5, 9);

struct Display {
    canvas: Canvas<Window>,
    creator: TextureCreator<WindowContext>,
    events: EventPump,
    frame: Vec<u32>,
    next_tick: Instant,
    started: Instant,
    presents: u32,
    ticks: u32,
    /// scripted input for tests: (tick, key name, pressed)
    script: Vec<(u32, String, bool)>,
    /// scripted mouse clicks for tests: (tick, x in lores pixels, y)
    clicks: Vec<(u32, i32, i32)>,
    controller_system: sdl2::GameControllerSubsystem,
    /// open gamepads (they stop sending events when dropped)
    pads: Vec<sdl2::controller::GameController>,
    /// left stick state: left, right, up, down
    stick: [bool; 4],
}

thread_local! {
    static DISPLAY: RefCell<Option<Display>> = const { RefCell::new(None) };
    static VIEW: RefCell<*mut View> = const { RefCell::new(std::ptr::null_mut()) };

    /// mouse pointer sprite data (Amiga sprite format), set by ChangeSprite
    static SPRITE: RefCell<*const u32> = const { RefCell::new(std::ptr::null()) };
}

/// Opens every gamepad that is already plugged in.
fn open_pads(system: &sdl2::GameControllerSubsystem) -> Vec<sdl2::controller::GameController> {
    let mut pads = Vec::new();

    for i in 0..system.num_joysticks().unwrap_or(0) {
        if system.is_game_controller(i)
            && let Ok(c) = system.open(i)
        {
            pads.push(c);
        }
    }

    pads
}

/// The scripted key presses for tests, plus the Space press that skips the intro.
fn initial_script() -> Vec<(u32, String, bool)> {
    let mut script = scripting::parse_script();

    if SKIP_INTRO.load(Ordering::Relaxed) {
        script.push((SKIP_INTRO_TICKS.0, "Space".to_string(), true));
        script.push((SKIP_INTRO_TICKS.1, "Space".to_string(), false));
    }

    script
}

pub fn init() -> Result<(), String> {
    hooks::start_watchdog();
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas
        .set_logical_size(WIDTH, HEIGHT * ROW_SCALE)
        .map_err(|e| e.to_string())?;
    let creator = canvas.texture_creator();
    let events = sdl.event_pump()?;
    let controller_system = sdl.game_controller()?;
    let pads = open_pads(&controller_system);
    super::input::load_config();
    sdl.mouse().show_cursor(false); // the game draws its own pointer

    match sdl.audio() {
        Ok(audio) => super::audio::init(&audio),
        Err(e) => eprintln!("No audio: {e}"),
    }

    std::mem::forget(sdl);
    std::mem::forget(video);
    let display = Display {
        canvas,
        creator,
        events,
        frame: vec![0; (WIDTH * HEIGHT) as usize],
        next_tick: Instant::now() + TICK,
        started: Instant::now(),
        presents: 0,
        ticks: 0,
        clicks: scripting::parse_clicks(),
        controller_system,
        pads,
        stick: [false; 4],
        script: initial_script(),
    };

    DISPLAY.with(|d| *d.borrow_mut() = Some(display));

    Ok(())
}

pub fn set_sprite(data: *const u32) {
    SPRITE.with(|c| *c.borrow_mut() = data);
}

pub fn set_view(v: *mut View) {
    VIEW.with(|c| *c.borrow_mut() = v);
}

/// Converts the current view to RGB and shows it.
fn draw(d: &mut Display, view: *mut View) {
    compose::compose(view, &mut d.frame);
    let bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            d.frame.as_ptr() as *const u8,
            d.frame.len() * BYTES_PER_PIXEL,
        )
    };

    let texture = d
        .creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, WIDTH, HEIGHT);

    match texture {
        Ok(mut texture) => {
            if let Err(e) = texture.update(None, bytes, WIDTH as usize * BYTES_PER_PIXEL) {
                eprintln!("Could not update the screen texture: {e}");
            }

            d.canvas.clear();

            if let Err(e) = d.canvas.copy(&texture, None, None) {
                eprintln!("Could not draw the screen texture: {e}");
            }

            d.canvas.present();
        }

        Err(e) => eprintln!("Could not create the screen texture: {e}"),
    }
}

/// Shows the current view on screen (the game calls this when it flips pages).
pub fn present() {
    BEAT.fetch_add(1, Ordering::Relaxed);
    let view = VIEW.with(|c| *c.borrow());
    DISPLAY.with(|d| {
        let mut d = d.borrow_mut();

        let Some(d) = d.as_mut() else { return };
        draw(d, view);
        d.presents += 1;
        events::handle_events(d);
        hooks::test_hooks(d);
    });
}

/// Redraws every tick. Some screens (the item list, story pages) never flip pages, but the pointer
/// still moves and text is still printed into the bitmaps.
fn redraw() {
    let view = VIEW.with(|c| *c.borrow());
    DISPLAY.with(|d| {
        if let Some(d) = d.borrow_mut().as_mut() {
            draw(d, view);
        }
    });
}

/// Moves the tick deadline on by one tick and returns how long to sleep until the current one.
fn pace(d: &mut Display) -> Duration {
    let now = Instant::now();
    let wait = d.next_tick.saturating_duration_since(now);
    d.next_tick = if now > d.next_tick + TICK * MAX_LAG_TICKS {
        now + TICK
    } else {
        d.next_tick + TICK
    };

    wait
}

/// Waits for the next 50 Hz tick (the Amiga's vertical blank), keeping the window alive.
pub fn wait_frame() {
    BEAT.fetch_add(1, Ordering::Relaxed);
    let wait = DISPLAY.with(|d| {
        let mut d = d.borrow_mut();
        let d = d.as_mut()?;
        events::handle_events(d);
        hooks::test_hooks(d);
        d.ticks += 1;
        scripting::run_script(d);

        Some(pace(d))
    });

    if let Some(w) = wait {
        // FTA_FAST=1 runs flat out (regression tests); game logic only counts ticks, not time
        if std::env::var_os("FTA_FAST").is_none() {
            std::thread::sleep(w);
        }

        redraw();
    }
}

/// AmigaDOS `Delay`: n ticks of 1/50 second.
pub fn delay(ticks: LONG) {
    for _ in 0..ticks.max(1) {
        wait_frame();
    }
}
