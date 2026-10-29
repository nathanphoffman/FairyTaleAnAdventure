//! Drawing the item list screen: a small picture for each item the hero holds.

use crate::game::data::state::Hero;
use crate::game::data::state::Video;
use crate::game::data::tables::item_list;
use crate::game::render::screen::{BitmapId, PortId, Screen, ViewPortId};
use crate::gfx::Bitmap;
use crate::gfx::bitmap::Rect;

/// Item slots below this are shown (the rest are gold and keys' counts).
const GOLDBASE: usize = 31;
/// The sprite sheet slot of the objects.
const OBJECTS: u8 = 1;

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

/// Draws every item the hero has, stacked, on a black page.
pub fn draw_item_list(video: &mut Video, hero: &mut Hero) {
    let page = video.screen.drawing_bitmap_id();
    video.screen.port_mut(PortId::Map).target = Some(page);
    video.screen.clear(PortId::Map, 0);
    let pictures = item_pictures(video);
    video.screen.set_bitmap(BitmapId::Items, pictures);

    draw_item_icons(&mut video.screen, hero, page);

    video
        .screen
        .set_color(ViewPortId::Page, BACKGROUND_PEN, (0, 0, 0));
}

/// The pictures of the objects as one long bitmap: the object sprite sheet holds each plane of an
/// image 32 bytes after the one before, so plane `p` starts `p * 32` bytes into the sheet.
fn item_pictures(video: &mut Video) -> Bitmap {
    {
        let sheet = video.graphics.seq_list[OBJECTS as usize].location;
        let available = SHAPE_AREA_BYTES.saturating_sub(sheet);
        let mut pictures = Bitmap::new(ITEM_IMAGE_WIDTH, ITEM_IMAGE_ROWS, ITEM_IMAGE_PLANES);

        for plane in 0..ITEM_IMAGE_PLANES {
            let start = plane * ITEM_IMAGE_BYTES;
            let bytes = pictures
                .plane(plane)
                .len()
                .min(available.saturating_sub(start));
            let from = sheet + start;
            let source = &video.graphics.shape[from..from + bytes];
            pictures.plane_mut(plane)[..bytes].copy_from_slice(source);
        }

        pictures
    }
}

/// Draws one small picture for each item the hero holds (up to the most the screen shows),
/// stacked down the item's column. The pictures are read from `Items`.
fn draw_item_icons(screen: &mut Screen, hero: &mut Hero, page: BitmapId) {
    for (slot, it) in item_list.iter().enumerate().take(GOLDBASE) {
        let shown = (hero.stuff()[slot]).min(it.maxshown);
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
            screen.copy(BitmapId::Items, picture, page, (x, y as i32), ALL_PLANES);

            y = y.wrapping_add(it.ydelta as u16);
        }
    }
}
