//! Swinging weapons and biting: who is hit, how far the weapon reaches and the whoosh of a near miss.

use super::{CARRIER, DEAD, RAFT, WALKING, chebyshev};
use crate::game::actors::interaction::hit_actor;
use crate::game::data::random::{bitrand, rand4, rand8, rand256};
use crate::game::data::state::{the_actors, the_hero, the_world};
use crate::game::data::types::Shape;
use crate::game::files::loading::play_effect;
use crate::game::world::map::{step_x, step_y};

/// A weapon with this bit is a bow (it is not swung).
pub(super) const BOW_BIT: i16 = 4;

/// Touch attacks (monsters' weapon codes 8 and up) are as strong as this weapon.
pub(super) const TOUCH_WEAPON: i16 = 5;
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
pub fn resolve_melee() {
    unsafe {
        let mut i = 0;

        while i < the_actors.active as usize {
            if i > 0 && the_world.freeze_timer != 0 {
                break;
            }

            let attacker = &raw const the_actors.list[i];

            if i != RAFT && (*attacker).state < WALKING {
                swing(i);
            }

            i += 1;
        }
    }
}

/// The weapon an actor swings in melee, or `None` for a bow (archers shoot instead). Anything
/// past the hand weapons (8 and up: bites, claws) counts as a touch.
pub(super) fn melee_weapon(weapon: i16) -> Option<i16> {
    if weapon & BOW_BIT != 0 {
        None
    } else if weapon >= 8 {
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

/// One actor's swing.
pub(super) unsafe fn swing(i: usize) {
    unsafe {
        let attacker = &raw const the_actors.list[i];
        let fc = (*attacker).facing as i32;
        let Some(base) = melee_weapon((*attacker).weapon as i16) else {
            return;
        };

        let wt = base + bitrand(2) as i16;
        let (xs, ys) = weapon_tip(attacker, fc, wt);
        let bv = if i == 0 {
            hero_reach(the_hero.brave)
        } else {
            enemy_reach(rand4() as i16)
        };

        let mut j = 0;

        while j < the_actors.active as usize {
            let target = &raw const the_actors.list[j];

            if j == RAFT || j == i || (*target).state == DEAD || (*attacker).kind == CARRIER {
                j += 1;

                continue;
            }

            let xd = ((*target).abs_x as i32 - xs as i32) as i16;
            let yd = ((*target).abs_y as i32 - ys as i32) as i16;
            let dist = chebyshev(xd, yd);

            match swing_result(
                i == 0,
                dist,
                bv,
                the_world.freeze_timer != 0,
                wt,
                the_hero.brave,
                rand256,
            ) {
                SwingResult::Hit => {
                    hit_actor(i as i32, j as i32, fc, wt);

                    break;
                }

                SwingResult::Whoosh => play_effect(SWING_SOUND, SWING_SOUND_PITCH + rand256()),
                SwingResult::Miss => {}
            }

            j += 1;
        }
    }
}

/// Where the tip of the weapon is: twice the weapon's strength in front of the attacker, with a
/// little random wobble.
unsafe fn weapon_tip(attacker: *const Shape, facing: i32, strength: i16) -> (i16, i16) {
    unsafe {
        let reach = (strength + strength) as i32;
        let xs = (step_x((*attacker).abs_x as i32, facing, reach) + rand8() - TIP_NOISE) as i16;
        let ys = (step_y((*attacker).abs_y as i32, facing, reach) + rand8() - TIP_NOISE) as i16;

        (xs, ys)
    }
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
