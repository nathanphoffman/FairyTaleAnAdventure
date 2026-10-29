//! What the rules ask of the screen. The rules say what should be shown (a line of text, new page
//! colours, the next frame) and an `Output` decides how: `Video` draws it into the Amiga-style
//! bitmaps, a test can just record it, and a new renderer could draw it in a different way.

use crate::game::data::state::{Hero, MapData, World};
use crate::game::render::screen::{FontKind, PortId};

/// The things the rules can ask for.
pub trait Output {
    /// Starts a new line in the text panel with these characters (the old lines scroll up).
    fn print_bytes(&mut self, text: &[u8]);

    /// Adds these characters to the last line of the text panel.
    fn print_cont_bytes(&mut self, text: &[u8]);

    /// Adds a number to the last line, right aligned in `len` characters, plus a trailing space.
    fn print_number(&mut self, value: i32, len: i32);

    /// Starts a new line in the text panel.
    fn print(&mut self, text: &str) {
        self.print_bytes(text.as_bytes());
    }

    /// Adds to the last line of the text panel.
    fn print_cont(&mut self, text: &str) {
        self.print_cont_bytes(text.as_bytes());
    }

    /// Makes these (12 bit, 0x0RGB) colours the page palette.
    fn load_page_colors(&mut self, colors: &[u16]);

    /// Swaps the page being drawn with the one being shown.
    fn swap_pages(&mut self);

    /// Shows the page that is on screen now.
    fn present(&mut self);

    /// Waits one tick of 1/50 second, showing the screen as it is now.
    fn wait_frame(&mut self);

    /// Ticks waited since the game started.
    fn ticks(&self) -> u32;

    /// Forgets all sprite sheets: the next one loads at the start of sprite memory.
    fn forget_sprites(&mut self);

    /// Makes the next sprite load replace sheet `slot` and everything loaded after it.
    fn rewind_sprites_to(&mut self, slot: u8);

    /// Sheet `slot` starts where the next sprite load goes.
    fn start_sprite_sheet(&mut self, slot: u8);

    /// Starts loading sprite file number `file` into the next free sprite memory.
    fn load_sprite_file(&mut self, file: i32);

    /// Waits for the sprite load to finish and builds the masks of sheet `slot`.
    fn finish_sprite_load(&mut self, slot: i16);

    /// Loads image bank `bank` (a quarter of the tile images, 5 planes) from disk block `first_block`.
    fn load_image_bank(&mut self, bank: usize, first_block: i16);

    // ---- drawing on the play field (`PortId::Map`), the message area (`Text`) and the menu area
    // (`Text2`). Each has its own font, pens and position. ----

    /// The port that print and print_cont draw on.
    fn current_port(&self) -> PortId;

    /// Makes `port` the one print and print_cont draw on.
    fn select_port(&mut self, port: PortId);

    /// Sets the font of `port`.
    fn set_font_kind(&mut self, port: PortId, kind: FontKind);

    /// Sets the colour text and lines are drawn in.
    fn set_a_pen(&mut self, port: PortId, pen: u8);

    /// Sets the colour behind text.
    fn set_b_pen(&mut self, port: PortId, pen: u8);

    /// Sets how drawing combines with what is there (JAM1, JAM2, ...).
    fn set_draw_mode(&mut self, port: PortId, mode: u8);

    /// Moves the drawing position of `port`.
    fn move_to(&mut self, port: PortId, x: i32, y: i32);

    /// Draws text at the position of `port`.
    fn text(&mut self, port: PortId, bytes: &[u8]);

    /// Draws a line from the position of `port` to (x, y).
    fn line_to(&mut self, port: PortId, x: i32, y: i32);

    /// Fills a rectangle with the pen colour.
    fn rect_fill(&mut self, port: PortId, from: (i32, i32), to: (i32, i32));

    // ---- whole screens and pictures ----

    /// Lights direction `dir` on the compass (9 and above light none).
    fn draw_compass(&mut self, dir: i16);

    /// Draws the pictures of everything the hero holds on a black page.
    fn draw_item_list(&mut self, hero: &mut Hero);

    /// Draws the overview map of the whole world around the map position (`mx`, `my`) on the page
    /// being drawn, and says in `world` which sector is at its top left.
    fn draw_overview_map(&mut self, map: &MapData, world: &mut World, mx: i32, my: i32);

    /// Marks the hero's place on the overview map, if it is inside the map.
    fn mark_overview_map(&mut self, at: Option<(i32, i32)>);

    /// Clears the page being drawn to colour `pen`.
    fn clear_drawing_page(&mut self, pen: u8);

    /// Shows the page being drawn from its top left corner (no scrolling).
    fn reset_page_scroll(&mut self);

    /// Gets the play field ready for a full page message: draws on the page being drawn, in the
    /// message pen, and clears it.
    fn prepare_message_page(&mut self, pen: u8);

    /// Shows or hides the text panel.
    fn set_panel_visible(&mut self, visible: bool);

    /// Makes the play field `2 * x` wide and `2 * y` high, centred on the screen, and the text panel
    /// the height that is left over (the intro zoom).
    fn set_zoom_size(&mut self, x: i32, y: i32);

    /// Loads the picture file `name` onto the page being drawn.
    fn load_picture_to_drawing_page(&mut self, name: &str);

    /// Makes these (12 bit, 0x0RGB) colours the text panel palette.
    fn load_panel_colors(&mut self, colors: &[u16]);
}
