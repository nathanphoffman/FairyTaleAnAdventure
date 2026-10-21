use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::time::Duration;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 200;
const SCALE: u32 = 3;

/// Amiga colours are 12 bit (4 bits per channel); expand to 8 bit.
fn amiga_rgb(r4: u32, g4: u32, b4: u32) -> u32 {
    (r4 * 17) << 16 | (g4 * 17) << 8 | (b4 * 17)
}

/// Fill the frame with a 32 colour test palette (the game uses 32 colours) over a gradient.
fn draw_test_pattern(pixels: &mut [u32]) {
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let c = if y < 140 {
                // 32 colour bars: 16 hues x 2 brightness levels
                let bar = x * 32 / WIDTH;
                let hue = bar % 16;
                let level = if bar < 16 { 15 } else { 8 };
                match hue % 6 {
                    0 => amiga_rgb(level, 0, 0),
                    1 => amiga_rgb(level, level / 2, 0),
                    2 => amiga_rgb(level, level, 0),
                    3 => amiga_rgb(0, level, 0),
                    4 => amiga_rgb(0, level, level),
                    _ => amiga_rgb(0, 0, level),
                }
            } else {
                // grey gradient strip, 16 steps
                let g = x * 16 / WIDTH;
                amiga_rgb(g, g, g)
            };
            pixels[(y * WIDTH + x) as usize] = c;
        }
    }
}

fn main() -> Result<(), String> {
    // Optional: `--frames N` exits after N frames, used to test without a person watching.
    let args: Vec<String> = std::env::args().collect();
    let max_frames: Option<u32> = args
        .iter()
        .position(|a| a == "--frames")
        .and_then(|i| args.get(i + 1))
        .and_then(|n| n.parse().ok());

    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("The Faery Tale Adventure", WIDTH * SCALE, HEIGHT * SCALE)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_logical_size(WIDTH, HEIGHT).map_err(|e| e.to_string())?;

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, WIDTH, HEIGHT)
        .map_err(|e| e.to_string())?;

    let mut pixels = vec![0u32; (WIDTH * HEIGHT) as usize];
    draw_test_pattern(&mut pixels);

    let mut events = sdl.event_pump()?;
    let mut frame = 0u32;
    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }

        texture
            .update(None, bytemuck_u32(&pixels), (WIDTH * 4) as usize)
            .map_err(|e| e.to_string())?;
        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();

        frame += 1;
        if max_frames.is_some_and(|m| frame >= m) {
            break;
        }
        std::thread::sleep(Duration::from_millis(20)); // ~50 Hz like the Amiga
    }
    Ok(())
}

fn bytemuck_u32(pixels: &[u32]) -> &[u8] {
    // SAFETY: u32 has no padding and any byte pattern is valid for u8.
    unsafe { std::slice::from_raw_parts(pixels.as_ptr() as *const u8, std::mem::size_of_val(pixels)) }
}
