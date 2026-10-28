//! What the rules ask of the screen. The rules say what should be shown (a line of text, new page
//! colours, the next frame) and an `Output` decides how: `Video` draws it into the Amiga-style
//! bitmaps, a test can just record it, and a new renderer could draw it in a different way.

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
}
