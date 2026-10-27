//! Putting back the background under last frame's sprites, and undoing the witch effect on that
//! page (from the main loop in csrc/fmain.c).

use crate::game::data::globals::current_sshape;
use crate::game::render::blit::{SShape, restore_background};
use crate::game::render::screen::{PortId, screen};
use crate::game::render::witch::witch_fx;

/// Puts back the background under last frame's sprites, then undoes the witch effect on that page.
pub fn undo_sprites() {
    unsafe {
        let screen = screen();
        screen.draw_target = screen.drawing_bitmap_id();
        let queue = screen.drawing_page().shape_queue.cast::<SShape>();

        for i in (1..=screen.drawing_page().obcount as isize).rev() {
            current_sshape = queue.offset(i - 1);
            restore_background((*current_sshape).backsave);
        }

        screen.port_mut(PortId::Map).target = Some(screen.drawing_bitmap_id());

        if screen.drawing_page().wflag != 0 {
            witch_fx(screen.drawing);
        }
    }
}
