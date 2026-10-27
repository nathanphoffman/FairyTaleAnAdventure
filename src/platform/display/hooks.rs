//! Test helpers driven by environment variables.

use super::{BEAT, Display, HEIGHT, WIDTH};
use std::sync::atomic::Ordering;
use std::time::Duration;

const WATCHDOG_CHECK: Duration = Duration::from_secs(1);
/// Used when FTA_STATE_AFTER / FTA_SHOT_AFTER are not set.
const DEFAULT_AFTER_TICKS: u32 = 1;

/// FTA_WATCHDOG=n aborts if the game goes n seconds without waiting for a frame (a stuck loop),
/// so a debugger shows where.
pub(super) fn start_watchdog() {
    let Some(limit) = std::env::var("FTA_WATCHDOG")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
    else {
        return;
    };
    std::thread::spawn(move || {
        let mut last = BEAT.load(Ordering::Relaxed);
        let mut still = 0;
        loop {
            std::thread::sleep(WATCHDOG_CHECK);
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

/// Reads a tick count from an environment variable.
fn ticks_from_env(name: &str) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_AFTER_TICKS)
}

/// FTA_STATE=file writes the game state as text at tick FTA_STATE_AFTER (then quits).
fn dump_state_hook(d: &Display) {
    let Some(path) = std::env::var_os("FTA_STATE") else {
        return;
    };
    if d.ticks < ticks_from_env("FTA_STATE_AFTER") {
        return;
    }
    crate::rsrc::statedump::dump_state(std::path::Path::new(&path));
    std::process::exit(0);
}

/// The frame as a binary PPM image.
fn frame_as_ppm(frame: &[u32]) -> Vec<u8> {
    let mut out = format!("P6\n{WIDTH} {HEIGHT}\n255\n").into_bytes();
    for px in frame {
        out.extend_from_slice(&[(px >> 16) as u8, (px >> 8) as u8, *px as u8]);
    }
    out
}

/// FTA_SHOT=file.ppm saves the screen after FTA_SHOT_AFTER ticks and quits.
fn screenshot_hook(d: &Display) {
    let Some(path) = std::env::var_os("FTA_SHOT") else {
        return;
    };
    if d.ticks < ticks_from_env("FTA_SHOT_AFTER") {
        return;
    }
    if let Err(e) = std::fs::write(&path, frame_as_ppm(&d.frame)) {
        eprintln!("Could not save the screenshot: {e}");
    }
    crate::platform::audio::dump();
    std::process::exit(0);
}

/// Test helpers: FTA_SHOT=file.ppm saves the screen after FTA_SHOT_AFTER ticks of 1/50 s (default 1)
/// and quits; FTA_SECONDS=n quits after n seconds; FTA_FAST=1 skips the 50 Hz pacing.
pub(super) fn test_hooks(d: &mut Display) {
    dump_state_hook(d);
    screenshot_hook(d);
    if let Some(secs) = std::env::var("FTA_SECONDS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        && d.started.elapsed().as_secs() >= secs
    {
        crate::platform::audio::dump();
        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ppm_has_header_and_three_bytes_per_pixel() {
        let ppm = frame_as_ppm(&[0xFF8040, 0x000001]);
        let header = format!("P6\n{WIDTH} {HEIGHT}\n255\n").len();
        assert_eq!(ppm.len(), header + 6);
        assert_eq!(&ppm[header..], &[0xFF, 0x80, 0x40, 0, 0, 1]);
    }
}
