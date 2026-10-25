//! Rust versions of the game code in csrc/, ported one file at a time with the same behaviour.
//! Each function keeps its C name so the C that is still left links against it unchanged.

pub mod iff_helpers;
pub mod iffsubs;
pub mod actors;
pub mod asm_blit;
pub mod asm_map;
pub mod asm_stubs;
pub mod asm_text;
pub mod hdrive;
pub mod input;
pub mod loading;
pub mod messages;
pub mod objects;
pub mod random;
pub mod savegame;
pub mod screen_fx;
pub mod types;
pub mod witch;
