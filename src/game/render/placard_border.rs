//! The decorative border drawn around story pages.

use crate::game::render::screen::{PortId, Screen};

// The decorative border drawn on story pages.
// The border is a closed path made of 16 small steps repeated with changing offsets.
const BORDER_X_STEPS: [i32; 16] = [-4, -4, -4, 0, 0, 0, 4, 4, 0, -4, 0, 4, 4, 0, 0, 0];
const BORDER_Y_STEPS: [i32; 16] = [0, 0, 0, 4, 4, 4, 0, 0, -4, 0, -4, 0, 0, 4, 4, 4];
const BORDER_START_X: i32 = 12;
const BORDER_PASSES: i32 = 17;
const BORDER_STEPS: usize = 16;

/// The page the border goes around is 284 x 124 (the long edges) and 268 x 112 (the short ones,
/// drawn turned a quarter), with the short edges offset by 16 and 12.
const LONG_EDGE_FAR: (i32, i32) = (284, 124);
const SHORT_EDGE_NEAR: (i32, i32) = (16, 12);
const SHORT_EDGE_FAR: (i32, i32) = (268, 112);

/// Passes numbered above this also draw the long (top and bottom) edges.
const LONG_EDGE_ABOVE_PASS: i32 = 9;
const BORDER_PEN: u8 = 1;
const BORDER_HIGHLIGHT_PEN: u8 = 24;

/// Strokes per step; the last one is drawn in the highlight pen.
const STROKES_PER_STEP: i32 = 5;

/// Draws a line from (x0, y0) to (x1, y1).
fn line(screen: &mut Screen, x0: i32, y0: i32, x1: i32, y1: i32) {
    screen.move_to(PortId::Map, x0, y0);
    screen.line_to(PortId::Map, x1, y1);
}

/// Draws the decorative border around a story page: many short strokes, five each with the last in
/// a highlight colour, winding around the four edges. The first passes also draw the top and bottom.
pub fn draw_placard_border(screen: &mut Screen) {
    let (mut xorg, mut yorg) = (BORDER_START_X, 0);

    for pass in (0..BORDER_PASSES).rev() {
        for step in 0..BORDER_STEPS {
            let dx = xorg + BORDER_X_STEPS[step];
            let dy = yorg + BORDER_Y_STEPS[step];

            for stroke in (0..STROKES_PER_STEP).rev() {
                let pen = if stroke == 0 {
                    BORDER_HIGHLIGHT_PEN
                } else {
                    BORDER_PEN
                };

                screen.set_a_pen(PortId::Map, pen);
                draw_border_stroke(screen, pass > LONG_EDGE_ABOVE_PASS, (xorg, yorg), (dx, dy));
            }

            xorg = dx;
            yorg = dy;
        }
    }
}

/// One stroke of the border from `from` to `to`, repeated on the opposite edge, and on the short
/// edges (turned a quarter) with the x and y swapped.
fn draw_border_stroke(
    screen: &mut Screen,
    long_edges: bool,
    (xorg, yorg): (i32, i32),
    (dx, dy): (i32, i32),
) {
    if long_edges {
        line(screen, xorg, yorg, dx, dy);
        line(
            screen,
            LONG_EDGE_FAR.0 - xorg,
            LONG_EDGE_FAR.1 - yorg,
            LONG_EDGE_FAR.0 - dx,
            LONG_EDGE_FAR.1 - dy,
        );
    }

    line(
        screen,
        SHORT_EDGE_NEAR.0 + yorg,
        SHORT_EDGE_NEAR.1 - xorg,
        SHORT_EDGE_NEAR.0 + dy,
        SHORT_EDGE_NEAR.1 - dx,
    );

    line(
        screen,
        SHORT_EDGE_FAR.0 - yorg,
        SHORT_EDGE_FAR.1 + xorg,
        SHORT_EDGE_FAR.0 - dy,
        SHORT_EDGE_FAR.1 + dx,
    );
}
