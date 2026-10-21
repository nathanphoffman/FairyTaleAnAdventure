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
use std::time::{Duration, Instant};

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
}

thread_local! {
    static DISPLAY: RefCell<Option<Display>> = const { RefCell::new(None) };
    static VIEW: RefCell<*mut View> = const { RefCell::new(std::ptr::null_mut()) };
}

pub fn init() -> Result<(), String> {
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
        })
    });
    Ok(())
}

pub fn set_view(v: *mut View) {
    VIEW.with(|c| *c.borrow_mut() = v);
}

/// 12 bit Amiga colour (0x0RGB) to 0xRRGGBB.
fn rgb4(c: u16) -> u32 {
    let (r, g, b) = ((c >> 8 & 15) as u32, (c >> 4 & 15) as u32, (c & 15) as u32);
    (r * 17) << 16 | (g * 17) << 8 | b * 17
}

/// Draws all viewports of the view into the 640x200 frame.
fn compose(view: *const View, frame: &mut [u32]) {
    frame.fill(0);
    if view.is_null() {
        return;
    }
    let mut vp = unsafe { (*view).ViewPort };
    while !vp.is_null() {
        let v = unsafe { &*vp };
        vp = v.Next;
        if v.RasInfo.is_null() || v.ColorMap.is_null() {
            continue;
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
}

fn handle_events(d: &mut Display) {
    for event in d.events.poll_iter() {
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                std::process::exit(0)
            }
            _ => {}
        }
    }
}

/// Test helpers: FTA_SHOT=file.ppm saves the screen after FTA_SHOT_AFTER presents (default 1)
/// and quits; FTA_SECONDS=n quits after n seconds.
fn test_hooks(d: &mut Display) {
    if let Some(path) = std::env::var_os("FTA_SHOT") {
        let after: u32 = std::env::var("FTA_SHOT_AFTER").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
        if d.presents >= after {
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
    let wait = DISPLAY.with(|d| {
        let mut d = d.borrow_mut();
        let Some(d) = d.as_mut() else { return None };
        handle_events(d);
        test_hooks(d);
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
