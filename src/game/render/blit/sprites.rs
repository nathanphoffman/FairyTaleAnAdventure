//! The sprite drawing steps built on the blitter: saving and restoring the background, the masks, and
//! drawing the shape.

use super::engine::{Blit, blit, blit_height, blit_width};
use super::{
    ALL_BITS, D_EQUALS_A_AND_NOT_C, D_EQUALS_A_THEN_B_ELSE_C, D_EQUALS_B, MASK_START, PLANE_COUNT,
    SHIFT_MASK, TILE_LINES, TILE_MASK_BYTES, USE_A, USE_B, USE_C, USE_D,
};

use crate::game::data::globals::{
    current_sshape, last_word_mask, shadow_mask_offset, sprite_mask_modulo, sprite_mask_offset,
    sprite_x_shift,
};

use crate::game::data::state::{the_memory, the_screen};

/// Where plane `p` of the play field starts for the current sprite.
pub(super) unsafe fn plane_at_sprite(p: i32) -> *mut u8 {
    unsafe { (*the_screen.planes.offset(p as isize)).offset((*current_sshape).coff as isize) }
}

/// Zeroes a rectangle of memory, sized like a BLTSIZE value.
pub fn clear_memory(mem: *mut u8, size: i32) {
    let bytes = blit_height(size as i16) * blit_width(size as i16) * 2;

    unsafe { std::ptr::write_bytes(mem, 0, bytes as usize) };
}

/// Copies the background under the sprite (all planes) to `savemem`.
pub fn save_background(savemem: *mut u8) {
    unsafe {
        for p in (0..PLANE_COUNT).rev() {
            let mut t = Blit::new(USE_B | USE_D, D_EQUALS_B, (*current_sshape).blitsize);
            t.b = plane_at_sprite(p);
            t.bmod = (*current_sshape).cmod as i32;
            t.d = savemem.offset((*current_sshape).savesize as isize * p as isize);
            blit(&mut t);
        }
    }
}

/// Puts the saved background back (this erases the sprite).
pub fn restore_background(savemem: *mut u8) {
    unsafe {
        for p in (0..PLANE_COUNT).rev() {
            let mut t = Blit::new(USE_B | USE_D, D_EQUALS_B, (*current_sshape).blitsize);
            t.b = savemem.offset((*current_sshape).savesize as isize * p as isize);
            t.d = plane_at_sprite(p);
            t.dmod = (*current_sshape).cmod as i32;
            blit(&mut t);
        }
    }
}

/// Combines the sprite's own mask with the scenery mask built by `add_scenery_mask`. The result goes in
/// `bmask_mem`: the sprite is opaque here and nothing is in front of it (D = A and not C).
pub fn combine_masks() {
    unsafe {
        let mut t = Blit::new(
            USE_A | USE_C | USE_D,
            D_EQUALS_A_AND_NOT_C,
            (*current_sshape).blitsize,
        );

        t.a = the_memory.mask_buffer.offset(sprite_mask_offset as isize);
        t.amod = sprite_mask_modulo as i32;

        // Masks with SHIFT_MASK, keeping only the bits that matter here.
        t.ashift = (sprite_x_shift & SHIFT_MASK) as u32;
        t.c = the_memory
            .bmask_mem
            .add(MASK_START)
            .offset(shadow_mask_offset as isize);
        t.d = t.c;
        blit(&mut t);
    }
}

/// Draws the sprite into the planes through the mask: D = mask ? shape : screen.
/// The first and last word masks stop the sprite's shifted edges leaking into neighbouring pixels.
pub fn draw_shape() {
    unsafe {
        // Masks with SHIFT_MASK, keeping only the bits that matter here.
        let bit_shift = (sprite_x_shift & SHIFT_MASK) as u32;

        // The first word is masked so the bits left of the sprite's pixel offset stay clear.
        let first = ALL_BITS >> bit_shift;

        // Masks with ALL_BITS, keeping only the bits that matter here.
        let mut last = !first & ALL_BITS;

        if last == 0 {
            last = ALL_BITS;
        }

        for p in (0..PLANE_COUNT).rev() {
            let mut t = Blit::new(
                USE_A | USE_B | USE_C | USE_D,
                D_EQUALS_A_THEN_B_ELSE_C,
                (*current_sshape).blitsize,
            );

            t.bshift = bit_shift;
            t.afwm = first & last_word_mask as u16 as u32;
            t.alwm = last;
            t.a = the_memory
                .bmask_mem
                .add(MASK_START)
                .offset(shadow_mask_offset as isize);
            t.b = the_memory
                .shapedata
                .offset(sprite_mask_offset as isize + the_screen.planesize as isize * p as isize);
            t.bmod = sprite_mask_modulo as i32;
            t.c = plane_at_sprite(p);
            t.d = t.c;
            t.cmod = (*current_sshape).cmod as i32;
            t.dmod = t.cmod;
            blit(&mut t);
        }
    }
}

/// Copies the 32 line mask of scenery tile `tile` (from `shadow_mem`) into the scenery mask at
/// word column `x`, tile row `y`; `mod_words` is the mask width in words.
pub fn add_scenery_mask(x: i32, y: i32, mod_words: i32, tile: i32) {
    let row_bytes = mod_words as isize * 2;
    unsafe {
        let mut dst = the_memory
            .bmask_mem
            .add(MASK_START)
            .offset(y as isize * row_bytes * TILE_LINES as isize + x as isize * 2);
        let mut src = the_memory
            .shadow_mem
            .offset(tile as isize * TILE_MASK_BYTES as isize);

        for _ in 0..TILE_LINES {
            *dst = *src;
            *dst.add(1) = *src.add(1);
            src = src.add(2);
            dst = dst.offset(row_bytes);
        }
    }
}

/// For every sprite frame builds a one plane mask: all 5 planes ANDed then inverted, so colour
/// 31 (all bits set) is transparent. The masks are stored right after the frames.
pub fn make_sprite_masks(src: *const u8, dest: *mut u8, words: i32, scans: i32, chars: i32) {
    let plane_bytes = (words * scans) as u16 as usize * 2;
    unsafe {
        let (mut src, mut dest) = (src, dest);

        for _ in 0..chars {
            let plane = |n: usize| src.add(n * plane_bytes);

            for i in 0..plane_bytes {
                let all = (0..PLANE_COUNT as usize).fold(0xff, |acc, n| acc & *plane(n).add(i));
                *dest = !all;
                dest = dest.add(1);
            }

            src = plane(PLANE_COUNT as usize);
        }
    }
}
