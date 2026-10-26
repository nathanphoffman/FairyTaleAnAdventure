//! Rust versions of the game code in csrc/, ported one file at a time with the same behaviour.
//! Each function keeps its C name so the C that is still left links against it unchanged.
//!
//! How the code is organised (roughly from the bottom of the stack to the top):
//!
//! - Shared data: `types` (the game's structs), `tables` (constant tables and the object lists).
//! - Files and pictures: `hdrive` (reads the game disk image), `iffsubs` and `iff_helpers` (load
//!   IFF pictures), `loading` (sprite files, songs, the story page turn).
//! - Input and numbers: `input` (keyboard and mouse), `random` (the game's random numbers).
//! - Drawing: `asm_blit` (sprite blitter in software), `asm_map` (map, collision and scrolling),
//!   `asm_text` (text and enemy steering), `screen_fx` (fades, compass), `witch` (the witch's beam).
//! - Game rules: `actors` (hits, tactics, rescue), `objects` (items and people in the world),
//!   `messages` (text panel), `savegame` (saving and loading).
//!
//! The `asm_` names are historical: those routines were 68000 assembly in the Amiga original.

// Chunk and line helpers for IFF pictures.
pub mod iff_helpers;
// Loads an IFF picture into a bitmap.
pub mod iffsubs;
// Opening doors with keys.
pub mod doors;
// Placing monsters and carriers, and deaths.
pub mod encounters;
// Choosing the music.
pub mod mood;
// Moving the hero and working out where it is.
pub mod places;
// Bringing the hero or the next brother back.
pub mod revive;
// Showing the finished page, the input device.
pub mod display;
// The text-panel menus.
pub mod menus;
// What choosing a menu entry does: talk, buy, use, keys, save, give.
pub mod options;
// The Items menu: the item list, taking things, looking.
pub mod items_menu;
// The Magic menu: the spells.
pub mod magic;
// Key presses and the hero's controls.
pub mod keys;
// Per-frame world bookkeeping: doors, beds, the clock, spawning, hunger, draw order.
pub mod world;
// Melee swings and missiles in flight.
pub mod combat;
// What each enemy decides to do.
pub mod ai;
// Moving and animating each actor every frame.
pub mod move_actor;
// Drawing the actors and the witch's beam into the page.
pub mod draw;
// Opening and closing everything, the title and intro, and the main loop.
pub mod startup;
// Hits, collision between actors, enemy tactics.
pub mod actors;
// The sprite blitter, done in software.
pub mod asm_blit;
// Map lookups, collision, camera, scrolling and the overview map.
pub mod asm_map;
// Amiga-only symbols the game still names.
pub mod asm_stubs;
// Number and text printing, story border, enemy steering.
pub mod asm_text;
// Reading blocks from the game disk image.
pub mod hdrive;
// Keyboard and mouse input, menu clicks, the compass.
pub mod input;
// Sprite files, songs, the story page turn.
pub mod loading;
// Printing and word-wrapping messages in the text panel.
pub mod messages;
// Turning the object lists into actors, taking items.
pub mod objects;
// Which region is loaded and loading the next one.
pub mod regions;
// The game's random number generator.
pub mod random;
// Saving and loading the game.
pub mod savegame;
// Colour fades, the compass, the print queue, the win sequence.
pub mod screen_fx;
// Constant tables and per-region object lists.
pub mod tables;
// Rust mirrors of the game's C structs.
pub mod types;
// The witch's spinning beam.
pub mod witch;
