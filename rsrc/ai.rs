//! What each enemy wants to do each frame (`update_enemy_goals`, from the main loop in csrc/fmain.c): picks a
//! goal and a tactic from how near the hero is and what the enemy carries, and starts or ends the
//! battle music.

use super::actors::turtle_eggs;
use super::actors::{report_battle_result, run_tactic};
use super::asm_text::{queue_print, steer_actor};
use super::mood::set_mood;
use super::random::{bitrand, rand2, rand4};
use super::state::{the_actors, the_hero, the_world};
use super::types::Shape;

// Actor kinds and states (csrc/ftale.h).
const CARRIER: i8 = 5;
const SETFIG: i8 = 4;
const FIGHTING: i8 = 0;
const WALKING: i8 = 12;
const STILL: i8 = 13;
const DEAD: i8 = 15;
const FALL: i8 = 22;
const SHOOT1: i8 = 24;
const SHOOT3: i8 = 25;

// Goals.
const ARCHER1: i16 = 3;
const ARCHER2: i16 = 4;
const FLEE: i16 = 5;
const STAND: i16 = 6;
const WAIT: i16 = 8;
const FOLLOWER: i16 = 9;
const CONFUSED: i16 = 10;

// Tactics.
const FRUST: i16 = 0;
const PURSUE: i16 = 1;
const FOLLOW: i32 = 2;
const RANDOM: i16 = 4;
const BACKUP: i32 = 5;
const EVADE: i16 = 6;
const SHOOT: i16 = 8;
const SHOOTFRUST: i16 = 9;
const EGG_SEEK: i16 = 10;

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
/// Weapon bit for a bow.
const BOW_BIT: i8 = 4;
/// Archers back off when closer than this, shoot inside the second pair, else close in.
const ARCHER_TOO_CLOSE: (i16, i16) = (40, 30);
const ARCHER_IN_RANGE: i16 = 70;
/// Distance at which a fighter closes to strike, minus its goal number (the dark knight: this).
const STRIKE_BASE: i16 = 14;
const DKNIGHT: u8 = 7;
const DKNIGHT_STRIKE: i16 = 16;
const SPIDER_EGGS_RACE: u8 = 4;
const WEAK: i16 = 6;
const NEARLY_DEAD: i16 = 2;
const DKNIGHT_FACING: i8 = 5;
const PRQ_STATS: i32 = 4;
const PRQ_VITALITY: i32 = 7;

/// Decides each enemy's goal and tactic, and starts or ends the battle mood.
pub fn update_enemy_goals() {
    unsafe {
        the_actors.on_screen = 0;
        the_actors.leader = 0;
        let battle2 = the_actors.battle_flag;
        the_actors.battle_flag = 0;
        let mut i = FIRST_AI_ACTOR;
        while i < the_actors.active as usize {
            if the_hero.good_fairy_timer != 0 && the_hero.good_fairy_timer < FAIRY_ARRIVES {
                break;
            }
            think(i, battle2);
            i += 1;
        }
        if battle2 == 0 && the_actors.battle_flag != 0 {
            set_mood(1);
        }
        if the_actors.battle_flag == 0 && battle2 != 0 {
            queue_print(PRQ_VITALITY);
            queue_print(PRQ_STATS);
            report_battle_result();
        }
    }
}

/// One actor's decision. Returns early (skipping the goal update) for carriers, people and
/// anyone out of hit points who is on screen.
unsafe fn think(i: usize, battle2: i8) {
    unsafe {
        let an = &raw mut the_actors.list[i];
        if (*an).kind == CARRIER {
            if the_world.day_clock & CARRIER_AIM_MASK == 0 {
                steer_actor(
                    i as i32,
                    the_hero.x as i32,
                    the_hero.y as i32,
                    CARRIER_AIM_MODE,
                );
            }
            return;
        } else if (*an).kind == SETFIG {
            return;
        }
        let mut mode = (*an).goal as i16;
        let mut tactic = (*an).tactic as i16;

        let xd = ((*an).abs_x as i32 - the_hero.x as i32) as i16;
        let yd = ((*an).abs_y as i32 - the_hero.y as i32) as i16;
        let (xd, yd) = (xd.wrapping_abs(), yd.wrapping_abs());
        if on_screen(xd, yd) {
            the_actors.on_screen = 1;
            if (*an).vitality < 1 {
                return;
            }
            if (*an).visible != 0 || battle2 != 0 {
                the_actors.battle_flag = 1;
            }
        }
        let mut r = bitrand(15) == 0;
        mode = adjust_mode_for_situation(an, mode);

        if tactic == FRUST || tactic == SHOOTFRUST {
            if (*an).weapon & BOW_BIT != 0 {
                run_tactic(i as i32, rand4() + 2);
            } else {
                run_tactic(i as i32, rand2() + 3);
            }
        } else if (*an).state == SHOOT1 {
            (*an).state = SHOOT3;
        } else if mode <= ARCHER2 {
            if mode & 2 == 0 {
                r = rand4() == 0;
            }
            if r {
                (mode, tactic) = choose_tactic(an, mode, xd, yd);
            }
            hostile_action(i, an, mode, tactic, xd, yd);
        } else if mode == FLEE {
            run_tactic(i as i32, BACKUP);
        } else if mode == FOLLOWER {
            run_tactic(i as i32, FOLLOW);
        } else if mode == STAND {
            steer_actor(i as i32, the_hero.x as i32, the_hero.y as i32, 0);
            (*an).state = STILL;
        } else if mode == WAIT {
            (*an).state = STILL;
        }

        (*an).goal = mode as i8;
        if the_actors.leader == 0 {
            the_actors.leader = i as i16;
        }
    }
}

/// Overrides an actor's mode when the situation demands: flee or follow when the hero is down,
/// flee when nearly dead.
unsafe fn adjust_mode_for_situation(an: *mut Shape, mode: i16) -> i16 {
    unsafe {
        adjusted_mode(
            mode,
            the_actors.list[0].state,
            the_actors.leader,
            (*an).vitality,
            the_world.extent_type,
            (*an).race,
            (*the_world.extent).v3,
        )
    }
}

/// Whether an actor `xd`, `yd` (absolute) pixels from the hero counts as on screen.
fn on_screen(xd: i16, yd: i16) -> bool {
    xd < SCREEN_RANGE && yd < SCREEN_RANGE
}

/// The mode after the hero's state and the actor's own health are taken into account: everyone
/// flees (or follows the leader) when the hero is dead or falling, the nearly dead flee, and so do
/// actors of a special encounter that are not the zone's own race in the 60+ zones.
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

/// A hostile enemy picks a new tactic. May also change its mode to wandering if it has no weapon.
unsafe fn choose_tactic(an: *mut Shape, mode: i16, xd: i16, yd: i16) -> (i16, i16) {
    unsafe {
        let facts = TacticFacts {
            race: (*an).race,
            weapon: (*an).weapon,
            vitality: (*an).vitality,
            eggs_to_guard: turtle_eggs != 0,
        };
        pick_tactic(&facts, mode, xd, yd, || rand2() != 0)
    }
}

/// What `pick_tactic` needs to know about the enemy.
struct TacticFacts {
    race: u8,
    weapon: i8,
    vitality: i16,
    /// The turtle's eggs are lying around.
    eggs_to_guard: bool,
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
unsafe fn hostile_action(i: usize, an: *mut Shape, mode: i16, tactic: i16, xd: i16, yd: i16) {
    unsafe {
        match hostile_choice((*an).race, (*an).weapon, (*an).vitality, mode, xd, yd) {
            Hostile::Strike => {
                steer_actor(i as i32, the_hero.x as i32, the_hero.y as i32, 0);
                if (*an).state >= WALKING {
                    (*an).state = FIGHTING;
                }
            }
            Hostile::KnightGuard => {
                (*an).state = STILL;
                (*an).facing = DKNIGHT_FACING;
            }
            Hostile::Tactic => run_tactic(i as i32, tactic as i32),
        }
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
    fn mode_adjustments() {
        assert_eq!(adjusted_mode(1, DEAD, 0, 10, 0, 0, 0), FLEE);
        assert_eq!(adjusted_mode(1, FALL, 3, 10, 0, 0, 0), FOLLOWER);
        assert_eq!(adjusted_mode(1, STILL, 0, NEARLY_DEAD - 1, 0, 0, 0), FLEE);
        assert_eq!(adjusted_mode(1, STILL, 0, NEARLY_DEAD, 0, 0, 0), 1);
        assert_eq!(adjusted_mode(1, STILL, 0, 10, SPECIAL_XTYPE + 1, 5, 6), FLEE);
        assert_eq!(adjusted_mode(1, STILL, 0, 10, SPECIAL_XTYPE + 1, 6, 6), 1);
        assert_eq!(adjusted_mode(1, STILL, 0, 10, SPECIAL_XTYPE, 5, 6), 1);
    }

    #[test]
    fn screen_range_is_exclusive() {
        assert!(on_screen(SCREEN_RANGE - 1, SCREEN_RANGE - 1));
        assert!(!on_screen(SCREEN_RANGE, 0));
    }

    #[test]
    fn strike_rules() {
        let near = STRIKE_BASE - ATTACK_MODE;
        assert_eq!(hostile_choice(0, 1, 10, ATTACK_MODE, near - 1, near - 1), Hostile::Strike);
        assert_eq!(hostile_choice(0, 1, 10, ATTACK_MODE, near, 5), Hostile::Tactic);
        // bows never strike
        assert_eq!(hostile_choice(0, BOW_BIT, 10, ATTACK_MODE, 1, 1), Hostile::Tactic);
        assert_eq!(
            hostile_choice(DKNIGHT, 1, 10, ATTACK_MODE, DKNIGHT_STRIKE - 1, DKNIGHT_STRIKE - 1),
            Hostile::Strike
        );
        assert_eq!(
            hostile_choice(DKNIGHT, 1, 10, ATTACK_MODE, DKNIGHT_STRIKE, 5),
            Hostile::KnightGuard
        );
        assert_eq!(hostile_choice(DKNIGHT, 1, 0, ATTACK_MODE, DKNIGHT_STRIKE, 5), Hostile::Tactic);
    }
}
