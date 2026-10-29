//! Saving and loading the game (from csrc/fmain2.c). The save file is written field by field in
//! big-endian order, so it does not depend on how the C compiler laid out the variables (see
//! `transfer`). Games are named `A.faery`, `B.faery` ... (one letter per menu choice) and the
//! platform layer puts the files in the `saves/` folder.

mod transfer;

use self::transfer::{Transfer, transfer_all};
use crate::game::data::state::Game;
use crate::game::files::loading::load_start_sprites;
use crate::game::output::Output;
use crate::game::render::screen::FontKind;
use crate::game::ui::menus::update_menu_availability;
use crate::game::ui::text::queue_print;
use crate::platform::paths::resolve;

/// The first letter of a save file name; the menu choice picks the letter.
const FIRST_SLOT_LETTER: u8 = b'A';

/// Value of `view_status` meaning the screen must be redrawn from scratch.
const VIEW_CORRUPT: i8 = 99;

/// Print queue requests that redraw the vitality line and the stats line.
const PRINT_VITALITY: i32 = 4;
const PRINT_STATS: i32 = 7;

/// Saves the game into slot `slot` (0 = A ...) when `saving`, or loads it from there.
pub fn savegame(game: &mut Game, out: &mut dyn Output, slot: i16, saving: bool) {
    set_current_font(out, FontKind::Panel);

    let name = format!(
        "{}.faery",
        (FIRST_SLOT_LETTER.wrapping_add(slot as u8)) as char
    );

    if saving {
        if write_game(game, &name).is_err() {
            out.print("ERROR: Couldn't save game.");
        }
    } else {
        if read_game(game, &name).is_err() {
            out.print("ERROR: Couldn't load game.");
        }

        reset_after_load(game, out);
    }

    set_current_font(out, FontKind::Story);
}

/// Writes the game to the save file `name`.
fn write_game(game: &mut Game, name: &str) -> std::io::Result<()> {
    let mut transfer = Transfer::saving();
    transfer_all(game, &mut transfer);

    let path = resolve(name, true).ok_or(std::io::ErrorKind::NotFound)?;

    std::fs::write(path, transfer.finish())
}

/// Reads the game from the save file `name`.
fn read_game(game: &mut Game, name: &str) -> std::io::Result<()> {
    let path = resolve(name, false).ok_or(std::io::ErrorKind::NotFound)?;
    let mut transfer = Transfer::loading(std::fs::read(path)?);
    transfer_all(game, &mut transfer);

    Ok(())
}

/// Writes in the given font from now on.
fn set_current_font(out: &mut dyn Output, kind: FontKind) {
    match kind {
        FontKind::Story => out.use_story_font(),
        FontKind::Panel => out.use_panel_font(),
    }
}

/// After loading a game: clears the encounter state and redraws the status panel for the new hero.
fn reset_after_load(game: &mut Game, out: &mut dyn Output) {
    game.encounter.weapon_type = 0;
    game.encounter.number = 0;
    load_start_sprites(game, out);
    update_menu_availability(game);
    game.ui.view_status = VIEW_CORRUPT;
    queue_print(&mut game.ui, PRINT_VITALITY);
    queue_print(&mut game.ui, PRINT_STATS);
    out.print("");
    out.print("");
    out.print("");
    game.encounter.kind = 0;
    game.actors.loading = 0;
}
