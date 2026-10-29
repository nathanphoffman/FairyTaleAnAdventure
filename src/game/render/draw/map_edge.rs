//! Finishing the scrolled map edge each frame, and the witch's beam and her damage.

use crate::game::actors::interaction::{actor_distance, hit_actor};
use crate::game::data::random::{rand2, rand4};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::data::state::Video;
use crate::game::render::map_tiles::{draw_map_column, draw_map_row};
use crate::game::render::witch::witch_fx;

// ---- repair_edges ----
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
pub fn repair_edges(game: &mut Game, video: &mut Video, dif_x: i16, dif_y: i16) {
    if dif_x == 1 {
        draw_map_column(game, video, STRIP_RIGHT);
    } else if dif_x == -1 {
        draw_map_column(game, video, STRIP_LEFT);
    }

    if dif_y == 1 {
        draw_map_row(game, video, ROW_BOTTOM);
    } else if dif_y == -1 {
        draw_map_row(game, video, ROW_TOP);
    }

    let witch = ActorRef::new(WITCH_SLOT);
    let page = video.screen.drawing_page_mut();

    page.witchx = (game.actors.list[witch.slot()].abs_x as i32
        - (game.world.map_x & TILE_X_MASK) as i32) as i16;
    page.witchy = ((game.actors.list[witch.slot()].abs_y as i32 - WITCH_ANCHOR_Y)
        - (game.world.map_y & TILE_Y_MASK) as i32) as i16;

    if rand4(&mut game.rng) == 0 {
        game.witch.direction = if game.witch.side_1 > 0 { 1 } else { -1 };
    }

    game.witch.angle = game.witch.angle.wrapping_add(game.witch.direction as u8);
    page.witchdir = game.witch.angle as i16;
    page.wflag = game.witch.on_screen as i16;

    if game.witch.on_screen != 0 {
        witch_fx(game, video, video.screen.drawing);

        if game.witch.side_1 > 0
            && game.witch.side_2 < 0
            && actor_distance(game, WITCH_SLOT as i32, 0) < WITCH_REACH
        {
            let facing = game.actors.list[witch.slot()].facing as i32;
            let damage = (rand2(&mut game.rng) + 1) as i16;
            hit_actor(game, video, -1, 0, facing, damage);
        }
    }
}
