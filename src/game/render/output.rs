//! `Video` as an `Output`: the Amiga-style drawing of what the rules ask for.

use crate::game::data::state::{Hero, MapData, Video, World};
use crate::game::files::disk::load_track_range;
use crate::game::files::iff_picture::load_picture;
use crate::game::output::Output;
use crate::game::render::overview_map::draw_overview_map;
use crate::game::render::placard_border::draw_placard_border;
use crate::game::render::screen::{FontKind, HIRES, PortId, SPRITES, VP_HIDE, ViewPortId};
use crate::game::render::screen_fx::{draw_compass, draw_item_list};
use crate::game::render::sprite_loading::{finish_sprite_load, load_sprite_file};
use crate::game::ui::text::{draw_text, format_number, move_to};
use crate::gfx::bitmap::Rect;
use crate::gfx::draw::JAM1;
use crate::platform::display;

// The scrolling message area of the text panel.
const TEXT_X_MIN: i32 = 16;
const TEXT_X_MAX: i32 = 400;
const TEXT_Y_MIN: i32 = 5;
const TEXT_Y_MAX: i32 = 44;

/// Pixels the panel scrolls up for each new line, and the baseline of that new line.
const LINE_HEIGHT: i32 = 10;
const NEW_LINE_Y: i32 = 42;

/// The tile images are in 5 planes of 16384 bytes; a bank is a quarter of each plane. A plane of a
/// bank is 8 disk blocks, and its request numbers start at 3.
const PLANES: usize = 5;
const IMAGE_PLANE_SIZE: usize = 16384;
const QUARTER_PLANE_SIZE: usize = 4096;
const IMAGE_PLANE_BLOCKS: i16 = 8;
const REQ_IMAGE_FIRST: i16 = 3;

/// The colour the hero's "+" is drawn in on the overview map.
const MAP_MARK_PEN: u8 = 31;

/// The intro zoom is centred on a 320 x 200 screen, and the text panel gets what is left of 95 lines.
const SCREEN_CENTER_X: i32 = 160;
const SCREEN_CENTER_Y: i32 = 100;
const ZOOM_TEXT_HEIGHT: i32 = 95;

/// A story page's text area (top left and bottom right corners), and its colours.
const PLACARD_AREA: ((i32, i32), (i32, i32)) = ((13, 13), (271, 107));
const PEN_BLACK: u8 = 0;
const PLACARD_TEXT_PEN: u8 = 24;

// Where the menu entries are drawn in the text panel: two columns, a row every 9 pixels.
const MENU_LEFT_COLUMN_X: i32 = 430;
const MENU_RIGHT_COLUMN_X: i32 = 482;
const MENU_FIRST_ROW_Y: i32 = 8;
const MENU_ROW_HEIGHT: i32 = 9;

/// The entry's text is inset from the edge of its coloured box by this much.
const MENU_TEXT_INSET: i32 = 4;

/// Each label is 5 characters; the box behind it is 6 wide.
const MENU_ENTRY_WIDTH: usize = 6;
const MENU_BOX: &[u8; MENU_ENTRY_WIDTH] = b"      ";

/// The top left corner of menu entry `slot`: even slots on the left, odd on the right.
fn menu_slot_position(slot: usize) -> (i32, i32) {
    let x = if slot & 1 != 0 {
        MENU_RIGHT_COLUMN_X
    } else {
        MENU_LEFT_COLUMN_X
    };

    (x, (slot / 2) as i32 * MENU_ROW_HEIGHT + MENU_FIRST_ROW_Y)
}

/// Characters in a formatted number (the last one is the trailing space).
const NUMBER_FIELD: i32 = 10;

impl Output for Video {
    /// The old lines scroll up by one text line (10 pixels) inside the message area.
    fn print_bytes(&mut self, text: &[u8]) {
        let area = Rect {
            x: TEXT_X_MIN,
            y: TEXT_Y_MIN,
            width: TEXT_X_MAX - TEXT_X_MIN + 1,
            height: TEXT_Y_MAX - TEXT_Y_MIN + 1,
        };

        self.screen
            .scroll(self.screen.current_port, area, (0, LINE_HEIGHT));

        move_to(self, TEXT_X_MIN, NEW_LINE_Y);
        draw_text(&mut self.screen, text);
    }

    fn print_cont_bytes(&mut self, text: &[u8]) {
        draw_text(&mut self.screen, text);
    }

    fn print_number(&mut self, value: i32, len: i32) {
        let text = format_number(value as u32);
        let from = (NUMBER_FIELD - len).max(0) as usize;

        draw_text(&mut self.screen, &text[from..]);
    }

    fn load_page_colors(&mut self, colors: &[u16]) {
        self.screen.load_colors(ViewPortId::Page, colors);
    }

    fn swap_pages(&mut self) {
        let finished = self.screen.drawing;
        std::mem::swap(&mut self.screen.drawing, &mut self.screen.viewing);
        self.screen.vp_page.ras = Some(self.screen.pages[finished].ras);
    }

    fn present(&mut self) {
        display::present(&self.screen.scene());
    }

    fn wait_frame(&mut self) {
        display::wait_frame(&self.screen.scene());
    }

    fn ticks(&self) -> u32 {
        display::ticks()
    }

    fn forget_sprites(&mut self) {
        self.graphics.nextshape = 0;
    }

    fn rewind_sprites_to(&mut self, slot: u8) {
        self.graphics.nextshape = self.graphics.seq_list[slot as usize].location;
    }

    fn start_sprite_sheet(&mut self, slot: u8) {
        self.graphics.seq_list[slot as usize].location = self.graphics.nextshape;
    }

    fn load_sprite_file(&mut self, file: i32) {
        load_sprite_file(self, file);
    }

    fn finish_sprite_load(&mut self, slot: i16) {
        finish_sprite_load(self, slot);
    }

    fn load_image_bank(&mut self, bank: usize, first_block: i16) {
        for plane in 0..PLANES {
            let from = bank * QUARTER_PLANE_SIZE + plane * IMAGE_PLANE_SIZE;
            let block = first_block + plane as i16 * IMAGE_PLANE_BLOCKS;

            load_track_range(
                block,
                IMAGE_PLANE_BLOCKS,
                &mut self.graphics.image[from..],
                REQ_IMAGE_FIRST + plane as i16,
            );
        }
    }

    fn set_text_position(&mut self, x: i32, y: i32) {
        self.screen.move_to(self.screen.current_port, x, y);
    }

    fn use_story_font(&mut self) {
        self.screen
            .set_font_kind(self.screen.current_port, FontKind::Story);
    }

    fn use_panel_font(&mut self) {
        self.screen
            .set_font_kind(self.screen.current_port, FontKind::Panel);
    }

    fn select_message_area(&mut self) {
        self.screen.current_port = PortId::Text;
    }

    fn clear_placard(&mut self) {
        let port = self.screen.current_port;
        self.screen.set_a_pen(port, PEN_BLACK);
        self.screen.rect_fill(port, PLACARD_AREA.0, PLACARD_AREA.1);
        self.screen.set_a_pen(port, PLACARD_TEXT_PEN);
    }

    fn draw_placard_border(&mut self) {
        draw_placard_border(&mut self.screen);
    }

    fn draw_menu_entry(&mut self, slot: usize, label: &[u8], selected: bool, box_pen: u8) {
        let (x, y) = menu_slot_position(slot);
        self.screen.set_a_pen(PortId::Text2, selected as u8);
        self.screen.set_b_pen(PortId::Text2, box_pen);
        self.screen.move_to(PortId::Text2, x, y);
        self.screen.text(PortId::Text2, MENU_BOX);
        self.screen.move_to(PortId::Text2, x + MENU_TEXT_INSET, y);
        self.screen.text(PortId::Text2, label);
    }

    fn blank_menu_entry(&mut self, slot: usize) {
        let (x, y) = menu_slot_position(slot);
        self.screen.set_b_pen(PortId::Text2, PEN_BLACK);
        self.screen.move_to(PortId::Text2, x, y);
        self.screen
            .text(PortId::Text2, &MENU_BOX[..MENU_ENTRY_WIDTH]);
    }

    fn draw_compass(&mut self, dir: i16) {
        draw_compass(&mut self.screen, dir);
    }

    fn draw_item_list(&mut self, hero: &mut Hero) {
        draw_item_list(self, hero);
    }

    fn draw_overview_map(&mut self, map: &MapData, world: &mut World, mx: i32, my: i32) {
        self.screen.draw_target = self.screen.drawing_bitmap_id();
        draw_overview_map(self, map, world, mx, my);
    }

    fn mark_overview_map(&mut self, at: Option<(i32, i32)>) {
        self.screen.port_mut(PortId::Map).target = Some(self.screen.draw_target);
        self.screen.set_draw_mode(PortId::Map, JAM1);
        self.screen.set_a_pen(PortId::Map, MAP_MARK_PEN);

        if let Some((x, y)) = at {
            self.screen.move_to(PortId::Map, x, y);
            self.screen.text(PortId::Map, b"+");
        }
    }

    fn clear_drawing_page(&mut self) {
        self.screen.port_mut(PortId::Map).target = Some(self.screen.drawing_bitmap_id());
        self.screen.clear(PortId::Map, PEN_BLACK);
        self.screen.port_mut(PortId::Map).target = Some(self.screen.viewing_bitmap_id());
    }

    fn reset_page_scroll(&mut self) {
        let ras = self.screen.drawing_page().ras;
        self.screen.ras[ras].x = 0;
        self.screen.ras[ras].y = 0;
    }

    fn prepare_message_page(&mut self) {
        self.screen.current_port = PortId::Map;
        self.screen.port_mut(PortId::Map).target = Some(self.screen.drawing_bitmap_id());
        self.screen.set_draw_mode(PortId::Map, JAM1);
        self.screen.set_a_pen(PortId::Map, PLACARD_TEXT_PEN);
        self.screen.clear(PortId::Map, 0);
    }

    fn set_panel_visible(&mut self, visible: bool) {
        self.screen.vp_text.modes = if visible {
            HIRES | SPRITES
        } else {
            HIRES | SPRITES | VP_HIDE
        };
    }

    fn set_zoom_size(&mut self, x: i32, y: i32) {
        let left = (SCREEN_CENTER_X - x) as i16;
        let top = (SCREEN_CENTER_Y - y) as i16;

        for ras in &mut self.screen.ras[..2] {
            ras.x = left;
            ras.y = top;
        }

        self.screen.vp_page.x = left;
        self.screen.vp_page.width = (x + x) as i16;
        self.screen.vp_page.y = top;
        self.screen.vp_page.height = (y + y) as i16;

        self.screen.vp_text.x = 0;
        self.screen.vp_text.y = 0;
        self.screen.vp_text.height = (ZOOM_TEXT_HEIGHT - y) as i16;
    }

    fn load_picture_to_drawing_page(&mut self, name: &str) {
        let id = self.screen.drawing_bitmap_id();
        self.screen.draw_target = id;
        load_picture(name, self.screen.bitmap_mut(id), 0, 0);
    }

    fn load_panel_colors(&mut self, colors: &[u16]) {
        self.screen.load_colors(ViewPortId::Text, colors);
    }
}
