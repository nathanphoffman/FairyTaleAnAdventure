//! Putting back the background under last frame's sprites, and undoing the witch effect on that
//! page (from the main loop in csrc/fmain.c).

use crate::game::data::state::Game;
use crate::game::render::blit::restore_background;
use crate::game::render::screen::PortId;
use crate::game::render::witch::witch_fx;

/// Puts back the background under last frame's sprites, then undoes the witch effect on that page.
pub fn undo_sprites(game: &mut Game) {
    game.screen.draw_target = game.screen.drawing_bitmap_id();

    for i in (1..=game.screen.drawing_page().obcount as usize).rev() {
        game.blitter.sshape = game.screen.drawing_page().shape_queue[i - 1];
        restore_background(game, game.blitter.sshape.backsave);
    }

    game.screen.port_mut(PortId::Map).target = Some(game.screen.drawing_bitmap_id());

    if game.screen.drawing_page().wflag != 0 {
        witch_fx(game, game.screen.drawing);
    }
}
