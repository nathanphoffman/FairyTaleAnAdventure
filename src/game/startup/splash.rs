//! The splash screens at the very start: a note from the remaster's author, then who made the
//! original game, what the copyright situation is (in David Joiner's own words, from the README of
//! his source release) and under which license this remaster is shared.

use crate::game::data::state::{Game, Video};
use crate::game::input::handler::{left_button_down, next_key, reset_key_buffer};
use crate::game::output::Output;
use crate::game::render::display::delay;
use crate::game::startup::intro::prepare_blue_page;
use crate::game::startup::launch_menu::wrap;
use crate::game::startup::story_pages::ESCAPE_KEY;
use crate::gfx::overlay::{GLYPH_SIZE, Overlay, Text, WHITE, WIDTH, text_width};

/// One screen: a heading and paragraphs of text under it. A paragraph that starts with `#` is a
/// small heading for the paragraphs after it.
struct Page {
    heading: &'static str,
    paragraphs: &'static [&'static str],
}

/// The words: Nathan Hoffman's note, then the legal details. The quoted sentences are David
/// Joiner's (Talin's) own words from the README of his source release on GitHub; the rest is
/// short and plain.
const PAGES: [Page; 1] = [Page {
    heading: "A note from Nathan Hoffman",
    paragraphs: &[
        "When I was just a few years old I would gaze over my sister's shoulder watching her play Faery Tale Adventure in the early-90s on one of my dad's Commodore Amiga computers. My childhood memories of the Amiga and this game would drive me into a programming career into a time in the early 2020s wondering how I might convert the game's original source code (released by David Joiner the creator) into a remastered version. Today, with the advent of AI I am at a special place and time that has allowed me to achieve that dream.",
        "I hope you enjoy,",
        "-Nathan Hoffman",
        "#About the license",
        "Fairy Tale: An Adventure is an adaptation by Nathan Hoffman of The Faery Tale Adventure, written by David Joiner (Talin) and published by MicroIllusions in 1987. In 2022 he sent a \"termination of transfer\" notice to Hollyware, Inc., the successors to MicroIllusions' rights, to reclaim his copyright, and has had no response. In his words: \"I cannot say for certain exactly what the copyright status of this code is. However, whatever rights I do have, I hereby make freely available under an MIT-style permissive license.\"",
        "Nathan Hoffman grants the MIT License on this adaptation, along with whatever rights the license above allows. The original game's art, music and data come from David Joiner's GitHub release, under his license above.",
    ],
}];

/// Everything here is in overlay pixels (640 x 400, see `gfx::overlay`). The heading is drawn at
/// the size of the classic game text and the rest at half that.
const HEADING_SCALE: i32 = 2;
const BODY_SCALE: i32 = 1;

/// Where the text goes: the heading's baseline, the first body line, and the left margin. Lines
/// are this many characters wide, this far apart, with this much extra between paragraphs.
const HEADING_Y: i32 = 40;
const BODY_Y: i32 = 72;
const LINE_WIDTH: usize = 76;
const LINE_SPACING: i32 = 12;
const PARAGRAPH_GAP: i32 = 8;

/// The bottom line of the screen holds the prompt.
const PROMPT_X: i32 = 16;
const PROMPT_Y: i32 = 390;

/// Colours of the heading, the text and the prompt.
const HEADING_COLOR: u32 = 0xffff_d060;
const TEXT_COLOR: u32 = WHITE;
const PROMPT_COLOR: u32 = 0xffa0_a0c0;

/// A page that nobody reacts to moves on by itself after this many ticks (40 seconds).
const AUTO_ADVANCE_TICKS: i32 = 2000;

/// What the player did on a page.
enum Reaction {
    Next,
    Skip,
}

/// Shows the pages one after another. Space, Return or a click goes to the next one, Escape skips
/// the rest of the splash (the intro comes next). `hint` is the message for the bottom right.
pub(super) fn show_splash(game: &mut Game, video: &mut Video, hint: &str) {
    prepare_blue_page(game, video);
    reset_key_buffer();

    for (number, page) in PAGES.iter().enumerate() {
        video.set_overlay(page_overlay(page, number, hint));

        // Escape only leaves the splash; it takes a second press to skip the intro after it.
        if let Reaction::Skip = wait_for_player(game, video) {
            break;
        }
    }

    video.set_overlay(Overlay::default());
}

/// Waits until the player moves on, or the page has been up long enough.
fn wait_for_player(game: &mut Game, video: &mut Video) -> Reaction {
    for _ in 0..AUTO_ADVANCE_TICKS {
        match next_key() {
            ESCAPE_KEY => return Reaction::Skip,
            key if key == b' ' as i32 || key == RETURN_KEY => return Reaction::Next,
            _ => {}
        }

        if left_button_down() {
            // Wait for the button to go up, so one click does not turn two pages.
            while left_button_down() {
                delay(game, video, 1);
            }

            return Reaction::Next;
        }

        delay(game, video, 1);
    }

    Reaction::Next
}

/// The Return key's code.
const RETURN_KEY: i32 = 13;

/// What the bottom line says: how to go on.
fn prompt(number: usize) -> String {
    if number + 1 == PAGES.len() {
        "Space or click: continue".to_string()
    } else {
        format!("Space: next page {}/{}", number + 1, PAGES.len())
    }
}

/// A page as an overlay: the heading, the text, the prompt and the skip message.
fn page_overlay(page: &Page, number: usize, hint: &str) -> Overlay {
    let mut overlay = Overlay::corner_hint(hint);

    overlay.texts.push(Text {
        text: page.heading.to_string(),
        x: (WIDTH - text_width(page.heading, HEADING_SCALE)) / 2,
        y: HEADING_Y,
        color: HEADING_COLOR,
        scale: HEADING_SCALE,
    });

    overlay.texts.extend(body_lines(page));
    overlay.texts.push(Text {
        text: prompt(number),
        x: PROMPT_X,
        y: PROMPT_Y,
        color: PROMPT_COLOR,
        scale: BODY_SCALE,
    });

    overlay
}

/// The wrapped lines of a page's paragraphs, with their positions: a block of text in the middle
/// of the screen, its lines starting at the same left edge.
fn body_lines(page: &Page) -> Vec<Text> {
    let left = (WIDTH - LINE_WIDTH as i32 * GLYPH_SIZE * BODY_SCALE) / 2;
    let mut lines = Vec::new();
    let mut y = BODY_Y;

    for paragraph in page.paragraphs {
        let (color, text) = match paragraph.strip_prefix('#') {
            Some(small_heading) => (HEADING_COLOR, small_heading),
            None => (TEXT_COLOR, *paragraph),
        };

        for line in wrap(text, LINE_WIDTH) {
            lines.push(Text {
                text: line.to_string(),
                x: left,
                y,
                color,
                scale: BODY_SCALE,
            });

            y += LINE_SPACING;
        }

        y += PARAGRAPH_GAP;
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Text must end above the prompt line.
    const LAST_TEXT_Y: i32 = 370;

    #[test]
    fn every_page_fits_above_the_prompt_line() {
        for page in &PAGES {
            let last = body_lines(page).last().map_or(0, |line| line.y);

            assert!(last <= LAST_TEXT_Y, "{} ends at line {last}", page.heading);
        }
    }

    #[test]
    fn no_line_is_wider_than_the_screen() {
        for page in &PAGES {
            for line in body_lines(page) {
                assert!(
                    line.x >= 0 && line.x + line.width() <= WIDTH,
                    "{:?}",
                    line.text
                );
            }
        }
    }

    #[test]
    fn the_legal_words_name_the_people_and_the_license() {
        let all: String = PAGES
            .iter()
            .flat_map(|page| page.paragraphs)
            .copied()
            .collect();

        for needle in [
            "David Joiner",
            "Hollyware",
            "MicroIllusions",
            "MIT-style",
            "2022",
        ] {
            assert!(all.contains(needle), "missing {needle}");
        }
    }

    #[test]
    fn the_prompt_and_the_skip_message_do_not_overlap() {
        let overlay = page_overlay(&PAGES[0], 0, "Press ESC to skip");
        let prompt = overlay.texts.last().map_or(0, |t| t.x + t.width());
        let hint = overlay.texts.first().map_or(0, |t| t.x);

        assert!(
            prompt < hint,
            "prompt ends at {prompt}, hint starts at {hint}"
        );
    }
}
