//! The camera: where the map is shown so the hero stays near the middle, and when the page turns.

use crate::game::data::state::the_world;

// The hero may wander this far from the camera centre before it moves.
pub(super) const CAMERA_CENTER_X: i32 = 144;
pub(super) const CAMERA_CENTER_Y: i32 = 70;
pub(super) const CAMERA_JUMP_X: i16 = 70;
pub(super) const CAMERA_CREEP_X: i16 = 20;
pub(super) const CAMERA_JUMP_UP: i16 = 24;
pub(super) const CAMERA_JUMP_DOWN: i16 = 44;
pub(super) const CAMERA_CREEP_Y: i16 = 10;

/// Sign-extends a 15 bit difference to 16 bits.
pub(super) fn sign_extend_15(v: i16) -> i16 {
    // Shifting left then arithmetically right copies bit 14 into bit 15, making a 15 bit signed number.
    ((v as i32) << 1) as i16 >> 1
}

/// Where the camera goes for a hero position: it creeps along once the hero is 20 pixels (10
/// vertically) off centre, and jumps once the hero is 70 (24 up, 44 down) off. The hero is kept
/// at about (144, 70) on screen.
pub(super) fn camera_after(cam_x: u16, cam_y: u16, x: i32, y: i32) -> (u16, u16) {
    let cx = (x - CAMERA_CENTER_X) as i16;
    let cy = (y - CAMERA_CENTER_Y) as i16;
    let dx = sign_extend_15((cx as i32 - cam_x as i32) as i16);
    let dy = sign_extend_15((cy as i32 - cam_y as i32) as i16);

    let new_x = if dx < -CAMERA_JUMP_X {
        (cx as i32 + CAMERA_JUMP_X as i32) as u16
    } else if dx > CAMERA_JUMP_X {
        (cx as i32 - CAMERA_JUMP_X as i32) as u16
    } else if dx < -CAMERA_CREEP_X {
        cam_x.wrapping_sub(1)
    } else if dx > CAMERA_CREEP_X {
        cam_x.wrapping_add(1)
    } else {
        cam_x
    };

    let new_y = if dy < -CAMERA_JUMP_UP {
        (cy as i32 + CAMERA_JUMP_UP as i32) as u16
    } else if dy > CAMERA_JUMP_DOWN {
        (cy as i32 - CAMERA_JUMP_DOWN as i32) as u16
    } else if dy < -CAMERA_CREEP_Y {
        cam_y.wrapping_sub(1)
    } else if dy > CAMERA_CREEP_Y {
        cam_y.wrapping_add(1)
    } else {
        cam_y
    };

    (new_x, new_y)
}

/// Moves the camera (`map_x`, `map_y`) after the hero.
pub fn move_camera_to(x: i32, y: i32) {
    unsafe {
        (the_world.map_x, the_world.map_y) = camera_after(the_world.map_x, the_world.map_y, x, y)
    };
}

/// Small lookup used by the page-turn animation: how far in from the top and bottom a strip of
/// the curling page starts, given its distance from the fold.
pub fn page_turn_inset(x: i32) -> i32 {
    const SMALL: [i32; 11] = [9, 9, 8, 7, 6, 5, 5, 5, 4, 4, 4];

    match x {
        ..=10 => SMALL[x as usize],
        137.. => 10,
        136 => 7,
        124..=135 => 6,
        99..=123 => 5,
        72..=98 => 4,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_det_covers_every_range() {
        assert_eq!(page_turn_inset(0), 9);
        assert_eq!(page_turn_inset(10), 4);
        assert_eq!(page_turn_inset(11), 3);
        assert_eq!(page_turn_inset(72), 4);
        assert_eq!(page_turn_inset(99), 5);
        assert_eq!(page_turn_inset(124), 6);
        assert_eq!(page_turn_inset(136), 7);
        assert_eq!(page_turn_inset(137), 10);
    }

    #[test]
    fn camera_stays_while_the_hero_is_near() {
        assert_eq!(camera_after(100, 100, 244 + 10, 170 + 5), (100, 100));
    }

    #[test]
    fn camera_creeps_then_jumps() {
        assert_eq!(camera_after(100, 100, 144 + 100 + 30, 170), (101, 100));
        assert_eq!(camera_after(100, 100, 144 + 100 + 80, 170), (110, 100));
        assert_eq!(camera_after(100, 100, 244, 70 + 100 + 50), (100, 106));
    }
}
