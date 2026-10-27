//! What each enemy wants to do each frame (`update_enemy_goals`, from the main loop in csrc/fmain.c): picks a
//! goal and a tactic from how near the hero is and what the enemy carries, and starts or ends the
//! battle music.

mod goals;

mod tactics;

pub use self::goals::update_enemy_goals;

const STILL: i8 = 13;
const BACKUP: i32 = 5;

/// Weapon bit for a bow.
const BOW_BIT: i8 = 4;
