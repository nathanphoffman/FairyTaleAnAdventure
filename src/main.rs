mod gfx;

mod platform;

mod game;

fn main() -> Result<(), String> {
    // The intro and the launch menu play unless you skip them: `cargo run -- --skip-launch`
    platform::host::install(&game::host::GameHost);
    platform::display::set_skip_launch(std::env::args().any(|a| a == "--skip-launch"));
    platform::display::init()?;

    game::startup::run_game();

    Ok(())
}
