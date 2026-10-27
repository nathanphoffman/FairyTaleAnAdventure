//! One hardware channel of the sound chip.

use super::tables::MAX_VOLUME;

/// One hardware channel: plays signed 8 bit data at a rate set by its period.
#[derive(Clone)]
pub(super) struct Channel {
    pub(super) data: Vec<i8>,
    pub(super) phase: f64,
    pub(super) step: f64,  // data bytes per output sample
    pub(super) volume: u8, // 0-MAX_VOLUME
    pub(super) active: bool,
    pub(super) looping: bool,
}

impl Channel {
    pub(super) fn new() -> Channel {
        Channel {
            data: Vec::new(),
            phase: 0.0,
            step: 0.0,
            volume: 0,
            active: false,
            looping: true,
        }
    }

    /// Next sample of the waveform, with the next one for interpolating, or None when silent.
    fn current_pair(&self) -> Option<(f32, f32)> {
        if !self.active || self.data.is_empty() {
            return None;
        }

        let len = self.data.len();
        let i = self.phase as usize;
        let a = self.data[i.min(len - 1)] as f32;
        let b = if i + 1 < len {
            self.data[i + 1] as f32
        } else if self.looping {
            self.data[0] as f32
        } else {
            a
        };

        Some((a, b))
    }

    /// Moves along the data, wrapping when looping and stopping at the end otherwise.
    fn advance(&mut self) {
        self.phase += self.step;
        let len = self.data.len() as f64;

        if self.phase >= len {
            if self.looping {
                self.phase -= len;
            } else {
                self.active = false;
            }
        }
    }

    /// Next output sample, -1.0 to 1.0.
    pub(super) fn next(&mut self) -> f32 {
        let Some((a, b)) = self.current_pair() else {
            return 0.0;
        };

        let frac = (self.phase - self.phase.floor()) as f32;
        let sample = (a + (b - a) * frac) / 128.0 * (self.volume as f32 / MAX_VOLUME as f32);
        self.advance();

        sample
    }
}
