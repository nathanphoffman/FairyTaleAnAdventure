//! The few Amiga-only symbols the game still names (from csrc/asm_stubs.c).
//!
//! - `trapper`: the 68000 trap handler hook; nothing to do on Linux
//! - `HandlerInterface`: the Amiga input handler; `input` replaces it

pub fn trapper() -> i32 {
    0
}
