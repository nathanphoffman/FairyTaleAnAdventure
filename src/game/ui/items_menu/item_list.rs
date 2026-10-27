//! Showing the list of the things the hero carries, with their pictures.

use super::{BltBitMap, GOLDBASE, InitBitMap, LoadRGB4, OBJECTS, SetRGB4, SetRast, item};
use crate::amiga::BitMap;
use crate::game::data::object_tables::page_colors;
use crate::game::data::state::{the_hero, the_memory, the_screen};
use crate::game::data::types::SeqInfo;
use crate::game::render::screen_fx::show_still_screen;
use crate::game::ui::text::queue_print;

// Print queue entries and events.
const PRQ_LIST: i32 = 5;

// Drawing the item list screen.
const ITEM_IMAGE_WIDTH: i32 = 16;
const ITEM_IMAGE_BYTES: usize = 32;
const ITEM_IMAGE_PLANES: i32 = 5;
const ITEM_IMAGE_ROWS: i32 = 8000;
const ITEM_ROW_HEIGHT: i32 = 80;
const ITEM_X_MARGIN: i32 = 20;
const BLIT_COPY: u32 = 0xC0;
const ALL_PLANES: u32 = 0xff;
const BACKGROUND_PEN: u32 = 31;

/// `view_status` value while the item list is shown.
const VIEW_ITEMS: i8 = 4;
pub(super) const VIEW_GAME_OVER: i8 = 2;
const COLOURS_SHOWN: i32 = 31;

/// Draws every item the hero has, stacked, on a black page and shows it.
pub(super) unsafe fn show_item_list() {
    unsafe {
        let bm = (*(*the_screen.drawing_page).ri_page).BitMap;
        the_screen.rp_map.BitMap = bm;
        SetRast(&raw mut the_screen.rp_map, 0);

        InitBitMap(
            &raw mut the_screen.pagea,
            ITEM_IMAGE_PLANES,
            ITEM_IMAGE_WIDTH,
            ITEM_IMAGE_ROWS,
        );

        let data = (*(&raw mut the_memory.seq_list)
            .cast::<SeqInfo>()
            .add(OBJECTS as usize))
        .location;

        for plane in 0..ITEM_IMAGE_PLANES as usize {
            the_screen.pagea.Planes[plane] = data.add(plane * ITEM_IMAGE_BYTES);
        }

        draw_item_icons(bm);

        SetRGB4(&raw mut the_screen.vp_page, BACKGROUND_PEN as i32, 0, 0, 0);
        show_still_screen();
        LoadRGB4(
            &raw mut the_screen.vp_page,
            (&raw const page_colors).cast(),
            COLOURS_SHOWN,
        );

        the_screen.view_status = VIEW_ITEMS;
        queue_print(PRQ_LIST);
    }
}

/// Draws one small picture for each item the hero holds (up to the most the screen shows),
/// stacked down the item's column. The pictures are read from `pagea`.
unsafe fn draw_item_icons(bm: *mut BitMap) {
    unsafe {
        for slot in 0..GOLDBASE {
            let it = item(slot);
            let shown = (*the_hero.stuff.add(slot)).min(it.maxshown);
            let x = it.xoff as i32 + ITEM_X_MARGIN;
            let mut y = it.yoff as u16;
            let source_row = it.image_number as i32 * ITEM_ROW_HEIGHT + it.img_off as i32;

            for _ in 0..shown {
                BltBitMap(
                    &raw mut the_screen.pagea,
                    0,
                    source_row,
                    bm,
                    x,
                    y as i32,
                    ITEM_IMAGE_WIDTH,
                    it.img_height as i32,
                    BLIT_COPY,
                    ALL_PLANES,
                    std::ptr::null_mut(),
                );

                y = y.wrapping_add(it.ydelta as u16);
            }
        }
    }
}
