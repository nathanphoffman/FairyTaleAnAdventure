//! Swinging weapons and biting: who is hit, how far the weapon reaches and the whoosh of a near miss.

use super::{CARRIER, DEAD, RAFT, WALKING, distance_to_point};
use crate::game::actors::interaction::hit_actor;
use crate::game::data::random::{bitrand, rand4, rand8, rand256};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::files::loading::play_effect;
use crate::game::world::map::{step_x, step_y};

/// A weapon with this bit is a bow (it is not swung).
pub(super) const BOW_BIT: i16 = 4;

/// Touch attacks (monsters' weapon codes 8 and up) are as strong as this weapon.
pub(super) const TOUCH_WEAPON: i16 = 5;

/// Weapon numbers from here on are natural weapons (bites, claws).
const FIRST_TOUCH_WEAPON: i16 = 8;
pub(super) const BRAVE_PER_REACH: i16 = 20;
pub(super) const HERO_BASE_REACH: i16 = 5;
pub(super) const ENEMY_BASE_REACH: i16 = 2;
pub(super) const MAX_REACH: i16 = 14;
pub(super) const REACH_LIMIT: i16 = 15;

/// The tip of the weapon is placed this far (plus a little noise) in front of the actor.
pub(super) const TIP_NOISE: i32 = 3;
pub(super) const SWING_SOUND: i16 = 1;
pub(super) const SWING_SOUND_PITCH: i32 = 150;

/// How much further than `reach` a swing still makes the whoosh sound.
pub(super) const WHOOSH_MARGIN: i16 = 2;

/// Everything not walking swings its weapon (or bites) at whatever is within reach of the tip:
/// `hit_actor` if it lands, a whoosh if it nearly does. Only the hero acts while time is frozen.
pub fn resolve_melee(game: &mut Game) {
    let mut i = 0;

    while i < game.actors.active as usize {
        if i > 0 && game.world.freeze_timer != 0 {
            break;
        }

        let attacker = ActorRef::new(i);

        if i != RAFT && game.actors.list[attacker.slot()].state < WALKING {
            swing(game, i);
        }

        i += 1;
    }
}

/// The weapon an actor swings in melee, or `None` for a bow (archers shoot instead). Anything
/// past the hand weapons (8 and up: bites, claws) counts as a touch.
pub(super) fn melee_weapon(weapon: i16) -> Option<i16> {
    // Masks with BOW_BIT, keeping only the bits that matter here.
    if weapon & BOW_BIT != 0 {
        None
    } else if weapon >= FIRST_TOUCH_WEAPON {
        Some(TOUCH_WEAPON)
    } else {
        Some(weapon)
    }
}

/// How close the weapon tip has to get to hit. Reach beyond the maximum drops to the limit.
pub(super) fn limited_reach(reach: i16) -> i16 {
    if reach > MAX_REACH {
        REACH_LIMIT
    } else {
        reach
    }
}

/// The hero's reach grows with bravery.
pub(super) fn hero_reach(brave: i16) -> i16 {
    limited_reach(brave / BRAVE_PER_REACH + HERO_BASE_REACH)
}

/// An enemy's reach is a fixed base plus a random `bonus`.
pub(super) fn enemy_reach(bonus: i16) -> i16 {
    limited_reach(ENEMY_BASE_REACH + bonus)
}

/// One actor's swing: where the weapon is, how far it reaches and with what strength.
struct Swing {
    attacker: usize,
    facing: i32,
    strength: i16,
    reach: i16,
    tip: (i16, i16),
}

/// One actor's swing.
pub(super) fn swing(game: &mut Game, i: usize) {
    let attacker = ActorRef::new(i);
    let Some(base) = melee_weapon(game.actors.list[attacker.slot()].weapon as i16) else {
        return;
    };

    let strength = base + bitrand(&mut game.rng, 2) as i16;
    let facing = game.actors.list[attacker.slot()].facing as i32;
    let swing = Swing {
        attacker: i,
        facing,
        strength,
        tip: weapon_tip(game, attacker, facing, strength),
        reach: if i == 0 {
            hero_reach(game.hero.brave)
        } else {
            enemy_reach(rand4(&mut game.rng) as i16)
        },
    };

    for j in 0..game.actors.active as usize {
        if swing_at(game, &swing, j) {
            break;
        }
    }
}

/// Swings at actor `j` if it is a target (not the raft, the attacker, the dead, or by a carrier).
/// Returns true when the swing hit, which ends it.
fn swing_at(game: &mut Game, swing: &Swing, j: usize) -> bool {
    let attacker = ActorRef::new(swing.attacker);
    let target = ActorRef::new(j);

    if j == RAFT
        || j == swing.attacker
        || game.actors.list[target.slot()].state == DEAD
        || game.actors.list[attacker.slot()].kind == CARRIER
    {
        return false;
    }

    match swing_result(
        swing.attacker == 0,
        distance_to_point(game, target, swing.tip.0, swing.tip.1),
        swing.reach,
        game.world.freeze_timer != 0,
        swing.strength,
        game.hero.brave,
        || rand256(&mut game.rng),
    ) {
        SwingResult::Hit => {
            hit_actor(
                game,
                swing.attacker as i32,
                j as i32,
                swing.facing,
                swing.strength,
            );

            true
        }

        SwingResult::Whoosh => {
            let pitch = SWING_SOUND_PITCH + rand256(&mut game.rng);
            play_effect(game, SWING_SOUND, pitch);

            false
        }

        SwingResult::Miss => false,
    }
}

/// Where the tip of the weapon is: twice the weapon's strength in front of the attacker, with a
/// little random wobble.
fn weapon_tip(game: &mut Game, attacker: ActorRef, facing: i32, strength: i16) -> (i16, i16) {
    let reach = (strength + strength) as i32;
    let xs = (step_x(
        game.actors.list[attacker.slot()].abs_x as i32,
        facing,
        reach,
    ) + rand8(&mut game.rng)
        - TIP_NOISE) as i16;
    let ys = (step_y(
        game.actors.list[attacker.slot()].abs_y as i32,
        facing,
        reach,
    ) + rand8(&mut game.rng)
        - TIP_NOISE) as i16;

    (xs, ys)
}

/// How a swing at one target turned out.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum SwingResult {
    Hit,
    /// Near enough to make the whoosh sound, but no hit.
    Whoosh,
    Miss,
}

/// The tip of the weapon is `dist` from the target and the reach is `reach`. The hero always hits
/// within reach; an enemy also has to get past the hero's bravery (`dodge_roll` is a 0-255 draw,
/// taken before anything else is checked, only for enemies). Nothing hits while time is frozen,
/// and touch attacks make no sound.
#[allow(clippy::too_many_arguments)] // the whole situation of one swing, from plain values
fn swing_result(
    by_hero: bool,
    dist: i16,
    reach: i16,
    frozen: bool,
    strength: i16,
    hero_bravery: i16,
    dodge_roll: impl FnOnce() -> i32,
) -> SwingResult {
    if (by_hero || dodge_roll() > hero_bravery as i32) && dist < reach && !frozen {
        SwingResult::Hit
    } else if dist < reach + WHOOSH_MARGIN && strength != TOUCH_WEAPON {
        SwingResult::Whoosh
    } else {
        SwingResult::Miss
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bows_are_not_swung() {
        assert_eq!(melee_weapon(BOW_BIT), None);
        assert_eq!(melee_weapon(1), Some(1));
    }

    #[test]
    fn natural_weapons_all_count_as_a_touch() {
        assert_eq!(melee_weapon(8), Some(TOUCH_WEAPON));
        assert_eq!(melee_weapon(10), Some(TOUCH_WEAPON));
    }

    #[test]
    fn reach_is_limited() {
        assert_eq!(limited_reach(MAX_REACH), MAX_REACH);
        assert_eq!(limited_reach(MAX_REACH + 1), REACH_LIMIT);
    }

    #[test]
    fn braver_heroes_reach_further_up_to_the_limit() {
        assert!(hero_reach(100) >= hero_reach(0));
        assert_eq!(hero_reach(i16::MAX / 2), REACH_LIMIT);
    }

    #[test]
    fn the_hero_always_hits_within_reach() {
        let never = || -> i32 { unreachable!("the hero does not roll") };
        assert_eq!(
            swing_result(true, 4, 5, false, 3, 100, never),
            SwingResult::Hit
        );

        assert_eq!(
            swing_result(true, 5, 5, false, 3, 100, never),
            SwingResult::Whoosh
        );

        assert_eq!(
            swing_result(true, 7, 5, false, 3, 100, never),
            SwingResult::Miss
        );
    }

    #[test]
    fn an_enemy_must_beat_the_heros_bravery() {
        assert_eq!(
            swing_result(false, 1, 5, false, 3, 100, || 101),
            SwingResult::Hit
        );

        assert_eq!(
            swing_result(false, 1, 5, false, 3, 100, || 100),
            SwingResult::Whoosh
        );

        assert_eq!(
            swing_result(false, 1, 5, false, 3, 100, || 0),
            SwingResult::Whoosh
        );
    }

    #[test]
    fn the_enemy_rolls_even_when_out_of_reach() {
        let mut drawn = 0;
        swing_result(false, 50, 5, false, 3, 100, || {
            drawn += 1;

            0
        });

        assert_eq!(drawn, 1);
    }

    #[test]
    fn nothing_hits_while_time_is_frozen_and_touches_are_silent() {
        assert_eq!(
            swing_result(true, 1, 5, true, 3, 0, || 0),
            SwingResult::Whoosh
        );

        assert_eq!(
            swing_result(true, 1, 5, true, TOUCH_WEAPON, 0, || 0),
            SwingResult::Miss
        );

        assert_eq!(
            swing_result(true, 5, 5, false, TOUCH_WEAPON, 0, || 0),
            SwingResult::Miss
        );
    }
}
