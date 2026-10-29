//! Handing over from a fallen brother to the next: his body, the placards and the new stats.

use crate::game::data::object_tables::ObjectLists;
use crate::game::data::state::ActorRef;
use crate::game::data::state::Actors;
use crate::game::data::state::Game;
use crate::game::data::state::{Hero, World};
use crate::game::data::tables::brother_stats;
use crate::game::data::types::{Bro, Shape};
use crate::game::files::loading::load_start_sprites;
use crate::game::output::Output;
use crate::game::render::display::delay;
use crate::game::render::screen::{FontKind, PortId};
use crate::game::render::screen_fx::{map_message, message_off};
use crate::game::ui::text::{draw_placard_border, placard_text, show_event};
use crate::game::world::map::move_camera_to;

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
const PEN_BLACK: u8 = 0;
const PEN_TEXT: u8 = 24;
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
pub(super) fn reset_raft_setfig(actors: &mut Actors) {
    let raft = ActorRef::new(1);
    actors.actor_mut(raft).kind = RAFT;
    actors.actor_mut(raft).abs_x = RAFT_START.0;
    actors.actor_mut(raft).abs_y = RAFT_START.1;
    actors.actor_mut(raft).index = 0;
    actors.actor_mut(raft).weapon = 0;
    actors.actor_mut(raft).environ = 0;

    let setfig = ActorRef::new(2);
    actors.actor_mut(setfig).kind = SETFIG;
    actors.actor_mut(setfig).abs_x = SETFIG_START.0;
    actors.actor_mut(setfig).abs_y = SETFIG_START.1;
    actors.actor_mut(setfig).index = 0;
    actors.actor_mut(setfig).weapon = 0;
}

/// The fallen brother's body is left where he died, and his ghost appears.
pub(super) fn leave_fallen_brother(objects: &mut ObjectLists, hero: &Hero) {
    if hero.brother > 0 && hero.brother < KEVIN {
        objects.global[hero.brother as usize].xc = hero.x;
        objects.global[hero.brother as usize].yc = hero.y;
        objects.global[hero.brother as usize].ob_stat = ON_GROUND;
        objects.global[hero.brother as usize + DEAD_BROTHER_PERSON].ob_stat = PERSON;
    }

    objects.region[8][SPECTRE_SLOT].ob_stat = PERSON;
}

/// The next brother takes over: his stats and item list, nothing in the timers, and the first
/// safe spot.
pub(super) fn take_over_next_brother(game: &mut Game, _out: &mut dyn Output, hero: ActorRef) {
    let stats = &brother_stats[game.hero.brother as usize];
    start_next_brother(&mut game.hero, &mut game.actors.list[hero.slot()], stats);

    reset_for_next_brother(&mut game.hero, &mut game.world);
    move_camera_to(
        &mut game.world,
        game.hero.safe_x as i32,
        game.hero.safe_y as i32,
    );
    game.ui.view_status = VIEW_REDRAW;
    game.actors.on_screen = 1;
    game.actors.loading = 0;
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
pub(super) fn start_next_brother(hero: &mut Hero, body: &mut Shape, next: &Bro) {
    hero.brave = next.brave as i16;
    hero.luck = next.luck as i16;
    hero.kindness = next.kind as i16;
    hero.wealth = next.wealth as i16;
    hero.brother += 1;

    let stuff = hero.stuff_mut();

    stuff[..GOLDBASE].fill(0); // has no stuff

    stuff[0] = 1; // okay, a dirk, then
    body.weapon = 1;
}

/// The story pages between brothers (and the game over page after the last).
pub(super) fn show_brother_placards(game: &mut Game, out: &mut dyn Output) {
    map_message(game, out);
    let current = out.current_port();
    out.set_font_kind(current, FontKind::Story);

    if game.hero.brother == JULIAN {
        placard_text(out, PLACARD_JULIAN);
        out.clear_drawing_page(PEN_BLACK);
    } else if game.hero.brother == PHILLIP {
        placard_text(out, PLACARD_PHILLIP);
    } else if game.hero.brother == KEVIN {
        placard_text(out, PLACARD_KEVIN);
    } else {
        placard_text(out, PLACARD_GAME_OVER);
    }

    draw_placard_border(out);
    delay(game, out, SHOW_PLACARD);

    if game.hero.brother > LAST_BROTHER {
        game.ui.quit_requested = 1;
        delay(game, out, GAME_OVER_PAUSE);
    } else if game.hero.brother > JULIAN {
        show_follow_up_placard(game, out);
    }
}

/// After the first story page the panel is cleared and the follow-up page for this brother is shown.
fn show_follow_up_placard(game: &mut Game, out: &mut dyn Output) {
    delay(game, out, SHORT_PAUSE);
    let current = out.current_port();
    out.set_a_pen(current, PEN_BLACK);
    out.rect_fill(current, (PANEL.0, PANEL.1), (PANEL.2, PANEL.3));

    delay(game, out, TINY_PAUSE);
    out.set_a_pen(current, PEN_TEXT);

    placard_text(
        out,
        if game.hero.brother == PHILLIP {
            PLACARD_PHILLIP_FOLLOW_UP
        } else {
            PLACARD_KEVIN_FOLLOW_UP
        },
    );

    delay(game, out, SHOW_PLACARD);
}

/// Loads the brother's sprites and tells where the story starts.
pub(super) fn announce_brother(game: &mut Game, out: &mut dyn Output) {
    let current = out.current_port();
    out.set_font_kind(current, FontKind::Panel);
    out.select_port(PortId::Text);

    game.actors.loaded_actor_file = HERO_SPRITE_FILE;
    game.actors.loaded_setfig_file = HERO_SET_FILE;
    load_start_sprites(game, out);

    if game.hero.brother <= LAST_BROTHER {
        message_off(game, out);
        game.hero.place = START_PLACE;
        show_event(game, out, EVENT_SETTING_OUT);

        if game.hero.brother == JULIAN {
            out.print_cont(".");
        } else if game.hero.brother == PHILLIP {
            show_event(game, out, EVENT_PHILLIP_SETS_OUT);
        } else if game.hero.brother == KEVIN {
            show_event(game, out, EVENT_KEVIN_SETS_OUT);
        }
    }
}
