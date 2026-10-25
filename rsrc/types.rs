//! Rust mirrors of the game's own C structs (csrc/ftale.h and csrc/fmain.c).

/// One actor (`struct shape`).
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

/// A rectangle of the world with a special meaning, e.g. where a monster type appears
/// (`struct extent`).
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

/// An item lying in the world (`struct object`).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_match_the_c_structs() {
        assert_eq!(size_of::<Shape>(), 22);
        assert_eq!(size_of::<Extent>(), 12);
        assert_eq!(size_of::<WorldObject>(), 6);
        assert_eq!(size_of::<Missile>(), 10);
    }
}
