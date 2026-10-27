//! Bringing the hero back (from csrc/fmain.c): starting a brother's adventure at the beginning, or
//! reviving the same brother at the last safe spot after the good fairy.
//!
//! The three brothers (Julian, Phillip, Kevin) play in turn. When one dies for good the next takes
//! over, with the first one's bones and goods left where they fell, until the third has died.

use super::asm_map::move_camera_to;
use super::asm_text::{draw_placard_border, placard_text, queue_print, show_event};
use super::display::reset_input;
use super::game_tables::brother_stats;
use super::globals::{panel_font, story_font};
use super::loading::load_start_sprites;
use super::menus::update_menu_availability;
use super::messages::print_cont;
use super::mood::set_mood;
use super::regions::load_region_now;
use super::screen_fx::{fade_down, map_message, message_off};
use super::state::{Hero, World, the_actors, the_hero, the_screen, the_ui, the_world};
use super::tables::{ob_list8, ob_listg};
use super::types::{Bro, Shape};
use crate::amiga::{RastPort, TextFont};

unsafe extern "C" {
    fn stopscore();
    fn Delay(ticks: i32);
    fn SetFont(current_rastport: *mut RastPort, font: *mut TextFont);
    fn SetAPen(current_rastport: *mut RastPort, pen: u32);
    fn SetRast(current_rastport: *mut RastPort, pen: u32);
    fn RectFill(current_rastport: *mut RastPort, x0: i32, y0: i32, x1: i32, y1: i32);
}

// Actor kinds and goals.
const PHIL: i8 = 0;
const RAFT: i8 = 3;
const SETFIG: i8 = 4;
const USER: i8 = 0;
const STILL: i8 = 13;

/// Where the raft and the actor in slot 2 start.
const RAFT_START: (u16, u16) = (13668, 14470);
const SETFIG_START: (u16, u16) = (13668, 15000);
/// Where the first brother starts, and the region it is in.
const FIRST_SAFE_SPOT: (u16, u16) = (19036, 15755);
const FIRST_REGION: u16 = 3;
/// The number of inventory slots that are goods (the rest are gold and arrows).
const GOLDBASE: usize = 31;
/// Where the hero is on the screen: the map is centred on the hero.
const CAMERA_OFFSET: (u16, u16) = (144, 90);
/// The last brother is number 3; one past that is the end of the game.
const LAST_BROTHER: i16 = 3;
/// Time of day when a brother starts or revives, and the light then.
const START_DAYNIGHT: u16 = 8000;
const START_LIGHT: u16 = 300;
/// Print queue entries: vitality and stats.
const PRQ_STATS: i32 = 4;
const PRQ_VITALITY: i32 = 7;
/// Object list slots: where the fallen brother's bones are and the objects that become people.
const DEAD_BROTHER_PERSON: usize = 2;
const SPECTRE_SLOT: usize = 9;
const PERSON: i8 = 3;
const ON_GROUND: i8 = 1;
/// Place message numbers: "You are in..." for the start village.
const START_PLACE: u16 = 2;
/// Sound/graphics: pen colours and the panel area that is cleared between messages.
const PEN_BLACK: u32 = 0;
const PEN_TEXT: u32 = 24;
const PANEL: (i32, i32, i32, i32) = (13, 13, 271, 107);
/// Delays, in 50ths of a second.
const SHOW_PLACARD: i32 = 120;
const SHORT_PAUSE: i32 = 80;
const TINY_PAUSE: i32 = 10;
const GAME_OVER_PAUSE: i32 = 500;
/// Value of `view_status` for "fade in" (3) and "game over screen" (2).
const VIEW_FADE_IN: i8 = 3;
const VIEW_GAME_OVER: i8 = 2;
const VIEW_REDRAW: i8 = 99;
/// Sprite file for the hero and its set number at the start.
const HERO_SPRITE_FILE: i16 = 6;
const HERO_SET_FILE: i16 = 13;

/// Revives the hero. With `new` set, a brother starts fresh: the previous one's body is left in
/// the world, the next brother's stats and an empty pack (just a dirk) are loaded, the story text
/// is shown, and the hero starts at the first safe spot. Without `new` (the good fairy), the same
/// brother returns to the last safe spot after a fade.
///
/// Either way the hero gets full health for the bravery, the clock is set to the morning
/// (8000), and hunger and fatigue are cleared.
pub fn revive(new: i16) {
    unsafe {
        let list = (&raw mut the_actors.list).cast::<Shape>();
        place_raft_and_setfig(list);

        let hero = list;
        (*hero).kind = PHIL;
        (*hero).goal = USER; // yes, we can usurp control from the user

        reset_input();
        the_actors.battle_flag = 0;
        the_hero.good_fairy_timer = 0;
        the_actors.next_missile = 0;
        if new != 0 {
            stopscore();
            leave_fallen_brother();
            take_over_next_brother(hero);
            show_brother_placards();
            announce_brother();
        } else {
            fade_down();
        }

        put_hero_at_safe_spot(hero);
        restore_hero(hero);
        if the_hero.brother > 3 {
            the_screen.view_status = VIEW_GAME_OVER;
        } else {
            the_screen.view_status = VIEW_FADE_IN;
            set_mood(1);
        }
        the_world.extent_type = 0;
        the_actors.fiery_death = 0;
    }
}

/// Puts the raft and the first person back where they start.
unsafe fn place_raft_and_setfig(list: *mut Shape) {
    unsafe {
        let raft = list.add(1);
        (*raft).kind = RAFT;
        (*raft).abs_x = RAFT_START.0;
        (*raft).abs_y = RAFT_START.1;
        (*raft).index = 0;
        (*raft).weapon = 0;
        (*raft).environ = 0;

        let setfig = list.add(2);
        (*setfig).kind = SETFIG;
        (*setfig).abs_x = SETFIG_START.0;
        (*setfig).abs_y = SETFIG_START.1;
        (*setfig).index = 0;
        (*setfig).weapon = 0;
    }
}

/// The fallen brother's body is left where he died, and his ghost appears.
unsafe fn leave_fallen_brother() {
    unsafe {
        if the_hero.brother > 0 && the_hero.brother < 3 {
            ob_listg[the_hero.brother as usize].xc = the_hero.x;
            ob_listg[the_hero.brother as usize].yc = the_hero.y;
            ob_listg[the_hero.brother as usize].ob_stat = ON_GROUND;
            ob_listg[the_hero.brother as usize + DEAD_BROTHER_PERSON].ob_stat = PERSON;
        }
        ob_list8[SPECTRE_SLOT].ob_stat = PERSON;
    }
}

/// The next brother takes over: his stats and item list, nothing in the timers, and the first
/// safe spot.
unsafe fn take_over_next_brother(hero: *mut Shape) {
    unsafe {
        start_next_brother(
            &mut *(&raw mut the_hero),
            &mut *hero,
            &brother_stats[the_hero.brother as usize],
        );

        reset_for_next_brother(&mut *(&raw mut the_hero), &mut *(&raw mut the_world));
        move_camera_to(the_hero.safe_x as i32, the_hero.safe_y as i32);
        the_screen.view_status = VIEW_REDRAW;
        the_actors.on_screen = 1;
        the_actors.loading = 0;
    }
}

/// No spells running, and the next brother starts from the first safe spot in the first region.
fn reset_for_next_brother(hero: &mut Hero, world: &mut World) {
    world.secret_timer = 0;
    world.light_timer = 0;
    world.freeze_timer = 0;
    hero.safe_x = FIRST_SAFE_SPOT.0;
    hero.safe_y = FIRST_SAFE_SPOT.1;
    hero.safe_r = FIRST_REGION;
    world.region = FIRST_REGION;
}

/// The next brother takes over: his starting stats, an empty pack except for a dirk (inventory
/// slot 0, which he also holds), and he counts as the next brother.
unsafe fn start_next_brother(hero: &mut Hero, body: &mut Shape, next: &Bro) {
    unsafe {
        hero.brave = next.brave as i16;
        hero.luck = next.luck as i16;
        hero.kindness = next.kind as i16;
        hero.wealth = next.wealth as i16;
        hero.stuff = next.stuff;
        hero.brother += 1;
        for slot in 0..GOLDBASE {
            *hero.stuff.add(slot) = 0; // has no stuff
        }
        *hero.stuff = 1; // okay, a dirk, then
        body.weapon = 1;
    }
}

/// The story pages between brothers (and the game over page after the last).
unsafe fn show_brother_placards() {
    unsafe {
        map_message();
        SetFont(the_screen.current_rastport, story_font);
        if the_hero.brother == 1 {
            placard_text(0);
            the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;
            SetRast(&raw mut the_screen.rp_map, PEN_BLACK);
            the_screen.rp_map.BitMap = (*(*the_screen.viewing_page).ri_page).BitMap;
        } else if the_hero.brother == 2 {
            placard_text(1);
        } else if the_hero.brother == 3 {
            placard_text(3);
        } else {
            placard_text(5);
        }

        draw_placard_border();
        Delay(SHOW_PLACARD);

        if the_hero.brother > LAST_BROTHER {
            the_ui.quit_requested = 1;
            Delay(GAME_OVER_PAUSE);
        } else if the_hero.brother > 1 {
            Delay(SHORT_PAUSE);
            SetAPen(the_screen.current_rastport, PEN_BLACK);
            RectFill(
                the_screen.current_rastport,
                PANEL.0,
                PANEL.1,
                PANEL.2,
                PANEL.3,
            );
            Delay(TINY_PAUSE);
            SetAPen(the_screen.current_rastport, PEN_TEXT);
            placard_text(if the_hero.brother == 2 { 2 } else { 4 });
            Delay(SHOW_PLACARD);
        }
    }
}

/// Loads the brother's sprites and tells where the story starts.
unsafe fn announce_brother() {
    unsafe {
        SetFont(the_screen.current_rastport, panel_font);
        the_screen.current_rastport = &raw mut the_screen.rp_text;

        the_actors.loaded_actor_file = HERO_SPRITE_FILE;
        the_actors.loaded_setfig_file = HERO_SET_FILE;
        load_start_sprites();

        if the_hero.brother < 4 {
            message_off();
            the_hero.place = START_PLACE;
            show_event(9);
            if the_hero.brother == 1 {
                print_cont(c".".as_ptr());
            } else if the_hero.brother == 2 {
                show_event(10);
            } else if the_hero.brother == 3 {
                show_event(11);
            }
        }
    }
}

/// The hero (re)appears at the last safe spot, and the region there is loaded.
unsafe fn put_hero_at_safe_spot(hero: *mut Shape) {
    unsafe {
        move_to_safe_spot(
            &mut *(&raw mut the_hero),
            &mut *hero,
            &mut *(&raw mut the_world),
        );
        load_region_now();
    }
}

/// The hero's position becomes the safe spot, the camera is centred on it, and its region is the
/// one to load next.
fn move_to_safe_spot(hero: &mut Hero, body: &mut Shape, world: &mut World) {
    hero.x = hero.safe_x;
    body.abs_x = hero.safe_x;
    hero.y = hero.safe_y;
    body.abs_y = hero.safe_y;
    world.map_x = hero.x.wrapping_sub(CAMERA_OFFSET.0);
    world.map_y = hero.y.wrapping_sub(CAMERA_OFFSET.1);
    world.new_region = hero.safe_r;
}

/// Fresh health, a new day, and no hunger or tiredness.
unsafe fn restore_hero(hero: *mut Shape) {
    unsafe {
        restore_stats(
            &mut *(&raw mut the_hero),
            &mut *hero,
            &mut *(&raw mut the_world),
        );
        the_actors.active = 3;
        update_menu_availability();
        queue_print(PRQ_VITALITY);
        queue_print(PRQ_STATS);
    }
}

/// Full health for the bravery, standing still on dry land, the morning clock, and no hunger or
/// fatigue.
fn restore_stats(hero: &mut Hero, body: &mut Shape, world: &mut World) {
    body.vitality = 15 + hero.brave / 4;
    body.environ = 0;
    body.state = STILL;
    body.race = 255; // -1
    world.day_clock = START_DAYNIGHT;
    world.light_level = START_LIGHT;
    hero.hunger = 0;
    hero.fatigue = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zeroed<T>() -> T {
        unsafe { std::mem::zeroed() }
    }

    #[test]
    fn a_new_brother_starts_clean_at_the_first_safe_spot() {
        let (mut hero, mut world): (Hero, World) = (zeroed(), zeroed());
        world.secret_timer = 5;
        world.light_timer = 6;
        world.freeze_timer = 7;
        world.region = 9;
        reset_for_next_brother(&mut hero, &mut world);
        assert_eq!((world.secret_timer, world.light_timer, world.freeze_timer), (0, 0, 0));
        assert_eq!(world.region, FIRST_REGION);
        assert_eq!(
            (hero.safe_x, hero.safe_y, hero.safe_r),
            (FIRST_SAFE_SPOT.0, FIRST_SAFE_SPOT.1, FIRST_REGION)
        );
    }

    #[test]
    fn the_hero_reappears_at_the_safe_spot_with_the_camera_centred() {
        let (mut hero, mut body, mut world): (Hero, Shape, World) = (zeroed(), zeroed(), zeroed());
        hero.safe_x = 1000;
        hero.safe_y = 2000;
        hero.safe_r = 5;
        move_to_safe_spot(&mut hero, &mut body, &mut world);
        assert_eq!((hero.x, hero.y, body.abs_x, body.abs_y), (1000, 2000, 1000, 2000));
        assert_eq!(
            (world.map_x, world.map_y),
            (1000 - CAMERA_OFFSET.0, 2000 - CAMERA_OFFSET.1)
        );
        assert_eq!(world.new_region, 5);
    }

    #[test]
    fn revived_hero_is_healthy_rested_and_it_is_morning() {
        let (mut hero, mut body, mut world): (Hero, Shape, World) = (zeroed(), zeroed(), zeroed());
        hero.brave = 40;
        hero.hunger = 90;
        hero.fatigue = 80;
        body.state = 15;
        body.environ = 3;
        world.day_clock = 100;
        restore_stats(&mut hero, &mut body, &mut world);
        assert_eq!((body.vitality, body.state, body.environ, body.race), (25, STILL, 0, 255));
        assert_eq!((hero.hunger, hero.fatigue), (0, 0));
        assert_eq!((world.day_clock, world.light_level), (START_DAYNIGHT, START_LIGHT));
    }

    #[test]
    fn next_brother_starts_with_his_own_stats_and_only_a_dirk() {
        let mut pack = [7u8; 64];
        let next = Bro {
            brave: 20,
            luck: 35,
            kind: 15,
            wealth: 15,
            stuff: pack.as_mut_ptr(),
        };
        let mut hero: Hero = unsafe { std::mem::zeroed() };
        hero.brother = 1;
        let mut body: Shape = unsafe { std::mem::zeroed() };
        unsafe { start_next_brother(&mut hero, &mut body, &next) };
        assert_eq!(
            (hero.brave, hero.luck, hero.kindness, hero.wealth),
            (20, 35, 15, 15)
        );
        assert_eq!(hero.brother, 2);
        assert_eq!(body.weapon, 1);
        assert_eq!(pack[0], 1);
        assert!(pack[1..GOLDBASE].iter().all(|&n| n == 0));
        assert_eq!(pack[GOLDBASE], 7); // what is past the item slots is left alone
    }
}
