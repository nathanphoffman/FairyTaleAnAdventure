//! The Magic menu actions (from `choose_menu_entry` in csrc/fmain.c): casting the spells the hero has
//! stones, vials and so on for. Each use takes one from the stack.

mod effects;

use self::effects::{heal, scare_enemies, show_map, stone_jump};
use crate::game::data::state::Game;
use crate::game::data::state::current_extent;
use crate::game::data::state::player;

use crate::game::data::state::{the_hero, the_world};
use crate::game::ui::menus::update_menu_availability;
use crate::game::ui::text::{show_event, speak};

const SPELL_STONE: i16 = 5;
const SPELL_LIGHT: i16 = 6;
const SPELL_HEAL: i16 = 7;
const SPELL_SECRET: i16 = 8;
const SPELL_MAP: i16 = 9;
const SPELL_FREEZE: i16 = 10;
const SPELL_FEAR: i16 = 11;
const FIRST_SPELL: i16 = 5;
const SPELL_SLOT_BASE: usize = 4;
const LIGHT_TIME: i16 = 760;
const SECRET_TIME: i16 = 360;
const FREEZE_TIME: i16 = 100;

/// Where the spells cannot be used: the extent type with this `v3` (a quiet place).
const NO_MAGIC_ZONE: u8 = 9;
const NO_MAGIC_SPEECH: i32 = 59;
const NO_SPELL_EVENT: i32 = 21;

/// Regions above this are indoors (no overview map there).
const LAST_OUTDOOR_REGION: u16 = 7;

/// Riding something bigger than the raft stops the freeze spell.
const RIDING_BIG: i16 = 1;

/// Handles an entry of the Magic menu. Returns false if `update_menu_availability` should not follow
/// (the spell did not work, so the stone is not used up).
pub unsafe fn magic_option(game: &mut Game, chosen_entry: i16) -> bool {
    unsafe {
        if let Some(result) = refuse_spell(game, chosen_entry) {
            return result;
        }

        if !cast_spell(game, chosen_entry) {
            return false;
        }

        use_up_one(chosen_entry);

        true
    }
}

/// Checks that the spell can be cast here and now. Gives `Some(result)` for `magic_option` to
/// return when it cannot, and `None` when the spell goes ahead.
unsafe fn refuse_spell(game: &mut Game, chosen_entry: i16) -> Option<bool> {
    unsafe {
        let count = if chosen_entry < FIRST_SPELL {
            0
        } else {
            player().stuff()[SPELL_SLOT_BASE + chosen_entry as usize]
        };

        let gate = spell_gate(
            chosen_entry,
            count,
            current_extent().v3,
            the_hero.riding,
            game.ui.cheat_enabled,
            the_world.region,
        );

        match gate {
            SpellGate::NoSpell => {
                show_event(game, NO_SPELL_EVENT);

                Some(true)
            }

            SpellGate::QuietZone => {
                speak(game, NO_MAGIC_SPEECH);

                Some(true)
            }

            SpellGate::Refused => Some(false),
            SpellGate::Proceed => None,
        }
    }
}

/// Casts the spell. Returns false if it did not work (the stone ring only works in a circle).
unsafe fn cast_spell(game: &mut Game, chosen_entry: i16) -> bool {
    unsafe {
        match chosen_entry {
            SPELL_LIGHT => the_world.light_timer += LIGHT_TIME,
            SPELL_SECRET => the_world.secret_timer += SECRET_TIME,
            SPELL_FREEZE => the_world.freeze_timer += FREEZE_TIME,
            SPELL_MAP => show_map(game),
            SPELL_STONE => {
                if !stone_jump(game) {
                    return false;
                }

                heal(game);
            }

            SPELL_HEAL => heal(game),
            SPELL_FEAR => scare_enemies(game),
            _ => {}
        }

        true
    }
}

/// Takes one of the spell's stones from the stack, and updates the menu when the last is gone.
fn use_up_one(chosen_entry: i16) {
    let slot = SPELL_SLOT_BASE + chosen_entry as usize;

    player().stuff_mut()[slot] = (player().stuff()[slot]).wrapping_sub(1);

    if player().stuff()[slot] == 0 {
        update_menu_availability();
    }
}

/// What stands between the hero and a spell.
#[derive(Debug, PartialEq, Eq)]
enum SpellGate {
    /// No such spell, or none of it left: the "no spell" event.
    NoSpell,
    /// In a quiet place: the hero says it does not work here.
    QuietZone,
    /// The spell cannot be cast now (freeze while riding, map indoors); nothing is used up.
    Refused,
    Proceed,
}

/// The checks before a spell is cast: `count` is how many of the spell's item the hero has and
/// `zone` the `v3` of the current extent.
fn spell_gate(entry: i16, count: u8, zone: u8, riding: i16, cheat: i16, region: u16) -> SpellGate {
    if entry < FIRST_SPELL || count == 0 {
        return SpellGate::NoSpell;
    }

    if zone == NO_MAGIC_ZONE {
        return SpellGate::QuietZone;
    }

    if entry == SPELL_FREEZE && riding > RIDING_BIG {
        return SpellGate::Refused;
    }

    if entry == SPELL_MAP && cheat == 0 && region > LAST_OUTDOOR_REGION {
        return SpellGate::Refused;
    }

    SpellGate::Proceed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell_gate_checks_in_order() {
        assert_eq!(spell_gate(4, 1, 0, 0, 0, 0), SpellGate::NoSpell);
        assert_eq!(spell_gate(SPELL_LIGHT, 0, 0, 0, 0, 0), SpellGate::NoSpell);
        assert_eq!(
            spell_gate(SPELL_LIGHT, 1, NO_MAGIC_ZONE, 0, 0, 0),
            SpellGate::QuietZone
        );

        assert_eq!(spell_gate(SPELL_FREEZE, 1, 0, 2, 0, 0), SpellGate::Refused);
        assert_eq!(spell_gate(SPELL_FREEZE, 1, 0, 1, 0, 0), SpellGate::Proceed);
        assert_eq!(spell_gate(SPELL_LIGHT, 1, 0, 5, 0, 9), SpellGate::Proceed);
        assert_eq!(spell_gate(SPELL_MAP, 1, 0, 0, 0, 8), SpellGate::Refused);
        assert_eq!(spell_gate(SPELL_MAP, 1, 0, 0, 1, 8), SpellGate::Proceed);
        assert_eq!(spell_gate(SPELL_MAP, 1, 0, 0, 0, 7), SpellGate::Proceed);
    }
}
