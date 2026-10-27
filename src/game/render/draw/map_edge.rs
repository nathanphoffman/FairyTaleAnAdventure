//! Finishing the scrolled map edge each frame, and the witch's beam and her damage.

use crate::game::actors::interaction::{actor_distance, hit_actor};
use crate::game::data::random::{rand2, rand4};
use crate::game::data::state::{the_actors, the_screen, the_witch, the_world};
use crate::game::render::witch::{SIDE_1 as s1, SIDE_2 as s2, witch_fx};
use crate::game::world::map::{draw_map_column, draw_map_row};

// ---- repair_map_and_witch ----
/// The map strip to redraw after a scroll: the right edge (36) or the left (0), and the bottom row
/// (10) or the top (0).
const STRIP_RIGHT: i32 = 36;
const STRIP_LEFT: i32 = 0;
const ROW_BOTTOM: i32 = 10;
const ROW_TOP: i32 = 0;

/// The witch's beam hurts the hero within this distance.
const WITCH_REACH: i32 = 100;

/// The witch's position is kept relative to the scrolled map: its x snaps to the 16 pixel tile grid
/// and its y (15 pixels above her feet) to the 32 pixel one.
const TILE_X_MASK: u16 = 0xfff0;
const TILE_Y_MASK: u16 = 0xffe0;
const WITCH_ANCHOR_Y: i32 = 15;
const WITCH_SLOT: usize = 2;

/// Finishes the scrolled map edge (`dif_x`, `dif_y` = scroll step) and runs the witch's beam and her
/// damage.
pub fn repair_map_and_witch(dif_x: i16, dif_y: i16) {
    unsafe {
        if dif_x == 1 {
            draw_map_column(STRIP_RIGHT);
        } else if dif_x == -1 {
            draw_map_column(STRIP_LEFT);
        }

        if dif_y == 1 {
            draw_map_row(ROW_BOTTOM);
        } else if dif_y == -1 {
            draw_map_row(ROW_TOP);
        }

        let witch = &raw const the_actors.list[WITCH_SLOT];
        let page = the_screen.drawing_page;
        (*page).witchx = ((*witch).abs_x as i32 - (the_world.map_x & TILE_X_MASK) as i32) as i16;
        (*page).witchy = (((*witch).abs_y as i32 - WITCH_ANCHOR_Y)
            - (the_world.map_y & TILE_Y_MASK) as i32) as i16;

        if rand4() == 0 {
            the_witch.direction = if s1 > 0 { 1 } else { -1 };
        }

        the_witch.angle = the_witch.angle.wrapping_add(the_witch.direction as u8);
        (*page).witchdir = the_witch.angle as i16;
        (*page).wflag = the_witch.on_screen as i16;

        if the_witch.on_screen != 0 {
            witch_fx(page);

            if s1 > 0 && s2 < 0 && actor_distance(WITCH_SLOT as i32, 0) < WITCH_REACH {
                hit_actor(-1, 0, (*witch).facing as i32, (rand2() + 1) as i16);
            }
        }
    }
}
