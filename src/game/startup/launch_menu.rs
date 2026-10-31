//! The launch menu, shown after the intro: the player picks which version of the game to play.
//! For now all three start the same game; the choice is kept in `Game::experience`. A Quit entry in
//! the bottom right corner leaves the game.

use crate::game::data::state::{Experience, Game, Video};
use crate::game::input::handler::{
    forget_last_menu, left_button_down, next_key, raw_pointer, reset_key_buffer,
};
use crate::game::output::Output;
use crate::game::render::display::delay;
use crate::game::startup::intro::prepare_blue_page;
use crate::game::startup::menu_background::Background;
use crate::gfx::overlay::{GLYPH_SIZE, Image, Overlay, Text, WHITE, WIDTH, text_width};
use crate::gfx::wood::{SHADOW, plank};
use crate::platform::display::show_system_cursor;
use std::sync::Arc;

/// The things on offer, top to bottom: the label, a description under it, and what it starts.
const OPTIONS: [(&str, &str, Experience); 3] = [
    (
        "Original Experience",
        "The way David Joiner made the game (converted for modern PCs).",
        Experience::Original,
    ),
    (
        "Remastered Experience",
        "The original experience with modern conveniences (maps, menus, visual upscales, etc.).",
        Experience::Remastered,
    ),
    (
        "Reimagined Experience",
        "The remastered experience with additional content by Nathan Hoffman to extend and \
         reimagine the game with more TTRPG flair, story, spells, monsters, and places.",
        Experience::Reimagined,
    ),
];

const HEADING: &str = "Choose your adventure";

/// Everything here is in overlay pixels (640 x 400, see `gfx::overlay`). Titles are drawn at the
/// size of the classic game text, descriptions at half that.
const TITLE_SCALE: i32 = 2;
const BODY_SCALE: i32 = 1;

/// The Quit entry sits in the bottom right corner. As a hover or a choice it is numbered one past
/// the real options.
const QUIT_LABEL: &str = "Quit";
const QUIT: usize = OPTIONS.len();
const QUIT_Y: i32 = 380;
const QUIT_RIGHT: i32 = WIDTH - 16;

/// Descriptions are wrapped to this many characters, and sit this far apart.
const DESCRIPTION_WIDTH: usize = 64;
const LINE_SPACING: i32 = 13;

/// Baseline of the heading and of the first option's title.
const HEADING_Y: i32 = 56;
const FIRST_OPTION_Y: i32 = 124;

/// A title is this far above its baseline, and its first description line this far below it.
const TITLE_ASCENT: i32 = 14 * TITLE_SCALE / 2;
const DESCRIPTION_OFFSET: i32 = 21;

/// Space between the text and the edge of its dark panel, and between two panels.
const PANEL_PADDING: i32 = 8;
const PANEL_GAP: i32 = 12;

/// An entry can be clicked from side margin to side margin.
const SIDE_MARGIN: i32 = 16;

/// Colours of the text on the wood: cream, gold for the heading, yellow under the pointer, with a
/// dark shadow behind each letter so it can be read on the grain.
const TITLE_COLOR: u32 = 0xffff_f4dc;
const HEADING_COLOR: u32 = 0xffff_e08a;
const HOVER_COLOR: u32 = 0xffff_e566;
const DESCRIPTION_COLOR: u32 = 0xffee_dcb8;
const LIT_DESCRIPTION_COLOR: u32 = WHITE;
const TEXT_SHADOW_COLOR: u32 = 0xff1e_1006;

/// Where one option sits: the baseline of its title and the lines of its description.
struct Block {
    title_y: i32,
    lines: Vec<&'static str>,
}

impl Block {
    /// The baseline of the last line of the block.
    fn last_baseline(&self) -> i32 {
        self.title_y + DESCRIPTION_OFFSET + (self.lines.len() as i32 - 1) * LINE_SPACING
    }

    /// The top and bottom of the dark panel behind the block.
    fn panel_span(&self) -> (i32, i32) {
        (
            self.title_y - TITLE_ASCENT - PANEL_PADDING,
            self.last_baseline() + PANEL_PADDING,
        )
    }
}

/// Shows the menu and waits for a click on an option (or its number key); returns the choice, or
/// `None` when the player chose Quit.
pub(super) fn choose_experience(game: &mut Game, video: &mut Video) -> Option<Experience> {
    let blocks = layout();

    prepare_blue_page(game, video);

    let background = Background::load(game, video);
    let mut tick = 0;

    show_system_cursor(true);
    let planks = Planks::new(&blocks);

    video.set_overlay(menu_overlay(&blocks, &planks, None));

    let mut drawn = None;

    let choice = loop {
        let pointer = raw_pointer();
        // The pointer is in lores game pixels; the menu is in overlay pixels, twice as many.
        let hovered = option_at(&blocks, pointer.0 * 2, pointer.1 * 2);

        if hovered != drawn {
            video.set_overlay(menu_overlay(&blocks, &planks, hovered));
            drawn = hovered;
        }

        background.draw(video, tick);
        tick += 1;

        if let Some(option) = hovered.filter(|_| left_button_down()) {
            break option;
        }

        if let Some(option) = option_for_key(next_key()) {
            break option;
        }

        delay(game, video, 1);
    };

    // Wait for the button to go up, so the click does not carry on into the game.
    while left_button_down() {
        delay(game, video, 1);
    }

    show_system_cursor(false);
    video.set_overlay(Overlay::default());
    reset_key_buffer();
    forget_last_menu();

    OPTIONS.get(choice).map(|option| option.2)
}

/// Works out where every option goes, one block under the other.
fn layout() -> Vec<Block> {
    let mut next_y = FIRST_OPTION_Y;

    OPTIONS
        .iter()
        .map(|(_, description, _)| {
            let block = Block {
                title_y: next_y,
                lines: wrap(description, DESCRIPTION_WIDTH),
            };

            next_y = block.panel_span().1 + PANEL_GAP + TITLE_ASCENT + PANEL_PADDING;

            block
        })
        .collect()
}

/// Breaks `text` into lines of at most `width` characters, between words.
pub(super) fn wrap(text: &'static str, width: usize) -> Vec<&'static str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut last_space = None;

    for (index, ch) in text.char_indices() {
        if ch == ' ' {
            last_space = Some(index);
        }

        if index + 1 - start > width
            && let Some(space) = last_space.take()
        {
            lines.push(&text[start..space]);
            start = space + 1;
        }
    }

    lines.push(&text[start..]);

    lines
}

/// The wooden boards behind the heading, each option and Quit, made once. Each entry has a plain
/// board and a lit one (for when the pointer is on it).
struct Planks {
    heading: Image,
    options: Vec<[Image; 2]>,
    quit: [Image; 2],
}

impl Planks {
    /// Makes a board for each area of the menu.
    fn new(blocks: &[Block]) -> Planks {
        let options = blocks
            .iter()
            .zip(&OPTIONS)
            .enumerate()
            .map(|(index, (block, option))| board(block_area(block, option.0), 21 + index as u32))
            .collect();

        Planks {
            heading: board(heading_area(), 11)[0].clone(),
            options,
            quit: board(quit_area(), 31),
        }
    }
}

/// A plain and a lit board for an area (left, top, width, height).
fn board(area: (i32, i32, i32, i32), seed: u32) -> [Image; 2] {
    let make = |lit| Image {
        x: area.0,
        y: area.1,
        width: area.2 + SHADOW as i32,
        height: area.3 + SHADOW as i32,
        pixels: Arc::from(plank(area.2 as usize, area.3 as usize, seed, lit)),
    };

    [make(false), make(true)]
}

/// The heading, the options and Quit as text on wooden boards, the hovered one lit.
fn menu_overlay(blocks: &[Block], planks: &Planks, hovered: Option<usize>) -> Overlay {
    let mut overlay = Overlay::default();

    overlay.images.push(planks.heading.clone());
    add_centered(&mut overlay, HEADING, HEADING_Y, HEADING_COLOR, TITLE_SCALE);

    for (index, (block, option)) in blocks.iter().zip(&OPTIONS).enumerate() {
        let lit = hovered == Some(index);
        let title_color = if lit { HOVER_COLOR } else { TITLE_COLOR };
        let line_color = if lit {
            LIT_DESCRIPTION_COLOR
        } else {
            DESCRIPTION_COLOR
        };

        overlay
            .images
            .push(planks.options[index][lit as usize].clone());
        add_centered(
            &mut overlay,
            option.0,
            block.title_y,
            title_color,
            TITLE_SCALE,
        );

        for (number, line) in block.lines.iter().enumerate() {
            let y = block.title_y + DESCRIPTION_OFFSET + number as i32 * LINE_SPACING;

            add_centered(&mut overlay, line, y, line_color, BODY_SCALE);
        }
    }

    let lit = hovered == Some(QUIT);
    let color = if lit { HOVER_COLOR } else { TITLE_COLOR };

    overlay.images.push(planks.quit[lit as usize].clone());
    add_text(
        &mut overlay,
        QUIT_LABEL,
        (quit_left(), QUIT_Y),
        color,
        TITLE_SCALE,
    );

    overlay
}

/// Adds a line of text in the middle of the screen with its baseline at `y`.
fn add_centered(overlay: &mut Overlay, text: &str, y: i32, color: u32, scale: i32) {
    let x = (WIDTH - text_width(text, scale)) / 2;

    add_text(overlay, text, (x, y), color, scale);
}

/// Adds a line of text with a dark shadow behind it, down and to the right.
fn add_text(overlay: &mut Overlay, text: &str, (x, y): (i32, i32), color: u32, scale: i32) {
    for (offset, color) in [(scale, TEXT_SHADOW_COLOR), (0, color)] {
        overlay.texts.push(Text {
            text: text.to_string(),
            x: x + offset,
            y: y + offset,
            color,
            scale,
        });
    }
}

/// The area behind the heading.
fn heading_area() -> (i32, i32, i32, i32) {
    let width = text_width(HEADING, TITLE_SCALE) + PANEL_PADDING * 2;

    (
        (WIDTH - width) / 2,
        HEADING_Y - TITLE_ASCENT - PANEL_PADDING,
        width,
        TITLE_ASCENT + PANEL_PADDING * 2,
    )
}

/// The area behind an option: as wide as its longest line plus some padding.
fn block_area(block: &Block, title: &str) -> (i32, i32, i32, i32) {
    let longest = block
        .lines
        .iter()
        .map(|line| text_width(line, BODY_SCALE))
        .max()
        .unwrap_or(0);

    let width = longest.max(text_width(title, TITLE_SCALE)) + PANEL_PADDING * 2;
    let (top, bottom) = block.panel_span();

    ((WIDTH - width) / 2, top, width, bottom - top)
}

/// The area behind the Quit entry.
fn quit_area() -> (i32, i32, i32, i32) {
    let left = quit_left() - PANEL_PADDING;

    (
        left,
        QUIT_Y - TITLE_ASCENT - PANEL_PADDING,
        WIDTH - left,
        TITLE_ASCENT + PANEL_PADDING * 2 + GLYPH_SIZE,
    )
}

/// The x position of the Quit entry: its text ends at the right margin.
fn quit_left() -> i32 {
    QUIT_RIGHT - text_width(QUIT_LABEL, TITLE_SCALE)
}

/// The option whose block the point (overlay pixels) is in, if any; `QUIT` for the Quit entry.
/// A block is hit anywhere from side margin to side margin.
fn option_at(blocks: &[Block], x: i32, y: i32) -> Option<usize> {
    let quit = quit_area();
    let on_quit = x >= quit.0 && y >= quit.1 && y < quit.1 + quit.3;

    if on_quit {
        return Some(QUIT);
    }

    blocks.iter().position(|block| {
        let (top, bottom) = block.panel_span();

        (SIDE_MARGIN..WIDTH - SIDE_MARGIN).contains(&x) && (top..bottom).contains(&y)
    })
}

/// The option a key chooses: 1, 2 or 3, or Q to quit.
fn option_for_key(key: i32) -> Option<usize> {
    if key == b'Q' as i32 {
        return Some(QUIT);
    }

    let number = (key - b'1' as i32) as usize;

    (key >= b'1' as i32 && number < OPTIONS.len()).then_some(number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_block_is_found_by_its_title_and_by_its_last_line() {
        let blocks = layout();

        for (index, block) in blocks.iter().enumerate() {
            let middle = WIDTH / 2;

            assert_eq!(option_at(&blocks, middle, block.title_y - 4), Some(index));
            assert_eq!(
                option_at(&blocks, middle, block.last_baseline()),
                Some(index)
            );
        }
    }

    #[test]
    fn the_heading_the_gaps_and_the_sides_pick_nothing() {
        let blocks = layout();
        let in_the_gap = blocks[0].panel_span().1 + 1;

        assert_eq!(option_at(&blocks, WIDTH / 2, HEADING_Y), None);
        assert_eq!(option_at(&blocks, WIDTH / 2, in_the_gap), None);
        assert_eq!(option_at(&blocks, 0, blocks[0].title_y), None);
    }

    #[test]
    fn everything_fits_above_the_quit_entry() {
        let blocks = layout();
        let bottom = blocks.last().map_or(0, |block| block.panel_span().1);

        assert!(bottom < quit_area().1, "the menu ends at line {bottom}");
        assert!(quit_area().1 + quit_area().3 <= 400);
    }

    #[test]
    fn the_quit_entry_is_found_in_the_bottom_right_corner() {
        let blocks = layout();

        assert_eq!(option_at(&blocks, WIDTH - 24, QUIT_Y - 4), Some(QUIT));
        assert_eq!(option_at(&blocks, 40, QUIT_Y - 4), None);
        assert_eq!(option_for_key(b'Q' as i32), Some(QUIT));
    }

    #[test]
    fn number_keys_pick_options() {
        assert_eq!(option_for_key(b'1' as i32), Some(0));
        assert_eq!(option_for_key(b'3' as i32), Some(2));
        assert_eq!(option_for_key(b'4' as i32), None);
        assert_eq!(option_for_key(0), None);
    }

    #[test]
    fn long_text_wraps_between_words() {
        let lines = wrap("one two three four", 9);

        assert_eq!(lines, ["one two", "three", "four"]);
        assert!(
            wrap(OPTIONS[2].1, DESCRIPTION_WIDTH)
                .iter()
                .all(|l| l.len() <= DESCRIPTION_WIDTH)
        );
    }
}
