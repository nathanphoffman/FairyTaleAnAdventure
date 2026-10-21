mod amiga;
mod platform;

use std::os::raw::{c_char, c_int};

unsafe extern "C" {
    /// `main` of the original game (fmain.c), renamed by build.rs.
    fn fta_main(argc: c_int, argv: *mut c_char);
}

fn main() -> Result<(), String> {
    amiga::check_layout();
    platform::display::init()?;
    // argc == 0 would mean "started from Workbench" in the original.
    unsafe { fta_main(1, std::ptr::null_mut()) };
    Ok(())
}
