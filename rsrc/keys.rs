//! Keys and the hero's controls (from csrc/fmain.c): `handle_key` reacts to one key code from the
//! key buffer (menus, direction keys, the fight key, debug keys) and `update_hero` turns the
//! controls into the hero's state each frame.

use super::actors::{blocked_at, center_extent_on, rescue_princess};
use super::asm_text::{queue_print, show_event};
use super::encounters::load_carrier;
use super::game_tables::{compass_lit_direction, menu_shortcuts, menus};
use super::input::{forget_last_menu, fta_pia, mouse_buttons};
use super::menus::print_options;
use super::menus::{go_to_menu, print_menu_entry, update_menu_availability};
use super::options::choose_menu_entry;
use super::random::{fta_rand, rand4, rnd};
use super::revive::revive;
use super::state::{the_actors, the_hero, the_screen, the_ui, the_world};
use super::types::Letter;

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
            if key & RELEASED == 0 {
                the_screen.view_status = VIEW_REDRAW;
            }
        } else if the_actors.list[0].state == DEAD {
            // ignored while dead
        } else if (FIRST_DIRECTION_KEY..=LAST_DIRECTION_KEY).contains(&key) {
            the_ui.held_direction = key as i16;
        } else if (key & CODE_MASK) as i16 == the_ui.held_direction {
            the_ui.held_direction = 0;
        } else if key == FIGHT_KEY {
            the_ui.fight_key_held = 1;
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
                choose_menu_entry((key - b'1' as i8 + 5) as i16);
            } else {
                go_to_menu(ITEMS);
            }
        } else if key == b' ' as i8 || notpause {
            menu_letter(key);
        }
    }
}

/// Debug keys that give the hero a carrier or items. Returns true if the key was one.
unsafe fn cheat_items(key: i8) -> bool {
    unsafe {
        if the_ui.cheat_enabled == 0 {
            return false;
        }
        if key == CHEAT_CARRIER {
            if the_actors.active_carrier == CARRIER_BIRD {
                *the_hero.stuff.add(RAFT_STONE_SLOT) = 1;
            }
            center_extent_on(
                0,
                (the_hero.x as i16).wrapping_add(CARRIER_OFFSET),
                (the_hero.y as i16).wrapping_add(CARRIER_OFFSET),
            );
            load_carrier(CARRIER_BIRD);
            true
        } else if key == CHEAT_STUFF {
            let slot = rnd(GOLDBASE) as usize;
            *the_hero.stuff.add(slot) = (*the_hero.stuff.add(slot)).wrapping_add(3);
            update_menu_availability();
            *the_hero.stuff.add(VIAL_SLOT) = 0;
            true
        } else {
            false
        }
    }
}

/// Debug keys that rescue, redraw or move the hero about. Returns true if the key was one.
unsafe fn cheat_move(key: i8) -> bool {
    unsafe {
        if the_ui.cheat_enabled == 0 {
            return false;
        }
        let hero = &raw mut the_actors.list[0];
        match key {
            CHEAT_RESCUE => rescue_princess(),
            CHEAT_PRQ_2 => queue_print(2),
            CHEAT_PRQ_3 => queue_print(3),
            CHEAT_TIME => the_world.day_clock = the_world.day_clock.wrapping_add(CHEAT_TIME_STEP),
            CHEAT_NORTH => {
                (*hero).abs_y = (*hero).abs_y.wrapping_sub(CHEAT_STEP_Y);
                the_world.map_y = the_world.map_y.wrapping_sub(CHEAT_STEP_Y);
            }
            CHEAT_SOUTH => {
                (*hero).abs_y = (*hero).abs_y.wrapping_add(CHEAT_STEP_Y);
                the_world.map_y = the_world.map_y.wrapping_add(CHEAT_STEP_Y);
            }
            CHEAT_EAST => {
                (*hero).abs_x = (*hero).abs_x.wrapping_add(CHEAT_STEP_X);
                the_world.map_x = the_world.map_x.wrapping_add(CHEAT_STEP_X);
            }
            CHEAT_WEST => {
                (*hero).abs_x = (*hero).abs_x.wrapping_sub(CHEAT_STEP_X);
                the_world.map_x = the_world.map_x.wrapping_sub(CHEAT_STEP_X);
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
        the_ui.chosen_entry = *(&raw const the_ui.slot_entries)
            .cast::<i8>()
            .offset(slot as isize);
        let chosen_entry = the_ui.chosen_entry as i16;
        let menu = &raw mut menus[the_ui.current_menu as usize];
        let valid = chosen_entry >= 0 && chosen_entry < (*menu).num as i16;
        if key & RELEASED != 0 {
            if valid {
                print_menu_entry(slot, 0);
            }
            return;
        }
        the_ui.menu_hit_enabled = 1;
        if !valid {
            return;
        }
        let entry = &raw mut (*menu).enabled[chosen_entry as usize];
        let kind = *entry as i32 & TYPE_MASK;
        if kind != TYPE_IMMEDIATE {
            forget_last_menu();
        }
        if kind == TYPE_TOGGLE {
            *entry ^= SELECTED;
            print_menu_entry(slot, (*entry & SELECTED) as i16);
            choose_menu_entry(chosen_entry);
        } else if kind == TYPE_IMMEDIATE {
            print_menu_entry(slot, 1);
            choose_menu_entry(chosen_entry);
        } else if kind == TYPE_RADIO {
            *entry |= SELECTED;
            print_menu_entry(slot, 1);
            choose_menu_entry(chosen_entry);
        } else if !notpause {
            // paused: only the menu types above work
        } else if kind == 0 && chosen_entry < 5 {
            the_ui.current_menu = chosen_entry;
            queue_print(5);
        } else {
            print_menu_entry(slot, (*entry & SELECTED) as i16);
        }
    }
}

/// A keyboard shortcut for a menu entry. Letters only work for the Save/Exit menu while it is open.
unsafe fn menu_letter(key: i8) {
    unsafe {
        for i in 0..LETTER_COUNT {
            let shortcut = &*(&raw const menu_shortcuts).cast::<Letter>().add(i);
            if shortcut.letter != key {
                continue;
            }
            if shortcut.menu as i16 == SAVEX && the_ui.current_menu != SAVEX {
                break;
            }
            the_ui.current_menu = shortcut.menu as i16;
            the_ui.chosen_entry = shortcut.choice;
            let entry =
                &raw mut menus[the_ui.current_menu as usize].enabled[the_ui.chosen_entry as usize];
            the_ui.menu_hit_enabled = *entry & DISPLAYED;
            if *entry as i32 & TYPE_MASK == TYPE_TOGGLE {
                *entry ^= SELECTED;
            }
            choose_menu_entry(the_ui.chosen_entry as i16);
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
        count_down(&raw mut the_world.light_timer);
        count_down(&raw mut the_world.secret_timer);
        count_down(&raw mut the_world.freeze_timer);

        the_actors.fiery_death = (the_world.map_x > LAVA_X.0
            && the_world.map_x < LAVA_X.1
            && the_world.map_y > LAVA_Y.0
            && the_world.map_y < LAVA_Y.1) as i8;

        let mut state = the_actors.list[0].state;
        if state == DEAD || state == FALL {
            state = fairy_or_revive(state, cycle);
        } else if state == DYING || state == SINK || state == SLEEP {
            // nothing to do
        } else if mouse_buttons() & RIGHT_BUTTON != 0
            || the_ui.fight_key_held != 0
            || (fta_pia & FIRE_BUTTON) == 0
        {
            state = fighting_state(state);
        } else if state == SHOOT1 {
            state = SHOOT3; // release the arrow
        } else {
            state = walking_state(state);
        }
        the_actors.list[0].state = state;
    }
}

unsafe fn count_down(timer: *mut i16) {
    unsafe {
        *timer = timer_step(*timer);
    }
}

/// A light, secret or freeze timer after one frame: it counts down to 0 and stays there.
fn timer_step(timer: i16) -> i16 {
    if timer != 0 { timer - 1 } else { timer }
}

/// The hero is dead or falling: the fairy flies in, then the hero revives (or, with no luck left,
/// the next brother takes over). Returns the hero's new state.
unsafe fn fairy_or_revive(state: i8, cycle: i16) -> i8 {
    unsafe {
        let (timer, action) = fairy_step(
            the_hero.good_fairy_timer,
            the_hero.luck,
            the_actors.list[0].state == FALL,
        );
        the_hero.good_fairy_timer = timer;
        match action {
            FairyAction::Revive => {
                revive(0);
                return STILL;
            }
            FairyAction::NextBrother => {
                revive(1);
                return STILL;
            }
            FairyAction::Wait => {}
            FairyAction::FlyIn => {
                let fairy = &raw mut the_actors.list[FAIRY_SLOT];
                the_actors.active = NO_ACTORS_LOADED;
                (*fairy).abs_x = the_hero
                    .x
                    .wrapping_add(the_hero.good_fairy_timer as u16 * FAIRY_SPEED)
                    .wrapping_sub(FAIRY_X_OFFSET);
                (*fairy).abs_y = the_hero.y;
                (*fairy).kind = OBJECTS;
                (*fairy).index = FAIRY_IMAGE + (cycle & 1) as i8;
                (*fairy).state = STILL;
                (*fairy).weapon = 0;
                (*fairy).environ = 0;
                (*fairy).race = FAIRY_RACE;
                the_actors.on_screen = 1;
                the_actors.battle_flag = 0;
            }
        }
        state
    }
}

/// What the good fairy does this frame.
#[derive(Debug, PartialEq)]
enum FairyAction {
    /// The hero gets up where they are.
    Revive,
    /// No luck is left: the next brother takes over.
    NextBrother,
    /// The fairy flies in toward the hero.
    FlyIn,
    /// Nothing yet (or the resurrection glow).
    Wait,
}

/// One step of the fairy countdown for a dead or fallen hero: returns the new timer value and
/// what to do. At timer 1 the hero is revived; below `FAIRY_RESURRECT` it is just the glow;
/// below `FAIRY_GIVES_UP` the hero is revived (a fallen one) or replaced (no luck); below
/// `FAIRY_ARRIVES` the fairy flies in.
fn fairy_step(timer: u8, luck: i16, falling: bool) -> (u8, FairyAction) {
    if timer == 1 {
        return (timer, FairyAction::Revive);
    }
    let timer = timer.wrapping_sub(1);
    let action = if timer < FAIRY_RESURRECT {
        FairyAction::Wait
    } else if luck < 1 && timer < FAIRY_GIVES_UP {
        FairyAction::NextBrother
    } else if falling && timer < FAIRY_GIVES_UP {
        FairyAction::Revive
    } else if timer < FAIRY_ARRIVES {
        FairyAction::FlyIn
    } else {
        FairyAction::Wait
    };
    (timer, action)
}

/// The fight button is down: shoot in a shooting zone, dismount, or fight with the weapon held.
unsafe fn fighting_state(state: i8) -> i8 {
    unsafe {
        let hero = &raw mut the_actors.list[0];
        let (dif_x, dif_y) = ((*hero).vel_x, (*hero).vel_y);
        if the_world.extent_type > SHOOTING_ZONE {
            if state != SHOOT1 && the_world.extent_type == ZONE_ARROW_SLOW {
                show_event(EVENT_ZONE_81);
            }
            if state != SHOOT1 && the_world.extent_type == ZONE_ARROW_FAST {
                show_event(EVENT_ZONE_82);
            }
            return SHOOT1;
        }
        if the_hero.riding == BIRD {
            if the_actors.fiery_death != 0 {
                show_event(EVENT_LAVA);
            } else if dif_x > -DISMOUNT_SPEED
                && dif_x < DISMOUNT_SPEED
                && dif_y > -DISMOUNT_SPEED
                && dif_y < DISMOUNT_SPEED
            {
                let ytest = (*hero).abs_y.wrapping_sub(DISMOUNT_STEP);
                if blocked_at((*hero).abs_x as i16, ytest as i16, 0) == 0
                    && blocked_at(
                        (*hero).abs_x as i16,
                        ytest.wrapping_add(DISMOUNT_CHECK) as i16,
                        0,
                    ) == 0
                {
                    the_hero.riding = 0;
                    (*hero).abs_y = ytest;
                }
            } else {
                show_event(EVENT_TOO_FAST);
            }
            return state;
        }
        if compass_lit_direction < NOT_WALKING {
            (*hero).facing = compass_lit_direction as i8;
        }
        if state >= WALKING {
            return fighting_state_for(state, (*hero).weapon);
        }
        state
    }
}

/// The state when the fight button is down and the hero is not already fighting: a bow shoots, a
/// wand keeps shooting, anything else fights.
fn fighting_state_for(state: i8, weapon: i8) -> i8 {
    match weapon {
        WEAPON_BOW => SHOOT1,
        WEAPON_WAND => state.max(SHOOT1),
        _ => FIGHTING,
    }
}

/// No fight button: walk the way the controls point (stumbling when starving), or stand still.
unsafe fn walking_state(state: i8) -> i8 {
    unsafe {
        if compass_lit_direction >= NOT_WALKING {
            return STILL;
        }
        compass_lit_direction = stumble(
            compass_lit_direction,
            the_hero.hunger,
            || rand4() == 0,
            || fta_rand() & 1 != 0,
        );
        the_actors.list[0].facing = compass_lit_direction as i8;
        if mouse_buttons() & LEFT_BUTTON != 0 || the_ui.held_direction != 0 {
            return WALKING;
        }
        state
    }
}

/// A starving hero sometimes stumbles one step off the direction walked. `trips` and `turns_right`
/// are random draws, taken in that order and only when needed.
fn stumble(
    direction: i16,
    hunger: i16,
    trips: impl FnOnce() -> bool,
    turns_right: impl FnOnce() -> bool,
) -> i16 {
    if hunger > STUMBLE_HUNGER && trips() {
        if turns_right() {
            (direction + 1) & 7
        } else {
            (direction - 1) & 7
        }
    } else {
        direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_starving_hero_stumbles() {
        let mut drawn = 0;
        let dir = stumble(
            3,
            STUMBLE_HUNGER,
            || {
                drawn += 1;
                true
            },
            || true,
        );
        assert_eq!((dir, drawn), (3, 0));
        assert_eq!(stumble(3, STUMBLE_HUNGER + 1, || false, || true), 3);
        assert_eq!(stumble(3, STUMBLE_HUNGER + 1, || true, || true), 4);
        assert_eq!(stumble(3, STUMBLE_HUNGER + 1, || true, || false), 2);
        // the compass wraps around
        assert_eq!(stumble(7, STUMBLE_HUNGER + 1, || true, || true), 0);
        assert_eq!(stumble(0, STUMBLE_HUNGER + 1, || true, || false), 7);
    }

    #[test]
    fn the_turn_is_only_drawn_after_a_trip() {
        let mut drawn = 0;
        stumble(
            3,
            STUMBLE_HUNGER + 1,
            || false,
            || {
                drawn += 1;
                true
            },
        );
        assert_eq!(drawn, 0);
    }

    #[test]
    fn timers_count_down_and_stop_at_zero() {
        assert_eq!(timer_step(3), 2);
        assert_eq!(timer_step(1), 0);
        assert_eq!(timer_step(0), 0);
        let (mut t, mut active) = (5, 0);
        while t != 0 {
            active += 1;
            t = timer_step(t);
        }
        assert_eq!(active, 5);
    }

    #[test]
    fn the_hero_revives_when_the_countdown_reaches_one() {
        assert_eq!(fairy_step(1, 50, false), (1, FairyAction::Revive));
    }

    #[test]
    fn the_fairy_flies_in_once_the_timer_is_low_enough() {
        assert_eq!(
            fairy_step(FAIRY_ARRIVES + 5, 50, false).1,
            FairyAction::Wait
        );
        assert_eq!(
            fairy_step(FAIRY_ARRIVES, 50, false),
            (FAIRY_ARRIVES - 1, FairyAction::FlyIn)
        );
    }

    #[test]
    fn a_fallen_hero_is_revived_before_the_fairy_arrives() {
        assert_eq!(fairy_step(FAIRY_GIVES_UP, 50, true).1, FairyAction::Revive);
    }

    #[test]
    fn with_no_luck_the_next_brother_takes_over() {
        assert_eq!(
            fairy_step(FAIRY_GIVES_UP, 0, false).1,
            FairyAction::NextBrother
        );
        // having luck left, a dead hero waits for the fairy instead
        assert_eq!(fairy_step(FAIRY_GIVES_UP, 1, false).1, FairyAction::Wait);
    }

    #[test]
    fn the_last_moments_are_only_a_glow() {
        assert_eq!(fairy_step(FAIRY_RESURRECT, 0, true).1, FairyAction::Wait);
    }

    #[test]
    fn weapons_decide_how_the_hero_fights() {
        assert_eq!(fighting_state_for(WALKING, WEAPON_BOW), SHOOT1);
        assert_eq!(fighting_state_for(WALKING, WEAPON_WAND), SHOOT1);
        assert_eq!(fighting_state_for(SHOOT1 + 3, WEAPON_WAND), SHOOT1 + 3);
        assert_eq!(fighting_state_for(WALKING, 1), FIGHTING);
    }
}
