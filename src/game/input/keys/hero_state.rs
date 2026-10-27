//! What the hero is doing each frame: walking, fighting or standing, and the timers that run down.

use super::fairy::fairy_or_revive;
use super::{
    BIRD, DEAD, DISMOUNT_CHECK, DISMOUNT_SPEED, DISMOUNT_STEP, DYING, EVENT_LAVA, EVENT_TOO_FAST,
    EVENT_ZONE_81, EVENT_ZONE_82, FACING_MASK, FALL, FIGHTING, FIRE_BUTTON, LAVA_X, LAVA_Y,
    LEFT_BUTTON, NOT_WALKING, RIGHT_BUTTON, SHOOT1, SHOOT3, SHOOTING_ZONE, SINK, SLEEP, STILL,
    STUMBLE_HUNGER, WALKING, WEAPON_BOW, WEAPON_WAND, ZONE_ARROW_FAST, ZONE_ARROW_SLOW,
};

use crate::game::actors::interaction::blocked_at;
use crate::game::data::random::{fta_rand, rand4};
use crate::game::data::state::{the_actors, the_hero, the_ui, the_world};
use crate::game::data::tables::compass_lit_direction;
use crate::game::data::types::Shape;
use crate::game::input::handler::{fta_pia, mouse_buttons};
use crate::game::ui::text::show_event;

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
            // Masks with FIRE_BUTTON, keeping only the bits that matter here.
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

pub(super) unsafe fn count_down(timer: *mut i16) {
    unsafe {
        *timer = timer_step(*timer);
    }
}

/// A light, secret or freeze timer after one frame: it counts down to 0 and stays there.
pub(super) fn timer_step(timer: i16) -> i16 {
    if timer != 0 { timer - 1 } else { timer }
}

/// The fight button is down: shoot in a shooting zone, dismount, or fight with the weapon held.
pub(super) unsafe fn fighting_state(state: i8) -> i8 {
    unsafe {
        let hero = &raw mut the_actors.list[0];

        if the_world.extent_type > SHOOTING_ZONE {
            return enter_shooting_zone(state);
        }

        if the_hero.riding == BIRD {
            try_to_dismount(hero);

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

/// In a shooting zone the hero shoots; the two special zones say so the first time.
unsafe fn enter_shooting_zone(state: i8) -> i8 {
    unsafe {
        if state != SHOOT1 && the_world.extent_type == ZONE_ARROW_SLOW {
            show_event(EVENT_ZONE_81);
        }

        if state != SHOOT1 && the_world.extent_type == ZONE_ARROW_FAST {
            show_event(EVENT_ZONE_82);
        }

        SHOOT1
    }
}

/// On the bird, the fight button tries to land: not over lava, not when moving too fast, and only
/// where there is room to stand.
unsafe fn try_to_dismount(hero: *mut Shape) {
    unsafe {
        let (dif_x, dif_y) = ((*hero).vel_x, (*hero).vel_y);

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
    }
}

/// The state when the fight button is down and the hero is not already fighting: a bow shoots, a
/// wand keeps shooting, anything else fights.
pub(super) fn fighting_state_for(state: i8, weapon: i8) -> i8 {
    match weapon {
        WEAPON_BOW => SHOOT1,
        WEAPON_WAND => state.max(SHOOT1),
        _ => FIGHTING,
    }
}

/// No fight button: walk the way the controls point (stumbling when starving), or stand still.
pub(super) unsafe fn walking_state(state: i8) -> i8 {
    unsafe {
        if compass_lit_direction >= NOT_WALKING {
            return STILL;
        }

        compass_lit_direction = stumble(
            compass_lit_direction,
            the_hero.hunger,
            || rand4() == 0,
            // Masks off the low bits (a cheap remainder or a flag test).
            || fta_rand() & 1 != 0,
        );

        the_actors.list[0].facing = compass_lit_direction as i8;

        // Masks with LEFT_BUTTON, keeping only the bits that matter here.
        if mouse_buttons() & LEFT_BUTTON != 0 || the_ui.held_direction != 0 {
            return WALKING;
        }

        state
    }
}

/// A starving hero sometimes stumbles one step off the direction walked. `trips` and `turns_right`
/// are random draws, taken in that order and only when needed.
pub(super) fn stumble(
    direction: i16,
    hunger: i16,
    trips: impl FnOnce() -> bool,
    turns_right: impl FnOnce() -> bool,
) -> i16 {
    if hunger > STUMBLE_HUNGER && trips() {
        if turns_right() {
            // Masks with FACING_MASK, keeping only the bits that matter here.
            (direction + 1) & FACING_MASK
        } else {
            // Masks with FACING_MASK, keeping only the bits that matter here.
            (direction - 1) & FACING_MASK
        }
    } else {
        direction
    }
}

#[cfg(test)]
mod tests {
    use super::super::{FIGHTING, SHOOT1, STUMBLE_HUNGER, WALKING, WEAPON_BOW, WEAPON_WAND};
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
    fn weapons_decide_how_the_hero_fights() {
        assert_eq!(fighting_state_for(WALKING, WEAPON_BOW), SHOOT1);
        assert_eq!(fighting_state_for(WALKING, WEAPON_WAND), SHOOT1);
        assert_eq!(fighting_state_for(SHOOT1 + 3, WEAPON_WAND), SHOOT1 + 3);
        assert_eq!(fighting_state_for(WALKING, 1), FIGHTING);
    }
}
