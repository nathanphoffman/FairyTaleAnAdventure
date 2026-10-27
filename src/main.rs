mod amiga;
mod platform;
// The game state lives in lower-case `static mut` variables (named for what they hold) until it
// moves into structs.
// `&mut *(&raw mut the_x)` is how a `static mut` is borrowed without the `static_mut_refs` lint, so
// clippy's `deref_addrof` advice (use the static directly) would not compile.
#[allow(non_upper_case_globals, clippy::deref_addrof)]
#[path = "../rsrc/mod.rs"]
mod rsrc;

use std::os::raw::{c_char, c_int};

unsafe extern "C" {
    /// `main` of the original game (fmain.c), renamed by build.rs.
    fn fta_main(argc: c_int, argv: *mut c_char);
}

fn main() -> Result<(), String> {
    amiga::check_layout();
    // The intro (story book) is skipped unless you ask for it: `cargo run -- --intro`
    platform::display::set_skip_intro(!std::env::args().any(|a| a == "--intro"));
    platform::display::init()?;
    // argc == 0 would mean "started from Workbench" in the original.
    unsafe { fta_main(1, std::ptr::null_mut()) };
    Ok(())
}
