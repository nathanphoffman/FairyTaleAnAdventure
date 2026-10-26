//! The text-panel menus (from csrc/fmain.c): which entries are switched on, and changing menu.
//!
//! `menus` holds the 10 menus; `current_menu` is the one being shown. Each entry's `enabled` byte says
//! how it is drawn: 8 is greyed out, 10 is available, and other values come from whether the hero
//! has the matching item (`item_menu_state`).

use super::asm_text::item_menu_state;
use super::game_tables::{key_colors, label1, menus};
use super::globals::{current_menu, rp_text2, slot_entries, wealth};
use super::input::forget_last_menu;
use crate::amiga::RastPort;
use std::ffi::c_char;

unsafe extern "C" {
    fn SetAPen(current_rastport: *mut RastPort, pen: u32);
    fn SetBPen(current_rastport: *mut RastPort, pen: u32);
    fn Move(current_rastport: *mut RastPort, x: i32, y: i32);
    fn Text(current_rastport: *mut RastPort, s: *const c_char, len: i32) -> i32;
}

// Menu numbers (`enum cmodes` in csrc/fmain.c).
const MAGIC: usize = 1;
const GAME: usize = 4;
const KEYS: usize = 6;
const GIVE: usize = 7;
const USE: usize = 8;
const FILE: usize = 9;
const SAVEX: usize = 5;

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
pub fn update_menu_availability() {
    unsafe {
        for i in 0..7 {
            menus[MAGIC].enabled[i + 5] = item_menu_state(i as i32 + MAGIC_ITEM_BASE) as i8;
            menus[USE].enabled[i] = item_menu_state(i as i32) as i8;
        }
        // "use a key" is available if any key slot reports available
        let mut keys = GREYED;
        for i in 0..6 {
            let flag = item_menu_state(i as i32 + KEY_ITEM_BASE) as i8;
            menus[KEYS].enabled[i + 5] = flag;
            if flag == AVAILABLE {
                keys = AVAILABLE;
            }
        }
        menus[USE].enabled[7] = keys;
        menus[USE].enabled[8] = item_menu_state(SUNSTONE) as i8;
        menus[GIVE].enabled[5] = if wealth > GIVE_GOLD_MIN {
            AVAILABLE
        } else {
            GREYED
        }; // gold
        menus[GIVE].enabled[6] = GREYED; // book
        menus[GIVE].enabled[7] = item_menu_state(WRIT) as i8;
        menus[GIVE].enabled[8] = item_menu_state(BONE) as i8;
    }
}

/// Most entries a menu shows at once (two columns of six).
const SLOTS: usize = 12;
/// An entry with this bit set is shown.
const SHOWN: i8 = 2;

/// Draws the current menu (`current_menu`): the entries that are switched on, in the order they are shown,
/// and blanks over the slots that are left. `slot_entries` remembers which entry each slot is.
pub fn print_options() {
    unsafe {
        let menu = &menus[current_menu as usize];
        let mut slot = 0usize;
        for entry in 0..menu.num as usize {
            let enabled = menu.enabled[entry];
            if enabled & SHOWN == 0 {
                continue;
            }
            slot_entries[slot] = entry as i8;
            print_menu_entry(slot as i16, (enabled & 1) as i16);
            slot += 1;
            if slot > SLOTS - 1 {
                break;
            }
        }
        SetBPen(&raw mut rp_text2, 0);
        while slot < SLOTS {
            let x = if slot & 1 != 0 {
                RIGHT_COLUMN_X
            } else {
                LEFT_COLUMN_X
            };
            let y = (slot / 2) as i32 * ROW_HEIGHT + FIRST_ROW_Y;
            Move(&raw mut rp_text2, x, y);
            Text(&raw mut rp_text2, c"      ".as_ptr(), 6);
            slot_entries[slot] = -1;
            slot += 1;
        }
    }
}

/// Switches to menu `mode` and redraws it, unless the game is paused.
pub fn go_to_menu(mode: i16) {
    unsafe {
        if menus[GAME].enabled[PAUSE_ENTRY] & 1 != 0 {
            return;
        }
        current_menu = mode;
        forget_last_menu();
        print_options();
    }
}

// Where the menu entries are drawn in the text panel: two columns, a row every 9 pixels.
const LEFT_COLUMN_X: i32 = 430;
const RIGHT_COLUMN_X: i32 = 482;
const FIRST_ROW_Y: i32 = 8;
const ROW_HEIGHT: i32 = 9;
/// The entry's text is inset from the edge of its coloured box by this much.
const TEXT_INSET: i32 = 4;
/// Each label is 5 characters; the box behind it is 6 wide.
const LABEL_LEN: i32 = 5;
const BOX: &[u8; 6] = b"      ";
/// The first 5 entries of every menu are the main menu names, taken from `label1`.
const MAIN_ENTRIES: i32 = 5;
const FILE_PEN: u32 = 13;
const USE_PEN: u32 = 14;
const MAIN_PEN: u32 = 4;

/// Draws entry slot `j` (0-11) of the current menu in the panel. `pena` is the text colour:
/// 1 for a selected entry, 0 for a plain one. The box colour depends on the menu.
pub fn print_menu_entry(j: i16, pena: i16) {
    unsafe {
        let r = &raw mut rp_text2;
        let entry = slot_entries[j as usize] as i32;
        let box_pen = if current_menu as usize == USE {
            USE_PEN
        } else if current_menu as usize == FILE {
            FILE_PEN
        } else if entry < MAIN_ENTRIES {
            MAIN_PEN
        } else if current_menu as usize == KEYS {
            key_colors[(entry - MAIN_ENTRIES) as usize] as u32
        } else if current_menu as usize == SAVEX {
            entry as u32
        } else {
            menus[current_menu as usize].color as u32
        };
        SetAPen(r, pena as u32);
        SetBPen(r, box_pen);

        let offset = entry * LABEL_LEN as i32;
        let x = if j & 1 != 0 {
            RIGHT_COLUMN_X
        } else {
            LEFT_COLUMN_X
        };
        let y = (j as i32 / 2) * ROW_HEIGHT + FIRST_ROW_Y;
        Move(r, x, y);
        Text(r, BOX.as_ptr().cast(), BOX.len() as i32);
        Move(r, x + TEXT_INSET, y);
        let labels = menus[current_menu as usize].label_list;
        // The main names come from `label1`; the other menus have their own list, which for the
        // first 5 slots is skipped over.
        let label = if current_menu as usize >= USE {
            labels.offset(offset as isize)
        } else if offset < MAIN_ENTRIES * LABEL_LEN {
            (&raw const label1).cast::<c_char>().offset(offset as isize)
        } else {
            labels.offset((offset - MAIN_ENTRIES * LABEL_LEN) as isize)
        };
        Text(r, label, LABEL_LEN);
    }
}
