//! The sprite drawing steps built on the blitter: saving and restoring the background, the masks, and
//! drawing the shape.

use super::engine::{Blit, BlitMem, blit, blit_height, blit_width};
use super::{
    ALL_BITS, Area, D_EQUALS_A_AND_NOT_C, D_EQUALS_A_THEN_B_ELSE_C, D_EQUALS_B, Loc, MASK_START,
    PLANE_COUNT, SHIFT_MASK, TILE_LINES, TILE_MASK_BYTES, USE_A, USE_B, USE_C, USE_D,
};

use crate::game::data::globals::{
    current_sshape, last_word_mask, shadow_mask_offset, sprite_mask_modulo, sprite_mask_offset,
    sprite_x_shift,
};

use crate::game::data::state::memory;
use crate::game::render::screen::screen;

/// Runs `job` on the memory a blit can reach.
fn with_mem<R>(job: impl FnOnce(&mut BlitMem) -> R) -> R {
    let (bitmap, queue, mask) = screen().blitter_memory();

    job(&mut BlitMem {
        bitmap,
        queue,
        mask,
        shapes: &mut memory().shape,
    })
}

/// Where plane `p` of the play field starts for the current sprite.
fn plane_at_sprite(p: i32) -> Loc {
    Loc {
        area: Area::Plane(p as u8),
        pos: unsafe { current_sshape.coff } as isize,
    }
}

/// Where the scenery mask of the current sprite starts.
fn scenery_mask_at() -> Loc {
    Loc::new(Area::Mask, MASK_START).add(unsafe { shadow_mask_offset } as isize)
}

/// Zeroes the start of the scenery mask, a rectangle sized like a BLTSIZE value.
pub fn clear_memory(size: i32) {
    let bytes = (blit_height(size as i16) * blit_width(size as i16) * 2) as usize;

    with_mem(|mem| {
        for i in 0..bytes {
            mem.set_byte(Loc::new(Area::Mask, i), 0);
        }
    });
}

/// Copies the background under the sprite (all planes) to `savemem`.
pub fn save_background(savemem: Loc) {
    let sshape = unsafe { current_sshape };

    with_mem(|mem| {
        for p in (0..PLANE_COUNT).rev() {
            let mut t = Blit::new(USE_B | USE_D, D_EQUALS_B, sshape.blitsize);
            t.b = plane_at_sprite(p);
            t.bmod = sshape.cmod as i32;
            t.d = savemem.add(sshape.savesize as isize * p as isize);
            blit(&mut t, mem);
        }
    });
}

/// Puts the saved background back (this erases the sprite).
pub fn restore_background(savemem: Loc) {
    let sshape = unsafe { current_sshape };

    with_mem(|mem| {
        for p in (0..PLANE_COUNT).rev() {
            let mut t = Blit::new(USE_B | USE_D, D_EQUALS_B, sshape.blitsize);
            t.b = savemem.add(sshape.savesize as isize * p as isize);
            t.d = plane_at_sprite(p);
            t.dmod = sshape.cmod as i32;
            blit(&mut t, mem);
        }
    });
}

/// Combines the sprite's own mask with the scenery mask built by `add_scenery_mask`. The result goes in
/// the scenery mask: the sprite is opaque here and nothing is in front of it (D = A and not C).
pub fn combine_masks() {
    let (sshape, mask_offset, mask_modulo, x_shift) = unsafe {
        (
            current_sshape,
            sprite_mask_offset,
            sprite_mask_modulo,
            sprite_x_shift,
        )
    };
    let mut t = Blit::new(USE_A | USE_C | USE_D, D_EQUALS_A_AND_NOT_C, sshape.blitsize);

    t.a = Loc::new(Area::Shapes, memory().mask_buffer).add(mask_offset as isize);
    t.amod = mask_modulo as i32;

    // Masks with SHIFT_MASK, keeping only the bits that matter here.
    t.ashift = (x_shift & SHIFT_MASK) as u32;
    t.c = scenery_mask_at();
    t.d = t.c;
    with_mem(|mem| blit(&mut t, mem));
}

/// Draws the sprite into the planes through the mask: D = mask ? shape : screen.
/// The first and last word masks stop the sprite's shifted edges leaking into neighbouring pixels.
pub fn draw_shape() {
    // Masks with SHIFT_MASK, keeping only the bits that matter here.
    let bit_shift = (unsafe { sprite_x_shift } & SHIFT_MASK) as u32;

    // The first word is masked so the bits left of the sprite's pixel offset stay clear.
    let first = ALL_BITS >> bit_shift;

    // Masks with ALL_BITS, keeping only the bits that matter here.
    let mut last = !first & ALL_BITS;

    if last == 0 {
        last = ALL_BITS;
    }

    with_mem(|mem| {
        for p in (0..PLANE_COUNT).rev() {
            draw_shape_plane(mem, p, bit_shift, (first, last));
        }
    });
}

/// Draws one bit plane of the sprite: the mask picks, bit by bit, the sprite's pixel or the screen's.
/// `(first, last)` are the masks for the first and last word of each row.
fn draw_shape_plane(mem: &mut BlitMem, p: i32, bit_shift: u32, (first, last): (u32, u32)) {
    let (sshape, mask_offset, mask_modulo, word_mask) = unsafe {
        (
            current_sshape,
            sprite_mask_offset,
            sprite_mask_modulo,
            last_word_mask,
        )
    };
    let mut t = Blit::new(
        USE_A | USE_B | USE_C | USE_D,
        D_EQUALS_A_THEN_B_ELSE_C,
        sshape.blitsize,
    );

    t.bshift = bit_shift;
    t.afwm = first & word_mask as u16 as u32;
    t.alwm = last;
    t.a = scenery_mask_at();
    t.b = Loc::new(Area::Shapes, memory().shapedata)
        .add(mask_offset as isize + screen().planesize as isize * p as isize);
    t.bmod = mask_modulo as i32;
    t.c = plane_at_sprite(p);
    t.d = t.c;
    t.cmod = sshape.cmod as i32;
    t.dmod = t.cmod;
    blit(&mut t, mem);
}

/// Copies the 32 line mask of scenery tile `tile` (from the shadow masks) into the scenery mask at
/// word column `x`, tile row `y`; `mod_words` is the mask width in words.
pub fn add_scenery_mask(x: i32, y: i32, mod_words: i32, tile: i32) {
    let row_bytes = mod_words as isize * 2;
    let mut dst = Loc::new(Area::Mask, MASK_START)
        .add(y as isize * row_bytes * TILE_LINES as isize + x as isize * 2);
    let shadow = &memory().shadow;
    let mut src = tile as usize * TILE_MASK_BYTES;

    with_mem(|mem| {
        for _ in 0..TILE_LINES {
            mem.set_byte(dst, shadow[src]);
            mem.set_byte(dst.add(1), shadow[src + 1]);
            src += 2;
            dst = dst.add(row_bytes);
        }
    });
}

/// For every sprite frame builds a one plane mask: all 5 planes ANDed then inverted, so colour
/// 31 (all bits set) is transparent. The frames start at `src` in the sprite memory and the masks
/// are stored from `dest` on.
pub fn make_sprite_masks(src: usize, dest: usize, words: i32, scans: i32, chars: i32) {
    let plane_bytes = (words * scans) as u16 as usize * 2;
    let shapes = &mut memory().shape;
    let (mut src, mut dest) = (src, dest);

    for _ in 0..chars {
        for i in 0..plane_bytes {
            let all = (0..PLANE_COUNT as usize).fold(0xff, |acc, n| {
                acc & shapes.get(src + n * plane_bytes + i).copied().unwrap_or(0)
            });

            if let Some(byte) = shapes.get_mut(dest) {
                *byte = !all;
            }

            dest += 1;
        }

        src += PLANE_COUNT as usize * plane_bytes;
    }
}
