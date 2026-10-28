//! Rust mirrors of the game's own C structs (csrc/ftale.h and csrc/fmain.c).
//!
//! (The layout does not have to match the C any more: nothing outside Rust reads these.)

use crate::game::render::blit::{Loc, SShape};

/// One actor (`struct shape`): the hero, a monster, a person or an object lying around.
///
/// Positions are world coordinates in pixels; `rel_x`/`rel_y` are the position on screen.
/// `kind` says what sort of thing it is (see the sequence numbers in csrc/ftale.h) and `race` its
/// species. `state` is what it is doing now (walking, fighting, dead ...), `goal` and `tactic` are
/// what an enemy is trying to do and how. `facing` is a direction 0-7.
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
pub struct WorldObject {
    pub xc: u16,
    pub yc: u16,
    pub ob_id: i8,
    pub ob_stat: i8,
}

/// An arrow, rock or fireball in flight (`struct missile`).
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
pub struct SeqInfo {
    pub width: i16,
    pub height: i16,
    pub count: i16,
    /// Where the frames start in the sprite memory, and where the masks start.
    pub location: usize,
    pub maskloc: usize,
    pub bytes: i16,
}

impl SeqInfo {
    pub const EMPTY: SeqInfo = SeqInfo {
        width: 0,
        height: 0,
        count: 0,
        location: 0,
        maskloc: 0,
        bytes: 0,
    };
}

/// One of the two display pages being drawn and shown (`struct fpage`). The game draws on one
/// while the other is on screen, then swaps them. It also remembers what is drawn there so it can
/// be erased later, and the witch's beam position.
pub struct FPage {
    /// Which of the screen's `ras` entries (the bitmap and its scroll offset) this page shows.
    pub ras: usize,
    pub isv_x: i32,
    pub isv_y: i32,
    pub obcount: i16,
    /// How each sprite drawn on this page was drawn, to undo them.
    pub shape_queue: Vec<SShape>,
    /// Where this page's saved backgrounds start.
    pub backsave: Loc,
    pub saveused: i32,
    pub witchx: i16,
    pub witchy: i16,
    pub witchdir: i16,
    pub wflag: i16,
}

impl FPage {
    pub const fn new() -> FPage {
        FPage {
            ras: 0,
            isv_x: 0,
            isv_y: 0,
            obcount: 0,
            shape_queue: Vec::new(),
            backsave: Loc::NONE,
            saveused: 0,
            witchx: 0,
            witchy: 0,
            witchdir: 0,
            wflag: 0,
        }
    }
}

/// One animation frame of an actor (`struct state`, the C table `animation_frames`): which figure image to
/// draw, which weapon image goes with it, and where the weapon sits relative to the actor.
pub struct StateEntry {
    pub figure: i8,
    pub wpn_no: i8,
    pub wpn_x: i8,
    pub wpn_y: i8,
}

/// The disk blocks one region needs (`struct need`): four image banks, two terrain tables, the
/// sector data, the region map and the character set.
#[derive(Clone, Copy)]
pub struct Need {
    pub image: [u16; 4],
    pub terra1: u16,
    pub terra2: u16,
    pub sector: u16,
    pub region: u16,
    #[allow(dead_code)] // in the original table; the game never reads it
    pub setchar: u16,
}

/// One of the game's menus (`struct menu`): the text labels, how many entries it has, the colour
/// of the entries, and for each entry whether it is enabled. In `enabled`, bit 0 means selected,
/// and the value (2 = plain, 3 = on, 8-10 = from an item the hero has) says how it is drawn.
pub struct Menu {
    pub label_list: &'static [u8],
    pub num: i8,
    pub color: i8,
    pub enabled: [i8; 12],
}

/// The brothers' starting stats and their item lists (`struct bro`).
pub struct Bro {
    pub brave: i8,
    pub luck: i8,
    pub kind: i8,
    pub wealth: i8,
}

/// One kind of item the hero can carry (`struct inv_item`): where its picture is, how it is
/// stacked on the item list screen, and its name.
pub struct InvItem {
    pub image_number: u8,
    pub xoff: u8,
    pub yoff: u8,
    pub ydelta: u8,
    pub img_off: u8,
    pub img_height: u8,
    pub maxshown: u8,
    pub name: &'static str,
}

/// A door between the outdoors and an inside (`struct door`): `xc1`/`yc1` is where it is outside,
/// `xc2`/`yc2` where it leads inside. `kind` is the sort of door (bit 0 set = horizontal) and
/// `secs` says which map section it joins.
pub struct Door {
    pub xc1: u16,
    pub yc1: u16,
    pub xc2: u16,
    pub yc2: u16,
    pub kind: i8,
    pub secs: i8,
}

/// A keyboard shortcut for a menu entry (`struct letters`).
pub struct Letter {
    pub letter: i8,
    pub menu: i8,
    pub choice: i8,
}
