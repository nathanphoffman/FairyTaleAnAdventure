//! Hitting an actor: who can be hurt, the sound, the knock-back, and what is said after a fight.

use super::blocking::step_figure;
use super::extents::summon_turtle;
use super::{DEAD, DRAGON, ENEMY, FLEE, HERO, SETFIG, turtle_eggs};
use crate::game::actors::encounters::check_death;
use crate::game::data::random::{bitrand, rand64, rand256};
use crate::game::data::state::{ActorRef, the_actors, the_hero, the_world};
use crate::game::files::loading::play_effect;
use crate::game::ui::messages::{print, print_cont};
use crate::game::ui::text::{print_number, speak};

// Races with special rules when hit.
pub(super) const RACE_WRAITH: u8 = 2;
const RACE_SPIRIT: u8 = 9;
const RACE_GHOST_KEEPER: u8 = 0x89;
const RACE_UNHITTABLE: [u8; 2] = [0x8a, 0x8b];

/// The weapon level needed to hurt spirits, and the item that also does.
const MIN_HURT_WEAPON: i8 = 4;
const SPIRIT_HURT_ITEM: usize = 7;
const MESSAGE_NEEDS_WEAPON: i32 = 58;

/// How far hit actors are pushed back.
const KNOCKBACK: i16 = 2;
const WORLD_DEAD_CHECK: i32 = 5;

// Sound effects and their speeds.
const EFFECT_ARROW_HIT: i16 = 2;

/// Base pitch of each hit sound, and the random spread added to some of them.
const PITCH_ARROW: i32 = 500;
const PITCH_ROCK: i32 = 3200;
const PITCH_HERO_HURT: i32 = 800;
const PITCH_FOE_HURT: i32 = 400;
const PITCH_SPREAD: i32 = 511;

/// Slots from here on in the actor list are monsters.
const FIRST_MONSTER_SLOT: i32 = 3;
const EFFECT_ROCK_HIT: i16 = 5;
const EFFECT_HERO_HURT: i16 = 0;
const EFFECT_FOE_HURT: i16 = 3;

/// Message count shown after a fight only while the area is this peaceful (`extent_type` below).
const PEACEFUL_EXTENT: u16 = 50;
const BRAVE_LOW_VITALITY: i16 = 5;

/// Actor `i`.
pub(super) fn actor_at(i: i32) -> ActorRef {
    ActorRef::new(i as usize)
}

/// A hit from actor `i` on actor `j`: `wt` damage, then the target is pushed back in direction `fc`.
/// `i` is -1 or -2 for hits by missiles (which have different sounds).
///
/// Special cases, as in the original: creatures of race 9 and the witch (race 0x89, a person of
/// kind 9) cannot be hurt by weak weapons (weapon below 4); the witch can be hurt anyway if the hero
/// holds the item in inventory slot 7 (the sun stone). Races 0x8a and 0x8b (the spectre and the
/// ghost brothers) are never hurt. Dragons and people are not pushed back.
pub fn hit_actor(i: i32, j: i32, fc: i32, wt: i16) {
    unsafe {
        let mut target = actor_at(j);
        let hero_weapon = actor_at(HERO as i32).weapon;
        let has_spirit_item = *the_hero.stuff.add(SPIRIT_HURT_ITEM) != 0;

        if too_weak_to_hurt(target.race, hero_weapon, has_spirit_item) {
            speak(MESSAGE_NEEDS_WEAPON);

            return;
        }

        if RACE_UNHITTABLE.contains(&target.race) {
            return;
        }

        target.vitality = vitality_after_hit(target.vitality, wt);

        match hit_sound(i, j) {
            HitSound::Arrow => play_effect(EFFECT_ARROW_HIT, PITCH_ARROW + rand64()),
            HitSound::Rock => play_effect(EFFECT_ROCK_HIT, PITCH_ROCK + bitrand(PITCH_SPREAD)),
            HitSound::HeroHurt => {
                play_effect(EFFECT_HERO_HURT, PITCH_HERO_HURT + bitrand(PITCH_SPREAD))
            }

            HitSound::FoeHurt => play_effect(EFFECT_FOE_HURT, PITCH_FOE_HURT + rand256()),
        }

        // push back the foe
        if can_be_pushed(target.kind) && step_figure(j as i16, fc as i16, KNOCKBACK) != 0 && i >= 0
        {
            step_figure(i as i16, fc as i16, KNOCKBACK);
        }

        check_death(j, WORLD_DEAD_CHECK);
    }
}

/// Which sound a hit makes: `i` is the attacker (-1 an arrow, -2 a fireball), `j` the target.
#[derive(Debug, PartialEq, Eq)]
enum HitSound {
    Arrow,
    Rock,
    HeroHurt,
    FoeHurt,
}

/// Which sound a hit makes, by who attacked (-1 an arrow, -2 a fireball, otherwise an actor slot) and
/// who was hit (slot 0 is the hero).
fn hit_sound(attacker: i32, target: i32) -> HitSound {
    match (attacker, target) {
        (-1, _) => HitSound::Arrow,
        (-2, _) => HitSound::Rock,
        (_, 0) => HitSound::HeroHurt,
        _ => HitSound::FoeHurt,
    }
}

/// Dragons and people are not pushed back by a hit.
fn can_be_pushed(kind: i8) -> bool {
    kind != DRAGON && kind != SETFIG
}

/// What to say after a fight.
#[derive(Debug, PartialEq, Eq)]
enum BattleReport {
    /// The hero is dead, or nothing worth saying.
    Silent,
    Bravely,
    Foes {
        dead: i32,
        fled: i32,
    },
}

/// `dead` and `fled` count the enemies that fell and that ran away.
fn battle_report(hero_vitality: i16, dead: i32, fled: i32, extent_type: u16) -> BattleReport {
    if hero_vitality < 1 {
        BattleReport::Silent
    } else if hero_vitality < BRAVE_LOW_VITALITY && dead != 0 {
        BattleReport::Bravely
    } else if extent_type < PEACEFUL_EXTENT {
        BattleReport::Foes { dead, fled }
    } else {
        BattleReport::Silent
    }
}

/// Spirits can only be hurt by a real weapon (`MIN_HURT_WEAPON` or better); the ghost keeper too,
/// unless the hero carries the item that makes him vulnerable.
fn too_weak_to_hurt(race: u8, hero_weapon: i8, has_spirit_item: bool) -> bool {
    hero_weapon < MIN_HURT_WEAPON
        && (race == RACE_SPIRIT || (race == RACE_GHOST_KEEPER && !has_spirit_item))
}

/// A hit of strength `wt` takes that much vitality, down to a minimum of 0.
fn vitality_after_hit(vitality: i16, wt: i16) -> i16 {
    (vitality - wt).max(0)
}

/// After a fight: reports how many foes were defeated or fled, or "Bravely done!" if the hero is
/// nearly dead. Then, if turtle eggs are around, the turtle story event is checked.
pub fn report_battle_result() {
    unsafe {
        let (dead, fled) = count_casualties();
        let vitality = actor_at(HERO as i32).vitality;

        match battle_report(vitality, dead, fled, the_world.extent_type) {
            BattleReport::Silent => {}
            BattleReport::Bravely => print("Bravely done!"),
            BattleReport::Foes { dead, fled } => {
                if dead != 0 {
                    print("");
                    print_number(dead, 1);
                    print_cont("foes were defeated in battle.");
                }

                if fled != 0 {
                    print("");
                    print_number(fled, 1);
                    print_cont("foes fled in retreat.");
                }
            }
        }

        // check for presence of turtle eggs
        if turtle_eggs != 0 {
            summon_turtle();
        }
    }
}

/// How many of the enemies are dead and how many have fled.
unsafe fn count_casualties() -> (i32, i32) {
    unsafe {
        let (mut dead, mut fled) = (0, 0);

        for i in FIRST_MONSTER_SLOT..the_actors.active as i32 {
            let a = actor_at(i);

            if a.kind != ENEMY {
                continue;
            }

            if a.state == DEAD {
                dead += 1;
            } else if a.goal == FLEE {
                fled += 1;
            }
        }

        (dead, fled)
    }
}

#[cfg(test)]
mod tests {
    use super::super::DRAGON;
    use super::super::SETFIG;
    use super::*;

    #[test]
    fn spirits_need_a_real_weapon() {
        assert!(too_weak_to_hurt(RACE_SPIRIT, MIN_HURT_WEAPON - 1, true));
        assert!(!too_weak_to_hurt(RACE_SPIRIT, MIN_HURT_WEAPON, false));
    }

    #[test]
    fn the_ghost_keeper_is_hurt_by_weak_weapons_only_with_the_item() {
        assert!(too_weak_to_hurt(RACE_GHOST_KEEPER, 0, false));
        assert!(!too_weak_to_hurt(RACE_GHOST_KEEPER, 0, true));
    }

    #[test]
    fn ordinary_foes_are_always_hurt() {
        assert!(!too_weak_to_hurt(0, 0, false));
    }

    #[test]
    fn a_hit_never_takes_vitality_below_zero() {
        assert_eq!(vitality_after_hit(10, 4), 6);
        assert_eq!(vitality_after_hit(3, 9), 0);
    }

    #[test]
    fn hit_sounds_by_who_hit_whom() {
        assert_eq!(hit_sound(-1, 0), HitSound::Arrow);
        assert_eq!(hit_sound(-2, 5), HitSound::Rock);
        assert_eq!(hit_sound(3, 0), HitSound::HeroHurt);
        assert_eq!(hit_sound(0, 3), HitSound::FoeHurt);
    }

    #[test]
    fn dragons_and_people_are_not_pushed_back() {
        assert!(!can_be_pushed(DRAGON));
        assert!(!can_be_pushed(SETFIG));
        assert!(can_be_pushed(2));
    }

    #[test]
    fn what_is_said_after_a_fight() {
        let calm = PEACEFUL_EXTENT - 1;
        assert_eq!(battle_report(0, 3, 0, calm), BattleReport::Silent);
        assert_eq!(
            battle_report(BRAVE_LOW_VITALITY - 1, 1, 0, calm),
            BattleReport::Bravely
        );

        // nearly dead but nobody died: the normal report
        assert_eq!(
            battle_report(BRAVE_LOW_VITALITY - 1, 0, 2, calm),
            BattleReport::Foes { dead: 0, fled: 2 }
        );

        assert_eq!(
            battle_report(50, 2, 1, calm),
            BattleReport::Foes { dead: 2, fled: 1 }
        );

        // peaceful places say nothing
        assert_eq!(
            battle_report(50, 2, 1, PEACEFUL_EXTENT),
            BattleReport::Silent
        );
    }
}
