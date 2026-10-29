//! The print queue: requests for the text panel (stats, vitality, menu, debug information).

use crate::game::data::state::Actors;
use crate::game::data::state::Game;
use crate::game::data::state::Hero;
use crate::game::data::state::World;
use crate::game::output::Output;
use crate::game::render::display::delay;
use crate::game::ui::menus::print_options;
use crate::game::ui::text::move_to;

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
pub fn handle_print_request(game: &mut Game, out: &mut dyn Output) {
    if game.ui.print_queue.write == game.ui.print_queue.read {
        delay(game, out, 1);

        return;
    }

    let request = game.ui.print_queue.items[game.ui.print_queue.read as usize] as i32;

    game.ui.print_queue.read = (game.ui.print_queue.read + 1) & QUEUE_MASK;

    match request {
        REQUEST_DEBUG_COORDS => show_debug_coords(&mut game.hero, out),
        REQUEST_DEBUG_SECTOR => show_debug_sector(&mut game.world, &mut game.hero, out),
        REQUEST_VITALITY => draw_vitality(&mut game.actors, out),
        REQUEST_OPTIONS => print_options(game, out),
        REQUEST_STATS => draw_stats(&mut game.hero, out),
        REQUEST_TAKE_WHAT => out.print("Take What?"),
        _ => {}
    }
}

/// Debug information: the hero's map position and the memory left.
fn show_debug_coords(hero: &mut Hero, out: &mut dyn Output) {
    out.print("Coords = ");
    out.print_number(hero.x as i32, COORD_DIGITS);
    out.print_number(hero.y as i32, COORD_DIGITS);
    out.print("Memory Available: ");
    out.print_number(FAKE_FREE_BYTES, COORD_DIGITS);
}

/// Debug information: the hero's sector and the type of the extent the hero is in.
fn show_debug_sector(world: &mut World, hero: &mut Hero, out: &mut dyn Output) {
    out.print("You are at: ");
    out.print_number(hero.x as i32 / SECTOR_PIXELS, SECTOR_DIGITS);
    out.print_number(hero.y as i32 / SECTOR_PIXELS, SECTOR_DIGITS);
    out.print("H/Sector = ");
    out.print_number(hero.sector as i32, SECTOR_DIGITS);
    out.print_cont_bytes(b" Extent = ");
    out.print_number(world.extent_type as i32, EXTENT_DIGITS);
}

/// The hero's vitality on the stats line.
fn draw_vitality(actors: &mut Actors, out: &mut dyn Output) {
    move_to(out, VITALITY_POSITION.0, VITALITY_POSITION.1);
    out.print_cont_bytes(b"Vit:");
    out.print_number(actors.list[0].vitality as i32, VITALITY_DIGITS);
}

/// Draws the hero's four stats (bravery, luck, kindness, wealth) along the stats line.
fn draw_stats(hero: &mut Hero, out: &mut dyn Output) {
    if hero.luck < 0 {
        hero.luck = 0;
    }

    for (x, label, value) in [
        (BRAVERY_X, b"Brv:".as_slice(), hero.brave),
        (LUCK_X, b"Lck:", hero.luck),
        (KINDNESS_X, b"Knd:", hero.kindness),
        (WEALTH_X, b"Wlth:", hero.wealth),
    ] {
        move_to(out, x, STATS_Y);
        out.print_cont_bytes(label);
        out.print_number(value as i32, STAT_DIGITS);
    }
}
