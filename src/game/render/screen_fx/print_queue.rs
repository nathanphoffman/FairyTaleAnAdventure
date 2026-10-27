//! The print queue: requests for the text panel (stats, vitality, menu, debug information).

use crate::game::data::object_tables::{pplay, prec, print_que};
use crate::game::data::state::{the_actors, the_hero, the_world};
use crate::game::ui::menus::print_options;
use crate::game::ui::messages::print;
use crate::game::ui::text::{draw_text, move_to, print_number};
use crate::platform::display::delay;
use crate::platform::exec::avail_mem;

// Field widths for printed numbers, and the map-coordinate shift (256 pixels to a sector).
const COORD_DIGITS: i32 = 6;
const SECTOR_DIGITS: i32 = 3;
const EXTENT_DIGITS: i32 = 2;
const VITALITY_DIGITS: i32 = 3;
const STAT_DIGITS: i32 = 3;
const SECTOR_PIXELS: i32 = 256;
const EXTENT_LABEL_LEN: i32 = 10;
const VITALITY_LABEL_LEN: i32 = 4;

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
pub fn handle_print_request() {
    unsafe {
        if prec == pplay {
            delay(1);

            return;
        }

        let request = (*(&raw const print_que))[pplay as usize] as i32;

        // Masks with QUEUE_MASK, keeping only the bits that matter here.
        pplay = (pplay + 1) & QUEUE_MASK;

        match request {
            REQUEST_DEBUG_COORDS => show_debug_coords(),
            REQUEST_DEBUG_SECTOR => show_debug_sector(),
            REQUEST_VITALITY => draw_vitality(),
            REQUEST_OPTIONS => print_options(),
            REQUEST_STATS => draw_stats(),
            REQUEST_TAKE_WHAT => print(c"Take What?".as_ptr()),
            _ => {}
        }
    }
}

/// Debug information: the hero's map position and the memory left.
unsafe fn show_debug_coords() {
    unsafe {
        print(c"Coords = ".as_ptr());
        print_number(the_hero.x as i32, COORD_DIGITS);
        print_number(the_hero.y as i32, COORD_DIGITS);
        print(c"Memory Available: ".as_ptr());
        print_number(avail_mem(0) as i32, COORD_DIGITS);
    }
}

/// Debug information: the hero's sector and the type of the extent the hero is in.
unsafe fn show_debug_sector() {
    unsafe {
        print(c"You are at: ".as_ptr());
        print_number(the_hero.x as i32 / SECTOR_PIXELS, SECTOR_DIGITS);
        print_number(the_hero.y as i32 / SECTOR_PIXELS, SECTOR_DIGITS);
        print(c"H/Sector = ".as_ptr());
        print_number(the_hero.sector as i32, SECTOR_DIGITS);
        draw_text(c" Extent = ".as_ptr(), EXTENT_LABEL_LEN);
        print_number(the_world.extent_type as i32, EXTENT_DIGITS);
    }
}

/// The hero's vitality on the stats line.
unsafe fn draw_vitality() {
    unsafe {
        move_to(VITALITY_POSITION.0, VITALITY_POSITION.1);
        draw_text(c"Vit:".as_ptr(), VITALITY_LABEL_LEN);
        print_number(the_actors.list[0].vitality as i32, VITALITY_DIGITS);
    }
}

/// Draws the hero's four stats (bravery, luck, kindness, wealth) along the stats line.
unsafe fn draw_stats() {
    unsafe {
        if the_hero.luck < 0 {
            the_hero.luck = 0;
        }

        for (x, label, len, value) in [
            (BRAVERY_X, c"Brv:", 4, the_hero.brave),
            (LUCK_X, c"Lck:", 4, the_hero.luck),
            (KINDNESS_X, c"Knd:", 4, the_hero.kindness),
            (WEALTH_X, c"Wlth:", 5, the_hero.wealth),
        ] {
            move_to(x, STATS_Y);
            draw_text(label.as_ptr(), len);
            print_number(value as i32, STAT_DIGITS);
        }
    }
}
