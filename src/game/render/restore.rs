//! Putting back the background under last frame's sprites, and undoing the witch effect on that
//! page (from the main loop in csrc/fmain.c).

use crate::game::data::state::Game;
use crate::game::data::state::Video;
use crate::game::render::blit::restore_background;
use crate::game::render::screen::PortId;
use crate::game::render::witch::witch_fx;

/// Puts back the background under last frame's sprites, then undoes the witch effect on that page.
pub fn undo_sprites(game: &mut Game, video: &mut Video) {
    video.screen.draw_target = video.screen.drawing_bitmap_id();

    for i in (1..=video.screen.drawing_page().obcount as usize).rev() {
        video.blitter.sshape = video.screen.drawing_page().shape_queue[i - 1];
        restore_background(video, video.blitter.sshape.backsave);
    }

    video.screen.port_mut(PortId::Map).target = Some(video.screen.drawing_bitmap_id());

    if video.screen.drawing_page().wflag != 0 {
        witch_fx(game, video, video.screen.drawing);
    }
}
