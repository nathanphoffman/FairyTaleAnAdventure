//! Choosing the background music (from csrc/fmain.c).
//!
//! The music depends on what the hero is doing: dying, in the astral plane, fighting, indoors,
//! or just walking by day or night. `track` holds the four voices of each tune in order.

use crate::game::data::state::Actors;
use crate::game::data::state::Game;
use crate::game::data::state::Hero;
use crate::game::data::state::World;
use crate::game::files::loading::tune_tracks;
use crate::platform::audio::{Track, playscore, set_instrument, setscore, stopscore};

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
const WAVE_CAVE: u16 = 0x0307;
const WAVE_INDOORS: u16 = 0x0100;

/// It is night when the light level is at or below this.
const NIGHT_LIGHT: u16 = 120;

/// The game menu entry that switches music on and off.
const GAME_MENU: usize = 4;
const MUSIC_ENTRY: usize = 6;

/// The tunes in order of their number, with the intro (3) where the game plays it at the start.
const TUNE_NAMES: [&str; 7] = [
    "day", "battle", "night", "intro", "astral", "indoors", "death",
];

/// Writes every tune as a WAV file into `dir` (the indoors tune twice: with the house and the cave
/// instrument). A tool for listening to the music; the game then ends.
pub fn export_tunes(dir: &std::path::Path) {
    let mut tunes: Vec<(&str, [Track; TRACKS_PER_TUNE])> = Vec::new();

    for (n, name) in TUNE_NAMES.iter().enumerate() {
        tunes.push((name, tune_tracks(n)));
    }

    tunes.push(("cave", tune_tracks(TUNE_INDOORS)));
    crate::platform::audio::export_tunes(dir, &tunes, |name| match name {
        "cave" => set_instrument(WAVE_SLOT, WAVE_CAVE),
        "indoors" => set_instrument(WAVE_SLOT, WAVE_INDOORS),
        _ => {}
    });
}

/// Picks the tune for the situation and plays it right away if `now` is set, otherwise changes
/// to it at the end of the current one. If music is switched off, stops the music.
pub fn set_mood(game: &mut Game, now: i8) {
    let tune = pick_tune(&game.actors, &game.world, &game.hero);

    if { game.menus[GAME_MENU].enabled[MUSIC_ENTRY] } & 1 != 0 {
        if now != 0 {
            playscore(tune_tracks(tune));
        } else {
            setscore(tune_tracks(tune));
        }
    } else {
        stopscore();
    }
}

/// The tune for what the hero is doing: dying, in the astral plane, fighting, indoors (which also
/// picks the instrument for houses or caves), or walking by day or by night.
fn pick_tune(actors: &Actors, world: &World, hero: &Hero) -> usize {
    if actors.list[0].vitality == 0 {
        TUNE_DEATH
    } else if hero.x > ASTRAL_X.0
        && hero.x < ASTRAL_X.1
        && hero.y > ASTRAL_Y.0
        && hero.y < ASTRAL_Y.1
    {
        TUNE_ASTRAL
    } else if actors.battle_flag != 0 {
        TUNE_BATTLE
    } else if world.region > INDOOR_REGION {
        set_instrument(
            WAVE_SLOT,
            if world.region == CAVE_REGION {
                WAVE_CAVE
            } else {
                WAVE_INDOORS
            },
        );

        TUNE_INDOORS
    } else if world.light_level > NIGHT_LIGHT {
        TUNE_DAY
    } else {
        TUNE_NIGHT
    }
}
