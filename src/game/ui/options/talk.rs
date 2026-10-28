//! The Talk and Yell menu entries: who the hero is talking to and what answers.

use super::speeches::person_speaks;
use crate::game::actors::interaction::find_nearest_figure;
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::data::state::the_actors;
use crate::game::data::tables::setfig_table;
use crate::game::ui::text::speak;
use crate::game::world::objects::SetFigEntry;

// Actor kinds and states (csrc/ftale.h).
const ENEMY: i8 = 2;
const SETFIG: i8 = 4;
const CARRIER: i8 = 5;
const DEAD: i8 = 15;
const TALKING: i8 = 19;

/// The tactic that makes a person stand and talk.
const TACTIC_TALK: i8 = 15;
const SHELL: usize = 6;

/// A person's race number is the low 7 bits of `race` (bit 7 marks a variant).
const PERSON_MASK: u8 = 0x7f;

// Speech numbers.
const SPEECH_TOO_CLOSE_TO_YELL: i32 = 8;
const SPEECH_TURTLE_HAS_SHELL: i32 = 57;
const SPEECH_TURTLE_GIVES_SHELL: i32 = 56;

/// `find_nearest_figure` search ranges for yelling and for the quieter talk options.
const YELL_RANGE: i16 = 100;
const TALK_RANGE: i16 = 50;

/// Anyone closer than this is told off for yelling.
const TOO_CLOSE_TO_YELL: i32 = 35;

// Talk menu entries and the people (`race & 0x7f`) that can be talked to.
const YELL: i16 = 5;
const CARRIER_TURTLE: i16 = 5;

/// Yell (louder, reaches further) or say something to the nearest person.
pub(super) unsafe fn talk_option(game: &mut Game, chosen_entry: i16) -> bool {
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

        let actor = ActorRef::new(the_actors.nearest_object as usize);
        let target = describe_target(actor);

        let action = talk_action(
            &target,
            chosen_entry == YELL,
            distance,
            the_actors.active_carrier,
            game.hero.stuff()[SHELL] != 0,
        );

        carry_out(game, action, actor);

        true
    }
}

/// What the Talk code needs to know about the actor the hero is talking to.
unsafe fn describe_target(actor: ActorRef) -> Target {
    unsafe {
        // Masks with PERSON_MASK, keeping only the bits that matter here.
        let person = actor.race & PERSON_MASK;
        let can_talk = (*(&raw const setfig_table)
            .cast::<SetFigEntry>()
            .add(person as usize))
        .can_talk
            != 0;

        Target {
            kind: actor.kind,
            state: actor.state,
            race: actor.race,
            can_talk,
        }
    }
}

/// Does what `talk_action` decided: speaks, and turns the person to talk or hands over the shell.
unsafe fn carry_out(game: &mut Game, action: TalkAction, mut actor: ActorRef) {
    unsafe {
        match action {
            TalkAction::Nothing => {}
            TalkAction::TooCloseToYell => speak(game, SPEECH_TOO_CLOSE_TO_YELL),
            TalkAction::Person { starts_talking } => {
                if starts_talking {
                    actor.state = TALKING;
                    actor.tactic = TACTIC_TALK;
                }

                // Masks with PERSON_MASK, keeping only the bits that matter here.
                person_speaks(game, actor.race & PERSON_MASK, actor);
            }

            TalkAction::TurtleHasShell => speak(game, SPEECH_TURTLE_HAS_SHELL),
            TalkAction::TurtleGivesShell => {
                game.hero.stuff_mut()[SHELL] = 1;
                speak(game, SPEECH_TURTLE_GIVES_SHELL);
            }

            TalkAction::EnemyShouts(race) => speak(game, race as i32),
        }
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

/// What a Talk or Yell does to the figure nearby. The dead say nothing; people answer (but not a yell
/// from right next to them); the turtle gives the shell once; an enemy shouts its line.
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
    fn people_answer_when_talked_to() {
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

        assert_eq!(
            talk_action(&figure, false, 1, 0, false),
            Person {
                starts_talking: true
            }
        );
    }

    #[test]
    fn yelling_at_someone_close_is_refused() {
        use TalkAction::*;
        let figure = target(SETFIG, 0, true);

        // far enough is fine; talking is always fine
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
    }

    #[test]
    fn the_dead_say_nothing() {
        use TalkAction::*;
        assert_eq!(
            talk_action(&target(SETFIG, DEAD, true), false, 10, 0, false),
            Nothing
        );
    }

    #[test]
    fn the_turtle_gives_the_shell_once() {
        use TalkAction::*;
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
    }

    #[test]
    fn enemies_shout_their_line() {
        use TalkAction::*;
        assert_eq!(
            talk_action(&target(ENEMY, 0, false), false, 10, 0, false),
            EnemyShouts(7)
        );
    }
}
