//! Hunger and fatigue (from the main loop in csrc/fmain.c): noting a safe spot, eating fruit, and
//! the messages and damage that come with being hungry or tired.

use crate::game::data::state::{Hero, the_actors, the_hero, the_witch, the_world};
use crate::game::data::types::Shape;
use crate::game::ui::music::set_mood;
use crate::game::ui::text::{queue_print, show_event};

// Actor kinds and states (csrc/ftale.h).
const DEAD: i8 = 15;
const SLEEP: i8 = 23;

// Print queue entries.
const PRQ_STATS: i32 = 4;

const SAFE_CHECK_MASK: u16 = 127;
const FRUIT_SLOT: usize = 24;
const FRUIT_FOOD: i16 = 30;
const EVENT_ATE_FRUIT: i32 = 37;
const HUNGRY: i16 = 35;
const VERY_HUNGRY: i16 = 60;
const STARVING_AT: i16 = 90;
const WEAK_VITALITY: i16 = 5;
const MUCH_HUNGER: i16 = 100;
const MUCH_FATIGUE: i16 = 160;
const COLLAPSE_FATIGUE: i16 = 170;
const FAINT_HUNGER: i16 = 140;
const FAINT_RESET: i16 = 130;
const TIRED_AT: i16 = 70;

/// Hunger, fatigue and the safe spot are checked when the clock is a multiple of 128; the
/// music is picked when it is a multiple of 8.
fn safe_check_due(clock: u16) -> bool {
    clock & SAFE_CHECK_MASK == 0
}

fn mood_due(clock: u16, battle: bool) -> bool {
    clock & 7 == 0 && !battle
}

/// The hero is hungrier and more tired on a safe-check frame while alive and awake.
fn hunger_due(clock: u16, vitality: i16, state: i8) -> bool {
    safe_check_due(clock) && vitality != 0 && state != SLEEP
}

/// Hungry enough to eat a piece of fruit at the safe spot (only when one is carried).
fn eats_fruit(hungry: i16, fruit: u8) -> bool {
    hungry > 30 && fruit != 0
}

/// Notes a safe spot, picks the music, and makes the hero hungrier and more tired.
pub fn survival_tick() {
    unsafe {
        let facts = SurvivalFacts {
            clock: the_world.day_clock,
            region: the_world.region,
            actors_on_screen: the_actors.on_screen != 0,
            actors_loading: the_actors.loading != 0,
            witch_on_screen: the_witch.on_screen != 0,
            battle: the_actors.battle_flag != 0,
        };

        let steps = survival_step(
            &facts,
            &mut *(&raw mut the_hero),
            &mut *(&raw mut the_actors.list[0]),
        );

        for step in steps {
            match step {
                SurvivalStep::Event(e) => show_event(e),
                SurvivalStep::RedrawStats => queue_print(PRQ_STATS),
                SurvivalStep::PickMusic => set_mood(0),
            }
        }
    }
}

/// What `survival_step` needs to know about the world.
struct SurvivalFacts {
    clock: u16,
    region: u16,
    actors_on_screen: bool,
    actors_loading: bool,
    witch_on_screen: bool,
    battle: bool,
}

/// What the survival tick wants done, in order.
#[derive(Debug, PartialEq, Eq)]
enum SurvivalStep {
    Event(i32),
    RedrawStats,
    PickMusic,
}

/// One frame of the survival rules: remembers a safe spot (eating fruit if hungry), then picks the
/// music, then makes the hero hungrier and more tired. `body` is the hero's actor.
fn survival_step(facts: &SurvivalFacts, hero: &mut Hero, body: &mut Shape) -> Vec<SurvivalStep> {
    let mut steps = Vec::new();

    if safe_check_due(facts.clock)
        && !facts.actors_on_screen
        && !facts.actors_loading
        && !facts.witch_on_screen
        && body.environ == 0
        && hero.safe_flag == 0
        && body.state != DEAD
    {
        hero.safe_r = facts.region;
        hero.safe_x = hero.x;
        hero.safe_y = hero.y;

        let fruit = unsafe { hero.stuff.add(FRUIT_SLOT) };

        if eats_fruit(hero.hunger, unsafe { *fruit }) {
            unsafe { *fruit -= 1 };
            hero.hunger -= FRUIT_FOOD;
            steps.push(SurvivalStep::Event(EVENT_ATE_FRUIT));
        }
    }

    if mood_due(facts.clock, facts.battle) {
        steps.push(SurvivalStep::PickMusic);
    }

    if hunger_due(facts.clock, body.vitality, body.state) {
        hero.hunger += 1;
        hero.fatigue += 1;
        let report = hunger_effects(hero, body);

        if report.redraw_stats {
            steps.push(SurvivalStep::RedrawStats);
        }

        steps.extend(report.events.into_iter().map(SurvivalStep::Event));

        if hero.fatigue == TIRED_AT {
            steps.push(SurvivalStep::Event(3));
        } else if hero.hunger == STARVING_AT {
            steps.push(SurvivalStep::Event(4));
        }
    }

    steps
}

/// What hunger and fatigue did this time: the event messages to show (in order) and whether the
/// stats line needs redrawing.
#[derive(Debug, Default, PartialEq)]
struct HungerReport {
    events: Vec<i32>,
    redraw_stats: bool,
}

/// The messages and damage that come with hunger and fatigue. Every 8 points of hunger the hero
/// loses vitality when very hungry or tired, and when weak either collapses from fatigue or faints
/// from hunger (falling asleep).
fn hunger_effects(hero: &mut Hero, body: &mut Shape) -> HungerReport {
    let mut report = HungerReport::default();

    if hero.hunger == HUNGRY {
        report.events.push(0);
    } else if hero.hunger == VERY_HUNGRY {
        report.events.push(1);
    } else if hero.hunger & 7 == 0 {
        if body.vitality > WEAK_VITALITY {
            if hero.hunger > MUCH_HUNGER || hero.fatigue > MUCH_FATIGUE {
                body.vitality -= 2;
                report.redraw_stats = true;
            }

            if hero.hunger > STARVING_AT {
                report.events.push(2);
            }
        } else if hero.fatigue > COLLAPSE_FATIGUE {
            report.events.push(12);
            body.state = SLEEP;
        } else if hero.hunger > FAINT_HUNGER {
            report.events.push(24);
            hero.hunger = FAINT_RESET;
            body.state = SLEEP;
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Actor state of a hero standing still.
    const STILL: i8 = 13;

    fn hero_and_body(hunger: i16, fatigue: i16, vitality: i16) -> (Hero, Shape) {
        let mut hero: Hero = unsafe { std::mem::zeroed() };

        let mut body: Shape = unsafe { std::mem::zeroed() };
        hero.hunger = hunger;
        hero.fatigue = fatigue;
        body.vitality = vitality;

        (hero, body)
    }

    /// A hero with a pack the test controls, standing in a calm place on a safe-check frame.
    fn calm_hero(pack: &mut [u8; 40]) -> (SurvivalFacts, Hero, Shape) {
        let mut hero: Hero = unsafe { std::mem::zeroed() };

        let mut body: Shape = unsafe { std::mem::zeroed() };
        hero.stuff = pack.as_mut_ptr();
        hero.x = 1234;
        hero.y = 5678;
        body.vitality = 50;
        body.state = STILL;
        let facts = SurvivalFacts {
            clock: 128,
            region: 3,
            actors_on_screen: false,
            actors_loading: false,
            witch_on_screen: false,
            battle: true,
        };

        (facts, hero, body)
    }

    #[test]
    fn a_calm_frame_notes_the_safe_spot_and_adds_hunger() {
        let mut pack = [0u8; 40];
        let (facts, mut hero, mut body) = calm_hero(&mut pack);
        let steps = survival_step(&facts, &mut hero, &mut body);
        assert_eq!((hero.safe_r, hero.safe_x, hero.safe_y), (3, 1234, 5678));
        assert_eq!((hero.hunger, hero.fatigue), (1, 1));
        assert!(steps.is_empty());
    }

    #[test]
    fn danger_stops_the_safe_spot_but_not_the_hunger() {
        let mut pack = [0u8; 40];

        for danger in 0..4 {
            let (mut facts, mut hero, mut body) = calm_hero(&mut pack);

            match danger {
                0 => facts.actors_on_screen = true,
                1 => facts.actors_loading = true,
                2 => facts.witch_on_screen = true,
                _ => hero.safe_flag = 1,
            }

            survival_step(&facts, &mut hero, &mut body);
            assert_eq!(hero.safe_r, 0);
            assert_eq!(hero.hunger, 1);
        }
    }

    #[test]
    fn a_hungry_hero_eats_fruit_at_the_safe_spot() {
        let mut pack = [0u8; 40];
        pack[FRUIT_SLOT] = 2;
        let (facts, mut hero, mut body) = calm_hero(&mut pack);
        hero.hunger = 40;
        let steps = survival_step(&facts, &mut hero, &mut body);
        assert_eq!(steps, vec![SurvivalStep::Event(EVENT_ATE_FRUIT)]);
        assert_eq!(hero.hunger, 40 - FRUIT_FOOD + 1);
        assert_eq!(pack[FRUIT_SLOT], 1);
    }

    #[test]
    fn music_is_picked_every_eighth_frame_outside_battle() {
        let mut pack = [0u8; 40];
        let (mut facts, mut hero, mut body) = calm_hero(&mut pack);
        facts.clock = 8;
        facts.battle = false;
        assert_eq!(
            survival_step(&facts, &mut hero, &mut body),
            vec![SurvivalStep::PickMusic]
        );

        facts.battle = true;
        assert!(survival_step(&facts, &mut hero, &mut body).is_empty());
    }

    #[test]
    fn hunger_messages_come_in_the_original_order() {
        let mut pack = [0u8; 40];
        let (mut facts, mut hero, mut body) = calm_hero(&mut pack);
        facts.clock = 0; // also a music frame; battle suppresses it
        hero.hunger = HUNGRY - 1;
        assert_eq!(
            survival_step(&facts, &mut hero, &mut body),
            vec![SurvivalStep::Event(0)]
        );

        // damage is shown (stats) before the starving messages, and the tired one comes last
        hero.hunger = 103;
        hero.fatigue = TIRED_AT - 1;
        let steps = survival_step(&facts, &mut hero, &mut body);
        assert_eq!(
            steps,
            vec![
                SurvivalStep::RedrawStats,
                SurvivalStep::Event(2),
                SurvivalStep::Event(3)
            ]
        );

        assert_eq!(body.vitality, 48);
    }

    #[test]
    fn no_hunger_while_asleep_or_dead() {
        let mut pack = [0u8; 40];
        let (facts, mut hero, mut body) = calm_hero(&mut pack);
        body.state = SLEEP;
        survival_step(&facts, &mut hero, &mut body);
        assert_eq!(hero.hunger, 0);
        body.state = STILL;
        body.vitality = 0;
        survival_step(&facts, &mut hero, &mut body);
        assert_eq!(hero.hunger, 0);
    }

    #[test]
    fn survival_schedule() {
        let fires: Vec<u16> = (0..=384).filter(|c| hunger_due(*c, 10, STILL)).collect();
        assert_eq!(fires, vec![0, 128, 256, 384]);
        assert!(!hunger_due(128, 0, STILL));
        assert!(!hunger_due(128, 10, SLEEP));
        assert!(mood_due(8, false) && mood_due(0, false));
        assert!(!mood_due(9, false) && !mood_due(8, true));
        assert!(eats_fruit(31, 1) && !eats_fruit(30, 1) && !eats_fruit(50, 0));
    }

    #[test]
    fn hunger_messages_at_the_first_two_thresholds() {
        let (mut hero, mut body) = hero_and_body(HUNGRY, 0, 50);
        assert_eq!(hunger_effects(&mut hero, &mut body).events, vec![0]);
        let (mut hero, mut body) = hero_and_body(VERY_HUNGRY, 0, 50);
        assert_eq!(hunger_effects(&mut hero, &mut body).events, vec![1]);
    }

    #[test]
    fn nothing_happens_between_eights_when_only_a_little_hungry() {
        let (mut hero, mut body) = hero_and_body(41, 0, 50);
        let report = hunger_effects(&mut hero, &mut body);
        assert_eq!(report, HungerReport::default());
        assert_eq!(body.vitality, 50);
    }

    #[test]
    fn tired_hero_loses_vitality_without_a_message_until_starving() {
        let (mut hero, mut body) = hero_and_body(88, MUCH_FATIGUE + 1, 50);
        let report = hunger_effects(&mut hero, &mut body);
        assert_eq!((body.vitality, report.redraw_stats), (48, true));
        assert!(report.events.is_empty());
        let (mut hero, mut body) = hero_and_body(96, MUCH_FATIGUE + 1, 50);
        let report = hunger_effects(&mut hero, &mut body);
        assert_eq!((body.vitality, report.events), (48, vec![2]));
    }

    #[test]
    fn weak_and_exhausted_hero_collapses_asleep() {
        let (mut hero, mut body) = hero_and_body(80, COLLAPSE_FATIGUE + 1, WEAK_VITALITY);
        let report = hunger_effects(&mut hero, &mut body);
        assert_eq!((report.events, body.state), (vec![12], SLEEP));
    }

    #[test]
    fn weak_and_starving_hero_faints_and_hunger_resets() {
        let (mut hero, mut body) =
            hero_and_body(FAINT_HUNGER + 8 - FAINT_HUNGER % 8, 0, WEAK_VITALITY);
        let report = hunger_effects(&mut hero, &mut body);
        assert_eq!(
            (report.events, body.state, hero.hunger),
            (vec![24], SLEEP, FAINT_RESET)
        );
    }

    #[test]
    fn weak_hero_not_yet_faint_is_left_alone() {
        let (mut hero, mut body) = hero_and_body(120, 0, WEAK_VITALITY);
        assert_eq!(
            hunger_effects(&mut hero, &mut body),
            HungerReport::default()
        );
    }
}
