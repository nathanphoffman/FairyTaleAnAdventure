//! What each kind of person says when talked to. Some also change things (the priest heals, the
//! sorceress gives luck).

use super::{PRQ_STATS, PRQ_VITALITY};
use crate::game::data::random::rand64;
use crate::game::data::state::ActorRef;
use crate::game::data::state::Game;
use crate::game::data::state::Video;
use crate::game::ui::text::{queue_print, speak};

const WRIT: usize = 28;
const WIZARD: u8 = 0;
const PRIEST: u8 = 1;
const GUARD_FRONT: u8 = 2;
const GUARD_BACK: u8 = 3;
const PRINCESS: u8 = 4;
const KING: u8 = 5;
const NOBLE: u8 = 6;
const SORCERESS: u8 = 7;
const BARTENDER: u8 = 8;
const WITCH: u8 = 9;
const SPECTRE: u8 = 10;
const GHOST: u8 = 11;
const RANGER: u8 = 12;
const BEGGAR: u8 = 13;

// Speech numbers (the lines in the narration table).
const SPEECH_WIZARD_UNKIND: i32 = 35;
const SPEECH_WIZARD_BASE: i32 = 27;
const SPEECH_GUARD: i32 = 15;
const SPEECH_PRINCESS: i32 = 16;
const SPEECH_KING: i32 = 17;
const SPEECH_NOBLE: i32 = 20;
const SPEECH_BARTENDER_RESTED: i32 = 13;
const SPEECH_BARTENDER_NIGHT: i32 = 12;
const SPEECH_BARTENDER_EVENING: i32 = 14;
const SPEECH_WITCH: i32 = 46;
const SPEECH_SPECTRE: i32 = 47;
const SPEECH_GHOST: i32 = 49;
const SPEECH_RANGER_SWAMP: i32 = 22;
const SPEECH_RANGER_BASE: i32 = 53;
const SPEECH_BEGGAR: i32 = 23;
const SPEECH_SORCERESS_HELP: i32 = 45;
const SPEECH_PRIEST_WRIT_REWARD: i32 = 39;
const SPEECH_PRIEST_WRIT_AGAIN: i32 = 19;
const SPEECH_PRIEST_SCOLD: i32 = 40;
const SPEECH_PRIEST_BLESSING_BASE: i32 = 36;

/// Kindness the hero needs for the wizard and priest to be friendly.
const KIND_ENOUGH: i16 = 10;

/// Object list slots whose state the conversations change.
const PRIEST_WRIT_SLOT: usize = 10;
const SORCERESS_SLOT: usize = 9;
const PRINCESS_FREED_SLOT: usize = 9;

/// Luck given by the sorceress, and the roll it must beat.
const SORCERESS_LUCK: i16 = 5;

/// Hero vitality when healed by the priest is `15 + brave / 4`.
const FULL_VITALITY_BASE: i16 = 15;
const PRIEST_BLESSINGS: u16 = 3;
const BRAVERY_DIVISOR: i16 = 4;
const NIGHT_PERIOD: i16 = 7;
const TIRED: i16 = 5;
const RANGER_SWAMP_REGION: u16 = 2;

/// What each kind of person says. Some of them also change the world (the priest heals, the
/// sorceress gives luck).
pub(super) fn person_speaks(game: &mut Game, video: &mut Video, person: u8, actor: ActorRef) {
    match person {
        PRIEST => priest_speaks(game, video),
        SORCERESS => sorceress_speaks(game, video),
        _ => {
            let facts = Talker {
                kindness: game.hero.kindness,
                fatigue: game.hero.fatigue,
                day_period: game.world.day_period,
                region: game.world.region,
                goal: game.actor(actor).goal as i32,
                princess_freed: game.objects.region[8][PRINCESS_FREED_SLOT].ob_stat != 0,
            };

            if let Some(line) = plain_speech(person, &facts) {
                speak(game, video, line);
            }
        }
    }
}

/// What decides the lines of the people who only talk.
struct Talker {
    kindness: i16,
    fatigue: i16,
    day_period: i16,
    region: u16,
    /// The person's goal, which picks one of several lines for the wizard and the ranger.
    goal: i32,
    princess_freed: bool,
}

/// The speech number a person says, if they say anything. The priest and the sorceress are not here
/// because they also change things.
fn plain_speech(person: u8, t: &Talker) -> Option<i32> {
    match person {
        WIZARD => Some(if t.kindness < KIND_ENOUGH {
            SPEECH_WIZARD_UNKIND
        } else {
            SPEECH_WIZARD_BASE + t.goal
        }),
        GUARD_FRONT | GUARD_BACK => Some(SPEECH_GUARD),
        PRINCESS if t.princess_freed => Some(SPEECH_PRINCESS),
        KING if t.princess_freed => Some(SPEECH_KING),
        NOBLE => Some(SPEECH_NOBLE),
        BARTENDER => Some(if t.fatigue < TIRED {
            SPEECH_BARTENDER_RESTED
        } else if t.day_period > NIGHT_PERIOD {
            SPEECH_BARTENDER_NIGHT
        } else {
            SPEECH_BARTENDER_EVENING
        }),
        WITCH => Some(SPEECH_WITCH),
        SPECTRE => Some(SPEECH_SPECTRE),
        GHOST => Some(SPEECH_GHOST),
        RANGER => Some(if t.region == RANGER_SWAMP_REGION {
            SPEECH_RANGER_SWAMP
        } else {
            SPEECH_RANGER_BASE + t.goal
        }),
        BEGGAR => Some(SPEECH_BEGGAR),
        _ => None,
    }
}

/// The sorceress gives luck once she has been freed; before that she asks for help.
fn sorceress_speaks(game: &mut Game, video: &mut Video) {
    let freed = game.objects.global[SORCERESS_SLOT].ob_stat != 0;

    match sorceress_action(freed, game.hero.luck, || rand64(&mut game.rng) as i16) {
        SorceressAction::GiveLuck => game.hero.luck += SORCERESS_LUCK,
        SorceressAction::Nothing => {}
        SorceressAction::AskForHelp => {
            speak(game, video, SPEECH_SORCERESS_HELP);
            game.objects.global[SORCERESS_SLOT].ob_stat = 1;
        }
    }

    queue_print(&mut game.ui, PRQ_VITALITY);
}

#[derive(Debug, PartialEq, Eq)]
enum SorceressAction {
    /// She is free and the roll was kind to the hero.
    GiveLuck,
    /// She is free but the roll was not high enough.
    Nothing,
    /// First meeting.
    AskForHelp,
}

/// `roll` is a 0-63 draw, taken only once she is free: luck is given when the hero's luck is below it.
fn sorceress_action(freed: bool, luck: i16, roll: impl FnOnce() -> i16) -> SorceressAction {
    if !freed {
        SorceressAction::AskForHelp
    } else if luck < roll() {
        SorceressAction::GiveLuck
    } else {
        SorceressAction::Nothing
    }
}

/// The priest gives the writ's reward, scolds the unkind, or heals.
fn priest_speaks(game: &mut Game, video: &mut Video) {
    let has_writ = game.hero.stuff()[WRIT] != 0;

    match priest_action(
        has_writ,
        game.objects.global[PRIEST_WRIT_SLOT].ob_stat,
        game.hero.kindness,
        game.world.day_clock,
    ) {
        PriestAction::WritReward => {
            speak(game, video, SPEECH_PRIEST_WRIT_REWARD);
            game.objects.global[PRIEST_WRIT_SLOT].ob_stat = 1;
        }

        PriestAction::WritAgain => speak(game, video, SPEECH_PRIEST_WRIT_AGAIN),
        PriestAction::Scold => speak(game, video, SPEECH_PRIEST_SCOLD),
        PriestAction::Bless { speech } => {
            speak(game, video, speech);
            game.actors.list[0].vitality = FULL_VITALITY_BASE + game.hero.brave / BRAVERY_DIVISOR;
            queue_print(&mut game.ui, PRQ_STATS);
        }
    }
}

/// What the priest does for the hero.
#[derive(Debug, PartialEq, Eq)]
enum PriestAction {
    /// Has the writ and the reward (object list slot) is still hidden: it appears.
    WritReward,
    /// Has the writ, reward already given.
    WritAgain,
    /// Not kind enough.
    Scold,
    /// Heals fully, with one of three blessings picked by the time of day.
    Bless { speech: i32 },
}

/// What the priest does: give the writ's reward once, say so again, scold an unkind hero, or bless and
/// heal one who is kind enough. The blessing is picked from the clock.
fn priest_action(has_writ: bool, reward_stat: i8, kindness: i16, clock: u16) -> PriestAction {
    if has_writ {
        if reward_stat == 0 {
            PriestAction::WritReward
        } else {
            PriestAction::WritAgain
        }
    } else if kindness < KIND_ENOUGH {
        PriestAction::Scold
    } else {
        PriestAction::Bless {
            speech: SPEECH_PRIEST_BLESSING_BASE + (clock % PRIEST_BLESSINGS) as i32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priest_rules() {
        assert_eq!(priest_action(true, 0, 0, 0), PriestAction::WritReward);
        assert_eq!(priest_action(true, 1, 99, 0), PriestAction::WritAgain);
        assert_eq!(
            priest_action(false, 0, KIND_ENOUGH - 1, 0),
            PriestAction::Scold
        );

        assert_eq!(
            priest_action(false, 0, KIND_ENOUGH, 0),
            PriestAction::Bless { speech: 36 }
        );

        assert_eq!(
            priest_action(false, 0, KIND_ENOUGH, 1),
            PriestAction::Bless { speech: 37 }
        );

        let wrap = PRIEST_BLESSINGS;
        assert_eq!(
            priest_action(false, 0, KIND_ENOUGH, wrap),
            PriestAction::Bless { speech: 36 }
        );
    }

    fn talker() -> Talker {
        Talker {
            kindness: 20,
            fatigue: 0,
            day_period: 3,
            region: 0,
            goal: 2,
            princess_freed: false,
        }
    }

    #[test]
    fn the_wizard_is_short_with_the_unkind() {
        let t = talker();
        assert_eq!(plain_speech(WIZARD, &t), Some(29));
        assert_eq!(
            plain_speech(
                WIZARD,
                &Talker {
                    kindness: KIND_ENOUGH - 1,
                    ..talker()
                }
            ),
            Some(35)
        );
    }

    #[test]
    fn guards_and_royalty_speak_once_the_princess_is_free() {
        let t = talker();
        assert_eq!(plain_speech(GUARD_BACK, &t), Some(15));
        assert_eq!(plain_speech(PRINCESS, &t), None);
        assert_eq!(
            plain_speech(
                PRINCESS,
                &Talker {
                    princess_freed: true,
                    ..talker()
                }
            ),
            Some(16)
        );

        assert_eq!(
            plain_speech(
                KING,
                &Talker {
                    princess_freed: true,
                    ..talker()
                }
            ),
            Some(17)
        );
    }

    #[test]
    fn the_ranger_depends_on_the_region() {
        let t = talker();
        assert_eq!(plain_speech(RANGER, &t), Some(55));
        assert_eq!(
            plain_speech(
                RANGER,
                &Talker {
                    region: RANGER_SWAMP_REGION,
                    ..talker()
                }
            ),
            Some(22)
        );
    }

    #[test]
    fn the_priest_and_sorceress_say_nothing_plainly() {
        let t = talker();
        assert_eq!(plain_speech(PRIEST, &t), None);
        assert_eq!(plain_speech(SORCERESS, &t), None);
    }

    #[test]
    fn the_bartender_notices_tiredness_and_the_hour() {
        let said = |fatigue, day_period| {
            plain_speech(
                BARTENDER,
                &Talker {
                    fatigue,
                    day_period,
                    ..talker()
                },
            )
        };

        assert_eq!(said(TIRED - 1, 0), Some(13));
        assert_eq!(said(TIRED, NIGHT_PERIOD), Some(14));
        assert_eq!(said(TIRED, NIGHT_PERIOD + 1), Some(12));
    }

    #[test]
    fn the_sorceress() {
        use SorceressAction::*;
        assert_eq!(sorceress_action(false, 0, || unreachable!()), AskForHelp);
        assert_eq!(sorceress_action(true, 10, || 11), GiveLuck);
        assert_eq!(sorceress_action(true, 10, || 10), Nothing);
    }
}
