//! The world's per-frame bookkeeping (from the main loop in csrc/fmain.c): how near the raft is,
//! sleeping on beds, going through doors, the day and night clock, new monsters, hunger, who is
//! the nearest person and the draw order of the actors.

use super::actors::{actor_distance, pick_encounter_spot};
use super::asm_blit::{SShape, restore_background};
use super::asm_map::{terrain_at, tile_address};
use super::asm_text::{queue_print, show_event, speak};
use super::encounters::load_enemy_sprites;
use super::encounters::place_encounter;
use super::game_tables::door_list;
use super::globals::{shp, sleepwait};
use super::hdrive::{motor_off, read_finished};
use super::loading::finish_sprite_load;
use super::mood::set_mood;
use super::places::{find_place, transfer_hero};
use super::random::{fta_rand, rand4, rand64};
use super::screen_fx::{day_fade, fade_page};
use super::state::{Hero, the_actors, the_encounter, the_hero, the_screen, the_witch, the_world};
use super::tables::{ob_list8, ob_listg, page_colors};
use super::types::{Door, Shape};
use super::witch::witch_fx;

unsafe extern "C" {
    fn OwnBlitter();
    fn DisownBlitter();
}

// Actor kinds and states (csrc/ftale.h).
const OBJECTS: i8 = 1;
const STILL: i8 = 13;
const DEAD: i8 = 15;
const SLEEP: i8 = 23;
const BIRD: i16 = 11;
const TURTLE_ENVIRON: i8 = -2;

// Print queue entries.
const PRQ_STATS: i32 = 4;

// ---- raft_proximity ----
/// The hero counts as near the raft or turtle within this many pixels (very near within the
/// smaller one).
const NEAR: i16 = 16;
const VERY_NEAR: i16 = 9;
/// The carrier is the raft (slot 1), or slot 3 when a turtle or bird is out.
const RAFT_SLOT: i16 = 1;
const CARRIER_SLOT: i16 = 3;
/// The hero's centre is a little off the carrier's.
const CARRIER_OFFSET: i32 = 4;

/// 0 when the hero is not near, 1 when near and 2 when very near the carrier (offsets in pixels).
fn nearness(dx: i16, dy: i16) -> i16 {
    if dx < VERY_NEAR && dx > -VERY_NEAR && dy < VERY_NEAR && dy > -VERY_NEAR {
        2
    } else if dx < NEAR && dx > -NEAR && dy < NEAR && dy > -NEAR {
        1
    } else {
        0
    }
}

/// How close the hero is to the raft or turtle (`raft_nearness` 1 = near, 2 = very near). Riding the
/// bird sets the hero's environ.
pub fn raft_proximity() {
    unsafe {
        the_actors.raft_nearness = 0;
        the_actors.turtle_nearness = 0;
        the_actors.carrier_slot = if the_actors.active_carrier != 0 {
            CARRIER_SLOT
        } else {
            RAFT_SLOT
        };
        let carrier = &raw const the_actors.list[the_actors.carrier_slot as usize];
        let hero = &raw mut the_actors.list[0];
        let dx = ((*hero).abs_x as i32 - (*carrier).abs_x as i32 - CARRIER_OFFSET) as i16;
        let dy = ((*hero).abs_y as i32 - (*carrier).abs_y as i32 - CARRIER_OFFSET) as i16;
        the_actors.raft_nearness = nearness(dx, dy);
        if the_hero.riding == BIRD {
            (*hero).environ = TURTLE_ENVIRON;
        }
    }
}

// ---- check_bed ----
/// Indoors region, the tiles that are beds, and how long to stand still on one before sleeping.
const INSIDE_REGION: u16 = 8;
const BED_TILES: [u8; 4] = [161, 52, 162, 53];
const BED_WAIT: i16 = 30;
const TOO_RESTED: i16 = 50;
const EVENT_NOT_TIRED: i32 = 25;
const EVENT_SLEEPING: i32 = 26;
/// Sleeping puts the hero at the bottom of the tile.
const BED_Y_MASK: u16 = 0x1f;

/// Copies the hero's position, and puts the hero to bed after standing on a bed for 30 ticks.
pub fn check_bed() {
    unsafe {
        let hero = &raw mut the_actors.list[0];
        if (*hero).state != STILL {
            sleepwait = 0;
        }
        the_hero.x = (*hero).abs_x;
        the_hero.y = (*hero).abs_y;

        if the_world.region == INSIDE_REGION {
            let tile = *tile_address((the_hero.x >> 4) as i32, (the_hero.y >> 5) as i32);
            let (wait, action) = bed_step(sleepwait, BED_TILES.contains(&tile), the_hero.fatigue);
            sleepwait = wait;
            match action {
                BedAction::Nothing => {}
                BedAction::NotTired => show_event(EVENT_NOT_TIRED),
                BedAction::FallAsleep => {
                    show_event(EVENT_SLEEPING);
                    (*hero).abs_y |= BED_Y_MASK;
                    the_hero.y = (*hero).abs_y;
                    (*hero).state = SLEEP;
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum BedAction {
    Nothing,
    /// Stood on the bed long enough, but is too rested to sleep.
    NotTired,
    FallAsleep,
}

/// One frame indoors: `wait` counts how long the hero has stood on a bed; after `BED_WAIT` frames
/// he sleeps if tired enough. Stepping off the bed starts the count again.
fn bed_step(wait: i16, on_bed: bool, fatigue: i16) -> (i16, BedAction) {
    if !on_bed {
        return (0, BedAction::Nothing);
    }
    let wait = wait + 1;
    let action = if wait != BED_WAIT {
        BedAction::Nothing
    } else if fatigue < TOO_RESTED {
        BedAction::NotTired
    } else {
        BedAction::FallAsleep
    };
    (wait, action)
}

// ---- check_door ----
const DOORCOUNT: i32 = 86;
const DESERT: i8 = 17;
const CAVE: i8 = 18;
/// Inventory slot of the gold statues, five are needed to enter the desert.
const STATBASE: usize = 25;
const STATUES_NEEDED: u8 = 5;
const REGION_INSIDE: u16 = 8;
const REGION_INSIDE_OTHER: u16 = 9;
const FIRST_INSIDE_REGION: u16 = 8;
const FADE_WHITE: i16 = 100;

/// Where to arrive on the other side of a door going in.
fn inside_spot(d: &Door) -> (u16, u16) {
    if d.kind == CAVE {
        (d.xc2.wrapping_add(24), d.yc2.wrapping_add(16))
    } else if d.kind & 1 != 0 {
        (d.xc2.wrapping_add(16), d.yc2)
    } else {
        (d.xc2.wrapping_sub(1), d.yc2.wrapping_add(16))
    }
}

/// Where to arrive going out.
fn outside_spot(d: &Door) -> (u16, u16) {
    if d.kind == CAVE {
        (d.xc1.wrapping_sub(4), d.yc1.wrapping_add(16))
    } else if d.kind & 1 != 0 {
        (d.xc1.wrapping_add(16), d.yc1.wrapping_add(34))
    } else {
        (d.xc1.wrapping_add(20), d.yc1.wrapping_add(16))
    }
}

/// The hero can go in when lined up with the door (horizontal doors by y, vertical ones by x), and
/// the desert door needs the gold statues.
fn may_enter(d: &Door, hero_x: u16, hero_y: u16, statues: u8) -> bool {
    let lined_up = if d.kind & 1 != 0 {
        hero_y & 0x10 == 0
    } else {
        hero_x & 15 <= 6
    };
    lined_up && (d.kind != DESERT || statues >= STATUES_NEEDED)
}

/// Which indoor region a door leads to.
fn region_inside(d: &Door) -> u16 {
    if d.secs == 1 { REGION_INSIDE } else { REGION_INSIDE_OTHER }
}

/// Whether a door's inside entrance is where the hero stands (`xtest`, `ytest`: the hero's position
/// rounded to a tile). Horizontal doors are also found one tile to the right.
fn is_exit_here(d: &Door, xtest: u16, ytest: u16) -> bool {
    let horizontal = d.kind & 1 != 0;
    d.yc2 == ytest && (d.xc2 == xtest || (d.xc2 as i32 == xtest as i32 - 16 && horizontal))
}

/// The hero can go out when lined up with the door the other way round to `may_enter`.
fn may_leave(d: &Door, hero_x: u16, hero_y: u16) -> bool {
    if d.kind & 1 != 0 {
        hero_y & 0x10 != 0
    } else {
        hero_x & 15 >= 2
    }
}

unsafe fn door_at(j: i32) -> &'static Door {
    unsafe { &*(&raw const door_list).cast::<Door>().add(j as usize) }
}

/// Is the hero standing at a door? Outdoors the doors are in sorted order and found by binary
/// search; indoors they are searched in turn. If so, the hero is moved to the other side.
/// Riding disables doors.
pub fn check_door() {
    unsafe {
        if the_hero.riding != 0 {
            return;
        }
        let xtest = the_hero.x & 0xfff0;
        let ytest = the_hero.y & 0xffe0;
        if the_world.region < FIRST_INSIDE_REGION {
            enter_door(xtest, ytest);
        } else {
            leave_door(xtest, ytest);
        }
    }
}

unsafe fn enter_door(xtest: u16, ytest: u16) {
    unsafe {
        let Some(index) = find_outdoor_door(xtest, ytest) else {
            return;
        };
        let d = door_at(index);
        let statues = *the_hero.stuff.add(STATBASE);
        if may_enter(d, the_hero.x, the_hero.y, statues) {
            let (x, y) = inside_spot(d);
            the_world.new_region = region_inside(d);
            transfer_hero(x, y, 0);
            find_place(2);
            fade_page(
                FADE_WHITE,
                FADE_WHITE,
                FADE_WHITE,
                1,
                (&raw const page_colors).cast(),
            );
        }
    }
}

/// Binary search of the (sorted) outdoor doors for the one whose entrance covers the point.
/// A door matches within 16 pixels to the right of its x; vertical doors need the exact x.
unsafe fn find_outdoor_door(xtest: u16, ytest: u16) -> Option<i32> {
    unsafe {
        let mut low = 0;
        let mut high = DOORCOUNT - 1;
        while high >= low {
            let j = (high + low) / 2;
            let d = door_at(j);
            if d.xc1 > xtest {
                high = j - 1;
            } else if d.xc1 as i32 + 16 < xtest as i32 {
                low = j + 1;
            } else if d.xc1 < xtest && d.kind & 1 == 0 {
                low = j + 1; // vertical door: its x must match exactly
            } else if d.yc1 > ytest {
                high = j - 1;
            } else if d.yc1 < ytest {
                low = j + 1;
            } else {
                return Some(j);
            }
            if low >= DOORCOUNT || high < 0 {
                break;
            }
        }
        None
    }
}

unsafe fn leave_door(xtest: u16, ytest: u16) {
    unsafe {
        for j in 0..DOORCOUNT {
            let d = door_at(j);
            if is_exit_here(d, xtest, ytest) {
                if may_leave(d, the_hero.x, the_hero.y) {
                    let (x, y) = outside_spot(d);
                    transfer_hero(x, y, 1);
                    find_place(0);
                }
                break;
            }
        }
    }
}

// ---- undo_sprites ----
/// Puts back the background under last frame's sprites, then undoes the witch effect on that page.
pub fn undo_sprites() {
    unsafe {
        the_screen.bm_draw = (*(*the_screen.drawing_page).ri_page).BitMap;
        the_screen.planes = (*the_screen.bm_draw).Planes.as_mut_ptr();

        OwnBlitter();
        let queue = (*the_screen.drawing_page).shape_queue.cast::<SShape>();
        for i in (1..=(*the_screen.drawing_page).obcount as isize).rev() {
            shp = queue.offset(i - 1);
            restore_background((*shp).backsave);
        }
        DisownBlitter();

        the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;
        if (*the_screen.drawing_page).wflag != 0 {
            witch_fx(the_screen.drawing_page);
        }
    }
}

// ---- day_night_tick ----
const DAY_LENGTH: u16 = 24000;
/// Sleeping makes time pass this much faster.
const SLEEP_TIME_STEP: u16 = 63;
/// The hero wakes at dawn (this window) if only a little tired.
const DAWN: (u16, u16) = (9000, 10000);
const LITTLE_TIRED: i16 = 30;
const LIGHT_PEAK: u16 = 300;
const DARK: u16 = 40;
/// The night-time object: the lamp post at this slot of `ob_listg` is on in the dark.
const LAMP_SLOT: usize = 5;
const LAMP_ON: i8 = 3;
const LAMP_OFF: i8 = 2;
const PERIOD_LENGTH: u16 = 2000;
/// Events said when a new part of the day starts.
const PERIOD_EVENTS: [(i16, i32); 4] = [(0, 28), (4, 29), (6, 30), (9, 31)];
/// The hero heals a little every time the clock passes a multiple of this (minus one).
const HEAL_MASK: u16 = 0x3ff;
const HEAL_BASE: i16 = 15;
const WAKE_Y_MASK: u16 = 0xffe0;

/// The day clock after one frame: it stands still while time is frozen and wraps to 0 once it
/// is past `DAY_LENGTH`.
fn clock_step(clock: u16, frozen: bool) -> u16 {
    if frozen {
        return clock;
    }
    if clock >= DAY_LENGTH { 0 } else { clock.wrapping_add(1) }
}

/// The light level for a clock value: it rises to `LIGHT_PEAK` at midday and falls again.
fn light_level(clock: u16) -> u16 {
    let level = clock / DARK;
    if level >= LIGHT_PEAK { 2 * LIGHT_PEAK - level } else { level }
}

/// The lamp post is lit when it is dark.
fn lamp_is_on(level: u16) -> bool {
    level < DARK
}

/// Which part of the day (0..=11) the clock is in.
fn day_period(clock: u16) -> i16 {
    (clock / PERIOD_LENGTH) as i16
}

/// The message said when `period` starts, if it has one.
fn period_event(period: i16) -> Option<i32> {
    PERIOD_EVENTS.iter().find(|(p, _)| *p == period).map(|(_, e)| *e)
}

/// Slow healing: once per `HEAL_MASK + 1` clock ticks, if hurt and alive.
fn heal_due(clock: u16, vitality: i16, bravery: i16, state: i8) -> bool {
    clock & HEAL_MASK == 0 && vitality < HEAL_BASE + bravery / 4 && state != DEAD
}

/// Sleeping, the day clock, the light level, the messages for the time of day and slow healing.
pub fn day_night_tick() {
    unsafe {
        let hero = &raw mut the_actors.list[0];
        if (*hero).state == SLEEP {
            sleep_tick(hero);
        }
        let tick = day_step(
            the_world.day_clock,
            the_world.freeze_timer != 0,
            the_world.day_period,
            (*hero).vitality,
            the_hero.brave,
            (*hero).state,
        );
        the_world.day_clock = tick.clock;
        the_world.light_level = tick.light;
        ob_listg[LAMP_SLOT].ob_stat = if tick.lamp_on { LAMP_ON } else { LAMP_OFF };
        the_world.day_period = tick.period;
        if let Some(e) = tick.period_event {
            show_event(e);
        }

        day_fade();

        if tick.heal {
            (*hero).vitality += 1;
            queue_print(PRQ_STATS);
        }
    }
}

/// What one frame of the day clock changes.
#[derive(Debug, PartialEq, Eq)]
struct DayTick {
    clock: u16,
    light: u16,
    lamp_on: bool,
    period: i16,
    /// The message for a new part of the day, if one started this frame and has one.
    period_event: Option<i32>,
    /// The hero heals one point.
    heal: bool,
}

/// The day clock, light, lamp, part of the day and slow healing after one frame.
fn day_step(
    clock: u16,
    frozen: bool,
    old_period: i16,
    vitality: i16,
    bravery: i16,
    state: i8,
) -> DayTick {
    let clock = clock_step(clock, frozen);
    let light = light_level(clock);
    let period = day_period(clock);
    DayTick {
        clock,
        light,
        lamp_on: lamp_is_on(light),
        period,
        period_event: if period != old_period { period_event(period) } else { None },
        heal: heal_due(clock, vitality, bravery, state),
    }
}

/// One frame of sleep: returns the new clock, the new fatigue and whether the hero wakes.
/// `fight_wakes` is only called (it draws a random number) when a battle is on and no other
/// reason to wake applied.
fn sleep_step(
    clock: u16,
    tired: i16,
    battle: bool,
    fight_wakes: impl FnOnce() -> bool,
) -> (u16, i16, bool) {
    let clock = clock.wrapping_add(SLEEP_TIME_STEP);
    let tired = if tired != 0 { tired - 1 } else { tired };
    let wake = tired == 0
        || (tired < LITTLE_TIRED && clock > DAWN.0 && clock < DAWN.1)
        || (battle && fight_wakes());
    (clock, tired, wake)
}

/// While asleep time runs fast; the hero wakes when rested, at dawn, or when a fight starts.
unsafe fn sleep_tick(hero: *mut Shape) {
    unsafe {
        let (clock, tired, wake) = sleep_step(
            the_world.day_clock,
            the_hero.fatigue,
            the_actors.battle_flag != 0,
            || rand64() == 0,
        );
        the_world.day_clock = clock;
        the_hero.fatigue = tired;
        if wake {
            (*hero).state = STILL;
            (*hero).abs_y &= WAKE_Y_MASK;
            the_hero.y = (*hero).abs_y;
        }
    }
}

// ---- spawn_encounters ----
const ENEMY_SLOT: i16 = 2;
const FIRST_ENEMY_ACTOR: i16 = 3;
const LAST_ENEMY_ACTOR: i16 = 7;
const DISK_REQUEST_ACTORS: i32 = 8;
const PLACE_TRIES: i32 = 10;
const SPREAD: u16 = 63;
const PLAIN_TERRAIN: i32 = 0;
const WRAITH: u8 = 2;
/// Xtype values: above this no mixed groups; the limit for random encounters; special zones.
const NO_MIX_XTYPE: u16 = 49;
const NO_RANDOM_XTYPE: u16 = 50;
const SPIDER_PIT_XTYPE: u16 = 8;
const DKNIGHT_XTYPE: u16 = 7;
const FINAL_XTYPE: u16 = 49;
const INDOOR_REGION: u16 = 7;

/// Says where the hero is, finishes loading monsters, places waiting ones and now and then
/// starts a new random encounter.
pub fn spawn_encounters() {
    unsafe {
        find_place(2);

        if the_actors.loading == 1 && read_finished(DISK_REQUEST_ACTORS) != 0 {
            finish_sprite_load(ENEMY_SLOT);
            motor_off();
            the_actors.loading = 0;
            the_actors.active = FIRST_ENEMY_ACTOR;
        }
        if the_world.day_clock & 15 == 0 && the_encounter.number != 0 && the_actors.loading == 0 {
            place_waiting_encounters();
        }
        if the_world.day_clock & 31 == 0
            && the_actors.on_screen == 0
            && the_actors.loading == 0
            && the_actors.active_carrier == 0
            && the_world.extent_type < NO_RANDOM_XTYPE
        {
            the_encounter.danger_level = (if the_world.region > INDOOR_REGION {
                5 + the_world.extent_type
            } else {
                2 + the_world.extent_type
            }) as u8;
            if rand64() <= the_encounter.danger_level as i32 {
                the_encounter.kind = rand4() as u16;
                if the_world.extent_type == DKNIGHT_XTYPE && the_encounter.kind == 2 {
                    the_encounter.kind = 4;
                }
                if the_world.extent_type == SPIDER_PIT_XTYPE {
                    the_encounter.kind = 6;
                    the_encounter.mix_flag = 0;
                }
                if the_world.extent_type == FINAL_XTYPE {
                    the_encounter.kind = 2;
                    the_encounter.mix_flag = 0;
                }
                load_enemy_sprites();
            }
        }
    }
}

/// Tries to find a clear spot near the hero and puts the waiting encounter there, into free
/// actor slots first and then into dead ones.
unsafe fn place_waiting_encounters() {
    unsafe {
        the_encounter.mix_flag = fta_rand() as i16;
        if the_world.extent_type > NO_MIX_XTYPE {
            the_encounter.mix_flag = 0;
        }
        the_encounter.weapon_type = rand4() as i16;
        if the_world.extent_type & 3 == 0 {
            the_encounter.mix_flag = 0;
        }
        for _ in 1..PLACE_TRIES {
            pick_encounter_spot();
            if terrain_at(the_encounter.x as i32, the_encounter.y as i32) != PLAIN_TERRAIN {
                continue;
            }
            while the_encounter.number != 0 && the_actors.active < LAST_ENEMY_ACTOR {
                if place_encounter(the_actors.active as u16, SPREAD) != 0 {
                    the_actors.active += 1;
                }
                the_encounter.number -= 1;
            }
            let mut i = FIRST_ENEMY_ACTOR as usize;
            while i < LAST_ENEMY_ACTOR as usize && the_encounter.number != 0 {
                let an = &raw const the_actors.list[i];
                if (*an).state == DEAD && ((*an).visible == 0 || (*an).race == WRAITH) {
                    place_encounter(i as u16, SPREAD);
                    the_encounter.number -= 1;
                }
                i += 1;
            }
            break;
        }
    }
}

// ---- speak_nearby ----
const BEGGAR: i16 = 0x8d;
const WITCH: i16 = 0x89;
const PRINCESS: i16 = 0x84;
const NECROMANCER: i16 = 9;
const DKNIGHT: i16 = 7;
const PRINCESS_FREED_SLOT: usize = 9;

/// The nearest person says something when they come near (once per person).
pub fn speak_nearby() {
    unsafe {
        let race = the_actors.list[the_actors.nearest_person as usize].race as i16;
        if the_actors.nearest_person != 0 && race != the_actors.last_person {
            match race {
                BEGGAR => speak(23),
                WITCH => speak(46),
                PRINCESS => {
                    if ob_list8[PRINCESS_FREED_SLOT].ob_stat != 0 {
                        speak(16);
                    }
                }
                NECROMANCER => speak(43),
                DKNIGHT => speak(41),
                _ => {}
            }
            the_actors.last_person = race;
        }
    }
}

// ---- survival_tick ----
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

// ---- sort_actors ----
/// Closest a person can be to count as `nearest_person`.
const PERSON_RANGE: i16 = 50;
/// Dead actors, the hero on a mount and the raft are drawn this much lower in the order.
const DRAW_LOWER: i16 = 32;
const WADING: i8 = 25;
const RAFT_ACTOR: u8 = 1;

/// Finds the nearest person, and sorts `draw_order` by y so that actors lower on the screen are
/// drawn later.
pub fn sort_actors() {
    unsafe {
        the_actors.nearest_person = 0;
        the_actors.nearest_person_distance = PERSON_RANGE;
        for i in 0..the_actors.drawn as usize {
            the_actors.draw_order[i] = i as u8;
        }
        for i in 0..the_actors.drawn as usize {
            let an = &raw const the_actors.list[i];
            if i != 0 && (*an).kind != OBJECTS && (*an).state != DEAD && the_hero.riding != BIRD {
                let dist = actor_distance(i as i32, 0) as i16;
                if dist < the_actors.nearest_person_distance {
                    the_actors.nearest_person_distance = dist;
                    the_actors.nearest_person = i as i16;
                }
            }
            for j in 1..the_actors.drawn as usize {
                bubble_step(j);
            }
        }
    }
}

/// One compare-and-swap of neighbours `j - 1` and `j` in the draw order.
unsafe fn bubble_step(j: usize) {
    unsafe {
        let k1 = the_actors.draw_order[j - 1];
        let k2 = the_actors.draw_order[j];
        let a = &raw const the_actors.list[k1 as usize];
        let b = &raw const the_actors.list[k2 as usize];
        let mut y1 = (*a).abs_y as i16;
        let mut y2 = (*b).abs_y as i16;
        if (*a).state == DEAD || (k2 == 0 && the_hero.riding != 0) || k1 == RAFT_ACTOR {
            y1 = y1.wrapping_sub(DRAW_LOWER);
        }
        if (*b).state == DEAD || (k1 == 0 && the_hero.riding != 0) || k2 == RAFT_ACTOR {
            y2 = y2.wrapping_sub(DRAW_LOWER);
        }
        if (*a).environ > WADING {
            y1 = y1.wrapping_add(DRAW_LOWER);
        }
        if (*b).environ > WADING {
            y2 = y2.wrapping_add(DRAW_LOWER);
        }
        if y2 < y1 {
            the_actors.draw_order[j - 1] = k2;
            the_actors.draw_order[j] = k1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hero_and_body(hunger: i16, fatigue: i16, vitality: i16) -> (Hero, Shape) {
        let mut hero: Hero = unsafe { std::mem::zeroed() };
        let mut body: Shape = unsafe { std::mem::zeroed() };
        hero.hunger = hunger;
        hero.fatigue = fatigue;
        body.vitality = vitality;
        (hero, body)
    }

    fn door(kind: i8, secs: i8) -> Door {
        Door { xc1: 100, yc1: 200, xc2: 300, yc2: 400, kind, secs }
    }

    #[test]
    fn entering_needs_the_hero_lined_up() {
        // horizontal doors (odd kind) go by y, vertical ones by x
        assert!(may_enter(&door(1, 1), 0, 0x00, 0));
        assert!(!may_enter(&door(1, 1), 0, 0x10, 0));
        assert!(may_enter(&door(2, 1), 6, 0, 0));
        assert!(!may_enter(&door(2, 1), 7, 0, 0));
    }

    #[test]
    fn the_desert_door_needs_five_statues() {
        assert!(!may_enter(&door(DESERT, 1), 0, 0, STATUES_NEEDED - 1));
        assert!(may_enter(&door(DESERT, 1), 0, 0, STATUES_NEEDED));
        assert!(may_enter(&door(2, 1), 0, 0, 0));
    }

    #[test]
    fn leaving_is_the_mirror_of_entering() {
        assert!(may_leave(&door(1, 1), 0, 0x10));
        assert!(!may_leave(&door(1, 1), 0, 0));
        assert!(may_leave(&door(2, 1), 2, 0));
        assert!(!may_leave(&door(2, 1), 1, 0));
    }

    #[test]
    fn exits_are_found_by_tile() {
        assert!(is_exit_here(&door(2, 1), 300, 400));
        assert!(!is_exit_here(&door(2, 1), 316, 400));
        assert!(is_exit_here(&door(1, 1), 316, 400)); // horizontal: also one tile right
        assert!(!is_exit_here(&door(1, 1), 300, 416));
    }

    #[test]
    fn doors_lead_to_two_indoor_regions() {
        assert_eq!(region_inside(&door(1, 1)), REGION_INSIDE);
        assert_eq!(region_inside(&door(1, 2)), REGION_INSIDE_OTHER);
    }

    #[test]
    fn arrival_spots_depend_on_the_door_kind() {
        assert_eq!(inside_spot(&door(CAVE, 1)), (324, 416));
        assert_eq!(inside_spot(&door(1, 1)), (316, 400));
        assert_eq!(inside_spot(&door(2, 1)), (299, 416));
        assert_eq!(outside_spot(&door(CAVE, 1)), (96, 216));
        assert_eq!(outside_spot(&door(1, 1)), (116, 234));
        assert_eq!(outside_spot(&door(2, 1)), (120, 216));
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
        assert_eq!(survival_step(&facts, &mut hero, &mut body), vec![SurvivalStep::PickMusic]);
        facts.battle = true;
        assert!(survival_step(&facts, &mut hero, &mut body).is_empty());
    }

    #[test]
    fn hunger_messages_come_in_the_original_order() {
        let mut pack = [0u8; 40];
        let (mut facts, mut hero, mut body) = calm_hero(&mut pack);
        facts.clock = 0; // also a music frame; battle suppresses it
        hero.hunger = HUNGRY - 1;
        assert_eq!(survival_step(&facts, &mut hero, &mut body), vec![SurvivalStep::Event(0)]);
        // damage is shown (stats) before the starving messages, and the tired one comes last
        hero.hunger = 103;
        hero.fatigue = TIRED_AT - 1;
        let steps = survival_step(&facts, &mut hero, &mut body);
        assert_eq!(
            steps,
            vec![SurvivalStep::RedrawStats, SurvivalStep::Event(2), SurvivalStep::Event(3)]
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
    fn a_frame_of_the_day() {
        let t = day_step(1999, false, 0, 50, 0, STILL);
        assert_eq!((t.clock, t.light, t.lamp_on, t.period, t.period_event, t.heal), (2000, 50, false, 1, None, false));
        // a new period with a message
        let t = day_step(7999, false, 3, 50, 0, STILL);
        assert_eq!((t.period, t.period_event), (4, Some(29)));
        // the same period again says nothing
        assert_eq!(day_step(8000, false, 4, 50, 0, STILL).period_event, None);
        // night: the lamp is on
        assert!(day_step(100, false, 0, 50, 0, STILL).lamp_on);
        // frozen time stands still
        assert_eq!(day_step(500, true, 0, 50, 0, STILL).clock, 500);
        // healing lands on multiples of 1024 for a hurt hero
        assert!(day_step(1023, false, 0, 10, 0, STILL).heal);
        assert!(!day_step(1023, false, 0, 10, 0, DEAD).heal);
        assert!(!day_step(1024, false, 0, 10, 0, STILL).heal);
    }

    #[test]
    fn nearness_of_the_carrier() {
        assert_eq!(nearness(0, 0), 2);
        assert_eq!(nearness(VERY_NEAR - 1, 0), 2);
        assert_eq!(nearness(VERY_NEAR, 0), 1);
        assert_eq!(nearness(-(NEAR - 1), NEAR - 1), 1);
        assert_eq!(nearness(NEAR, 0), 0);
        assert_eq!(nearness(0, -NEAR), 0);
    }

    #[test]
    fn a_tired_hero_falls_asleep_after_thirty_frames_on_a_bed() {
        let (mut wait, mut last) = (0, BedAction::Nothing);
        for frame in 1..=BED_WAIT {
            (wait, last) = bed_step(wait, true, TOO_RESTED);
            assert_eq!(last == BedAction::FallAsleep, frame == BED_WAIT);
        }
        assert_eq!(wait, BED_WAIT);
        assert_eq!(last, BedAction::FallAsleep);
    }

    #[test]
    fn a_rested_hero_is_told_he_is_not_tired() {
        assert_eq!(bed_step(BED_WAIT - 1, true, TOO_RESTED - 1).1, BedAction::NotTired);
    }

    #[test]
    fn leaving_the_bed_resets_the_count() {
        assert_eq!(bed_step(20, false, 100), (0, BedAction::Nothing));
    }

    #[test]
    fn clock_counts_wraps_and_freezes() {
        assert_eq!(clock_step(0, false), 1);
        assert_eq!(clock_step(500, true), 500);
        assert_eq!(clock_step(DAY_LENGTH - 1, false), DAY_LENGTH);
        assert_eq!(clock_step(DAY_LENGTH, false), 0);
    }

    #[test]
    fn light_follows_the_sun() {
        assert_eq!(light_level(0), 0);
        assert_eq!(light_level(39), 0);
        assert_eq!(light_level(40), 1);
        assert_eq!(light_level(300 * 40 - 1), 299);
        assert_eq!(light_level(300 * 40), 300);
        assert_eq!(light_level(301 * 40), 299);
        assert_eq!(light_level(23999), 1);
        assert!(lamp_is_on(light_level(0)) && lamp_is_on(light_level(39 * 40 + 39)));
        assert!(!lamp_is_on(light_level(40 * 40)));
    }

    #[test]
    fn day_periods_and_messages() {
        assert_eq!(day_period(0), 0);
        assert_eq!(day_period(1999), 0);
        assert_eq!(day_period(2000), 1);
        assert_eq!(day_period(23999), 11);
        let said: Vec<_> = (0..12).filter_map(|p| period_event(p).map(|e| (p, e))).collect();
        assert_eq!(said, vec![(0, 28), (4, 29), (6, 30), (9, 31)]);
    }

    #[test]
    fn healing_every_1024_ticks() {
        assert!(heal_due(0, 10, 0, STILL));
        assert!(heal_due(1024, 10, 0, STILL));
        assert!(!heal_due(1023, 10, 0, STILL));
        assert!(!heal_due(1024, 15, 0, STILL));
        assert!(heal_due(1024, 15, 4, STILL));
        assert!(!heal_due(1024, 10, 0, DEAD));
    }

    #[test]
    fn sleep_steps_fast_and_wakes() {
        let mut drawn = 0;
        let (clock, tired, wake) = sleep_step(100, 50, false, || {
            drawn += 1;
            true
        });
        assert_eq!((clock, tired, wake, drawn), (163, 49, false, 0));
        assert!(sleep_step(100, 1, false, || false).2);
        assert!(sleep_step(9000, 20, false, || false).2);
        assert!(!sleep_step(9000, 40, false, || false).2);
        assert!(!sleep_step(8000, 20, false, || false).2);
        let mut drawn = 0;
        assert!(
            sleep_step(100, 50, true, || {
                drawn += 1;
                true
            })
            .2
        );
        assert_eq!(drawn, 1);
        let mut drawn = 0;
        sleep_step(100, 1, true, || {
            drawn += 1;
            true
        });
        assert_eq!(drawn, 0);
        assert_eq!(sleep_step(0, 0, false, || false).1, 0);
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

    /// Doors whose entrance the binary search cannot reach, because a neighbouring door's x range
    /// (16 pixels) overlaps theirs. This is how the original data behaves; they are listed so a
    /// change to the search is noticed.
    const UNREACHABLE: &[i32] = &[38, 67];

    #[test]
    fn every_other_door_is_found_at_its_own_entrance() {
        unsafe {
            for j in (0..DOORCOUNT).filter(|j| !UNREACHABLE.contains(j)) {
                let d = door_at(j);
                let found = find_outdoor_door(d.xc1, d.yc1)
                    .unwrap_or_else(|| panic!("door {j} at {},{} not found", d.xc1, d.yc1));
                let f = door_at(found);
                assert_eq!((f.xc1, f.yc1), (d.xc1, d.yc1));
            }
            for j in UNREACHABLE {
                let d = door_at(*j);
                assert_eq!(find_outdoor_door(d.xc1, d.yc1), None);
            }
        }
    }

    #[test]
    fn empty_ground_has_no_door() {
        unsafe {
            assert_eq!(find_outdoor_door(1, 1), None);
        }
    }
}

