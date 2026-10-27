//! The Talk and Yell menu entries: who the hero is talking to and what answers.

use super::*;

// Actor kinds and states (csrc/ftale.h).
const ENEMY: i8 = 2;
const SETFIG: i8 = 4;
const CARRIER: i8 = 5;
const DEAD: i8 = 15;
const TALKING: i8 = 19;
/// The tactic that makes a person stand and talk.
const TACTIC_TALK: i8 = 15;
const SHELL: usize = 6;
/// `find_nearest_figure` search ranges for yelling and for the quieter talk options.
const YELL_RANGE: i16 = 100;
const TALK_RANGE: i16 = 50;
/// Anyone closer than this is told off for yelling.
const TOO_CLOSE_TO_YELL: i32 = 35;

// Talk menu entries and the people (`race & 0x7f`) that can be talked to.
const YELL: i16 = 5;
const CARRIER_TURTLE: i16 = 5;

/// Yell (louder, reaches further) or say something to the nearest person.
pub(super) unsafe fn talk_option(chosen_entry: i16) -> bool {
    unsafe {
        let distance = find_nearest_figure(
            1,
            if chosen_entry == YELL {
                YELL_RANGE
            } else {
                TALK_RANGE
            },
        );
        if the_actors.nearest_object == 0 {
            return true;
        }
        let an = &raw mut the_actors.list[the_actors.nearest_object as usize];
        let person = (*an).race & 0x7f;
        let can_talk = (*(&raw const setfig_table)
            .cast::<SetFigEntry>()
            .add(person as usize))
        .can_talk
            != 0;
        let target = Target {
            kind: (*an).kind,
            state: (*an).state,
            race: (*an).race,
            can_talk,
        };
        match talk_action(
            &target,
            chosen_entry == YELL,
            distance,
            the_actors.active_carrier,
            *the_hero.stuff.add(SHELL) != 0,
        ) {
            TalkAction::Nothing => {}
            TalkAction::TooCloseToYell => speak(8),
            TalkAction::Person { starts_talking } => {
                if starts_talking {
                    (*an).state = TALKING;
                    (*an).tactic = TACTIC_TALK;
                }
                person_speaks(person, an);
            }
            TalkAction::TurtleHasShell => speak(57),
            TalkAction::TurtleGivesShell => {
                *the_hero.stuff.add(SHELL) = 1;
                speak(56);
            }
            TalkAction::EnemyShouts(race) => speak(race as i32),
        }
        true
    }
}

/// Who the hero is talking to.
struct Target {
    kind: i8,
    state: i8,
    race: u8,
    /// The set figure's table entry says it can talk.
    can_talk: bool,
}

/// What a Talk or Yell does.
#[derive(Debug, PartialEq, Eq)]
enum TalkAction {
    Nothing,
    /// Yelling at someone standing right next to you.
    TooCloseToYell,
    /// A person answers (and turns to talk if it can).
    Person {
        starts_talking: bool,
    },
    TurtleHasShell,
    TurtleGivesShell,
    /// An enemy says its line (the speech number is its race).
    EnemyShouts(u8),
}

fn talk_action(
    target: &Target,
    yelling: bool,
    distance: i32,
    active_carrier: i16,
    has_shell: bool,
) -> TalkAction {
    if target.state == DEAD {
        return TalkAction::Nothing;
    }
    match target.kind {
        SETFIG if yelling && (distance as u32) < TOO_CLOSE_TO_YELL as u32 => {
            TalkAction::TooCloseToYell
        }
        SETFIG => TalkAction::Person {
            starts_talking: target.can_talk,
        },
        CARRIER if active_carrier == CARRIER_TURTLE => {
            if has_shell {
                TalkAction::TurtleHasShell
            } else {
                TalkAction::TurtleGivesShell
            }
        }
        ENEMY => TalkAction::EnemyShouts(target.race),
        _ => TalkAction::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(kind: i8, state: i8, can_talk: bool) -> Target {
        Target {
            kind,
            state,
            race: 7,
            can_talk,
        }
    }

    #[test]
    fn talking() {
        use TalkAction::*;
        let figure = target(SETFIG, 0, true);
        assert_eq!(
            talk_action(&figure, false, 10, 0, false),
            Person {
                starts_talking: true
            }
        );
        assert_eq!(
            talk_action(&target(SETFIG, 0, false), false, 10, 0, false),
            Person {
                starts_talking: false
            }
        );
        // yelling at someone close is refused; far enough is fine; talking is always fine
        assert_eq!(
            talk_action(&figure, true, TOO_CLOSE_TO_YELL - 1, 0, false),
            TooCloseToYell
        );
        assert_eq!(
            talk_action(&figure, true, TOO_CLOSE_TO_YELL, 0, false),
            Person {
                starts_talking: true
            }
        );
        assert_eq!(
            talk_action(&figure, false, 1, 0, false),
            Person {
                starts_talking: true
            }
        );
        // the dead say nothing
        assert_eq!(
            talk_action(&target(SETFIG, DEAD, true), false, 10, 0, false),
            Nothing
        );
        // the turtle gives the shell once
        let turtle = target(CARRIER, 0, false);
        assert_eq!(
            talk_action(&turtle, false, 10, CARRIER_TURTLE, false),
            TurtleGivesShell
        );
        assert_eq!(
            talk_action(&turtle, false, 10, CARRIER_TURTLE, true),
            TurtleHasShell
        );
        assert_eq!(
            talk_action(&turtle, false, 10, CARRIER_TURTLE + 1, false),
            Nothing
        );
        assert_eq!(
            talk_action(&target(ENEMY, 0, false), false, 10, 0, false),
            EnemyShouts(7)
        );
    }
}
