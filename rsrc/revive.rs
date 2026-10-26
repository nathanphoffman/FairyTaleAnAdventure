//! Bringing the hero back (from csrc/fmain.c): starting a brother's adventure at the beginning, or
//! reviving the same brother at the last safe spot after the good fairy.
//!
//! The three brothers (Julian, Phillip, Kevin) play in turn. When one dies for good the next takes
//! over, with the first one's bones and goods left where they fell, until the third has died.

use super::asm_map::map_adjust;
use super::asm_text::{event, placard, placard_text, prq};
use super::display::reset_input;
use super::loading::shape_read;
use super::menus::set_options;
use super::messages::print_cont;
use super::mood::setmood;
use super::regions::load_all;
use super::screen_fx::{fade_down, map_message, message_off};
use super::tables::{ob_list8, ob_listg};
use super::types::Shape;
use crate::amiga::{RastPort, TextFont};
use super::game_tables::{blist, region_num};
use super::globals::{actor_file, actors_loading, actors_on_screen, afont, anim_list, anix, battleflag, brave, brother, daynight, fatigue, fiery_death, fp_drawing, fp_viewing, freeze_timer, goodfairy, hero_place, hero_x, hero_y, hunger, kind as hero_kind, light_timer, lightlevel, luck, map_x, map_y, mdex, new_region, quitflag, rp, rp_map, rp_text, safe_r, safe_x, safe_y, secret_timer, set_file, stuff, tfont, viewstatus, wealth, xtype};

unsafe extern "C" {
    fn stopscore();
    fn Delay(ticks: i32);
    fn SetFont(rp: *mut RastPort, font: *mut TextFont);
    fn SetAPen(rp: *mut RastPort, pen: u32);
    fn SetRast(rp: *mut RastPort, pen: u32);
    fn RectFill(rp: *mut RastPort, x0: i32, y0: i32, x1: i32, y1: i32);
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
/// Value of `viewstatus` for "fade in" (3) and "game over screen" (2).
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
        let list = (&raw mut anim_list).cast::<Shape>();
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

        let hero = list;
        (*hero).kind = PHIL;
        (*hero).goal = USER; // yes, we can usurp control from the user

        reset_input();
        battleflag = 0;
        goodfairy = 0;
        mdex = 0;
        if new != 0 {
            stopscore();
            if brother > 0 && brother < 3 {
                // the fallen brother's body is left where he died, and his ghost appears
                ob_listg[brother as usize].xc = hero_x;
                ob_listg[brother as usize].yc = hero_y;
                ob_listg[brother as usize].ob_stat = ON_GROUND;
                ob_listg[brother as usize + DEAD_BROTHER_PERSON].ob_stat = PERSON;
            }
            ob_list8[SPECTRE_SLOT].ob_stat = PERSON;
            let next = &blist[brother as usize];
            brave = next.brave as i16;
            luck = next.luck as i16;
            hero_kind = next.kind as i16;
            wealth = next.wealth as i16;
            stuff = next.stuff;
            brother += 1;

            for slot in 0..GOLDBASE {
                *stuff.add(slot) = 0; // has no stuff
            }
            *stuff = 1; // okay, a dirk, then
            (*hero).weapon = 1;

            secret_timer = 0;
            light_timer = 0;
            freeze_timer = 0;
            safe_x = FIRST_SAFE_SPOT.0;
            safe_y = FIRST_SAFE_SPOT.1;
            safe_r = FIRST_REGION;
            region_num = FIRST_REGION;
            map_adjust(safe_x as i32, safe_y as i32);
            viewstatus = VIEW_REDRAW;
            actors_on_screen = 1;
            actors_loading = 0;

            map_message();
            SetFont(rp, afont);
            if brother == 1 {
                placard_text(0);
                rp_map.BitMap = (*(*fp_drawing).ri_page).BitMap;
                SetRast(&raw mut rp_map, PEN_BLACK);
                rp_map.BitMap = (*(*fp_viewing).ri_page).BitMap;
            } else if brother == 2 {
                placard_text(1);
            } else if brother == 3 {
                placard_text(3);
            } else {
                placard_text(5);
            }

            placard();
            Delay(SHOW_PLACARD);

            if brother > LAST_BROTHER {
                quitflag = 1;
                Delay(GAME_OVER_PAUSE);
            } else if brother > 1 {
                Delay(SHORT_PAUSE);
                SetAPen(rp, PEN_BLACK);
                RectFill(rp, PANEL.0, PANEL.1, PANEL.2, PANEL.3);
                Delay(TINY_PAUSE);
                SetAPen(rp, PEN_TEXT);
                placard_text(if brother == 2 { 2 } else { 4 });
                Delay(SHOW_PLACARD);
            }

            SetFont(rp, tfont);
            rp = &raw mut rp_text;

            actor_file = HERO_SPRITE_FILE;
            set_file = HERO_SET_FILE;
            shape_read();

            if brother < 4 {
                message_off();
                hero_place = START_PLACE;
                event(9);
                if brother == 1 {
                    print_cont(c".".as_ptr());
                } else if brother == 2 {
                    event(10);
                } else if brother == 3 {
                    event(11);
                }
            }
        } else {
            fade_down();
        }

        hero_x = safe_x;
        (*hero).abs_x = safe_x;
        hero_y = safe_y;
        (*hero).abs_y = safe_y;
        map_x = hero_x.wrapping_sub(CAMERA_OFFSET.0);
        map_y = hero_y.wrapping_sub(CAMERA_OFFSET.1);
        new_region = safe_r;
        load_all();
        (*hero).vitality = 15 + brave / 4;
        (*hero).environ = 0;
        (*hero).state = STILL;
        (*hero).race = 255; // -1
        daynight = START_DAYNIGHT;
        lightlevel = START_LIGHT;
        hunger = 0;
        fatigue = 0;
        anix = 3;
        set_options();
        prq(PRQ_VITALITY);
        prq(PRQ_STATS);
        if brother > 3 {
            viewstatus = VIEW_GAME_OVER;
        } else {
            viewstatus = VIEW_FADE_IN;
            setmood(1);
        }
        xtype = 0;
        fiery_death = 0;
    }
}
