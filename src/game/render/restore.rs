//! Putting back the background under last frame's sprites, and undoing the witch effect on that
//! page (from the main loop in csrc/fmain.c).

use crate::game::data::globals::current_sshape;
use crate::game::data::state::the_screen;
use crate::game::render::blit::{SShape, restore_background};
use crate::game::render::witch::witch_fx;

use crate::platform::gfx::{disown_blitter, own_blitter};

/// Puts back the background under last frame's sprites, then undoes the witch effect on that page.
pub fn undo_sprites() {
    unsafe {
        the_screen.bm_draw = (*(*the_screen.drawing_page).ri_page).BitMap;
        the_screen.planes = (*the_screen.bm_draw).Planes.as_mut_ptr();

        own_blitter();
        let queue = (*the_screen.drawing_page).shape_queue.cast::<SShape>();

        for i in (1..=(*the_screen.drawing_page).obcount as isize).rev() {
            current_sshape = queue.offset(i - 1);
            restore_background((*current_sshape).backsave);
        }

        disown_blitter();

        the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;

        if (*the_screen.drawing_page).wflag != 0 {
            witch_fx(the_screen.drawing_page);
        }
    }
}
