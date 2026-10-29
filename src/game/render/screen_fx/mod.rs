//! Screen colour effects and panel updates (from csrc/fmain2.c): the compass, fading the page
//! colours with the time of day, the teleport flash, the print queue and the win sequence.
//! The colour tables live in `tables`; the view ports are still defined in csrc/fmain.c.
//!
//! Colours: the screen has 32 colours, each 12 bits (4 bits red, green, blue). The game
//! fades the whole picture by scaling those components (`fade_page`) and loads them as the page palette.
//! Night is a darker, bluer scaling; indoors it is always full brightness.

mod compass;

mod fading;

mod item_screen;

mod message_page;

mod print_queue;

mod win;

mod zoom;

pub use self::compass::draw_compass;
pub use self::fading::{day_fade, fade_down, fade_normal, fade_page, flicker_colors};
pub use self::item_screen::draw_item_list;
pub use self::message_page::{map_message, message_off, show_still_screen};
pub use self::print_queue::handle_print_request;
pub use self::win::win_colors;
pub use self::zoom::set_intro_zoom;

const COLOR_COUNT: usize = 32;
