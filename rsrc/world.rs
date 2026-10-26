//! The world's per-frame bookkeeping (from the main loop in csrc/fmain.c): how near the raft is,
//! sleeping on beds, going through doors, the day and night clock, new monsters, hunger, who is
//! the nearest person and the draw order of the actors.

use super::actors::{actor_distance, pick_encounter_spot};
use super::asm_blit::{SShape, restore_background};
use super::asm_map::{terrain_at, tile_address};
use super::asm_text::{queue_print, show_event, speak};
use super::encounters::load_enemy_sprites;
use super::encounters::place_encounter;
use super::game_tables::{door_list, region_num};
use super::globals::{
    active_actor_count, active_carrier, actor_list, actors_loading, actors_on_screen, battle_flag,
    bm_draw, brave, carrier_slot, danger_level, day_clock, day_period, draw_order, drawing_page,
    drawn_actor_count, encounter_number, encounter_type, encounter_x, encounter_y, extent_type,
    fatigue, freeze_timer, hero_x, hero_y, hunger, last_person, light_level, mixflag,
    nearest_person, nearest_person_distance, new_region, planes, raft_nearness, riding, rp_map,
    safe_flag, safe_r, safe_x, safe_y, shp, sleepwait, stuff, turtle_nearness, witch_on_screen, wt,
};
use super::hdrive::{motor_off, read_finished};
use super::loading::finish_sprite_load;
use super::mood::set_mood;
use super::places::{find_place, transfer_hero};
use super::random::{fta_rand, rand4, rand64};
use super::screen_fx::{day_fade, fade_page};
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

/// How close the hero is to the raft or turtle (`raft_nearness` 1 = near, 2 = very near). Riding the
/// bird sets the hero's environ.
pub fn raft_proximity() {
    unsafe {
        raft_nearness = 0;
        turtle_nearness = 0;
        carrier_slot = if active_carrier != 0 {
            CARRIER_SLOT
        } else {
            RAFT_SLOT
        };
        let carrier = &raw const actor_list[carrier_slot as usize];
        let hero = &raw mut actor_list[0];
        let dx = ((*hero).abs_x as i32 - (*carrier).abs_x as i32 - CARRIER_OFFSET) as i16;
        let dy = ((*hero).abs_y as i32 - (*carrier).abs_y as i32 - CARRIER_OFFSET) as i16;
        if dx < NEAR && dx > -NEAR && dy < NEAR && dy > -NEAR {
            raft_nearness = 1;
        }
        if dx < VERY_NEAR && dx > -VERY_NEAR && dy < VERY_NEAR && dy > -VERY_NEAR {
            raft_nearness = 2;
        }
        if riding == BIRD {
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
        let hero = &raw mut actor_list[0];
        if (*hero).state != STILL {
            sleepwait = 0;
        }
        hero_x = (*hero).abs_x;
        hero_y = (*hero).abs_y;

        if region_num == INSIDE_REGION {
            let tile = *tile_address((hero_x >> 4) as i32, (hero_y >> 5) as i32);
            if BED_TILES.contains(&tile) {
                sleepwait += 1;
                if sleepwait == BED_WAIT {
                    if fatigue < TOO_RESTED {
                        show_event(EVENT_NOT_TIRED);
                    } else {
                        show_event(EVENT_SLEEPING);
                        (*hero).abs_y |= BED_Y_MASK;
                        hero_y = (*hero).abs_y;
                        (*hero).state = SLEEP;
                    }
                }
            } else {
                sleepwait = 0;
            }
        }
    }
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

unsafe fn door_at(j: i32) -> &'static Door {
    unsafe { &*(&raw const door_list).cast::<Door>().add(j as usize) }
}

/// Is the hero standing at a door? Outdoors the doors are in sorted order and found by binary
/// search; indoors they are searched in turn. If so, the hero is moved to the other side.
/// Riding disables doors.
pub fn check_door() {
    unsafe {
        if riding != 0 {
            return;
        }
        let xtest = hero_x & 0xfff0;
        let ytest = hero_y & 0xffe0;
        if region_num < FIRST_INSIDE_REGION {
            enter_door(xtest, ytest);
        } else {
            leave_door(xtest, ytest);
        }
    }
}

unsafe fn enter_door(xtest: u16, ytest: u16) {
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
                let horizontal = d.kind & 1 != 0;
                let lined_up = if horizontal {
                    hero_y & 0x10 == 0
                } else {
                    hero_x & 15 <= 6
                };
                let allowed = d.kind != DESERT || *stuff.add(STATBASE) >= STATUES_NEEDED;
                if lined_up && allowed {
                    let (x, y) = inside_spot(d);
                    new_region = if d.secs == 1 {
                        REGION_INSIDE
                    } else {
                        REGION_INSIDE_OTHER
                    };
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
                break;
            }
            if low >= DOORCOUNT || high < 0 {
                break;
            }
        }
    }
}

unsafe fn leave_door(xtest: u16, ytest: u16) {
    unsafe {
        for j in 0..DOORCOUNT {
            let d = door_at(j);
            let horizontal = d.kind & 1 != 0;
            if d.yc2 == ytest
                && (d.xc2 == xtest || (d.xc2 as i32 == xtest as i32 - 16 && horizontal))
            {
                let lined_up = if horizontal {
                    hero_y & 0x10 != 0
                } else {
                    hero_x & 15 >= 2
                };
                if lined_up {
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
        bm_draw = (*(*drawing_page).ri_page).BitMap;
        planes = (*bm_draw).Planes.as_mut_ptr();

        OwnBlitter();
        let queue = (*drawing_page).shape_queue.cast::<SShape>();
        for i in (1..=(*drawing_page).obcount as isize).rev() {
            shp = queue.offset(i - 1);
            restore_background((*shp).backsave);
        }
        DisownBlitter();

        rp_map.BitMap = (*(*drawing_page).ri_page).BitMap;
        if (*drawing_page).wflag != 0 {
            witch_fx(drawing_page);
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

/// Sleeping, the day clock, the light level, the messages for the time of day and slow healing.
pub fn day_night_tick() {
    unsafe {
        let hero = &raw mut actor_list[0];
        if (*hero).state == SLEEP {
            sleep_tick(hero);
        }
        if freeze_timer == 0 {
            let old = day_clock;
            day_clock = old.wrapping_add(1);
            if old >= DAY_LENGTH {
                day_clock = 0;
            }
        }
        light_level = day_clock / DARK;
        if light_level >= LIGHT_PEAK {
            light_level = 2 * LIGHT_PEAK - light_level;
        }
        ob_listg[LAMP_SLOT].ob_stat = if light_level < DARK {
            LAMP_ON
        } else {
            LAMP_OFF
        };
        let period = (day_clock / PERIOD_LENGTH) as i16;
        if period != day_period {
            day_period = period;
            if let Some((_, e)) = PERIOD_EVENTS.iter().find(|(p, _)| *p == period) {
                show_event(*e);
            }
        }

        day_fade();

        if day_clock & HEAL_MASK == 0
            && (*hero).vitality < HEAL_BASE + brave / 4
            && (*hero).state != DEAD
        {
            (*hero).vitality += 1;
            queue_print(PRQ_STATS);
        }
    }
}

/// While asleep time runs fast; the hero wakes when rested, at dawn, or when a fight starts.
unsafe fn sleep_tick(hero: *mut Shape) {
    unsafe {
        day_clock = day_clock.wrapping_add(SLEEP_TIME_STEP);
        if fatigue != 0 {
            fatigue -= 1;
        }
        if fatigue == 0
            || (fatigue < LITTLE_TIRED && day_clock > DAWN.0 && day_clock < DAWN.1)
            || (battle_flag != 0 && rand64() == 0)
        {
            (*hero).state = STILL;
            (*hero).abs_y &= WAKE_Y_MASK;
            hero_y = (*hero).abs_y;
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

        if actors_loading == 1 && read_finished(DISK_REQUEST_ACTORS) != 0 {
            finish_sprite_load(ENEMY_SLOT);
            motor_off();
            actors_loading = 0;
            active_actor_count = FIRST_ENEMY_ACTOR;
        }
        if day_clock & 15 == 0 && encounter_number != 0 && actors_loading == 0 {
            place_waiting_encounters();
        }
        if day_clock & 31 == 0
            && actors_on_screen == 0
            && actors_loading == 0
            && active_carrier == 0
            && extent_type < NO_RANDOM_XTYPE
        {
            danger_level = (if region_num > INDOOR_REGION {
                5 + extent_type
            } else {
                2 + extent_type
            }) as u8;
            if rand64() <= danger_level as i32 {
                encounter_type = rand4() as u16;
                if extent_type == DKNIGHT_XTYPE && encounter_type == 2 {
                    encounter_type = 4;
                }
                if extent_type == SPIDER_PIT_XTYPE {
                    encounter_type = 6;
                    mixflag = 0;
                }
                if extent_type == FINAL_XTYPE {
                    encounter_type = 2;
                    mixflag = 0;
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
        mixflag = fta_rand() as i16;
        if extent_type > NO_MIX_XTYPE {
            mixflag = 0;
        }
        wt = rand4() as i16;
        if extent_type & 3 == 0 {
            mixflag = 0;
        }
        for _ in 1..PLACE_TRIES {
            pick_encounter_spot();
            if terrain_at(encounter_x as i32, encounter_y as i32) != PLAIN_TERRAIN {
                continue;
            }
            while encounter_number != 0 && active_actor_count < LAST_ENEMY_ACTOR {
                if place_encounter(active_actor_count as u16, SPREAD) != 0 {
                    active_actor_count += 1;
                }
                encounter_number -= 1;
            }
            let mut i = FIRST_ENEMY_ACTOR as usize;
            while i < LAST_ENEMY_ACTOR as usize && encounter_number != 0 {
                let an = &raw const actor_list[i];
                if (*an).state == DEAD && ((*an).visible == 0 || (*an).race == WRAITH) {
                    place_encounter(i as u16, SPREAD);
                    encounter_number -= 1;
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
        let race = actor_list[nearest_person as usize].race as i16;
        if nearest_person != 0 && race != last_person {
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
            last_person = race;
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

/// Notes a safe spot, picks the music, and makes the hero hungrier and more tired.
pub fn survival_tick() {
    unsafe {
        let hero = &raw mut actor_list[0];
        if day_clock & SAFE_CHECK_MASK == 0
            && actors_on_screen == 0
            && actors_loading == 0
            && witch_on_screen == 0
            && (*hero).environ == 0
            && safe_flag == 0
            && (*hero).state != DEAD
        {
            safe_r = region_num;
            safe_x = hero_x;
            safe_y = hero_y;
            if hunger > 30 && *stuff.add(FRUIT_SLOT) != 0 {
                *stuff.add(FRUIT_SLOT) -= 1;
                hunger -= FRUIT_FOOD;
                show_event(EVENT_ATE_FRUIT);
            }
        }
        if day_clock & 7 == 0 && battle_flag == 0 {
            set_mood(0);
        }
        if day_clock & SAFE_CHECK_MASK == 0 && (*hero).vitality != 0 && (*hero).state != SLEEP {
            hunger += 1;
            fatigue += 1;
            hunger_effects(hero);
            if fatigue == TIRED_AT {
                show_event(3);
            } else if hunger == STARVING_AT {
                show_event(4);
            }
        }
    }
}

/// The messages and damage that come with hunger and fatigue.
unsafe fn hunger_effects(hero: *mut Shape) {
    unsafe {
        if hunger == HUNGRY {
            show_event(0);
        } else if hunger == VERY_HUNGRY {
            show_event(1);
        } else if hunger & 7 == 0 {
            if (*hero).vitality > WEAK_VITALITY {
                if hunger > MUCH_HUNGER || fatigue > MUCH_FATIGUE {
                    (*hero).vitality -= 2;
                    queue_print(PRQ_STATS);
                }
                if hunger > STARVING_AT {
                    show_event(2);
                }
            } else if fatigue > COLLAPSE_FATIGUE {
                show_event(12);
                (*hero).state = SLEEP;
            } else if hunger > FAINT_HUNGER {
                show_event(24);
                hunger = FAINT_RESET;
                (*hero).state = SLEEP;
            }
        }
    }
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
        nearest_person = 0;
        nearest_person_distance = PERSON_RANGE;
        for i in 0..drawn_actor_count as usize {
            draw_order[i] = i as u8;
        }
        for i in 0..drawn_actor_count as usize {
            let an = &raw const actor_list[i];
            if i != 0 && (*an).kind != OBJECTS && (*an).state != DEAD && riding != BIRD {
                let dist = actor_distance(i as i32, 0) as i16;
                if dist < nearest_person_distance {
                    nearest_person_distance = dist;
                    nearest_person = i as i16;
                }
            }
            for j in 1..drawn_actor_count as usize {
                bubble_step(j);
            }
        }
    }
}

/// One compare-and-swap of neighbours `j - 1` and `j` in the draw order.
unsafe fn bubble_step(j: usize) {
    unsafe {
        let k1 = draw_order[j - 1];
        let k2 = draw_order[j];
        let a = &raw const actor_list[k1 as usize];
        let b = &raw const actor_list[k2 as usize];
        let mut y1 = (*a).abs_y as i16;
        let mut y2 = (*b).abs_y as i16;
        if (*a).state == DEAD || (k2 == 0 && riding != 0) || k1 == RAFT_ACTOR {
            y1 = y1.wrapping_sub(DRAW_LOWER);
        }
        if (*b).state == DEAD || (k1 == 0 && riding != 0) || k2 == RAFT_ACTOR {
            y2 = y2.wrapping_sub(DRAW_LOWER);
        }
        if (*a).environ > WADING {
            y1 = y1.wrapping_add(DRAW_LOWER);
        }
        if (*b).environ > WADING {
            y2 = y2.wrapping_add(DRAW_LOWER);
        }
        if y2 < y1 {
            draw_order[j - 1] = k2;
            draw_order[j] = k1;
        }
    }
}
