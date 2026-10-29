//! Choosing a tactic for an enemy that wants to fight, and what a hostile enemy does about it.

use super::{BACKUP, BOW_BIT, STILL};
use crate::game::actors::interaction::run_tactic;
use crate::game::actors::steering::steer_actor;
use crate::game::data::random::rand2;
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;

const FIGHTING: i8 = 0;
const WALKING: i8 = 12;

// Goals.
const ARCHER1: i16 = 3;
const CONFUSED: i16 = 10;
const PURSUE: i16 = 1;
const RANDOM: i16 = 4;
const EVADE: i16 = 6;
const SHOOT: i16 = 8;
const EGG_SEEK: i16 = 10;

/// Archers back off when closer than this, shoot inside the second pair, else close in.
const ARCHER_TOO_CLOSE: (i16, i16) = (40, 30);
const ARCHER_IN_RANGE: i16 = 70;

/// Distance at which a fighter closes to strike, minus its goal number (the dark knight: this).
const STRIKE_BASE: i16 = 14;
const DKNIGHT: u8 = 7;
const DKNIGHT_STRIKE: i16 = 16;
const SPIDER_EGGS_RACE: u8 = 4;
const WEAK: i16 = 6;
const DKNIGHT_FACING: i8 = 5;

/// A hostile enemy picks a new tactic. May also change its mode to wandering if it has no weapon.
pub(super) fn choose_tactic(
    game: &mut Game,
    actor: ActorRef,
    mode: i16,
    xd: i16,
    yd: i16,
) -> (i16, i16) {
    let facts = TacticFacts {
        race: game.actor(actor).race,
        weapon: game.actor(actor).weapon,
        vitality: game.actor(actor).vitality,
        eggs_to_guard: game.actors.turtle_eggs != 0,
    };

    pick_tactic(&facts, mode, xd, yd, || rand2(&mut game.rng) != 0)
}

/// What `pick_tactic` needs to know about the enemy.
struct TacticFacts {
    pub(super) race: u8,
    pub(super) weapon: i8,
    pub(super) vitality: i16,
    /// The turtle's eggs are lying around.
    pub(super) eggs_to_guard: bool,
}

/// The tactic for an enemy in `mode`, `xd` and `yd` pixels (absolute) from the hero: seek the
/// eggs, wander if unarmed (which also changes the mode), evade when weak (`coin` decides, and is
/// only asked then), keep an archer at a distance and shoot, or just pursue.
fn pick_tactic(
    enemy: &TacticFacts,
    mut mode: i16,
    xd: i16,
    yd: i16,
    coin: impl FnOnce() -> bool,
) -> (i16, i16) {
    let tactic;

    if enemy.race == SPIDER_EGGS_RACE && enemy.eggs_to_guard {
        tactic = EGG_SEEK;
    } else if enemy.weapon < 1 {
        mode = CONFUSED;
        tactic = RANDOM;
    } else if enemy.vitality < WEAK && coin() {
        tactic = EVADE;
    } else if mode >= ARCHER1 {
        tactic = if xd < ARCHER_TOO_CLOSE.0 && yd < ARCHER_TOO_CLOSE.1 {
            BACKUP as i16
        } else if xd < ARCHER_IN_RANGE && yd < ARCHER_IN_RANGE {
            SHOOT
        } else {
            PURSUE
        };
    } else {
        tactic = PURSUE;
    }

    (mode, tactic)
}

/// A hostile enemy strikes if close enough, otherwise acts on its tactic.
pub(super) fn hostile_action(
    game: &mut Game,
    actor: ActorRef,
    mode: i16,
    tactic: i16,
    (xd, yd): (i16, i16),
) {
    let i = actor.slot();

    match hostile_choice(
        game.actor(actor).race,
        game.actor(actor).weapon,
        game.actor(actor).vitality,
        mode,
        xd,
        yd,
    ) {
        Hostile::Strike => {
            steer_actor(game, i as i32, game.hero.x as i32, game.hero.y as i32, 0);

            if game.actor(actor).state >= WALKING {
                game.actor_mut(actor).state = FIGHTING;
            }
        }

        Hostile::KnightGuard => {
            game.actor_mut(actor).state = STILL;
            game.actor_mut(actor).facing = DKNIGHT_FACING;
        }

        Hostile::Tactic => run_tactic(game, i as i32, tactic as i32),
    }
}

/// What a hostile enemy does this frame.
#[derive(Debug, PartialEq, Eq)]
enum Hostile {
    /// Close enough with a hand weapon: face the hero and fight.
    Strike,
    /// The dark knight stands guard, facing one way.
    KnightGuard,
    /// Otherwise follow the tactic.
    Tactic,
}

/// What a hostile enemy does this frame: strike if close enough with a hand weapon (the dark knight
/// has its own, longer reach), the knight stands guard while alive, and anything else follows its tactic.
fn hostile_choice(race: u8, weapon: i8, vitality: i16, mode: i16, xd: i16, yd: i16) -> Hostile {
    let mut thresh = STRIKE_BASE - mode;

    if race == DKNIGHT {
        thresh = DKNIGHT_STRIKE;
    }

    if weapon & BOW_BIT == 0 && xd < thresh && yd < thresh {
        Hostile::Strike
    } else if race == DKNIGHT && vitality != 0 {
        Hostile::KnightGuard
    } else {
        Hostile::Tactic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mode of an ordinary melee enemy (below the archers).
    const ATTACK_MODE: i16 = 1;

    fn enemy(weapon: i8, vitality: i16) -> TacticFacts {
        TacticFacts {
            race: 0,
            weapon,
            vitality,
            eggs_to_guard: false,
        }
    }

    #[test]
    fn unarmed_enemy_wanders_aimlessly() {
        assert_eq!(
            pick_tactic(&enemy(0, 20), ATTACK_MODE, 0, 0, || false),
            (CONFUSED, RANDOM)
        );
    }

    #[test]
    fn healthy_swordsman_pursues() {
        assert_eq!(
            pick_tactic(&enemy(1, 20), ATTACK_MODE, 0, 0, || true),
            (ATTACK_MODE, PURSUE)
        );
    }

    #[test]
    fn weak_enemy_evades_only_when_the_coin_says_so() {
        assert_eq!(
            pick_tactic(&enemy(1, WEAK - 1), ATTACK_MODE, 0, 0, || true).1,
            EVADE
        );

        assert_eq!(
            pick_tactic(&enemy(1, WEAK - 1), ATTACK_MODE, 0, 0, || false).1,
            PURSUE
        );
    }

    #[test]
    fn strong_enemy_never_flips_the_coin() {
        pick_tactic(&enemy(1, WEAK), ATTACK_MODE, 0, 0, || {
            panic!("coin flipped")
        });
    }

    #[test]
    fn archer_backs_off_when_close_shoots_in_range_and_otherwise_closes_in() {
        let archer = enemy(BOW_BIT, 20);
        let (close_x, close_y) = (ARCHER_TOO_CLOSE.0 - 1, ARCHER_TOO_CLOSE.1 - 1);
        assert_eq!(
            pick_tactic(&archer, ARCHER1, close_x, close_y, || false).1,
            BACKUP as i16
        );

        let mid = ARCHER_IN_RANGE - 1;
        assert_eq!(
            pick_tactic(&archer, ARCHER1, ARCHER_TOO_CLOSE.0, mid, || false).1,
            SHOOT
        );

        assert_eq!(
            pick_tactic(&archer, ARCHER1, ARCHER_IN_RANGE, ARCHER_IN_RANGE, || false).1,
            PURSUE
        );
    }

    #[test]
    fn spider_guards_the_eggs_while_the_turtle_has_them() {
        let spider = TacticFacts {
            race: SPIDER_EGGS_RACE,
            weapon: 1,
            vitality: 20,
            eggs_to_guard: true,
        };

        assert_eq!(
            pick_tactic(&spider, ATTACK_MODE, 0, 0, || false).1,
            EGG_SEEK
        );
    }

    #[test]
    fn strike_rules() {
        let near = STRIKE_BASE - ATTACK_MODE;
        assert_eq!(
            hostile_choice(0, 1, 10, ATTACK_MODE, near - 1, near - 1),
            Hostile::Strike
        );

        assert_eq!(
            hostile_choice(0, 1, 10, ATTACK_MODE, near, 5),
            Hostile::Tactic
        );

        // bows never strike
        assert_eq!(
            hostile_choice(0, BOW_BIT, 10, ATTACK_MODE, 1, 1),
            Hostile::Tactic
        );

        assert_eq!(
            hostile_choice(
                DKNIGHT,
                1,
                10,
                ATTACK_MODE,
                DKNIGHT_STRIKE - 1,
                DKNIGHT_STRIKE - 1
            ),
            Hostile::Strike
        );

        assert_eq!(
            hostile_choice(DKNIGHT, 1, 10, ATTACK_MODE, DKNIGHT_STRIKE, 5),
            Hostile::KnightGuard
        );

        assert_eq!(
            hostile_choice(DKNIGHT, 1, 0, ATTACK_MODE, DKNIGHT_STRIKE, 5),
            Hostile::Tactic
        );
    }
}
