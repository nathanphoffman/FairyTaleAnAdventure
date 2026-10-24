//! Music and sound effects, replacing gdriver.asm and the Amiga's four-channel sound chip.
//!
//! The game plays music from four tracks of (command, value) events, one per voice, and one-shot
//! sound effects on voice 2. The original ran the sequencer from the vertical blank interrupt and
//! let the chip play the waveforms by DMA. Here an SDL audio callback does both: it advances the
//! sequencer 60 times a second and mixes the four channels (voices 1 and 4 left, 2 and 3 right,
//! as on the Amiga).
//!
//! Track format (see docs/data-formats.md): command byte < 128 is a note number and the value's
//! low 6 bits index `NOTE_LENGTHS`; 128 = rest, 129 = set instrument, 0x90 = set tempo, 255 = end
//! of track (value non-zero = loop).

mod calls;
mod channel;
mod dump;
mod sequencer;
mod tables;

pub use dump::dump;

use sdl2::AudioSubsystem;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sequencer::Sound;
use std::sync::{Arc, Mutex, OnceLock};
use tables::{BUFFER_SAMPLES, OUTPUT_CHANNELS, SAMPLE_RATE};

struct Callback(Arc<Mutex<Sound>>);

impl AudioCallback for Callback {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        match self.0.lock() {
            Ok(mut s) => s.mix(out),
            Err(_) => out.fill(0.0),
        }
        if dump::enabled() {
            dump::capture(out);
        }
    }
}

static SOUND: OnceLock<Arc<Mutex<Sound>>> = OnceLock::new();

/// Opens the audio device. Without one the game runs silent.
pub fn init(audio: &AudioSubsystem) {
    let desired = AudioSpecDesired {
        freq: Some(SAMPLE_RATE),
        channels: Some(OUTPUT_CHANNELS),
        samples: Some(BUFFER_SAMPLES),
    };
    let mut shared = None;
    let device: Result<AudioDevice<Callback>, String> =
        audio.open_playback(None, &desired, |spec| {
            let sound = Arc::new(Mutex::new(Sound::new(spec.freq as f64)));
            shared = Some(sound.clone());
            Callback(sound)
        });
    match device {
        Ok(d) => {
            d.resume();
            std::mem::forget(d); // keep playing for the life of the program
            if let Some(s) = shared {
                let _ = SOUND.set(s);
            }
        }
        Err(e) => eprintln!("No audio: {e}"),
    }
}

/// Runs `f` on the sound state; does nothing when there is no audio device.
fn with_sound(f: impl FnOnce(&mut Sound)) {
    if let Some(s) = SOUND.get()
        && let Ok(mut s) = s.lock()
    {
        f(&mut s);
    }
}
