//! Keys and the hero's controls (from csrc/fmain.c): `handle_key` reacts to one key code from the
//! key buffer (menus, direction keys, the fight key, debug keys) and `update_hero` turns the
//! controls into the hero's state each frame.

use super::actors::{move_extent, proxcheck, rescue};
use super::asm_text::{event, prq};
use super::encounters::load_carrier;
use super::input::{forget_last_menu, fta_pia, handler_qualifier};
use super::menus::{gomenu, propt, set_options};
use super::options::do_option;
use super::random::{fta_rand, rand4, rnd};
use super::revive::revive;
use super::types::Letter;
use super::menus::{print_options};
use super::game_tables::{letter_list, menus, oldir};
use super::globals::{active_carrier, actors_on_screen, anim_list, anix, battleflag, cheat1, cmode, daynight, fiery_death, freeze_timer, goodfairy, hero_x, hero_y, hit as menu_hit, hitgo, hunger, keydir, keyfight, light_timer, luck, map_x, map_y, real_options, riding, secret_timer, stuff, viewstatus, xtype};


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

/// Number of shortcut letters in `letter_list`.
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
        if viewstatus != 0 && notpause {
            if key & RELEASED == 0 {
                viewstatus = VIEW_REDRAW;
            }
        } else if anim_list[0].state == DEAD {
            // ignored while dead
        } else if (FIRST_DIRECTION_KEY..=LAST_DIRECTION_KEY).contains(&key) {
            keydir = key as i16;
        } else if (key & CODE_MASK) as i16 == keydir {
            keydir = 0;
        } else if key == FIGHT_KEY {
            keyfight = 1;
        } else if key & CODE_MASK == FIGHT_KEY {
            keyfight = 0;
        } else if cheat_items(key) {
            // handled
        } else if key & CODE_MASK >= FIRST_MENU_CLICK {
            menu_click(key, notpause);
        } else if cheat_move(key) {
            // handled
        } else if cmode == KEYS && key & RELEASED == 0 {
            if (b'1' as i8..=b'6' as i8).contains(&key) {
                do_option((key - b'1' as i8 + 5) as i16);
            } else {
                gomenu(ITEMS);
            }
        } else if key == b' ' as i8 || notpause {
            menu_letter(key);
        }
    }
}

/// Debug keys that give the hero a carrier or items. Returns true if the key was one.
unsafe fn cheat_items(key: i8) -> bool {
    unsafe {
        if cheat1 == 0 {
            return false;
        }
        if key == CHEAT_CARRIER {
            if active_carrier == CARRIER_BIRD {
                *stuff.add(RAFT_STONE_SLOT) = 1;
            }
            move_extent(
                0,
                (hero_x as i16).wrapping_add(CARRIER_OFFSET),
                (hero_y as i16).wrapping_add(CARRIER_OFFSET),
            );
            load_carrier(CARRIER_BIRD);
            true
        } else if key == CHEAT_STUFF {
            let slot = rnd(GOLDBASE) as usize;
            *stuff.add(slot) = (*stuff.add(slot)).wrapping_add(3);
            set_options();
            *stuff.add(VIAL_SLOT) = 0;
            true
        } else {
            false
        }
    }
}

/// Debug keys that rescue, redraw or move the hero about. Returns true if the key was one.
unsafe fn cheat_move(key: i8) -> bool {
    unsafe {
        if cheat1 == 0 {
            return false;
        }
        let hero = &raw mut anim_list[0];
        match key {
            CHEAT_RESCUE => rescue(),
            CHEAT_PRQ_2 => prq(2),
            CHEAT_PRQ_3 => prq(3),
            CHEAT_TIME => daynight = daynight.wrapping_add(CHEAT_TIME_STEP),
            CHEAT_NORTH => {
                (*hero).abs_y = (*hero).abs_y.wrapping_sub(CHEAT_STEP_Y);
                map_y = map_y.wrapping_sub(CHEAT_STEP_Y);
            }
            CHEAT_SOUTH => {
                (*hero).abs_y = (*hero).abs_y.wrapping_add(CHEAT_STEP_Y);
                map_y = map_y.wrapping_add(CHEAT_STEP_Y);
            }
            CHEAT_EAST => {
                (*hero).abs_x = (*hero).abs_x.wrapping_add(CHEAT_STEP_X);
                map_x = map_x.wrapping_add(CHEAT_STEP_X);
            }
            CHEAT_WEST => {
                (*hero).abs_x = (*hero).abs_x.wrapping_sub(CHEAT_STEP_X);
                map_x = map_x.wrapping_sub(CHEAT_STEP_X);
            }
            _ => return false,
        }
        true
    }
}

/// A click on a menu slot (the mouse sends 'a' + slot for the button going down, and the same
/// with bit 7 set for it coming up).
unsafe fn menu_click(key: i8, notpause: bool) {
    unsafe {
        let slot = ((key & CODE_MASK) - FIRST_MENU_CLICK) as i16;
        menu_hit = *(&raw const real_options).cast::<i8>().offset(slot as isize);
        let hit = menu_hit as i16;
        let menu = &raw mut menus[cmode as usize];
        let valid = hit >= 0 && hit < (*menu).num as i16;
        if key & RELEASED != 0 {
            if valid {
                propt(slot, 0);
            }
            return;
        }
        hitgo = 1;
        if !valid {
            return;
        }
        let entry = &raw mut (*menu).enabled[hit as usize];
        let kind = *entry as i32 & TYPE_MASK;
        if kind != TYPE_IMMEDIATE {
            forget_last_menu();
        }
        if kind == TYPE_TOGGLE {
            *entry ^= SELECTED;
            propt(slot, (*entry & SELECTED) as i16);
            do_option(hit);
        } else if kind == TYPE_IMMEDIATE {
            propt(slot, 1);
            do_option(hit);
        } else if kind == TYPE_RADIO {
            *entry |= SELECTED;
            propt(slot, 1);
            do_option(hit);
        } else if !notpause {
            // paused: only the menu types above work
        } else if kind == 0 && hit < 5 {
            cmode = hit;
            prq(5);
        } else {
            propt(slot, (*entry & SELECTED) as i16);
        }
    }
}

/// A keyboard shortcut for a menu entry. Letters only work for the Save/Exit menu while it is open.
unsafe fn menu_letter(key: i8) {
    unsafe {
        for i in 0..LETTER_COUNT {
            let shortcut = &*(&raw const letter_list).cast::<Letter>().add(i);
            if shortcut.letter != key {
                continue;
            }
            if shortcut.menu as i16 == SAVEX && cmode != SAVEX {
                break;
            }
            cmode = shortcut.menu as i16;
            menu_hit = shortcut.choice;
            let entry = &raw mut menus[cmode as usize].enabled[menu_hit as usize];
            hitgo = *entry & DISPLAYED;
            if *entry as i32 & TYPE_MASK == TYPE_TOGGLE {
                *entry ^= SELECTED;
            }
            do_option(menu_hit as i16);
            print_options();
            break;
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

/// Counts the timers down and sets the hero's state from the controls: the good fairy when dead
/// or falling, shooting, riding, fighting or walking. `cycle` is the main loop's frame counter.
pub fn update_hero(cycle: i16) {
    unsafe {
        count_down(&raw mut light_timer);
        count_down(&raw mut secret_timer);
        count_down(&raw mut freeze_timer);

        fiery_death =
            (map_x > LAVA_X.0 && map_x < LAVA_X.1 && map_y > LAVA_Y.0 && map_y < LAVA_Y.1) as i8;

        let mut state = anim_list[0].state;
        if state == DEAD || state == FALL {
            state = fairy_or_revive(state, cycle);
        } else if state == DYING || state == SINK || state == SLEEP {
            // nothing to do
        } else if handler_qualifier() & RIGHT_BUTTON != 0
            || keyfight != 0
            || (fta_pia & FIRE_BUTTON) == 0
        {
            state = fighting_state(state);
        } else if state == SHOOT1 {
            state = SHOOT3; // release the arrow
        } else {
            state = walking_state(state);
        }
        anim_list[0].state = state;
    }
}

unsafe fn count_down(timer: *mut i16) {
    unsafe {
        if *timer != 0 {
            *timer -= 1;
        }
    }
}

/// The hero is dead or falling: the fairy flies in, then the hero revives (or, with no luck left,
/// the next brother takes over). Returns the hero's new state.
unsafe fn fairy_or_revive(state: i8, cycle: i16) -> i8 {
    unsafe {
        if goodfairy == 1 {
            revive(0);
            return STILL;
        }
        goodfairy = goodfairy.wrapping_sub(1);
        if goodfairy < FAIRY_RESURRECT {
            // the resurrection glow
        } else if luck < 1 && goodfairy < FAIRY_GIVES_UP {
            revive(1);
            return STILL;
        } else if anim_list[0].state == FALL && goodfairy < FAIRY_GIVES_UP {
            revive(0);
            return STILL;
        } else if goodfairy < FAIRY_ARRIVES {
            let fairy = &raw mut anim_list[FAIRY_SLOT];
            anix = NO_ACTORS_LOADED;
            (*fairy).abs_x = hero_x
                .wrapping_add(goodfairy as u16 * FAIRY_SPEED)
                .wrapping_sub(FAIRY_X_OFFSET);
            (*fairy).abs_y = hero_y;
            (*fairy).kind = OBJECTS;
            (*fairy).index = FAIRY_IMAGE + (cycle & 1) as i8;
            (*fairy).state = STILL;
            (*fairy).weapon = 0;
            (*fairy).environ = 0;
            (*fairy).race = FAIRY_RACE;
            actors_on_screen = 1;
            battleflag = 0;
        }
        state
    }
}

/// The fight button is down: shoot in a shooting zone, dismount, or fight with the weapon held.
unsafe fn fighting_state(state: i8) -> i8 {
    unsafe {
        let hero = &raw mut anim_list[0];
        let (dif_x, dif_y) = ((*hero).vel_x, (*hero).vel_y);
        if xtype > SHOOTING_ZONE {
            if state != SHOOT1 && xtype == ZONE_ARROW_SLOW {
                event(EVENT_ZONE_81);
            }
            if state != SHOOT1 && xtype == ZONE_ARROW_FAST {
                event(EVENT_ZONE_82);
            }
            return SHOOT1;
        }
        if riding == BIRD {
            if fiery_death != 0 {
                event(EVENT_LAVA);
            } else if dif_x > -DISMOUNT_SPEED
                && dif_x < DISMOUNT_SPEED
                && dif_y > -DISMOUNT_SPEED
                && dif_y < DISMOUNT_SPEED
            {
                let ytest = (*hero).abs_y.wrapping_sub(DISMOUNT_STEP);
                if proxcheck((*hero).abs_x as i16, ytest as i16, 0) == 0
                    && proxcheck(
                        (*hero).abs_x as i16,
                        ytest.wrapping_add(DISMOUNT_CHECK) as i16,
                        0,
                    ) == 0
                {
                    riding = 0;
                    (*hero).abs_y = ytest;
                }
            } else {
                event(EVENT_TOO_FAST);
            }
            return state;
        }
        if (oldir as i16) < NOT_WALKING {
            (*hero).facing = oldir as i8;
        }
        if state >= WALKING {
            return match (*hero).weapon {
                WEAPON_BOW => SHOOT1,
                WEAPON_WAND => state.max(SHOOT1),
                _ => FIGHTING,
            };
        }
        state
    }
}

/// No fight button: walk the way the controls point (stumbling when starving), or stand still.
unsafe fn walking_state(state: i8) -> i8 {
    unsafe {
        if oldir >= NOT_WALKING {
            return STILL;
        }
        if hunger > STUMBLE_HUNGER && rand4() == 0 {
            oldir = if fta_rand() & 1 != 0 {
                (oldir + 1) & 7
            } else {
                (oldir - 1) & 7
            };
        }
        anim_list[0].facing = oldir as i8;
        if handler_qualifier() & LEFT_BUTTON != 0 || keydir != 0 {
            return WALKING;
        }
        state
    }
}
