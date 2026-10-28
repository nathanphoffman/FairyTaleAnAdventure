//! What an enemy wants each frame: noticing the hero, choosing a goal and acting on it.

use super::tactics::{choose_tactic, hostile_action};
use super::{BACKUP, BOW_BIT, STILL};
use crate::game::actors::interaction::{report_battle_result, run_tactic};
use crate::game::actors::steering::steer_actor;
use crate::game::data::random::{bitrand, rand2, rand4};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::ui::music::set_mood;
use crate::game::ui::text::queue_print;

// Actor kinds and states (csrc/ftale.h).
const CARRIER: i8 = 5;
const SETFIG: i8 = 4;
const DEAD: i8 = 15;
const FALL: i8 = 22;
const SHOOT1: i8 = 24;
const SHOOT3: i8 = 25;
const ARCHER2: i16 = 4;
const FLEE: i16 = 5;
const STAND: i16 = 6;
const WAIT: i16 = 8;
const FOLLOWER: i16 = 9;

// Tactics.
const FRUST: i16 = 0;

/// A new tactic is rolled one frame in 16.
const NEW_TACTIC_ODDS: i32 = 16;

/// After being frustrated, archers pick one of tactics 2 to 5 and the others 3 or 4.
const ARCHER_FRUSTRATION_TACTIC: i32 = 2;
const MELEE_FRUSTRATION_TACTIC: i32 = 3;
const FOLLOW: i32 = 2;
const SHOOTFRUST: i16 = 9;

/// The first actors are the hero and the raft.
const FIRST_AI_ACTOR: usize = 2;

/// The good fairy is on screen below this countdown; nobody acts then.
const FAIRY_ARRIVES: u8 = 120;

/// An actor within this many pixels (in x and y) of the hero counts as on screen.
const SCREEN_RANGE: i16 = 300;

/// Special encounters (extent_type above this) only fight if they are the encounter's race.
const SPECIAL_XTYPE: u16 = 59;

/// Carriers (the turtle) re-aim every 16 ticks, steering mode 5.
const CARRIER_AIM_MASK: u16 = 15;
const CARRIER_AIM_MODE: i32 = 5;
const NEARLY_DEAD: i16 = 2;
const PRQ_STATS: i32 = 4;
const PRQ_VITALITY: i32 = 7;

/// Decides each enemy's goal and tactic, and starts or ends the battle mood.
pub fn update_enemy_goals(game: &mut Game) {
    game.actors.on_screen = 0;
    game.actors.leader = 0;
    let battle2 = game.actors.battle_flag;
    game.actors.battle_flag = 0;
    let mut i = FIRST_AI_ACTOR;

    while i < game.actors.active as usize {
        if game.hero.good_fairy_timer != 0 && game.hero.good_fairy_timer < FAIRY_ARRIVES {
            break;
        }

        think(game, i, battle2);
        i += 1;
    }

    if battle2 == 0 && game.actors.battle_flag != 0 {
        set_mood(game, 1);
    }

    if game.actors.battle_flag == 0 && battle2 != 0 {
        queue_print(game, PRQ_VITALITY);
        queue_print(game, PRQ_STATS);
        report_battle_result(game);
    }
}

/// One actor's decision. Returns early (skipping the goal update) for carriers, people and
/// anyone out of hit points who is on screen.
fn think(game: &mut Game, i: usize, battle2: i8) {
    let actor = ActorRef::new(i);

    if game.actor(actor).kind == CARRIER {
        aim_carrier(game, i);

        return;
    } else if game.actor(actor).kind == SETFIG {
        return;
    }

    let (xd, yd) = distance_from_hero(game, actor);

    if !notice_actor(game, actor, xd, yd, battle2) {
        return;
    }

    let tactic = game.actor(actor).tactic as i16;
    let new_tactic_roll = bitrand(&mut game.rng, NEW_TACTIC_ODDS - 1) == 0;
    let mode = adjust_mode_for_situation(game, actor, game.actor(actor).goal as i16);
    let mode = act_on_goal(game, i, actor, mode, tactic, (xd, yd), new_tactic_roll);

    game.actor_mut(actor).goal = mode as i8;

    if game.actors.leader == 0 {
        game.actors.leader = i as i16;
    }
}

/// A carrier (raft, turtle, swan) keeps steering toward the hero every few frames.
fn aim_carrier(game: &mut Game, i: usize) {
    if game.world.day_clock & CARRIER_AIM_MASK == 0 {
        steer_actor(
            game,
            i as i32,
            game.hero.x as i32,
            game.hero.y as i32,
            CARRIER_AIM_MODE,
        );
    }
}

/// How far the actor is from the hero along each axis, never negative.
fn distance_from_hero(game: &mut Game, actor: ActorRef) -> (i16, i16) {
    let xd = (game.actor(actor).abs_x as i32 - game.hero.x as i32) as i16;
    let yd = (game.actor(actor).abs_y as i32 - game.hero.y as i32) as i16;

    (xd.wrapping_abs(), yd.wrapping_abs())
}

/// Notes that an actor is on screen, and that a battle is on if it is awake and in view. Returns
/// false for an actor that is on screen but out of hit points (it does nothing more).
fn notice_actor(game: &mut Game, actor: ActorRef, xd: i16, yd: i16, battle2: i8) -> bool {
    if on_screen(xd, yd) {
        game.actors.on_screen = 1;

        if game.actor(actor).vitality < 1 {
            return false;
        }

        if game.actor(actor).visible != 0 || battle2 != 0 {
            game.actors.battle_flag = 1;
        }
    }

    true
}

/// A stuck actor picks a new tactic: archers one of the ranged ones, the others a melee one.
fn try_something_else(game: &mut Game, i: usize, actor: ActorRef) {
    if game.actor(actor).weapon & BOW_BIT != 0 {
        let tactic = rand4(&mut game.rng) + ARCHER_FRUSTRATION_TACTIC;
        run_tactic(game, i as i32, tactic);
    } else {
        let tactic = rand2(&mut game.rng) + MELEE_FRUSTRATION_TACTIC;
        run_tactic(game, i as i32, tactic);
    }
}

/// Carries out the actor's goal for this frame: a stuck actor tries something else, a hostile one
/// picks a tactic now and then and acts on it, the others flee, follow, stand or wait. Returns the
/// goal for next frame, which a hostile actor may change.
#[allow(clippy::too_many_arguments)] // the whole situation of one actor, plus the game state
fn act_on_goal(
    game: &mut Game,
    i: usize,
    actor: ActorRef,
    mut mode: i16,
    mut tactic: i16,
    (xd, yd): (i16, i16),
    mut new_tactic_roll: bool,
) -> i16 {
    if tactic == FRUST || tactic == SHOOTFRUST {
        try_something_else(game, i, actor);
    } else if game.actor(actor).state == SHOOT1 {
        game.actor_mut(actor).state = SHOOT3;
    } else if mode <= ARCHER2 {
        if mode & 2 == 0 {
            new_tactic_roll = rand4(&mut game.rng) == 0;
        }

        if new_tactic_roll {
            (mode, tactic) = choose_tactic(game, actor, mode, xd, yd);
        }

        hostile_action(game, i, actor, mode, tactic, xd, yd);
    } else if mode == FLEE {
        run_tactic(game, i as i32, BACKUP);
    } else if mode == FOLLOWER {
        run_tactic(game, i as i32, FOLLOW);
    } else if mode == STAND {
        steer_actor(game, i as i32, game.hero.x as i32, game.hero.y as i32, 0);
        game.actor_mut(actor).state = STILL;
    } else if mode == WAIT {
        game.actor_mut(actor).state = STILL;
    }

    mode
}

/// Overrides an actor's mode when the situation demands: flee or follow when the hero is down,
/// flee when nearly dead.
fn adjust_mode_for_situation(game: &mut Game, actor: ActorRef, mode: i16) -> i16 {
    adjusted_mode(
        mode,
        game.actors.list[0].state,
        game.actors.leader,
        game.actor(actor).vitality,
        game.world.extent_type,
        game.actor(actor).race,
        game.extents[game.world.extent].v3,
    )
}

/// Whether an actor `xd`, `yd` (absolute) pixels from the hero counts as on screen.
fn on_screen(xd: i16, yd: i16) -> bool {
    xd < SCREEN_RANGE && yd < SCREEN_RANGE
}

/// The mode after the hero's state and the actor's own health are taken into account: everyone
/// flees (or follows the leader) when the hero is dead or falling, the nearly dead flee, and so do
/// actors of a special encounter that are not the zone's own race in the 60+ zones.
#[allow(clippy::too_many_arguments)] // the whole situation of one actor, from plain values
fn adjusted_mode(
    mode: i16,
    hero_state: i8,
    leader_slot: i16,
    vitality: i16,
    extent_type: u16,
    race: u8,
    zone_race: u8,
) -> i16 {
    if hero_state == DEAD || hero_state == FALL {
        if leader_slot == 0 { FLEE } else { FOLLOWER }
    } else if vitality < NEARLY_DEAD || (extent_type > SPECIAL_XTYPE && race != zone_race) {
        FLEE
    } else {
        mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_adjustments() {
        assert_eq!(adjusted_mode(1, DEAD, 0, 10, 0, 0, 0), FLEE);
        assert_eq!(adjusted_mode(1, FALL, 3, 10, 0, 0, 0), FOLLOWER);
        assert_eq!(adjusted_mode(1, STILL, 0, NEARLY_DEAD - 1, 0, 0, 0), FLEE);
        assert_eq!(adjusted_mode(1, STILL, 0, NEARLY_DEAD, 0, 0, 0), 1);
        assert_eq!(
            adjusted_mode(1, STILL, 0, 10, SPECIAL_XTYPE + 1, 5, 6),
            FLEE
        );

        assert_eq!(adjusted_mode(1, STILL, 0, 10, SPECIAL_XTYPE + 1, 6, 6), 1);
        assert_eq!(adjusted_mode(1, STILL, 0, 10, SPECIAL_XTYPE, 5, 6), 1);
    }

    #[test]
    fn screen_range_is_exclusive() {
        assert!(on_screen(SCREEN_RANGE - 1, SCREEN_RANGE - 1));
        assert!(!on_screen(SCREEN_RANGE, 0));
    }
}
