//! The main loop (from csrc/fmain.c): one pass per frame, reading input, moving the actors, updating
//! the world and drawing.

use crate::game::data::state::Game;
use crate::game::render::draw::{draw_sprites, repair_edges};

use crate::game::actors::ai::update_enemy_goals;
use crate::game::actors::carriers::raft_proximity;
use crate::game::actors::combat::{advance_missiles, queue_missile_sprites, resolve_melee};
use crate::game::actors::movement::move_actor;
use crate::game::actors::nearby_speech::speak_nearby;
use crate::game::actors::spawning::spawn_encounters;
use crate::game::files::loading::load_pending_region;
use crate::game::input::handler::{next_key, read_walk_direction};
use crate::game::input::keys::{handle_key, update_hero};
use crate::game::render::display::flip_pages;
use crate::game::render::restore::undo_sprites;
use crate::game::render::screen_fx::{fade_normal, handle_print_request};
use crate::game::render::sprite_order::sort_actors;
use crate::game::world::bed::check_bed;
use crate::game::world::clock::day_night_tick;
use crate::game::world::map::{draw_map, scroll_map};
use crate::game::world::objects::spawn_object_actors;
use crate::game::world::regions::refresh_minimap;
use crate::game::world::survival::survival_tick;
use crate::game::world::travel_doors::check_door;

use crate::game::render::display::delay;
use crate::game::render::screen::{PortId, ViewPortId};

// Screen modes (`view_status`).
const VIEW_NORMAL: i8 = 0;
const VIEW_CHANGED: i8 = 3;
const VIEW_BIG: i8 = 1;
const VIEW_BIG_STILL: i8 = 4;
const VIEW_PAUSED: i8 = 2;
const VIEW_REDRAW_2: i8 = 98;
const VIEW_REDRAW: i8 = 99;

/// The game menu entry that pauses the game; bit 0 set means paused.
const GAME_MENU: usize = 4;
const PAUSE_ENTRY: usize = 5;

/// A region number below this means the next region is still being loaded.
const NO_REGION: u16 = 10;

/// Pens: the title text, and the colour of the flashing pointer.
const FLASH_COLOR: u32 = 15;
const SPRITE_PEN: u8 = 31;

/// Waits, in fiftieths of a second.
const PAUSE_WAIT: i32 = 200;

/// How `dif_x` wraps: a 10 bit signed value.
const DIF_SIGN_BIT: i32 = 0x200;
const DIF_SIGN_EXTEND: i32 = 0xfc00;
const DIF_MASK: i32 = 0x03ff;

/// The scroll directions `scroll_map` takes, by (dif_x, dif_y).
const SCROLL_UP_LEFT: i32 = 5;
const SCROLL_LEFT: i32 = 4;
const SCROLL_DOWN_LEFT: i32 = 3;
const SCROLL_UP: i32 = 6;
const SCROLL_DOWN: i32 = 2;
const SCROLL_UP_RIGHT: i32 = 7;
const SCROLL_RIGHT: i32 = 0;
const SCROLL_DOWN_RIGHT: i32 = 1;

/// Tiles are 16 pixels wide and 32 high: the shifts that turn map positions into tile positions,
/// and the masks of the part of a tile the map is offset by.
const TILE_WIDTH_SHIFT: u32 = 4;
const TILE_HEIGHT_SHIFT: u32 = 5;
const TILE_X_MASK: i32 = 15;
const TILE_Y_MASK: i32 = 31;

/// The status colour (31) flashes on and off with this bit of the flasher counter.
const FLASH_BIT: i8 = 16;
const STATUS_COLOR: usize = 31;

/// How the map moved since the last frame, in sectors (x wraps at 10 bits).
fn scroll_step(game: &mut Game) -> (i16, i16) {
    // Shifts by TILE_WIDTH_SHIFT, converting between pixels, tiles, sectors or bytes.
    game.world.tile_x = game.world.map_x >> TILE_WIDTH_SHIFT;

    // Shifts by TILE_HEIGHT_SHIFT, converting between pixels, tiles, sectors or bytes.
    game.world.tile_y = game.world.map_y >> TILE_HEIGHT_SHIFT;
    let page = game.screen.drawing_page();
    let dif_x = (game.world.tile_x as i32 - page.isv_x) as i16;
    let dif_y = (game.world.tile_y as i32 - page.isv_y) as i16;

    (wrap_scroll_difference(dif_x), dif_y)
}

/// The map's horizontal step as a 10 bit signed number (the map wraps round at the edges).
fn wrap_scroll_difference(difference: i16) -> i16 {
    // Masks with DIF_SIGN_BIT, keeping only the bits that matter here.
    if difference as i32 & DIF_SIGN_BIT != 0 {
        (difference as i32 | DIF_SIGN_EXTEND) as i16
    } else {
        // Masks with DIF_MASK, keeping only the bits that matter here.
        (difference as i32 & DIF_MASK) as i16
    }
}

/// What the frame loop carries from one frame to the next.
struct FrameState {
    /// The frame counter (the C left it uninitialised).
    cycle: i16,
    /// The terrain of the last actor moved. It carries over to the next actor (a dead actor reads
    /// it without setting it, see `move_actor`), and from the drawing loop to the next frame.
    last_terrain: i16,
}

/// The main loop: one pass per frame until the player quits.
pub(super) fn main_loop(game: &mut Game) {
    let mut state = FrameState {
        cycle: 0,
        last_terrain: 0,
    };

    game.ui.cheat_enabled = 0;
    game.ui.quit_requested = 0;

    while game.ui.quit_requested == 0 {
        run_frame(game, &mut state);
    }
}

/// One frame: input, moving everything, updating the map and the world, then drawing.
fn run_frame(game: &mut Game, state: &mut FrameState) {
    state.cycle = state.cycle.wrapping_add(1);
    game.screen.flasher = game.screen.flasher.wrapping_add(1);
    crate::game::statedump::run_pending_command(game); // test commands, does nothing normally

    read_keyboard(game);

    if handle_view_status(game) {
        return;
    }

    read_walk_direction(game);

    // Masks with GAME_MENU, keeping only the bits that matter here.
    if game.menus[GAME_MENU].enabled[PAUSE_ENTRY] & 1 != 0 {
        delay(game, 1);

        return;
    }

    update_hero(game, state.cycle);
    move_everything(game, state);
    check_bed(game);
    check_door(game);

    game.hero.x = game.actors.list[0].abs_x;
    game.hero.y = game.actors.list[0].abs_y;

    undo_sprites(game);
    let (scroll_x, scroll_y, world_update) = update_map_view(game);

    if world_update {
        run_world_update(game);
    }

    draw_frame(game, state, scroll_x, scroll_y);
}

/// Reads one key and handles it.
fn read_keyboard(game: &mut Game) {
    let key = next_key() as i8;

    // Masks with GAME_MENU, keeping only the bits that matter here.
    let notpause = game.menus[GAME_MENU].enabled[PAUSE_ENTRY] & 1 == 0;

    if key != 0 {
        handle_key(game, key as i32, notpause as i32);
    }
}

/// The screen modes that stop a normal frame. Returns true if this frame ends here (the item list
/// and the like only flash the pointer and handle the print queue).
fn handle_view_status(game: &mut Game) -> bool {
    if game.screen.view_status == VIEW_PAUSED {
        delay(game, PAUSE_WAIT);
        game.screen.view_status = VIEW_REDRAW;
    } else if game.screen.view_status == VIEW_BIG || game.screen.view_status == VIEW_BIG_STILL {
        // Masks with FLASH_BIT, keeping only the bits that matter here.
        if game.screen.flasher & FLASH_BIT != 0 && game.screen.view_status == VIEW_BIG {
            game.screen.set_color(
                ViewPortId::Page,
                STATUS_COLOR,
                (FLASH_COLOR, FLASH_COLOR, FLASH_COLOR),
            );
        } else {
            game.screen
                .set_color(ViewPortId::Page, STATUS_COLOR, (0, 0, 0));
        }

        handle_print_request(game);

        return true;
    }

    false
}

/// Moves the hero, the raft and every monster and person (the actor count is read each time round,
/// as in the original).
fn move_everything(game: &mut Game, state: &mut FrameState) {
    raft_proximity(game);
    let mut i = 0;

    while i < game.actors.active as i32 {
        move_actor(game, i, state.cycle, &mut state.last_terrain);
        i += 1;
    }
}

/// The scroll direction `scroll_map` takes for the map moving one tile by (x, y), if it can scroll.
fn scroll_direction(scroll_x: i16, scroll_y: i16) -> Option<i32> {
    match (scroll_x, scroll_y) {
        (1, 1) => Some(SCROLL_UP_LEFT),
        (1, 0) => Some(SCROLL_LEFT),
        (1, -1) => Some(SCROLL_DOWN_LEFT),
        (0, 1) => Some(SCROLL_UP),
        (0, -1) => Some(SCROLL_DOWN),
        (-1, 1) => Some(SCROLL_UP_RIGHT),
        (-1, 0) => Some(SCROLL_RIGHT),
        (-1, -1) => Some(SCROLL_DOWN_RIGHT),
        _ => None,
    }
}

/// Redraws the whole map when the view asks for it (two frames in a row after a change, so both
/// pages get it). Returns true if it did.
fn redraw_whole_map(game: &mut Game) -> bool {
    if game.screen.view_status != VIEW_REDRAW
        && game.screen.view_status != VIEW_REDRAW_2
        && game.screen.view_status != VIEW_CHANGED
    {
        return false;
    }

    refresh_minimap(game);
    draw_map(game);

    if game.screen.view_status == VIEW_REDRAW {
        game.screen.view_status = VIEW_REDRAW_2;
    } else if game.screen.view_status == VIEW_REDRAW_2 {
        game.screen.view_status = VIEW_NORMAL;
    }

    true
}

/// Brings the map in line with the camera: loads a waiting region, then redraws the whole map or
/// scrolls it by one tile. Returns the scroll step used and whether the world gets its turn this
/// frame (it does when the map is redrawn from scratch, or did not move at all).
fn update_map_view(game: &mut Game) -> (i16, i16, bool) {
    let (mut scroll_x, mut scroll_y) = scroll_step(game);

    if game.world.new_region < NO_REGION {
        load_pending_region(game);
    }

    if redraw_whole_map(game) {
        return (0, 0, true);
    }

    if scroll_x != 0 || scroll_y != 0 {
        refresh_minimap(game);
    }

    if let Some(direction) = scroll_direction(scroll_x, scroll_y) {
        scroll_map(game, direction);

        return (scroll_x, scroll_y, false);
    }

    if scroll_x == 0 && scroll_y == 0 {
        // no motion: the world gets its turn here
        handle_print_request(game);

        return (scroll_x, scroll_y, true);
    }

    // moved too far to scroll: redraw the whole map
    scroll_x = 0;
    scroll_y = 0;
    draw_map(game);

    (scroll_x, scroll_y, false)
}

/// The once-per-frame world update: the clock, then (unless time is frozen) new monsters, talking,
/// what the monsters want to do, and hunger.
fn run_world_update(game: &mut Game) {
    day_night_tick(game);

    if game.world.freeze_timer == 0 {
        spawn_encounters(game);
        speak_nearby(game);
        update_enemy_goals(game);
        survival_tick(game);
    }
}

/// Fights, missiles and drawing, then shows the page that was just drawn.
fn draw_frame(game: &mut Game, state: &mut FrameState, scroll_x: i16, scroll_y: i16) {
    let page = game.screen.drawing_page_mut();
    page.isv_x = game.world.tile_x as i32;
    page.isv_y = game.world.tile_y as i32;

    game.screen.set_a_pen(PortId::Map, SPRITE_PEN);

    resolve_melee(game);
    advance_missiles(game);

    game.actors.drawn = game.actors.active;
    spawn_object_actors(game);

    // the flying missiles, then sort the actors by position
    queue_missile_sprites(game);
    sort_actors(game);

    // now that the scroll blit is done, finish repairing the map
    repair_edges(game, scroll_x, scroll_y);

    draw_sprites(game, state.cycle);

    // The drawing loop used to leave `j` here; the next frame's actor loop reads it.
    state.last_terrain = game.actors.drawn;
    let ras = game.screen.drawing_page().ras;
    // Masks with TILE_X_MASK, keeping only the bits that matter here.
    game.screen.ras[ras].x = (game.world.map_x as i32 & TILE_X_MASK) as i16;
    // Masks with TILE_Y_MASK, keeping only the bits that matter here.
    game.screen.ras[ras].y = (game.world.map_y as i32 & TILE_Y_MASK) as i16;
    flip_pages(game);

    if game.screen.view_status == VIEW_CHANGED {
        fade_normal(game);
        game.screen.view_status = VIEW_NORMAL;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_scroll_steps_stay_as_they_are() {
        assert_eq!(wrap_scroll_difference(0), 0);
        assert_eq!(wrap_scroll_difference(1), 1);
        assert_eq!(wrap_scroll_difference(-1), -1);
    }

    #[test]
    fn scroll_steps_wrap_at_ten_bits() {
        // 1023 is one step the other way round the wrapped map; 1024 is no step at all
        assert_eq!(wrap_scroll_difference(0x3ff), -1);
        assert_eq!(wrap_scroll_difference(0x400), 0);
        assert_eq!(wrap_scroll_difference(0x201), -511);
    }

    #[test]
    fn the_map_scrolls_one_tile_in_each_direction() {
        assert_eq!(scroll_direction(1, 0), Some(SCROLL_LEFT));
        assert_eq!(scroll_direction(-1, 0), Some(SCROLL_RIGHT));
        assert_eq!(scroll_direction(0, 1), Some(SCROLL_UP));
        assert_eq!(scroll_direction(0, -1), Some(SCROLL_DOWN));
        assert_eq!(scroll_direction(1, 1), Some(SCROLL_UP_LEFT));
        assert_eq!(scroll_direction(-1, -1), Some(SCROLL_DOWN_RIGHT));
    }

    #[test]
    fn no_motion_and_big_jumps_do_not_scroll() {
        assert_eq!(scroll_direction(0, 0), None);
        assert_eq!(scroll_direction(2, 0), None);
        assert_eq!(scroll_direction(0, -3), None);
    }
}
