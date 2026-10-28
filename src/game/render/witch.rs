//! The witch's spinning beam (from csrc/fmain2.c): a filled quadrilateral drawn in XOR mode
//! in a temporary layer, with the sign of the hero's side of each edge kept for hit tests.
//!
//! The beam is the area between two of 64 directions around the witch (`witchpoints` lists the near
//! and far end of each). `witch_fx` draws it with the exclusive-or drawing mode, so drawing it a
//! second time erases it. It also records, in `s1` and `s2`, on which side of each edge the hero
//! stands, which csrc/fmain.c uses to decide whether the beam hits the hero.

use crate::game::data::object_tables::witchpoints;
use crate::game::data::state::Game;
use crate::game::data::state::Video;
use crate::game::data::types::FPage;
use crate::game::render::screen::PortId;
use crate::gfx::bitmap::Rect;
use crate::gfx::draw::COMPLEMENT;

/// The beam is clipped to the 19 x 6 tiles of the play field.
const CLIP_WIDTH: i32 = 16 * 19;
const CLIP_HEIGHT: i32 = 6 * 32;

/// The map position is cut to whole tiles (16 wide, 32 tall).
const TILE_X_MASK: u16 = 0xfff0;
const TILE_Y_MASK: u16 = 0xffe0;

/// The beam has 64 directions, 4 bytes (two points) each.
const POINT_BYTES: i32 = 4;
const DIRECTIONS_BEFORE: i32 = 63;
const DIRECTIONS_AFTER: i32 = 1;

/// An edge is two (x, y) points of one byte each, so 4 bytes; the beam polygon has 4 corners.
const EDGE_BYTES: usize = 4;
const BEAM_CORNERS: usize = 4;

/// The two points of the beam edge for direction `dir`, relative to the witch.
fn edge(dir: i16, fp: &FPage) -> ((i16, i16), (i16, i16)) {
    {
        let start = ((dir as i32 * POINT_BYTES) as u8) as usize;
        let w = &witchpoints[start..start + EDGE_BYTES];

        (
            (w[0] as i16 + fp.witchx, w[1] as i16 + fp.witchy),
            (w[2] as i16 + fp.witchx, w[3] as i16 + fp.witchy),
        )
    }
}

/// Which side of the line (p1, p2) the point (xh, yh) is on: the cross product, as the 16 bit
/// value the original kept.
fn side(p1: (i16, i16), p2: (i16, i16), xh: i16, yh: i16) -> i16 {
    let (dx, dy) = (p2.0.wrapping_sub(p1.0), p2.1.wrapping_sub(p1.1));
    let (dx1, dy1) = (xh.wrapping_sub(p1.0), yh.wrapping_sub(p1.1));
    (dx as i32 * dy1 as i32 - dy as i32 * dx1 as i32) as i16
}

/// Draws the witch's beam on page `page` and records on which side of its edges the hero is.
/// The drawing is clipped to the map area.
pub fn witch_fx(game: &mut Game, video: &mut Video, page: usize) {
    let fp = &video.screen.pages[page];

    let xh = game.hero.x.wrapping_sub(game.world.map_x & TILE_X_MASK) as i16;

    let yh = game.hero.y.wrapping_sub(game.world.map_y & TILE_Y_MASK) as i16;

    let (p1, p2) = edge(fp.witchdir.wrapping_add(DIRECTIONS_BEFORE as i16), fp);
    let (p3, p4) = edge(fp.witchdir.wrapping_add(DIRECTIONS_AFTER as i16), fp);
    game.witch.side_1 = side(p1, p2, xh, yh);
    game.witch.side_2 = side(p3, p4, xh, yh);

    paint_beam(video, [p1, p2, p4, p3]);
}

/// Fills the beam's quadrilateral with corners `p` (in drawing order) in complement mode, so
/// drawing it a second time erases it.
fn paint_beam(video: &mut Video, p: [(i16, i16); BEAM_CORNERS]) {
    let points = p.map(|(x, y)| (x as i32, y as i32));
    let clip = Rect {
        x: 0,
        y: 0,
        width: CLIP_WIDTH + 1,
        height: CLIP_HEIGHT + 1,
    };

    video.screen.set_draw_mode(PortId::Map, COMPLEMENT);
    video.screen.fill_polygon(PortId::Map, &points, clip);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_tells_which_way_the_hero_is() {
        // line along x; a point below it (positive y) has a positive cross product
        assert_eq!(side((0, 0), (10, 0), 5, 5), 50);
        assert_eq!(side((0, 0), (10, 0), 5, -5), -50);
    }
}
