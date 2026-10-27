//! Showing the list of the things the hero carries, with their pictures.

use super::{GOLDBASE, OBJECTS, item};
use crate::game::data::object_tables::page_colors;
use crate::game::data::state::{memory, the_hero};
use crate::game::render::screen::{BitmapId, PortId, ViewPortId, screen};
use crate::game::render::screen_fx::show_still_screen;
use crate::game::ui::text::queue_print;
use crate::gfx::Bitmap;
use crate::gfx::bitmap::Rect;

// Print queue entries and events.
const PRQ_LIST: i32 = 5;

// Drawing the item list screen.
const ITEM_IMAGE_WIDTH: usize = 16;
const ITEM_IMAGE_BYTES: usize = 32;
const ITEM_IMAGE_PLANES: usize = 5;
const ITEM_IMAGE_ROWS: usize = 8000;

/// The shape memory the object pictures are read from has this many bytes.
const SHAPE_AREA_BYTES: usize = 78000;
const ITEM_ROW_HEIGHT: i32 = 80;
const ITEM_X_MARGIN: i32 = 20;
const ALL_PLANES: u32 = 0xff;
const BACKGROUND_PEN: usize = 31;

/// `view_status` value while the item list is shown.
const VIEW_ITEMS: i8 = 4;
pub(super) const VIEW_GAME_OVER: i8 = 2;

/// Draws every item the hero has, stacked, on a black page and shows it.
pub(super) unsafe fn show_item_list() {
    let screen = screen();
    let page = screen.drawing_bitmap_id();
    screen.port_mut(PortId::Map).target = Some(page);
    screen.clear(PortId::Map, 0);
    screen.set_bitmap(BitmapId::Items, item_pictures());

    draw_item_icons(page);

    screen.set_color(ViewPortId::Page, BACKGROUND_PEN, (0, 0, 0));
    show_still_screen();
    screen.load_colors(ViewPortId::Page, unsafe { &*(&raw const page_colors) });

    screen.view_status = VIEW_ITEMS;
    queue_print(PRQ_LIST);
}

/// The pictures of the objects as one long bitmap: the object sprite sheet holds each plane of an
/// image 32 bytes after the one before, so plane `p` starts `p * 32` bytes into the sheet.
fn item_pictures() -> Bitmap {
    {
        let sheet = memory().seq_list[OBJECTS as usize].location;
        let available = SHAPE_AREA_BYTES.saturating_sub(sheet);
        let mut pictures = Bitmap::new(ITEM_IMAGE_WIDTH, ITEM_IMAGE_ROWS, ITEM_IMAGE_PLANES);

        for plane in 0..ITEM_IMAGE_PLANES {
            let start = plane * ITEM_IMAGE_BYTES;
            let bytes = pictures
                .plane(plane)
                .len()
                .min(available.saturating_sub(start));
            let from = sheet + start;
            let source = &memory().shape[from..from + bytes];
            pictures.plane_mut(plane)[..bytes].copy_from_slice(source);
        }

        pictures
    }
}

/// Draws one small picture for each item the hero holds (up to the most the screen shows),
/// stacked down the item's column. The pictures are read from `Items`.
fn draw_item_icons(page: BitmapId) {
    for slot in 0..GOLDBASE {
        let it = item(slot);
        let shown = unsafe { (*the_hero.stuff.add(slot)).min(it.maxshown) };
        let x = it.xoff as i32 + ITEM_X_MARGIN;
        let mut y = it.yoff as u16;
        let source_row = it.image_number as i32 * ITEM_ROW_HEIGHT + it.img_off as i32;
        let picture = Rect {
            x: 0,
            y: source_row,
            width: ITEM_IMAGE_WIDTH as i32,
            height: it.img_height as i32,
        };

        for _ in 0..shown {
            screen().copy(BitmapId::Items, picture, page, (x, y as i32), ALL_PLANES);

            y = y.wrapping_add(it.ydelta as u16);
        }
    }
}
