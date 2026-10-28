//! What the hero is doing each frame: walking, fighting or standing, and the timers that run down.

use super::fairy::fairy_revival;
use super::{
    BIRD, DEAD, DISMOUNT_CHECK, DISMOUNT_SPEED, DISMOUNT_STEP, DYING, EVENT_LAVA, EVENT_TOO_FAST,
    EVENT_ZONE_81, EVENT_ZONE_82, FACING_MASK, FALL, FIGHTING, LAVA_X, LAVA_Y, LEFT_BUTTON,
    NOT_WALKING, RIGHT_BUTTON, SHOOT1, SHOOT3, SHOOTING_ZONE, SINK, SLEEP, STILL, STUMBLE_HUNGER,
    WALKING, WEAPON_BOW, WEAPON_WAND, ZONE_ARROW_FAST, ZONE_ARROW_SLOW,
};
use crate::game::data::state::Game;
use crate::game::data::state::Video;

use crate::game::actors::interaction::blocked_at;
use crate::game::data::random::fta_rand;
use crate::game::data::state::ActorRef;
use crate::game::input::handler::{joystick_fire, mouse_buttons};
use crate::game::ui::text::show_event;

/// Counts the timers down and sets the hero's state from the controls: the good fairy when dead
/// or falling, shooting, riding, fighting or walking. `cycle` is the main loop's frame counter.
pub fn update_hero(game: &mut Game, video: &mut Video, cycle: i16) {
    game.world.light_timer = timer_step(game.world.light_timer);
    game.world.secret_timer = timer_step(game.world.secret_timer);
    game.world.freeze_timer = timer_step(game.world.freeze_timer);

    game.actors.fiery_death = (game.world.map_x > LAVA_X.0
        && game.world.map_x < LAVA_X.1
        && game.world.map_y > LAVA_Y.0
        && game.world.map_y < LAVA_Y.1) as i8;

    let mut state = game.actors.list[0].state;

    if state == DEAD || state == FALL {
        state = fairy_revival(game, video, state, cycle);
    } else if state == DYING || state == SINK || state == SLEEP {
        // nothing to do
    } else if mouse_buttons() & RIGHT_BUTTON != 0 || game.ui.fight_key_held != 0 || joystick_fire()
    {
        state = fighting_state(game, video, state);
    } else if state == SHOOT1 {
        state = SHOOT3; // release the arrow
    } else {
        state = walking_state(game, state);
    }

    game.actors.list[0].state = state;
}

/// A light, secret or freeze timer after one frame: it counts down to 0 and stays there.
pub(super) fn timer_step(timer: i16) -> i16 {
    if timer != 0 { timer - 1 } else { timer }
}

/// The fight button is down: shoot in a shooting zone, dismount, or fight with the weapon held.
pub(super) fn fighting_state(game: &mut Game, video: &mut Video, state: i8) -> i8 {
    let hero = ActorRef::new(0);

    if game.world.extent_type > SHOOTING_ZONE {
        return enter_shooting_zone(game, video, state);
    }

    if game.hero.riding == BIRD {
        try_to_dismount(game, video, hero);

        return state;
    }

    if game.ui.compass_lit_direction < NOT_WALKING {
        game.actor_mut(hero).facing = game.ui.compass_lit_direction as i8;
    }

    if state >= WALKING {
        return fighting_state_for(state, game.actor(hero).weapon);
    }

    state
}

/// In a shooting zone the hero shoots; the two special zones say so the first time.
fn enter_shooting_zone(game: &mut Game, video: &mut Video, state: i8) -> i8 {
    if state != SHOOT1 && game.world.extent_type == ZONE_ARROW_SLOW {
        show_event(game, video, EVENT_ZONE_81);
    }

    if state != SHOOT1 && game.world.extent_type == ZONE_ARROW_FAST {
        show_event(game, video, EVENT_ZONE_82);
    }

    SHOOT1
}

/// On the bird, the fight button tries to land: not over lava, not when moving too fast, and only
/// where there is room to stand.
fn try_to_dismount(game: &mut Game, video: &mut Video, hero: ActorRef) {
    let (dif_x, dif_y) = (game.actor(hero).vel_x, game.actor(hero).vel_y);

    if game.actors.fiery_death != 0 {
        show_event(game, video, EVENT_LAVA);
    } else if dif_x > -DISMOUNT_SPEED
        && dif_x < DISMOUNT_SPEED
        && dif_y > -DISMOUNT_SPEED
        && dif_y < DISMOUNT_SPEED
    {
        let ytest = game.actor(hero).abs_y.wrapping_sub(DISMOUNT_STEP);

        if blocked_at(game, game.actor(hero).abs_x as i16, ytest as i16, 0) == 0
            && blocked_at(
                game,
                game.actor(hero).abs_x as i16,
                ytest.wrapping_add(DISMOUNT_CHECK) as i16,
                0,
            ) == 0
        {
            game.hero.riding = 0;
            game.actor_mut(hero).abs_y = ytest;
        }
    } else {
        show_event(game, video, EVENT_TOO_FAST);
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
pub(super) fn walking_state(game: &mut Game, state: i8) -> i8 {
    if game.ui.compass_lit_direction >= NOT_WALKING {
        return STILL;
    }

    let rng = &mut game.rng;
    game.ui.compass_lit_direction = stumble(game.ui.compass_lit_direction, game.hero.hunger, |n| {
        fta_rand(rng) & (n - 1)
    });

    game.actors.list[0].facing = game.ui.compass_lit_direction as i8;

    if mouse_buttons() & LEFT_BUTTON != 0 || game.ui.held_direction != 0 {
        return WALKING;
    }

    state
}

/// A starving hero sometimes stumbles one step off the direction walked. `dice(n)` is a random number
/// from 0 to n - 1: one draw of 4 says whether the hero trips (0), and only then a draw of 2 says
/// which way (1 is right).
pub(super) fn stumble(direction: i16, hunger: i16, mut dice: impl FnMut(i32) -> i32) -> i16 {
    if hunger > STUMBLE_HUNGER && dice(4) == 0 {
        if dice(2) != 0 {
            (direction + 1) & FACING_MASK
        } else {
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

    /// Dice that give a trip (or not) and then a turn to the right (or not).
    fn dice(trips: bool, right: bool) -> impl FnMut(i32) -> i32 {
        move |n| match n {
            4 => !trips as i32,
            _ => right as i32,
        }
    }

    #[test]
    fn only_a_starving_hero_stumbles() {
        let mut drawn = 0;
        let dir = stumble(3, STUMBLE_HUNGER, |_| {
            drawn += 1;

            0
        });

        assert_eq!((dir, drawn), (3, 0));
        assert_eq!(stumble(3, STUMBLE_HUNGER + 1, dice(false, true)), 3);
        assert_eq!(stumble(3, STUMBLE_HUNGER + 1, dice(true, true)), 4);
        assert_eq!(stumble(3, STUMBLE_HUNGER + 1, dice(true, false)), 2);

        // the compass wraps around
        assert_eq!(stumble(7, STUMBLE_HUNGER + 1, dice(true, true)), 0);
        assert_eq!(stumble(0, STUMBLE_HUNGER + 1, dice(true, false)), 7);
    }

    #[test]
    fn the_turn_is_only_drawn_after_a_trip() {
        let mut turns = 0;
        stumble(3, STUMBLE_HUNGER + 1, |n| {
            if n == 2 {
                turns += 1;
            }

            1
        });

        assert_eq!(turns, 0);
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
