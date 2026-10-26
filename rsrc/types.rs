//! Rust mirrors of the game's own C structs (csrc/ftale.h and csrc/fmain.c).
//!
//! The layout must match the C exactly (`#[repr(C)]`), because the C code that is left reads and
//! writes the same memory. The unit tests at the bottom check the sizes.

use crate::amiga::RasInfo;
use std::ffi::c_void;

/// One actor (`struct shape`): the hero, a monster, a person or an object lying around.
///
/// Positions are world coordinates in pixels; `rel_x`/`rel_y` are the position on screen.
/// `kind` says what sort of thing it is (see the sequence numbers in csrc/ftale.h) and `race` its
/// species. `state` is what it is doing now (walking, fighting, dead ...), `goal` and `tactic` are
/// what an enemy is trying to do and how. `facing` is a direction 0-7.
#[repr(C)]
pub struct Shape {
    pub abs_x: u16,
    pub abs_y: u16,
    pub rel_x: u16,
    pub rel_y: u16,
    pub kind: i8,
    pub race: u8,
    pub index: i8,
    pub visible: i8,
    pub weapon: i8,
    pub environ: i8,
    pub goal: i8,
    pub tactic: i8,
    pub state: i8,
    pub facing: i8,
    pub vitality: i16,
    pub vel_x: i8,
    pub vel_y: i8,
}

/// A rectangle of the world with a special meaning, for example where a kind of monster appears
/// (`struct extent`). `etype` says what it is; `v1`-`v3` are extra values for that type.
#[repr(C)]
pub struct Extent {
    pub x1: u16,
    pub y1: u16,
    pub x2: u16,
    pub y2: u16,
    pub etype: u8,
    pub v1: u8,
    pub v2: u8,
    pub v3: u8,
}

/// An item or person standing in the world (`struct object`).
/// `ob_stat` is its state: 0 does not exist, 1 on the ground, 2 taken, 3 a person, 4 a dead
/// person, 5 hidden until looked for.
#[repr(C)]
pub struct WorldObject {
    pub xc: u16,
    pub yc: u16,
    pub ob_id: i8,
    pub ob_stat: i8,
}

/// An arrow, rock or fireball in flight (`struct missile`).
#[repr(C)]
pub struct Missile {
    pub abs_x: u16,
    pub abs_y: u16,
    pub missile_type: i8,
    pub time_of_flight: i8,
    pub speed: i8,
    pub direction: i8,
    pub archer: i8,
}

/// Where a sprite sequence (all the frames of one kind of character) is in memory (`struct seq_info`),
/// and where its transparency mask is.
#[repr(C)]
pub struct SeqInfo {
    pub width: i16,
    pub height: i16,
    pub count: i16,
    pub location: *mut u8,
    pub maskloc: *mut u8,
    pub bytes: i16,
    pub current_file: i16,
}

/// One of the two display pages being drawn and shown (`struct fpage`). The game draws on one
/// while the other is on screen, then swaps them. It also remembers what is drawn there so it can
/// be erased later, and the witch's beam position.
#[repr(C)]
pub struct FPage {
    pub ri_page: *mut RasInfo,
    pub savecop: *mut c_void,
    pub isv_x: i32,
    pub isv_y: i32,
    pub obcount: i16,
    pub shape_queue: *mut c_void,
    pub backsave: *mut u8,
    pub saveused: i32,
    pub witchx: i16,
    pub witchy: i16,
    pub witchdir: i16,
    pub wflag: i16,
}

/// One animation frame of an actor (`struct state`, the C table `statelist`): which figure image to
/// draw, which weapon image goes with it, and where the weapon sits relative to the actor.
#[repr(C)]
pub struct StateEntry {
    pub figure: i8,
    pub wpn_no: i8,
    pub wpn_x: i8,
    pub wpn_y: i8,
}

/// The disk blocks one region needs (`struct need`): four image banks, two terrain tables, the
/// sector data, the region map and the character set.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Need {
    pub image: [u16; 4],
    pub terra1: u16,
    pub terra2: u16,
    pub sector: u16,
    pub region: u16,
    pub setchar: u16,
}

/// One of the game's menus (`struct menu`): the text labels, how many entries it has, the colour
/// of the entries, and for each entry whether it is enabled. In `enabled`, bit 0 means selected,
/// and the value (2 = plain, 3 = on, 8-10 = from an item the hero has) says how it is drawn.
#[repr(C)]
pub struct Menu {
    pub label_list: *const std::ffi::c_char,
    pub num: i8,
    pub color: i8,
    pub enabled: [i8; 12],
}

/// The brothers' starting stats and their item lists (`struct bro`).
#[repr(C)]
pub struct Bro {
    pub brave: i8,
    pub luck: i8,
    pub kind: i8,
    pub wealth: i8,
    pub stuff: *mut u8,
}

/// One kind of item the hero can carry (`struct inv_item`): where its picture is, how it is
/// stacked on the item list screen, and its name.
#[repr(C)]
pub struct InvItem {
    pub image_number: u8,
    pub xoff: u8,
    pub yoff: u8,
    pub ydelta: u8,
    pub img_off: u8,
    pub img_height: u8,
    pub maxshown: u8,
    pub name: *const std::ffi::c_char,
}

/// A door between the outdoors and an inside (`struct door`): `xc1`/`yc1` is where it is outside,
/// `xc2`/`yc2` where it leads inside. `kind` is the sort of door (bit 0 set = horizontal) and
/// `secs` says which map section it joins.
#[repr(C)]
pub struct Door {
    pub xc1: u16,
    pub yc1: u16,
    pub xc2: u16,
    pub yc2: u16,
    pub kind: i8,
    pub secs: i8,
}

/// A keyboard shortcut for a menu entry (`struct letters`).
#[repr(C)]
pub struct Letter {
    pub letter: i8,
    pub menu: i8,
    pub choice: i8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_match_the_c_structs() {
        assert_eq!(size_of::<Shape>(), 22);
        assert_eq!(size_of::<Extent>(), 12);
        assert_eq!(size_of::<WorldObject>(), 6);
        assert_eq!(size_of::<Missile>(), 10);
        assert_eq!(size_of::<SeqInfo>(), 32);
        assert_eq!(size_of::<FPage>(), 64);
        assert_eq!(size_of::<Need>(), 18);
        assert_eq!(size_of::<Menu>(), 24);
        assert_eq!(size_of::<Bro>(), 16);
        assert_eq!(size_of::<InvItem>(), 16);
        assert_eq!(size_of::<Letter>(), 3);
        assert_eq!(size_of::<Door>(), 10);
        assert_eq!(size_of::<StateEntry>(), 4);
    }
}
