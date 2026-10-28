//! `Video` as an `Output`: the Amiga-style drawing of what the rules ask for.

use crate::game::data::state::Video;
use crate::game::files::disk::load_track_range;
use crate::game::files::loading::{finish_sprite_load, load_sprite_file};
use crate::game::output::Output;
use crate::game::render::screen::ViewPortId;
use crate::game::ui::text::{draw_text, format_number, move_to};
use crate::gfx::bitmap::Rect;
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
}
