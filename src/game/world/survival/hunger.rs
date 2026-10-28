//! What hunger and tiredness do to the hero: messages, lost vitality, collapsing and fainting.

use super::{STARVING_AT, safe_check_due};
use crate::game::data::state::Hero;
use crate::game::data::types::Shape;

pub(super) const SLEEP: i8 = 23;
pub(super) const HUNGRY: i16 = 35;
const VERY_HUNGRY: i16 = 60;
const WEAK_VITALITY: i16 = 5;
const MUCH_HUNGER: i16 = 100;
const MUCH_FATIGUE: i16 = 160;
const COLLAPSE_FATIGUE: i16 = 170;
const FAINT_HUNGER: i16 = 140;
const FAINT_RESET: i16 = 130;
const HUNGER_STEP_MASK: i16 = 7;

/// Hungry enough to eat fruit at the safe spot.
const EATS_FRUIT_AT: i16 = 30;

// Event messages for hunger and tiredness.
const EVENT_HUNGRY: i32 = 0;
const EVENT_VERY_HUNGRY: i32 = 1;
const EVENT_STARVING_WARNING: i32 = 2;
const EVENT_COLLAPSE: i32 = 12;
const EVENT_FAINT: i32 = 24;

/// The hero is hungrier and more tired on a safe-check frame while alive and awake.
pub(super) fn hunger_due(clock: u16, vitality: i16, state: i8) -> bool {
    safe_check_due(clock) && vitality != 0 && state != SLEEP
}

/// Hungry enough to eat a piece of fruit at the safe spot (only when one is carried).
pub(super) fn eats_fruit(hungry: i16, fruit: u8) -> bool {
    hungry > EATS_FRUIT_AT && fruit != 0
}

/// What hunger and fatigue did this time: the event messages to show (in order) and whether the
/// stats line needs redrawing.
#[derive(Debug, Default, PartialEq)]
pub(super) struct HungerReport {
    pub(super) events: Vec<i32>,
    pub(super) redraw_stats: bool,
}

/// The messages and damage that come with hunger and fatigue. Every 8 points of hunger the hero
/// loses vitality when very hungry or tired, and when weak either collapses from fatigue or faints
/// from hunger (falling asleep).
pub(super) fn hunger_effects(hero: &mut Hero, body: &mut Shape) -> HungerReport {
    let mut report = HungerReport::default();

    if hero.hunger == HUNGRY {
        report.events.push(EVENT_HUNGRY);
    } else if hero.hunger == VERY_HUNGRY {
        report.events.push(EVENT_VERY_HUNGRY);
    } else if hero.hunger & HUNGER_STEP_MASK == 0 {
        if body.vitality > WEAK_VITALITY {
            if hero.hunger > MUCH_HUNGER || hero.fatigue > MUCH_FATIGUE {
                body.vitality -= 2;
                report.redraw_stats = true;
            }

            if hero.hunger > STARVING_AT {
                report.events.push(EVENT_STARVING_WARNING);
            }
        } else if hero.fatigue > COLLAPSE_FATIGUE {
            report.events.push(EVENT_COLLAPSE);
            body.state = SLEEP;
        } else if hero.hunger > FAINT_HUNGER {
            report.events.push(EVENT_FAINT);
            hero.hunger = FAINT_RESET;
            body.state = SLEEP;
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hero_with_body(hunger: i16, fatigue: i16, vitality: i16) -> (Hero, Shape) {
        let mut hero: Hero = Default::default();

        let mut body: Shape = Default::default();
        hero.hunger = hunger;
        hero.fatigue = fatigue;
        body.vitality = vitality;

        (hero, body)
    }

    #[test]
    fn hunger_messages_at_the_first_two_thresholds() {
        let (mut hero, mut body) = hero_with_body(HUNGRY, 0, 50);
        assert_eq!(hunger_effects(&mut hero, &mut body).events, vec![0]);
        let (mut hero, mut body) = hero_with_body(VERY_HUNGRY, 0, 50);
        assert_eq!(hunger_effects(&mut hero, &mut body).events, vec![1]);
    }

    #[test]
    fn nothing_happens_between_eights_when_only_a_little_hungry() {
        let (mut hero, mut body) = hero_with_body(41, 0, 50);
        let report = hunger_effects(&mut hero, &mut body);
        assert_eq!(report, HungerReport::default());
        assert_eq!(body.vitality, 50);
    }

    #[test]
    fn tired_hero_loses_vitality_without_a_message_until_starving() {
        let (mut hero, mut body) = hero_with_body(88, MUCH_FATIGUE + 1, 50);
        let report = hunger_effects(&mut hero, &mut body);
        assert_eq!((body.vitality, report.redraw_stats), (48, true));
        assert!(report.events.is_empty());
        let (mut hero, mut body) = hero_with_body(96, MUCH_FATIGUE + 1, 50);
        let report = hunger_effects(&mut hero, &mut body);
        assert_eq!((body.vitality, report.events), (48, vec![2]));
    }

    #[test]
    fn weak_and_exhausted_hero_collapses_asleep() {
        let (mut hero, mut body) = hero_with_body(80, COLLAPSE_FATIGUE + 1, WEAK_VITALITY);
        let report = hunger_effects(&mut hero, &mut body);
        assert_eq!((report.events, body.state), (vec![12], SLEEP));
    }

    #[test]
    fn weak_and_starving_hero_faints_and_hunger_resets() {
        let (mut hero, mut body) =
            hero_with_body(FAINT_HUNGER + 8 - FAINT_HUNGER % 8, 0, WEAK_VITALITY);
        let report = hunger_effects(&mut hero, &mut body);
        assert_eq!(
            (report.events, body.state, hero.hunger),
            (vec![24], SLEEP, FAINT_RESET)
        );
    }

    #[test]
    fn weak_hero_not_yet_faint_is_left_alone() {
        let (mut hero, mut body) = hero_with_body(120, 0, WEAK_VITALITY);
        assert_eq!(
            hunger_effects(&mut hero, &mut body),
            HungerReport::default()
        );
    }
}
