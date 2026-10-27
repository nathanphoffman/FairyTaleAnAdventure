//! Keys and the hero's controls (from csrc/fmain.c): `handle_key` reacts to one key code from the
//! key buffer (menus, direction keys, the fight key, debug keys) and `update_hero` turns the
//! controls into the hero's state each frame.

mod cheats;

mod fairy;

mod hero_state;

mod menu_keys;

pub use self::hero_state::update_hero;

use self::cheats::{cheat_items, cheat_move};
use self::menu_keys::{menu_click, menu_letter};

use crate::game::data::state::{the_actors, the_screen, the_ui};
use crate::game::ui::menus::go_to_menu;
use crate::game::ui::options::choose_menu_entry;

// Menu numbers (`enum cmodes`).
const ITEMS: i16 = 0;
const SAVEX: i16 = 5;
const KEYS: i16 = 6;

// Actor states (csrc/ftale.h).
const FIGHTING: i8 = 0;
const WALKING: i8 = 12;
const STILL: i8 = 13;
const DYING: i8 = 14;
const DEAD: i8 = 15;
const SINK: i8 = 16;
const FALL: i8 = 22;
const SLEEP: i8 = 23;
const SHOOT1: i8 = 24;
const SHOOT3: i8 = 25;
const OBJECTS: i8 = 1;

/// Number of shortcut letters in `menu_shortcuts`.
const LETTER_COUNT: usize = 37;

/// A key code with this bit set is the key being released (or the mouse button going up).
const RELEASED: i8 = 0x80u8 as i8;
const CODE_MASK: i8 = 0x7f;

/// Codes 20-29 are the direction keys.
const FIRST_DIRECTION_KEY: i8 = 20;
const LAST_DIRECTION_KEY: i8 = 29;

/// Menu clicks arrive as 'a' for the first entry slot, 'b' for the next and so on.
const FIRST_MENU_CLICK: i8 = 0x61;
const FIGHT_KEY: i8 = b'0' as i8;
const VIEW_REDRAW: i8 = 99;

// Bits in a menu entry's `enabled` byte.
/// The facing of the hero is 0-7.
const FACING_MASK: i16 = 7;

/// The key menu's entries start at 5: key `1` is entry 5. A menu number below 5 is a menu to
/// switch to, and the print request 5 redraws the options.
const KEY_ENTRY_BASE: i8 = 5;
const FIRST_ACTION_MENU: i16 = 5;
const PRQ_OPTIONS: i32 = 5;

const SELECTED: i8 = 1;
const DISPLAYED: i8 = 2;
const TYPE_MASK: i32 = 0xfc;
const TYPE_TOGGLE: i32 = 4;
const TYPE_IMMEDIATE: i32 = 8;
const TYPE_RADIO: i32 = 12;

// Mouse button bits in the input qualifier.
const LEFT_BUTTON: i16 = 0x4000;
const RIGHT_BUTTON: i16 = 0x2000;
const FIRE_BUTTON: u8 = 0x80;

// Debug keys.
const CHEAT_CARRIER: i8 = b'B' as i8;
const CHEAT_STUFF: i8 = b'.' as i8;
const CHEAT_RESCUE: i8 = b'R' as i8;
const CHEAT_PRQ_2: i8 = b'=' as i8;
const CHEAT_PRQ_3: i8 = 19;
const CHEAT_TIME: i8 = 18;
const CHEAT_NORTH: i8 = 1;
const CHEAT_SOUTH: i8 = 2;
const CHEAT_EAST: i8 = 3;
const CHEAT_WEST: i8 = 4;
const CHEAT_STEP_Y: u16 = 150;
const CHEAT_STEP_X: u16 = 280;
const CHEAT_TIME_STEP: u16 = 1000;
const GOLDBASE: i32 = 31;
const CARRIER_BIRD: i16 = 11;
const CARRIER_OFFSET: i16 = 20;
const RAFT_STONE_SLOT: usize = 5;
const VIAL_SLOT: usize = 22;

/// Handles one key code. `notpause` is true unless the game is paused.
///
/// Codes with bit 7 set are releases. Keys that do nothing in the current state fall through
/// to the menu shortcut letters at the end.
pub fn handle_key(key: i32, notpause: i32) {
    unsafe {
        let key = key as i8;
        let notpause = notpause as i16 != 0;

        if the_screen.view_status != 0 && notpause {
            // Masks with RELEASED, keeping only the bits that matter here.
            if key & RELEASED == 0 {
                the_screen.view_status = VIEW_REDRAW;
            }
        } else if the_actors.list[0].state == DEAD {
            // ignored while dead
        } else if (FIRST_DIRECTION_KEY..=LAST_DIRECTION_KEY).contains(&key) {
            the_ui.held_direction = key as i16;
        // Masks with CODE_MASK, keeping only the bits that matter here.
        } else if (key & CODE_MASK) as i16 == the_ui.held_direction {
            the_ui.held_direction = 0;
        } else if key == FIGHT_KEY {
            the_ui.fight_key_held = 1;
        // Masks with CODE_MASK, keeping only the bits that matter here.
        } else if key & CODE_MASK == FIGHT_KEY {
            the_ui.fight_key_held = 0;
        } else if cheat_items(key) {
            // handled
        } else if key & CODE_MASK >= FIRST_MENU_CLICK {
            menu_click(key, notpause);
        } else if cheat_move(key) {
            // handled
        } else if the_ui.current_menu == KEYS && key & RELEASED == 0 {
            if (b'1' as i8..=b'6' as i8).contains(&key) {
                choose_menu_entry((key - b'1' as i8 + KEY_ENTRY_BASE) as i16);
            } else {
                go_to_menu(ITEMS);
            }
        } else if key == b' ' as i8 || notpause {
            menu_letter(key);
        }
    }
}

// Timers and the bounds of the fiery region (the lava).
const LAVA_X: (u16, u16) = (8802, 13562);
const LAVA_Y: (u16, u16) = (24744, 29544);

// The good fairy countdown: she appears below this, and the hero revives when it runs out.
const FAIRY_ARRIVES: u8 = 120;
const FAIRY_RESURRECT: u8 = 20;
const FAIRY_GIVES_UP: u8 = 200;
const FAIRY_SLOT: usize = 3;
const FAIRY_IMAGE: i8 = 100;
const FAIRY_SPEED: u16 = 2;
const FAIRY_X_OFFSET: u16 = 20;
const FAIRY_RACE: u8 = 0xff;
const NO_ACTORS_LOADED: i16 = 4;

// Riding and shooting.
const SHOOTING_ZONE: u16 = 80;
const ZONE_ARROW_SLOW: u16 = 81;
const ZONE_ARROW_FAST: u16 = 82;
const EVENT_ZONE_81: i32 = 15;
const EVENT_ZONE_82: i32 = 16;
const BIRD: i16 = 11;
const EVENT_LAVA: i32 = 32;
const EVENT_TOO_FAST: i32 = 33;
const DISMOUNT_SPEED: i8 = 15;
const DISMOUNT_STEP: u16 = 14;
const DISMOUNT_CHECK: u16 = 10;
const WEAPON_BOW: i8 = 4;
const WEAPON_WAND: i8 = 5;

/// Hunger above this makes the hero stumble sideways now and then.
const STUMBLE_HUNGER: i16 = 120;
const NOT_WALKING: i16 = 9;
