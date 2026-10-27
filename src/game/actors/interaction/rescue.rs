//! Rescuing the princess: the story page, the reward and the keys.

use super::*;

// The rescue scene.
const PLACARD_PEN: u32 = 24;
const RESCUE_PAUSE: i32 = 380;
const CLEAR_PAUSE: i32 = 10;
const PLACARD_AREA: (i32, i32, i32, i32) = (13, 13, 271, 107);
const PRINCESS_HOME: (u16, u16) = (5511, 33780);
const PRINCESS_EXTENT: (i16, i16) = (22205, 21231);
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
pub fn rescue_princess() {
    unsafe {
        map_message();
        SetFont(the_screen.current_rastport, story_font);
        let line = princesses_rescued as i32 * 3;
        placard_text(8 + line);
        name();
        placard_text(9 + line);
        name();
        placard_text(10 + line);
        draw_placard_border();
        Delay(RESCUE_PAUSE);
        SetAPen(the_screen.current_rastport, 0);
        let (x0, y0, x1, y1) = PLACARD_AREA;
        RectFill(the_screen.current_rastport, x0, y0, x1, y1);
        Delay(CLEAR_PAUSE);
        SetAPen(the_screen.current_rastport, PLACARD_PEN);
        placard_text(17);
        name();
        placard_text(18);
        Delay(RESCUE_PAUSE);
        message_off();

        princesses_rescued += 1;
        transfer_hero(PRINCESS_HOME.0, PRINCESS_HOME.1, 0); // do princess transfer
        center_extent_on(0, PRINCESS_EXTENT.0, PRINCESS_EXTENT.1);
        let objects = (&raw mut ob_list8).cast::<WorldObject>();
        (*objects.add(PRINCESS_OBJECT)).ob_id = PRINCESS_OBJECT_ID;
        *the_hero.stuff.add(PRINCESS_STUFF) = 1;
        speak(REWARD_MESSAGE);
        the_hero.wealth = the_hero.wealth.wrapping_add(REWARD_GOLD);
        (*objects.add(CAPTIVE_OBJECT)).ob_stat = 0;
        for item in KEY_ITEMS {
            *the_hero.stuff.add(item) = (*the_hero.stuff.add(item)).wrapping_add(KEYS_REWARD);
        }
    }
}
