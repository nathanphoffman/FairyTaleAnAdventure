//! The game itself: Rust versions of the code in csrc/, ported one file at a time with the same
//! behaviour. The folders group the code by what it does:
//!
//! - `data`: the structs, the game state (`the_hero`, `the_world`, ...), the constant tables and the
//!   random numbers.
//! - `files`: the game disk, IFF pictures, sprite files, songs and saved games.
//! - `world`: the map, regions, places, doors, objects and the day clock.
//! - `actors`: enemies, how actors move and fight, encounters and reviving the hero.
//! - `ui`: the text panel, the menus and what choosing an entry does, and the music.
//! - `input`: keyboard and mouse input and the hero's controls.
//! - `render`: drawing the sprites and the page, colour effects and the witch's beam.
//! - `startup`: opening and closing everything, the title and intro, and the main loop.
//! - `statedump`: test helper that writes the game state as text.
//!
//! Functions the C still calls keep their C names. The old file names (`asm_blit`, `hdrive`,
//! `iffsubs`, ...) are listed in modern_engine_plan.md.

pub mod actors;
pub mod data;
pub mod files;
pub mod input;
pub mod render;
pub mod startup;
pub mod statedump;
pub mod ui;
pub mod world;
