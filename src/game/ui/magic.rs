//! The Magic menu actions (from `choose_menu_entry` in csrc/fmain.c): casting the spells the hero has
//! stones, vials and so on for. Each use takes one from the stack.

use crate::amiga::RastPort;
use crate::game::actors::encounters::check_death;
use crate::game::data::random::rand8;
use crate::game::data::state::{the_actors, the_hero, the_screen, the_ui, the_world};
use crate::game::data::tables::stone_ring_sectors;
use crate::game::data::types::Shape;
use crate::game::render::screen_fx::{flicker_colors, show_still_screen};
use crate::game::ui::menus::update_menu_availability;
use crate::game::ui::messages::print;
use crate::game::ui::text::{queue_print, show_event, speak};
use crate::game::world::map::draw_overview_map;
use crate::game::world::places::transfer_hero;
use std::ffi::c_char;

unsafe extern "C" {
    fn SetDrMd(current_rastport: *mut RastPort, mode: u32);

    fn SetAPen(current_rastport: *mut RastPort, pen: u32);

    fn Move(current_rastport: *mut RastPort, x: i32, y: i32);

    fn Text(current_rastport: *mut RastPort, s: *const c_char, len: i32) -> i32;
}

// The menu entries: the spells are entries 5-11 and use the item slots 9-15 (`4 + entry`).
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
const BATTLE_EVENT: i32 = 34;

/// Regions above this are indoors (no overview map there).
const LAST_OUTDOOR_REGION: u16 = 7;

/// Riding something bigger than the raft stops the freeze spell.
const RIDING_BIG: i16 = 1;

// The stone circle: sector, and where in the sector the hero must stand.
const STONE_SECTOR: i16 = 144;
const STONE_COLUMN_WIDTH: u16 = 85;
const STONE_ROW_HEIGHT: u16 = 64;
const STONE_CIRCLES: usize = 11;

// The overview map and where the hero's position is marked.
const MAP_MARK_PEN: u32 = 31;
const JAM1: u32 = 0;
const MARK_X_OFFSET: i64 = 4;
const MARK_Y_OFFSET: i64 = 3;
const MARK_MAX: (u32, u32) = (320, 143);
const VIEW_MAP: i8 = 1;

const PRQ_STATS: i32 = 4;
const PRQ_MAP: i32 = 5;

/// Healing is capped at this plus a quarter of bravery.
const HEAL_CAP_BASE: i16 = 15;
const HEAL_MIN: i32 = 4;

/// The fear spell only hits enemies of races below this.
const FEAR_RACE_LIMIT: u8 = 7;
const ENEMY: i8 = 2;

/// Handles an entry of the Magic menu. Returns false if `update_menu_availability` should not follow
/// (the spell did not work, so the stone is not used up).
pub unsafe fn magic_option(chosen_entry: i16) -> bool {
    unsafe {
        if let Some(result) = refuse_spell(chosen_entry) {
            return result;
        }

        if !cast_spell(chosen_entry) {
            return false;
        }

        use_up_one(chosen_entry);

        true
    }
}

/// Checks that the spell can be cast here and now. Gives `Some(result)` for `magic_option` to
/// return when it cannot, and `None` when the spell goes ahead.
unsafe fn refuse_spell(chosen_entry: i16) -> Option<bool> {
    unsafe {
        let count = if chosen_entry < FIRST_SPELL {
            0
        } else {
            *the_hero.stuff.add(SPELL_SLOT_BASE + chosen_entry as usize)
        };

        let gate = spell_gate(
            chosen_entry,
            count,
            (*the_world.extent).v3,
            the_hero.riding,
            the_ui.cheat_enabled,
            the_world.region,
        );

        match gate {
            SpellGate::NoSpell => {
                show_event(NO_SPELL_EVENT);

                Some(true)
            }

            SpellGate::QuietZone => {
                speak(NO_MAGIC_SPEECH);

                Some(true)
            }

            SpellGate::Refused => Some(false),
            SpellGate::Proceed => None,
        }
    }
}

/// Casts the spell. Returns false if it did not work (the stone ring only works in a circle).
unsafe fn cast_spell(chosen_entry: i16) -> bool {
    unsafe {
        match chosen_entry {
            SPELL_LIGHT => the_world.light_timer += LIGHT_TIME,
            SPELL_SECRET => the_world.secret_timer += SECRET_TIME,
            SPELL_FREEZE => the_world.freeze_timer += FREEZE_TIME,
            SPELL_MAP => show_map(),
            SPELL_STONE => {
                if !stone_jump() {
                    return false;
                }

                heal();
            }

            SPELL_HEAL => heal(),
            SPELL_FEAR => scare_enemies(),
            _ => {}
        }

        true
    }
}

/// Takes one of the spell's stones from the stack, and updates the menu when the last is gone.
unsafe fn use_up_one(chosen_entry: i16) {
    unsafe {
        let slot = SPELL_SLOT_BASE + chosen_entry as usize;

        *the_hero.stuff.add(slot) = (*the_hero.stuff.add(slot)).wrapping_sub(1);

        if *the_hero.stuff.add(slot) == 0 {
            update_menu_availability();
        }
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

/// Draws the overview map with a "+" where the hero is, and shows it.
unsafe fn show_map() {
    unsafe {
        the_screen.bm_draw = (*(*the_screen.drawing_page).ri_page).BitMap;
        the_screen.planes = (*the_screen.bm_draw).Planes.as_mut_ptr();
        draw_overview_map(the_world.map_x as i32, the_world.map_y as i32);

        // The hero on the overview map: his tile column minus the left edge of the shown sectors (16 tiles to a sector).
        let x = ((the_hero.x >> 4) as i64
            - ((the_world.sector_x as i64 + the_world.origin_x as i64) << 4)
            - MARK_X_OFFSET) as u32;
        // The hero on the overview map: his position in 16 pixel steps minus the top edge of the shown sectors.
        let y = ((the_hero.y >> 4) as i64
            - ((the_world.sector_y as i64 + the_world.origin_y as i64) << 4)
            + MARK_Y_OFFSET) as u32;
        the_screen.rp_map.BitMap = the_screen.bm_draw;
        SetDrMd(&raw mut the_screen.rp_map, JAM1);
        SetAPen(&raw mut the_screen.rp_map, MAP_MARK_PEN);

        if x > 0 && x < MARK_MAX.0 && y > 0 && y < MARK_MAX.1 {
            Move(&raw mut the_screen.rp_map, x as i32, y as i32);
            Text(&raw mut the_screen.rp_map, c"+".as_ptr(), 1);
        }

        the_screen.view_status = VIEW_MAP;
        show_still_screen();
        queue_print(PRQ_MAP);
    }
}

/// The stone ring spell: only works standing in the middle of a stone circle. Takes the hero to
/// the next circle in the direction faced. Returns false if the spell did not work.
unsafe fn stone_jump() -> bool {
    unsafe {
        let ring_list = stone_ring_sectors;

        match stone_target(
            the_hero.sector,
            the_hero.x,
            the_hero.y,
            the_actors.list[0].facing,
            &ring_list,
        ) {
            StoneJump::Refused => false,
            StoneJump::NoRing => true,
            StoneJump::To(x, y) => {
                flicker_colors();
                transfer_hero(x, y, 1);

                if the_hero.riding != 0 {
                    the_actors.list[the_actors.carrier_slot as usize].abs_x =
                        the_actors.list[0].abs_x;
                    the_actors.list[the_actors.carrier_slot as usize].abs_y =
                        the_actors.list[0].abs_y;
                }

                true
            }
        }
    }
}

/// Where the stone ring spell takes the hero.
#[derive(Debug, PartialEq, Eq)]
enum StoneJump {
    /// Not standing in the middle of a stone circle: the spell fails.
    Refused,
    /// In the right sector but at no circle of the list: the stone is used up, nothing happens.
    NoRing,
    /// Jump to this position (the same spot inside the sector of the target circle).
    To(u16, u16),
}

/// The pure part of `stone_jump`: the hero goes `facing + 1` circles further along the list,
/// wrapping around after the last one.
fn stone_target(sector: i16, px: u16, py: u16, facing: i8, list: &[u8]) -> StoneJump {
    if sector != STONE_SECTOR {
        return StoneJump::Refused;
    }

    if (px & 255) / STONE_COLUMN_WIDTH != 1 || (py & 255) / STONE_ROW_HEIGHT != 1 {
        return StoneJump::Refused;
    }

    // Sectors are 256 pixels wide and high.
    let (sector_x, sector_y) = (px >> 8, py >> 8);

    for ring in 0..STONE_CIRCLES {
        if list[ring * 2] as u16 == sector_x && list[ring * 2 + 1] as u16 == sector_y {
            let mut target = ring + (facing + 1) as usize;

            if target > STONE_CIRCLES - 1 {
                target -= STONE_CIRCLES;
            }

            // Back to pixels: the target sector, then the same spot inside it.
            let x = ((list[target * 2] as u16) << 8) + (px & 255);
            let y = ((list[target * 2 + 1] as u16) << 8) + (py & 255);

            return StoneJump::To(x, y);
        }
    }

    StoneJump::NoRing
}

/// Heals the hero a little, up to a limit set by bravery.
unsafe fn heal() {
    unsafe {
        let hero = &raw mut the_actors.list[0];
        let (vitality, capped) = healed_vitality((*hero).vitality, the_hero.brave, rand8());
        (*hero).vitality = vitality;

        if !capped {
            print(c"That feels a lot better!".as_ptr());
        }

        queue_print(PRQ_STATS);
    }
}

/// Vitality after the heal spell: it adds 4 plus a roll (`roll`, 0-7), up to a limit that grows with
/// bravery. Also says whether the limit was hit (then the hero was already nearly full).
fn healed_vitality(vitality: i16, brave: i16, roll: i32) -> (i16, bool) {
    let cap = HEAL_CAP_BASE + brave / 4;
    let healed = vitality.wrapping_add((roll + HEAL_MIN) as i16);

    if healed > cap {
        (cap, true)
    } else {
        (healed, false)
    }
}

/// Whether the fear spell kills this actor: a living enemy of one of the weaker races.
fn is_scared_by_fear_spell(actor: &Shape) -> bool {
    actor.vitality != 0 && actor.kind == ENEMY && actor.race < FEAR_RACE_LIMIT
}

/// Kills every weak enemy on the screen (and costs a little bravery for each).
unsafe fn scare_enemies() {
    unsafe {
        let mut i = 1;

        while i < the_actors.active as usize {
            let actor = &raw mut the_actors.list[i];

            if is_scared_by_fear_spell(&*actor) {
                (*actor).vitality = 0;
                check_death(i as i32, 0);
                the_hero.brave -= 1;
            }

            i += 1;
        }

        if the_actors.battle_flag != 0 {
            show_event(BATTLE_EVENT);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: [u8; 22] = [
        54, 43, 71, 77, 78, 102, 66, 121, 12, 85, 79, 40, 107, 38, 73, 21, 12, 26, 26, 53, 84, 60,
    ];

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

    #[test]
    fn stone_jump_rules() {
        // Middle of the first circle: x&255 in 85..170, y&255 in 64..128.
        let (x, y) = ((54 << 8) + 100, (43 << 8) + 70);
        assert_eq!(stone_target(143, x, y, 0, &LIST), StoneJump::Refused);
        assert_eq!(stone_target(144, x + 100, y, 0, &LIST), StoneJump::Refused);
        assert_eq!(stone_target(144, x, y + 100, 0, &LIST), StoneJump::Refused);
        assert_eq!(
            stone_target(144, 0x0500 + 100, 0x0500 + 70, 0, &LIST),
            StoneJump::NoRing
        );

        // Facing 0 goes one circle on, same offset inside the sector.
        assert_eq!(
            stone_target(144, x, y, 0, &LIST),
            StoneJump::To((71 << 8) + 100, (77 << 8) + 70)
        );

        // From the last circle the list wraps: 10 + 8 - 11 = 7.
        let (x, y) = ((84 << 8) + 90, (60 << 8) + 65);
        assert_eq!(
            stone_target(144, x, y, 7, &LIST),
            StoneJump::To((73 << 8) + 90, (21 << 8) + 65)
        );

        // Landing exactly on the last index does not wrap.
        let (x, y) = ((54 << 8) + 90, (43 << 8) + 65);
        assert_eq!(
            stone_target(144, x, y, 9, &LIST),
            StoneJump::To((84 << 8) + 90, (60 << 8) + 65)
        );
    }

    #[test]
    fn heal_adds_four_plus_the_roll() {
        assert_eq!(healed_vitality(5, 0, 3), (12, false));
    }

    #[test]
    fn heal_stops_at_a_cap_that_grows_with_bravery() {
        assert_eq!(healed_vitality(14, 0, 7), (HEAL_CAP_BASE, true));
        assert_eq!(healed_vitality(10, 40, 7), (21, false));
        assert_eq!(healed_vitality(25, 40, 7), (HEAL_CAP_BASE + 10, true));
        assert_eq!(healed_vitality(21, 40, 0), (HEAL_CAP_BASE + 10, false));
    }

    #[test]
    fn fear_kills_living_weak_enemies_only() {
        let mut a: Shape = unsafe { std::mem::zeroed() };
        a.kind = ENEMY;
        a.vitality = 5;
        a.race = FEAR_RACE_LIMIT - 1;
        assert!(is_scared_by_fear_spell(&a));
        a.race = FEAR_RACE_LIMIT;
        assert!(!is_scared_by_fear_spell(&a));
        a.race = 0;
        a.vitality = 0;
        assert!(!is_scared_by_fear_spell(&a));
        a.vitality = 5;
        a.kind = 0;
        assert!(!is_scared_by_fear_spell(&a));
    }
}
