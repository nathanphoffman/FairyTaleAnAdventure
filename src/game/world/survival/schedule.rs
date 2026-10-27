//! The survival tick: noting the safe spot, picking the music and applying hunger once a frame.

use super::hunger::{eats_fruit, hunger_due, hunger_effects};
use super::{MOOD_PERIOD_MASK, STARVING_AT, safe_check_due};
use crate::game::data::state::{Hero, the_actors, the_hero, the_witch, the_world};
use crate::game::data::types::Shape;
use crate::game::ui::music::set_mood;
use crate::game::ui::text::{queue_print, show_event};

// Actor kinds and states (csrc/ftale.h).
const DEAD: i8 = 15;

// Print queue entries.
const PRQ_STATS: i32 = 4;
const FRUIT_SLOT: usize = 24;
const FRUIT_FOOD: i16 = 30;
const EVENT_ATE_FRUIT: i32 = 37;
const TIRED_AT: i16 = 70;
const EVENT_TIRED: i32 = 3;
const EVENT_STARVING: i32 = 4;

fn mood_due(clock: u16, battle: bool) -> bool {
    // Masks with MOOD_PERIOD_MASK, keeping only the bits that matter here.
    clock & MOOD_PERIOD_MASK == 0 && !battle
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
    pub(super) clock: u16,
    pub(super) region: u16,
    pub(super) actors_on_screen: bool,
    pub(super) actors_loading: bool,
    pub(super) witch_on_screen: bool,
    pub(super) battle: bool,
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

    if safe_check_due(facts.clock) && is_safe_place(facts, hero, body) {
        note_safe_spot(facts, hero, &mut steps);
    }

    if mood_due(facts.clock, facts.battle) {
        steps.push(SurvivalStep::PickMusic);
    }

    if hunger_due(facts.clock, body.vitality, body.state) {
        add_hunger(hero, body, &mut steps);
    }

    steps
}

/// A safe spot can be noted when nothing is around, the hero is on dry land and alive.
fn is_safe_place(facts: &SurvivalFacts, hero: &Hero, body: &Shape) -> bool {
    !facts.actors_on_screen
        && !facts.actors_loading
        && !facts.witch_on_screen
        && body.environ == 0
        && hero.safe_flag == 0
        && body.state != DEAD
}

/// Remembers where the hero stands as the safe spot, and eats fruit if hungry enough.
fn note_safe_spot(facts: &SurvivalFacts, hero: &mut Hero, steps: &mut Vec<SurvivalStep>) {
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

/// One more point of hunger and tiredness, with what that does and the warnings that come with it.
fn add_hunger(hero: &mut Hero, body: &mut Shape, steps: &mut Vec<SurvivalStep>) {
    hero.hunger += 1;
    hero.fatigue += 1;
    let report = hunger_effects(hero, body);

    if report.redraw_stats {
        steps.push(SurvivalStep::RedrawStats);
    }

    steps.extend(report.events.into_iter().map(SurvivalStep::Event));

    if hero.fatigue == TIRED_AT {
        steps.push(SurvivalStep::Event(EVENT_TIRED));
    } else if hero.hunger == STARVING_AT {
        steps.push(SurvivalStep::Event(EVENT_STARVING));
    }
}

#[cfg(test)]
mod tests {
    use super::super::hunger::{HUNGRY, SLEEP, eats_fruit, hunger_due};
    use super::*;

    const STILL: i8 = 13;

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
                SurvivalStep::Event(EVENT_TIRED)
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
}
