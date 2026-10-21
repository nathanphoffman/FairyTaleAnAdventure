//! The window. Converts the planar bitmaps the game draws into (view -> viewports -> pixels) an
//! RGB frame and shows it with SDL2. Also paces frames at 50 Hz and handles window events.
//!
//! The original screen is built from viewports: the play field (lores, 5 bitplanes, 32 colours)
//! on top and the text panel (hires, 4 bitplanes) below. Output is 640x200; lores pixels are
//! doubled horizontally.

use crate::amiga::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;
use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// Counts calls to wait_frame/present; the test watchdog uses it to see if the game is stuck.
static BEAT: AtomicU32 = AtomicU32::new(0);

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 200;
const TICK: Duration = Duration::from_millis(20); // 50 Hz

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
}

thread_local! {
    static DISPLAY: RefCell<Option<Display>> = const { RefCell::new(None) };
    static VIEW: RefCell<*mut View> = const { RefCell::new(std::ptr::null_mut()) };
    /// mouse pointer sprite data (Amiga sprite format), set by ChangeSprite
    static SPRITE: RefCell<*const u32> = const { RefCell::new(std::ptr::null()) };
}

pub fn init() -> Result<(), String> {
    start_watchdog();
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("The Faery Tale Adventure", 1280, 800)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    // 640x200 pixels are shown on a 4:3 screen, so each row is drawn two pixels tall
    canvas.set_logical_size(WIDTH, HEIGHT * 2).map_err(|e| e.to_string())?;
    let creator = canvas.texture_creator();
    let events = sdl.event_pump()?;
    sdl.mouse().show_cursor(false); // the game draws its own pointer
    std::mem::forget(sdl);
    std::mem::forget(video);
    DISPLAY.with(|d| {
        *d.borrow_mut() = Some(Display {
            canvas,
            creator,
            events,
            frame: vec![0; (WIDTH * HEIGHT) as usize],
            next_tick: Instant::now() + TICK,
            started: Instant::now(),
            presents: 0,
            ticks: 0,
            script: parse_script(),
        })
    });
    Ok(())
}

pub fn set_sprite(data: *const u32) {
    SPRITE.with(|c| *c.borrow_mut() = data);
}

pub fn set_view(v: *mut View) {
    VIEW.with(|c| *c.borrow_mut() = v);
}

/// 12 bit Amiga colour (0x0RGB) to 0xRRGGBB.
fn rgb4(c: u16) -> u32 {
    let (r, g, b) = ((c >> 8 & 15) as u32, (c >> 4 & 15) as u32, (c & 15) as u32);
    (r * 17) << 16 | (g * 17) << 8 | b * 17
}

const VP_HIDE: u16 = 0x2000;
const SPRITES: u16 = 0x4000;

/// Draws the mouse pointer: a 16 pixel wide sprite, 2 frame pixels per sprite pixel. Each line is
/// two 16 bit planes (the high word of each 32 bit value is plane 0); colour = 16 + value.
fn draw_pointer(frame: &mut [u32], colors: &ColorMap) {
    let data = SPRITE.with(|c| *c.borrow());
    if data.is_null() {
        return;
    }
    let (px, py) = super::input::pointer();
    let (ox, oy) = (px * 2, py);
    let mut line = 0;
    loop {
        let word = unsafe { *data.add(1 + line) };
        if word == 0 || line >= 32 {
            break;
        }
        let (a, b) = ((word >> 16) as u16, word as u16);
        for bit in 0..16 {
            let v = (a >> (15 - bit) & 1) | (b >> (15 - bit) & 1) << 1;
            let idx = 16 + v as usize;
            if v == 0 || idx >= colors.Count as usize {
                continue;
            }
            let color = rgb4(unsafe { *colors.ColorTable.add(idx) });
            for dx in 0..2 {
                let (x, y) = (ox + bit * 2 + dx, oy + line as i32);
                if x >= 0 && x < WIDTH as i32 && y >= 0 && y < HEIGHT as i32 {
                    frame[y as usize * WIDTH as usize + x as usize] = color;
                }
            }
        }
        line += 1;
    }
}

/// Draws all viewports of the view into the 640x200 frame.
fn compose(view: *const View, frame: &mut [u32]) {
    frame.fill(0);
    if view.is_null() {
        return;
    }
    let mut pointer_colors: *const ColorMap = std::ptr::null();
    let mut vp = unsafe { (*view).ViewPort };
    while !vp.is_null() {
        let v = unsafe { &*vp };
        vp = v.Next;
        if v.RasInfo.is_null() || v.ColorMap.is_null() || v.Modes & VP_HIDE != 0 {
            continue;
        }
        if v.Modes & SPRITES != 0 {
            pointer_colors = v.ColorMap;
        }
        let ri = unsafe { &*v.RasInfo };
        if ri.BitMap.is_null() {
            continue;
        }
        let bm = unsafe { &*ri.BitMap };
        let cm = unsafe { &*v.ColorMap };
        let hires = v.Modes & HIRES != 0;
        let depth = bm.Depth as usize;
        for row in 0..v.DHeight as i32 {
            let sy = ri.RyOffset as i32 + row;
            let oy = v.DyOffset as i32 + row;
            if sy < 0 || sy >= bm.Rows as i32 || oy < 0 || oy >= HEIGHT as i32 {
                continue;
            }
            for col in 0..v.DWidth as i32 {
                let sx = ri.RxOffset as i32 + col;
                if sx < 0 || sx >= bm.BytesPerRow as i32 * 8 {
                    continue;
                }
                let mut index = 0usize;
                for p in 0..depth {
                    let plane = bm.Planes[p];
                    if plane.is_null() {
                        continue;
                    }
                    let byte = unsafe { *plane.add(sy as usize * bm.BytesPerRow as usize + (sx >> 3) as usize) };
                    if byte & (0x80 >> (sx & 7)) != 0 {
                        index |= 1 << p;
                    }
                }
                let color = if index < cm.Count as usize {
                    rgb4(unsafe { *cm.ColorTable.add(index) })
                } else {
                    0
                };
                let ox = if hires { v.DxOffset as i32 + col } else { (v.DxOffset as i32 + col) * 2 };
                let base = oy as usize * WIDTH as usize;
                if ox >= 0 && ox < WIDTH as i32 {
                    frame[base + ox as usize] = color;
                }
                if !hires && ox + 1 < WIDTH as i32 {
                    frame[base + ox as usize + 1] = color;
                }
            }
        }
    }
    if !pointer_colors.is_null() {
        draw_pointer(frame, unsafe { &*pointer_colors });
    }
}

/// SDL already reports mouse positions in the 640x400 logical screen (it does that when a logical
/// size is set, and it handles the letterboxing), so only the doubled rows need undoing.
fn to_frame(_d: &Display, lx: i32, ly: i32) -> (i32, i32) {
    (lx, ly / 2)
}

fn handle_events(d: &mut Display) {
    let events: Vec<Event> = d.events.poll_iter().collect();
    for event in events {
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                std::process::exit(0)
            }
            Event::KeyDown { keycode: Some(k), repeat: false, .. } => super::input::key_event(k, true),
            Event::KeyUp { keycode: Some(k), .. } => super::input::key_event(k, false),
            Event::MouseMotion { x, y, .. } => {
                let (fx, fy) = to_frame(d, x, y);
                super::input::mouse_move(fx, fy);
            }
            Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                let (fx, fy) = to_frame(d, x, y);
                super::input::mouse_move(fx, fy);
                super::input::mouse_button(mouse_btn, true);
            }
            Event::MouseButtonUp { mouse_btn, .. } => super::input::mouse_button(mouse_btn, false),
            _ => {}
        }
    }
}

/// FTA_WATCHDOG=n aborts if the game goes n seconds without waiting for a frame (a stuck loop),
/// so a debugger shows where.
fn start_watchdog() {
    let Some(limit) = std::env::var("FTA_WATCHDOG").ok().and_then(|s| s.parse::<u64>().ok()) else {
        return;
    };
    std::thread::spawn(move || {
        let mut last = BEAT.load(Ordering::Relaxed);
        let mut still = 0;
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let now = BEAT.load(Ordering::Relaxed);
            still = if now == last { still + 1 } else { 0 };
            last = now;
            if still >= limit {
                eprintln!("WATCHDOG: game stuck for {limit}s");
                std::process::abort();
            }
        }
    });
}

/// FTA_INJECT="100:Space,200:Right:60" presses keys at those 1/50 s ticks, held for 4 ticks or
/// the given number.
fn parse_script() -> Vec<(u32, String, bool)> {
    let mut script = Vec::new();
    for item in std::env::var("FTA_INJECT").unwrap_or_default().split(',') {
        let mut parts = item.split(':');
        let (Some(t), Some(name)) = (parts.next(), parts.next()) else { continue };
        let hold: u32 = parts.next().and_then(|h| h.parse().ok()).unwrap_or(4);
        if let Ok(t) = t.parse::<u32>() {
            script.push((t, name.to_string(), true));
            script.push((t + hold, name.to_string(), false));
        }
    }
    script
}

fn run_script(d: &mut Display) {
    let tick = d.ticks;
    for (t, name, down) in d.script.clone() {
        if t == tick {
            if let Some(k) = sdl2::keyboard::Keycode::from_name(&name) {
                super::input::key_event(k, down);
            }
        }
    }
}

/// Test helpers: FTA_SHOT=file.ppm saves the screen after FTA_SHOT_AFTER ticks of 1/50 s (default 1)
/// and quits; FTA_SECONDS=n quits after n seconds.
fn test_hooks(d: &mut Display) {
    if let Some(path) = std::env::var_os("FTA_SHOT") {
        let after: u32 = std::env::var("FTA_SHOT_AFTER").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
        if d.ticks >= after {
            let mut out = format!("P6\n{WIDTH} {HEIGHT}\n255\n").into_bytes();
            for px in &d.frame {
                out.extend_from_slice(&[(px >> 16) as u8, (px >> 8) as u8, *px as u8]);
            }
            let _ = std::fs::write(path, out);
            std::process::exit(0);
        }
    }
    if let Some(secs) = std::env::var("FTA_SECONDS").ok().and_then(|s| s.parse::<u64>().ok()) {
        if d.started.elapsed().as_secs() >= secs {
            std::process::exit(0);
        }
    }
}

/// Shows the current view on screen.
pub fn present() {
    BEAT.fetch_add(1, Ordering::Relaxed);
    let view = VIEW.with(|c| *c.borrow());
    DISPLAY.with(|d| {
        let mut d = d.borrow_mut();
        let Some(d) = d.as_mut() else { return };
        compose(view, &mut d.frame);
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(d.frame.as_ptr() as *const u8, d.frame.len() * 4)
        };
        if let Ok(mut texture) =
            d.creator.create_texture_streaming(PixelFormatEnum::ARGB8888, WIDTH, HEIGHT)
        {
            let _ = texture.update(None, bytes, WIDTH as usize * 4);
            d.canvas.clear();
            let _ = d.canvas.copy(&texture, None, None);
            d.canvas.present();
        }
        d.presents += 1;
        handle_events(d);
        test_hooks(d);
    });
}

/// Waits for the next 50 Hz tick (the Amiga's vertical blank), keeping the window alive.
pub fn wait_frame() {
    BEAT.fetch_add(1, Ordering::Relaxed);
    let wait = DISPLAY.with(|d| {
        let mut d = d.borrow_mut();
        let Some(d) = d.as_mut() else { return None };
        handle_events(d);
        test_hooks(d);
        d.ticks += 1;
        run_script(d);
        let now = Instant::now();
        let wait = d.next_tick.saturating_duration_since(now);
        d.next_tick = if now > d.next_tick + TICK * 4 { now + TICK } else { d.next_tick + TICK };
        Some(wait)
    });
    if let Some(w) = wait {
        std::thread::sleep(w);
    }
}

/// AmigaDOS `Delay`: n ticks of 1/50 second.
#[unsafe(no_mangle)]
pub extern "C" fn Delay(ticks: LONG) {
    for _ in 0..ticks.max(1) {
        wait_frame();
    }
}
