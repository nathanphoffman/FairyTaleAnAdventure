//! Rust versions of the game code in csrc/, ported one file at a time with the same behaviour.
//! Each function keeps its C name so the C that is still left links against it unchanged.

pub mod iff_helpers;
pub mod iffsubs;
pub mod hdrive;
pub mod input;
pub mod random;
