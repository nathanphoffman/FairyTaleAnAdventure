//! The text-panel menus (from csrc/fmain.c): which entries are switched on, and changing menu.
//!
//! `menus` holds the 10 menus; `cmode` is the one being shown. Each entry's `enabled` byte says
//! how it is drawn: 8 is greyed out, 10 is available, and other values come from whether the hero
//! has the matching item (`stuff_flag`).

use super::asm_text::stuff_flag;
use super::input::forget_last_menu;
use super::types::Menu;

unsafe extern "C" {
    static mut menus: [Menu; 10];
    static mut cmode: i16;
    static wealth: i16;

    fn print_options() -> i32;
}

// Menu numbers (`enum cmodes` in csrc/fmain.c).
const MAGIC: usize = 1;
const GAME: usize = 4;
const KEYS: usize = 6;
const GIVE: usize = 7;
const USE: usize = 8;

/// An entry that is shown but cannot be used, and one that can.
const GREYED: i8 = 8;
const AVAILABLE: i8 = 10;
/// The game menu entry that pauses the game; bit 0 set means paused.
const PAUSE_ENTRY: usize = 5;

// Inventory slots, counted from the start of `stuff`, for each use/give entry.
const MAGIC_ITEM_BASE: i32 = 9;
const KEY_ITEM_BASE: i32 = 16;
const SUNSTONE: i32 = 7;
const WRIT: i32 = 28;
const BONE: i32 = 29;
/// The hero must have more than this much gold to be able to give some away.
const GIVE_GOLD_MIN: i16 = 2;

/// Switches the use, magic, keys and give entries on or off to match what the hero is carrying.
#[unsafe(no_mangle)]
pub extern "C" fn set_options() {
    unsafe {
        for i in 0..7 {
            menus[MAGIC].enabled[i + 5] = stuff_flag(i as i32 + MAGIC_ITEM_BASE) as i8;
            menus[USE].enabled[i] = stuff_flag(i as i32) as i8;
        }
        // "use a key" is available if any key slot reports available
        let mut keys = GREYED;
        for i in 0..6 {
            let flag = stuff_flag(i as i32 + KEY_ITEM_BASE) as i8;
            menus[KEYS].enabled[i + 5] = flag;
            if flag == AVAILABLE {
                keys = AVAILABLE;
            }
        }
        menus[USE].enabled[7] = keys;
        menus[USE].enabled[8] = stuff_flag(SUNSTONE) as i8;
        menus[GIVE].enabled[5] = if wealth > GIVE_GOLD_MIN { AVAILABLE } else { GREYED }; // gold
        menus[GIVE].enabled[6] = GREYED; // book
        menus[GIVE].enabled[7] = stuff_flag(WRIT) as i8;
        menus[GIVE].enabled[8] = stuff_flag(BONE) as i8;
    }
}

/// Switches to menu `mode` and redraws it, unless the game is paused.
#[unsafe(no_mangle)]
pub extern "C" fn gomenu(mode: i16) {
    unsafe {
        if menus[GAME].enabled[PAUSE_ENTRY] & 1 != 0 {
            return;
        }
        cmode = mode;
        forget_last_menu();
        print_options();
    }
}
