//! Handing over from a fallen brother to the next: his body, the placards and the new stats.

use crate::game::data::globals::{panel_font, story_font};
use crate::game::data::object_tables::{ob_list8, ob_listg};
use crate::game::data::state::{Hero, World, the_actors, the_hero, the_screen, the_ui, the_world};
use crate::game::data::tables::brother_stats;
use crate::game::data::types::{Bro, Shape};
use crate::game::files::loading::load_start_sprites;
use crate::game::render::screen_fx::{map_message, message_off};
use crate::game::ui::messages::print_cont;
use crate::game::ui::text::{draw_placard_border, placard_text, show_event};
use crate::game::world::map::move_camera_to;
use crate::platform::display::delay;
use crate::platform::gfx::{rect_fill, set_a_pen, set_font, set_rast};

const RAFT: i8 = 3;
const SETFIG: i8 = 4;

/// Where the raft and the actor in slot 2 start.
const RAFT_START: (u16, u16) = (13668, 14470);
const SETFIG_START: (u16, u16) = (13668, 15000);

/// Where the first brother starts, and the region it is in.
pub(super) const FIRST_SAFE_SPOT: (u16, u16) = (19036, 15755);
pub(super) const FIRST_REGION: u16 = 3;

/// The number of inventory slots that are goods (the rest are gold and arrows).
pub(super) const GOLDBASE: usize = 31;

/// The last brother is number 3; one past that is the end of the game.
pub(super) const LAST_BROTHER: i16 = 3;

/// The brothers in order; their numbers go up with each fallen one.
const JULIAN: i16 = 1;
const PHILLIP: i16 = 2;
const KEVIN: i16 = 3;

/// Placard text numbers: the first brother's, the second's, ... the game-over one, then the
/// follow-up placards that tell why the next brother sets out.
const PLACARD_JULIAN: i32 = 0;
const PLACARD_PHILLIP: i32 = 1;
const PLACARD_PHILLIP_FOLLOW_UP: i32 = 2;
const PLACARD_KEVIN: i32 = 3;
const PLACARD_KEVIN_FOLLOW_UP: i32 = 4;
const PLACARD_GAME_OVER: i32 = 5;

/// Event messages for setting out: always the first, then one for the second and third brother.
const EVENT_SETTING_OUT: i32 = 9;
const EVENT_PHILLIP_SETS_OUT: i32 = 10;
const EVENT_KEVIN_SETS_OUT: i32 = 11;

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
const VIEW_REDRAW: i8 = 99;

/// Sprite file for the hero and its set number at the start.
const HERO_SPRITE_FILE: i16 = 6;
const HERO_SET_FILE: i16 = 13;

/// Puts the raft and the first person back where they start.
pub(super) unsafe fn reset_raft_setfig(list: *mut Shape) {
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
pub(super) unsafe fn leave_fallen_brother() {
    unsafe {
        if the_hero.brother > 0 && the_hero.brother < KEVIN {
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
pub(super) unsafe fn take_over_next_brother(hero: *mut Shape) {
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
pub(super) fn reset_for_next_brother(hero: &mut Hero, world: &mut World) {
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
pub(super) unsafe fn start_next_brother(hero: &mut Hero, body: &mut Shape, next: &Bro) {
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
pub(super) unsafe fn show_brother_placards() {
    unsafe {
        map_message();
        set_font(the_screen.current_rastport, story_font);

        if the_hero.brother == JULIAN {
            placard_text(PLACARD_JULIAN);
            the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;
            set_rast(&raw mut the_screen.rp_map, PEN_BLACK);
            the_screen.rp_map.BitMap = (*(*the_screen.viewing_page).ri_page).BitMap;
        } else if the_hero.brother == PHILLIP {
            placard_text(PLACARD_PHILLIP);
        } else if the_hero.brother == KEVIN {
            placard_text(PLACARD_KEVIN);
        } else {
            placard_text(PLACARD_GAME_OVER);
        }

        draw_placard_border();
        delay(SHOW_PLACARD);

        if the_hero.brother > LAST_BROTHER {
            the_ui.quit_requested = 1;
            delay(GAME_OVER_PAUSE);
        } else if the_hero.brother > JULIAN {
            show_follow_up_placard();
        }
    }
}

/// After the first story page the panel is cleared and the follow-up page for this brother is shown.
unsafe fn show_follow_up_placard() {
    unsafe {
        delay(SHORT_PAUSE);
        set_a_pen(the_screen.current_rastport, PEN_BLACK);
        rect_fill(
            the_screen.current_rastport,
            PANEL.0,
            PANEL.1,
            PANEL.2,
            PANEL.3,
        );

        delay(TINY_PAUSE);
        set_a_pen(the_screen.current_rastport, PEN_TEXT);

        placard_text(if the_hero.brother == PHILLIP {
            PLACARD_PHILLIP_FOLLOW_UP
        } else {
            PLACARD_KEVIN_FOLLOW_UP
        });

        delay(SHOW_PLACARD);
    }
}

/// Loads the brother's sprites and tells where the story starts.
pub(super) unsafe fn announce_brother() {
    unsafe {
        set_font(the_screen.current_rastport, panel_font);
        the_screen.current_rastport = &raw mut the_screen.rp_text;

        the_actors.loaded_actor_file = HERO_SPRITE_FILE;
        the_actors.loaded_setfig_file = HERO_SET_FILE;
        load_start_sprites();

        if the_hero.brother <= LAST_BROTHER {
            message_off();
            the_hero.place = START_PLACE;
            show_event(EVENT_SETTING_OUT);

            if the_hero.brother == JULIAN {
                print_cont(c".".as_ptr());
            } else if the_hero.brother == PHILLIP {
                show_event(EVENT_PHILLIP_SETS_OUT);
            } else if the_hero.brother == KEVIN {
                show_event(EVENT_KEVIN_SETS_OUT);
            }
        }
    }
}
