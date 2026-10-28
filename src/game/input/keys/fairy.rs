//! The good fairy: after the hero dies she flies in and revives him, or the next brother takes over.

use super::{
    FAIRY_ARRIVES, FAIRY_GIVES_UP, FAIRY_IMAGE, FAIRY_RACE, FAIRY_RESURRECT, FAIRY_SLOT,
    FAIRY_SPEED, FAIRY_X_OFFSET, FALL, NO_ACTORS_LOADED, OBJECTS, STILL,
};
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;

use crate::game::actors::revive::revive;

/// The hero is dead or falling: the fairy flies in, then the hero revives (or, with no luck left,
/// the next brother takes over). Returns the hero's new state.
pub(super) fn fairy_revival(game: &mut Game, state: i8, cycle: i16) -> i8 {
    let (timer, action) = fairy_step(
        game.hero.good_fairy_timer,
        game.hero.luck,
        game.actors.list[0].state == FALL,
    );

    game.hero.good_fairy_timer = timer;

    match action {
        FairyAction::Revive => {
            revive(game, 0);

            return STILL;
        }

        FairyAction::NextBrother => {
            revive(game, 1);

            return STILL;
        }

        FairyAction::Wait => {}
        FairyAction::FlyIn => place_fairy(game, cycle),
    }

    state
}

/// Puts the fairy near the hero, closer each frame, and clears the other actors away.
fn place_fairy(game: &mut Game, cycle: i16) {
    let fairy = ActorRef::new(FAIRY_SLOT);
    game.actors.active = NO_ACTORS_LOADED;
    game.actors.list[fairy.slot()].abs_x = game
        .hero
        .x
        .wrapping_add(game.hero.good_fairy_timer as u16 * FAIRY_SPEED)
        .wrapping_sub(FAIRY_X_OFFSET);
    game.actors.list[fairy.slot()].abs_y = game.hero.y;
    game.actors.list[fairy.slot()].kind = OBJECTS;

    // Masks with FAIRY_IMAGE, keeping only the bits that matter here.
    game.actors.list[fairy.slot()].index = FAIRY_IMAGE + (cycle & 1) as i8;
    game.actors.list[fairy.slot()].state = STILL;
    game.actors.list[fairy.slot()].weapon = 0;
    game.actors.list[fairy.slot()].environ = 0;
    game.actors.list[fairy.slot()].race = FAIRY_RACE;
    game.actors.on_screen = 1;
    game.actors.battle_flag = 0;
}

/// What the good fairy does this frame.
#[derive(Debug, PartialEq)]
pub(super) enum FairyAction {
    /// The hero gets up where they are.
    Revive,
    /// No luck is left: the next brother takes over.
    NextBrother,
    /// The fairy flies in toward the hero.
    FlyIn,
    /// Nothing yet (or the resurrection glow).
    Wait,
}

/// One step of the fairy countdown for a dead or fallen hero: returns the new timer value and
/// what to do. At timer 1 the hero is revived; below `FAIRY_RESURRECT` it is just the glow;
/// below `FAIRY_GIVES_UP` the hero is revived (a fallen one) or replaced (no luck); below
/// `FAIRY_ARRIVES` the fairy flies in.
pub(super) fn fairy_step(timer: u8, luck: i16, falling: bool) -> (u8, FairyAction) {
    if timer == 1 {
        return (timer, FairyAction::Revive);
    }

    let timer = timer.wrapping_sub(1);
    let action = if timer < FAIRY_RESURRECT {
        FairyAction::Wait
    } else if luck < 1 && timer < FAIRY_GIVES_UP {
        FairyAction::NextBrother
    } else if falling && timer < FAIRY_GIVES_UP {
        FairyAction::Revive
    } else if timer < FAIRY_ARRIVES {
        FairyAction::FlyIn
    } else {
        FairyAction::Wait
    };

    (timer, action)
}

#[cfg(test)]
mod tests {
    use super::super::{FAIRY_ARRIVES, FAIRY_GIVES_UP, FAIRY_RESURRECT};
    use super::*;

    #[test]
    fn the_hero_revives_when_the_countdown_reaches_one() {
        assert_eq!(fairy_step(1, 50, false), (1, FairyAction::Revive));
    }

    #[test]
    fn the_fairy_flies_in_once_the_timer_is_low_enough() {
        assert_eq!(
            fairy_step(FAIRY_ARRIVES + 5, 50, false).1,
            FairyAction::Wait
        );

        assert_eq!(
            fairy_step(FAIRY_ARRIVES, 50, false),
            (FAIRY_ARRIVES - 1, FairyAction::FlyIn)
        );
    }

    #[test]
    fn a_fallen_hero_is_revived_before_the_fairy_arrives() {
        assert_eq!(fairy_step(FAIRY_GIVES_UP, 50, true).1, FairyAction::Revive);
    }

    #[test]
    fn with_no_luck_the_next_brother_takes_over() {
        assert_eq!(
            fairy_step(FAIRY_GIVES_UP, 0, false).1,
            FairyAction::NextBrother
        );

        // having luck left, a dead hero waits for the fairy instead
        assert_eq!(fairy_step(FAIRY_GIVES_UP, 1, false).1, FairyAction::Wait);
    }

    #[test]
    fn the_last_moments_are_only_a_glow() {
        assert_eq!(fairy_step(FAIRY_RESURRECT, 0, true).1, FairyAction::Wait);
    }
}
