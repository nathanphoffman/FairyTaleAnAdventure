//! Printing messages in the text panel (from csrc/fmain2.c): scrolling a line in, continuing it,
//! word-wrapping story text and the "found ..." announcements.
//!
//! The text panel at the bottom of the screen shows the last few lines of messages. `print` scrolls
//! the panel up one line and writes a new line at the bottom; `print_cont` adds more text to the
//! line that is already there. `print_wrapped` is the general routine: it takes a message string from the
//! story tables, breaks it into lines that fit the panel and prints each one.

use crate::game::data::state::Game;
use crate::game::data::state::the_hero;
use crate::game::data::tables::brother_names;
use crate::game::ui::text::{draw_text, move_to};
use crate::gfx::bitmap::Rect;

// The scrolling message area of the text panel.
const TEXT_X_MIN: i32 = 16;
const TEXT_X_MAX: i32 = 400;
const TEXT_Y_MIN: i32 = 5;
const TEXT_Y_MAX: i32 = 44;

/// Pixels the panel scrolls up for each new line, and the baseline of that new line.
const LINE_HEIGHT: i32 = 10;
const NEW_LINE_Y: i32 = 42;

/// Characters on a line before `print_wrapped` looks for a place to wrap it.
const WRAP_COLUMN: i16 = 37;

/// Past this count `print_wrapped` knows the message is finished.
const END_COLUMN: i16 = 38;

/// Set when the message text ends, to leave the line loop.
const MESSAGE_ENDED: i16 = 1000;
const CARRIAGE_RETURN: u8 = 13;

/// In message text: replaced by the current brother's name.
const NAME_MARK: u8 = b'%';

/// Characters in the line buffer of `print_wrapped`.
const LINE_BUFFER: usize = 200;

/// The current brother's name.
fn brother_name() -> &'static str {
    brother_names[unsafe { the_hero.brother } as usize - 1]
}

/// Starts a new line in the panel and prints `s` on it.
/// The old lines scroll up by one text line (10 pixels) inside the message area.
pub fn print(game: &mut Game, s: &str) {
    print_bytes(game, s.as_bytes());
}

fn print_bytes(game: &mut Game, s: &[u8]) {
    let area = Rect {
        x: TEXT_X_MIN,
        y: TEXT_Y_MIN,
        width: TEXT_X_MAX - TEXT_X_MIN + 1,
        height: TEXT_Y_MAX - TEXT_Y_MIN + 1,
    };

    game.screen
        .scroll(game.screen.current_port, area, (0, LINE_HEIGHT));

    move_to(game, TEXT_X_MIN, NEW_LINE_Y);
    draw_text(game, s);
}

/// Continues the current line with `s`.
pub fn print_cont(game: &mut Game, s: &str) {
    draw_text(game, s.as_bytes());
}

/// The line buffer of `print_wrapped`: message characters are collected here, and `end` is where the
/// next one goes.
struct LineBuffer {
    bytes: [u8; LINE_BUFFER],
    end: usize,
}

impl LineBuffer {
    fn put(&mut self, c: u8) {
        if let Some(byte) = self.bytes.get_mut(self.end) {
            *byte = c;
        }

        self.end += 1;
    }

    /// Ends the text at the position `at` (without moving the end).
    fn terminate(&mut self, at: usize) {
        if let Some(byte) = self.bytes.get_mut(at) {
            *byte = 0;
        }
    }

    /// The line that starts at `start` and runs to the zero byte.
    fn line(&self, start: usize) -> &[u8] {
        let rest = self.bytes.get(start..).unwrap_or(&[]);
        let length = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());

        &rest[..length]
    }

    /// Adds the current brother's name. Returns its length.
    fn put_brother_name(&mut self) -> i16 {
        let name = brother_name();

        for c in name.bytes() {
            self.put(c);
        }

        name.len() as i16
    }
}

/// Prints message text, wrapping it into lines at spaces. A carriage return (13) starts a new
/// line and a `%` is replaced by the current brother's name.
///
/// How it works: characters are copied into a buffer, remembering where the last space was. When
/// a line reaches 37 characters it is cut at that space (the space becomes the end of the string)
/// and printed, and the leftover characters start the next line. The message ends at a zero byte
/// (or the end of `message`).
pub fn print_wrapped(game: &mut Game, message: impl AsRef<[u8]>) {
    let message = message.as_ref();
    let mut buffer = LineBuffer {
        bytes: [0; LINE_BUFFER],
        end: 0,
    };
    let mut chars = message.iter().copied().chain(std::iter::repeat(0));
    let mut line_start = 0;
    let mut i: i16 = 0;

    loop {
        let line_break = copy_up_to_a_line(&mut chars, &mut buffer, &mut i);

        if let Some(line_break) = line_break {
            // found a wrap point: end the line and print it
            buffer.terminate(line_break);
            print_bytes(game, buffer.line(line_start));

            if i > END_COLUMN {
                break;
            }

            i = (buffer.end as isize - (line_break + 1) as isize) as i16;
            line_start = line_break + 1;
        } else {
            // no wrap point, so print the whole thing
            buffer.put(0);

            if i > END_COLUMN {
                break;
            }

            print_bytes(game, buffer.line(line_start));
            i = 0;
            line_start = buffer.end;
        }
    }
}

/// Copies message characters into the buffer until a line is full or the text ends, counting them in
/// `i` and returning where the last space (or the end) was, or `None` if there was none.
fn copy_up_to_a_line(
    chars: &mut impl Iterator<Item = u8>,
    buffer: &mut LineBuffer,
    i: &mut i16,
) -> Option<usize> {
    let mut line_break = None; // end of the current line

    while *i < WRAP_COLUMN {
        let c = chars.next().unwrap_or(0);

        if c == b' ' {
            line_break = Some(buffer.end);
        }

        if c == 0 {
            line_break = Some(buffer.end);
            buffer.terminate(buffer.end);
            *i = MESSAGE_ENDED;
        }

        if c == CARRIAGE_RETURN {
            line_break = Some(buffer.end);
            buffer.terminate(buffer.end);
            break;
        }

        if c == NAME_MARK {
            *i += buffer.put_brother_name();
        } else {
            buffer.put(c);
        }

        *i += 1;
    }

    line_break
}

/// Prints "<Brother> found <container> containing ", ready for the contents to be added.
pub fn announce_container(game: &mut Game, s: &str) {
    print(game, brother_name());
    print_cont(game, " found ");
    print_cont(game, s);
    print_cont(game, " containing ");
}

/// Prints "<Brother> found <treasure>".
pub fn announce_treasure(game: &mut Game, s: &str) {
    print(game, brother_name());
    print_cont(game, " found ");
    print_cont(game, s);
}

/// Adds the current brother's name to the line being printed.
pub fn name(game: &mut Game) {
    print_cont(game, brother_name());
}
