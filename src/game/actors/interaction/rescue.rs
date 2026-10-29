//! Rescuing the princess: the story page, the reward and the keys.

use super::extents::center_extent_on;
use crate::game::data::state::Game;
use crate::game::output::Output;
use crate::game::render::display::delay;
use crate::game::render::screen::FontKind;

use crate::game::render::screen_fx::{map_message, message_off};
use crate::game::ui::messages::name;
use crate::game::ui::text::{draw_placard_border, placard_text, speak};
use crate::game::world::places::transfer_hero;

// The rescue scene.
const PLACARD_PEN: u8 = 24;

/// Each princess has three story placards, starting at 8 for the first; the closing two follow.
const RESCUE_PLACARD_BASE: i32 = 8;
const RESCUE_PLACARDS_EACH: i32 = 3;
const AFTER_RESCUE_PLACARDS: (i32, i32) = (17, 18);
const RESCUE_PAUSE: i32 = 380;
const CLEAR_PAUSE: i32 = 10;
const PLACARD_AREA: (i32, i32, i32, i32) = (13, 13, 271, 107);
const PRINCESS_HOME: (u16, u16) = (5511, 33780);
const PRINCESS_EXTENT: (i16, i16) = (22205, 21231);

/// The object list of the interior regions.
const INSIDE_OBJECTS: usize = 8;

/// Entries of the interior object list (`ob_list8`) that change when the princess is rescued.
const PRINCESS_OBJECT: usize = 2;
const PRINCESS_OBJECT_ID: i8 = 4;
const CAPTIVE_OBJECT: usize = 9;
const PRINCESS_STUFF: usize = 28;

/// Items 16 to 21 (the keys) get this many extra.
const KEY_ITEMS: std::ops::Range<usize> = 16..22;
const KEYS_REWARD: u8 = 3;
const REWARD_MESSAGE: i32 = 18;
const REWARD_GOLD: i16 = 100;

/// The scene when a princess is rescued, and her reward.
///
/// Shows the story pages, moves the hero to the princess's home, repositions the extent for the
/// next princess, changes the princess object, gives 100 gold and three of each key, and bumps
/// the `princess` count (each rescue shows different story pages).
pub fn rescue_princess(game: &mut Game, out: &mut dyn Output) {
    map_message(game, out);
    let current = out.current_port();
    out.set_font_kind(current, FontKind::Story);
    let line = game.hero.princesses_rescued as i32 * RESCUE_PLACARDS_EACH;
    placard_text(out, RESCUE_PLACARD_BASE + line);
    name(game, out);
    placard_text(out, RESCUE_PLACARD_BASE + 1 + line);
    name(game, out);
    placard_text(out, RESCUE_PLACARD_BASE + 2 + line);
    draw_placard_border(out);
    delay(game, out, RESCUE_PAUSE);
    out.set_a_pen(current, 0);
    let (x0, y0, x1, y1) = PLACARD_AREA;
    out.rect_fill(current, (x0, y0), (x1, y1));
    delay(game, out, CLEAR_PAUSE);
    out.set_a_pen(current, PLACARD_PEN);
    placard_text(out, AFTER_RESCUE_PLACARDS.0);
    name(game, out);
    placard_text(out, AFTER_RESCUE_PLACARDS.1);
    delay(game, out, RESCUE_PAUSE);
    message_off(game, out);

    game.hero.princesses_rescued += 1;
    transfer_hero(game, out, PRINCESS_HOME.0, PRINCESS_HOME.1, 0); // do princess transfer
    center_extent_on(game, 0, PRINCESS_EXTENT.0, PRINCESS_EXTENT.1);
    game.objects.region(INSIDE_OBJECTS)[PRINCESS_OBJECT].ob_id = PRINCESS_OBJECT_ID;
    game.hero.stuff_mut()[PRINCESS_STUFF] = 1;
    speak(game, out, REWARD_MESSAGE);
    game.hero.wealth = game.hero.wealth.wrapping_add(REWARD_GOLD);
    game.objects.region(INSIDE_OBJECTS)[CAPTIVE_OBJECT].ob_stat = 0;

    for item in KEY_ITEMS {
        game.hero.stuff_mut()[item] = (game.hero.stuff()[item]).wrapping_add(KEYS_REWARD);
    }
}
