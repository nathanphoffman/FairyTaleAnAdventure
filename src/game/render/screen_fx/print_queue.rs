//! The print queue: requests for the text panel (stats, vitality, menu, debug information).

use crate::game::data::state::Game;
use crate::game::render::display::delay;
use crate::game::ui::menus::print_options;
use crate::game::ui::messages::print;
use crate::game::ui::text::{draw_text, move_to, print_number};

// Field widths for printed numbers, and the map-coordinate shift (256 pixels to a sector).
/// There is no memory shortage to report; the debug display shows this.
const FAKE_FREE_BYTES: i32 = 1_000_000;
const COORD_DIGITS: i32 = 6;
const SECTOR_DIGITS: i32 = 3;
const EXTENT_DIGITS: i32 = 2;
const VITALITY_DIGITS: i32 = 3;
const STAT_DIGITS: i32 = 3;
const SECTOR_PIXELS: i32 = 256;

// The print queue requests.
const REQUEST_DEBUG_COORDS: i32 = 2;
const REQUEST_DEBUG_SECTOR: i32 = 3;
const REQUEST_VITALITY: i32 = 4;
const REQUEST_OPTIONS: i32 = 5;
const REQUEST_STATS: i32 = 7;
const REQUEST_TAKE_WHAT: i32 = 10;
const QUEUE_MASK: i16 = 31;
const VITALITY_POSITION: (i32, i32) = (245, 52);
const STATS_Y: i32 = 52;
const BRAVERY_X: i32 = 14;
const LUCK_X: i32 = 90;
const KINDNESS_X: i32 = 168;
const WEALTH_X: i32 = 321;

/// Handles one request from the print queue (or waits a tick if there is none).
/// Requests: 2 and 3 debug information, 4 the vitality figure, 5 the menu, 7 the stats line
/// (bravery, luck, kindness, wealth), 10 "Take What?".
pub fn handle_print_request(game: &mut Game) {
    if game.ui.print_queue.write == game.ui.print_queue.read {
        delay(game, 1);

        return;
    }

    let request = game.ui.print_queue.items[game.ui.print_queue.read as usize] as i32;

    // Masks with QUEUE_MASK, keeping only the bits that matter here.
    game.ui.print_queue.read = (game.ui.print_queue.read + 1) & QUEUE_MASK;

    match request {
        REQUEST_DEBUG_COORDS => show_debug_coords(game),
        REQUEST_DEBUG_SECTOR => show_debug_sector(game),
        REQUEST_VITALITY => draw_vitality(game),
        REQUEST_OPTIONS => print_options(game),
        REQUEST_STATS => draw_stats(game),
        REQUEST_TAKE_WHAT => print(game, "Take What?"),
        _ => {}
    }
}

/// Debug information: the hero's map position and the memory left.
fn show_debug_coords(game: &mut Game) {
    print(game, "Coords = ");
    print_number(game, game.hero.x as i32, COORD_DIGITS);
    print_number(game, game.hero.y as i32, COORD_DIGITS);
    print(game, "Memory Available: ");
    print_number(game, FAKE_FREE_BYTES, COORD_DIGITS);
}

/// Debug information: the hero's sector and the type of the extent the hero is in.
fn show_debug_sector(game: &mut Game) {
    print(game, "You are at: ");
    print_number(game, game.hero.x as i32 / SECTOR_PIXELS, SECTOR_DIGITS);
    print_number(game, game.hero.y as i32 / SECTOR_PIXELS, SECTOR_DIGITS);
    print(game, "H/Sector = ");
    print_number(game, game.hero.sector as i32, SECTOR_DIGITS);
    draw_text(game, b" Extent = ");
    print_number(game, game.world.extent_type as i32, EXTENT_DIGITS);
}

/// The hero's vitality on the stats line.
fn draw_vitality(game: &mut Game) {
    move_to(game, VITALITY_POSITION.0, VITALITY_POSITION.1);
    draw_text(game, b"Vit:");
    print_number(game, game.actors.list[0].vitality as i32, VITALITY_DIGITS);
}

/// Draws the hero's four stats (bravery, luck, kindness, wealth) along the stats line.
fn draw_stats(game: &mut Game) {
    if game.hero.luck < 0 {
        game.hero.luck = 0;
    }

    for (x, label, value) in [
        (BRAVERY_X, b"Brv:".as_slice(), game.hero.brave),
        (LUCK_X, b"Lck:", game.hero.luck),
        (KINDNESS_X, b"Knd:", game.hero.kindness),
        (WEALTH_X, b"Wlth:", game.hero.wealth),
    ] {
        move_to(game, x, STATS_Y);
        draw_text(game, label);
        print_number(game, value as i32, STAT_DIGITS);
    }
}
