mod amiga;

mod gfx;

mod platform;

// The game's tables and state are lower-case `static mut` variables named for what they hold.
// `&mut *(&raw mut the_x)` is how a `static mut` is borrowed without the `static_mut_refs` lint, so
// clippy's `deref_addrof` advice (use the static directly) would not compile.
#[allow(non_upper_case_globals, clippy::deref_addrof)]
mod game;

fn main() -> Result<(), String> {
    // The intro (story book) is skipped unless you ask for it: `cargo run -- --intro`
    platform::display::set_skip_intro(!std::env::args().any(|a| a == "--intro"));
    platform::display::init()?;

    game::startup::run_game();

    Ok(())
}
