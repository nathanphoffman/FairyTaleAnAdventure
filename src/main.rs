mod gfx;

mod platform;

mod game;

fn main() -> Result<(), String> {
    // The intro (story book) is skipped unless you ask for it: `cargo run -- --intro`
    platform::display::set_skip_intro(!std::env::args().any(|a| a == "--intro"));
    platform::display::init()?;

    game::startup::run_game();

    Ok(())
}
