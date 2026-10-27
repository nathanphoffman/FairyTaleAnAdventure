//! Choosing the background music (from csrc/fmain.c).
//!
//! The music depends on what the hero is doing: dying, in the astral plane, fighting, indoors,
//! or just walking by day or night. `track` holds the four voices of each tune in order.

use super::game_tables::{instrument_waves, menus};
use super::globals::{battle_flag, track};
use super::state::{the_actors, the_hero, the_world};

unsafe extern "C" {
    fn playscore(t1: *const u8, t2: *const u8, t3: *const u8, t4: *const u8);
    fn setscore(t1: *const u8, t2: *const u8, t3: *const u8, t4: *const u8);
    fn stopscore();
}

/// Each tune uses four tracks.
const TRACKS_PER_TUNE: usize = 4;
// Tune numbers, times four to index `track`.
const TUNE_DAY: usize = 0;
const TUNE_BATTLE: usize = 1;
const TUNE_NIGHT: usize = 2;
const TUNE_ASTRAL: usize = 4;
const TUNE_INDOORS: usize = 5;
const TUNE_DEATH: usize = 6;
/// The astral plane (a rectangle of the world).
const ASTRAL_X: (u16, u16) = (0x2400, 0x3100);
const ASTRAL_Y: (u16, u16) = (0x8200, 0x8a00);
/// Indoors, the instrument changes: caves (region 9) use another wave from the dungeons.
const INDOOR_REGION: u16 = 7;
const CAVE_REGION: u16 = 9;
const WAVE_SLOT: usize = 10;
const WAVE_CAVE: i16 = 0x0307;
const WAVE_INDOORS: i16 = 0x0100;
/// It is night when the light level is at or below this.
const NIGHT_LIGHT: u16 = 120;
/// The game menu entry that switches music on and off.
const GAME_MENU: usize = 4;
const MUSIC_ENTRY: usize = 6;

/// Picks the tune for the situation and plays it right away if `now` is set, otherwise changes
/// to it at the end of the current one. If music is switched off, stops the music.
pub fn set_mood(now: i8) {
    unsafe {
        let tune = if the_actors.list[0].vitality == 0 {
            TUNE_DEATH
        } else if the_hero.x > ASTRAL_X.0
            && the_hero.x < ASTRAL_X.1
            && the_hero.y > ASTRAL_Y.0
            && the_hero.y < ASTRAL_Y.1
        {
            TUNE_ASTRAL
        } else if battle_flag != 0 {
            TUNE_BATTLE
        } else if the_world.region > INDOOR_REGION {
            let wave = (&raw mut instrument_waves).cast::<i16>().add(WAVE_SLOT);
            *wave = if the_world.region == CAVE_REGION {
                WAVE_CAVE
            } else {
                WAVE_INDOORS
            };
            TUNE_INDOORS
        } else if the_world.light_level > NIGHT_LIGHT {
            TUNE_DAY
        } else {
            TUNE_NIGHT
        };
        let at = tune * TRACKS_PER_TUNE;

        if menus[GAME_MENU].enabled[MUSIC_ENTRY] & 1 != 0 {
            if now != 0 {
                playscore(track[at], track[at + 1], track[at + 2], track[at + 3]);
            } else {
                setscore(track[at], track[at + 1], track[at + 2], track[at + 3]);
            }
        } else {
            stopscore();
        }
    }
}
