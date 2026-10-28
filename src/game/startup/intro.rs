//! The title screen, the intro and getting the first brother started (from csrc/fmain.c), up to the
//! first frame of the game.

use crate::game::actors::revive::revive;
use crate::game::data::state::Game;
use crate::game::data::state::Video;
use crate::game::data::story_text::TITLE_TEXT;
use crate::game::data::tables::{black_colors, text_colors};
use crate::game::files::disk::load_track_range;
use crate::game::files::iff_picture::load_picture;
use crate::game::files::loading::{
    copy_protection_check, copy_protection_riddle, intro_skipped, read_sample, read_score,
    show_story_page, track,
};
use crate::game::input::handler::reset_key_buffer;
use crate::game::render::display::{delay, flip_pages};
use crate::game::render::screen::Screen;
use crate::game::render::screen::{BitmapId, FontKind, PortId, ViewPortId};
use crate::game::render::screen_fx::{set_intro_zoom, show_still_screen};
use crate::game::ui::menus::print_options;
use crate::game::ui::text::{placard_text, print_positioned};
use crate::gfx::bitmap::Rect;
use crate::platform::audio::{playscore, stopscore};

// Where the view ports sit once the game starts.
const PAGE_HEIGHT: i16 = 143;
const TEXT_HEIGHT: i16 = 57;
const LOWRES_WIDTH: i16 = 288;
const PAGE_VIEW_HEIGHT: i16 = 140;
const PAGE_VIEW_X: i16 = 16;

// Size of a picture page.
const PAGE_SIZE: (i32, i32) = (320, 200);
const ALL_FIVE_PLANES: u32 = 0x1f;

// Screen modes (`view_status`).
const VIEW_REDRAW: i8 = 99;

/// Intro: how big the picture zooms (half the width in pixels), and the step.
const ZOOM_FULL: i32 = 160;
const ZOOM_NORMAL: i32 = 156;
const ZOOM_STEP: usize = 4;

/// Pens: the title text, and the colour of the flashing pointer.
const TITLE_PEN: u8 = 1;

// The title screen's palette: a blue page background and white text; the play screen's background
// is darker while the panel loads.
const BACKGROUND_COLOR: usize = 0;
const TEXT_COLOR: usize = 1;
const TITLE_BLUE: (u32, u32, u32) = (0, 0, 6);
const WHITE_RGB: (u32, u32, u32) = (15, 15, 15);
const LOADING_BLUE: (u32, u32, u32) = (0, 0, 3);

/// Pens of the story text.
const STORY_PEN: u8 = 10;
const STORY_BACKGROUND_PEN: u8 = 11;

/// The tracks of the intro tune.
const INTRO_TRACKS: [usize; 4] = [12, 13, 14, 15];

/// The story pages: file names and the delays in ticks before and after each is shown.
const STORY_PAGES: [(&str, &str, i16, i16); 3] = [
    ("p1a", "p1b", 21, 29),
    ("p2a", "p2b", 20, 29),
    ("p3a", "p3b", 20, 33),
];

/// Waits, in fiftieths of a second.
const TITLE_WAIT: i32 = 50;
const INTRO_PAGES_WAIT: i32 = 190;
const PLACARD_WAIT: i32 = 20;

/// The placard shown while the copy protection question is asked.
const PLACARD_COPYRIGHT: i32 = 19;

/// Disk track range holding the shadow masks.
const SHADOW_BLOCK: (i16, i16) = (896, 24);

/// Returns false if the player failed the copy protection.
pub(super) fn start_up(game: &mut Game, video: &mut Video) -> bool {
    stopscore();
    show_title_screen(game, video);
    prepare_intro(game, video);

    if let Some(dir) = std::env::var_os("FTA_MUSIC_EXPORT") {
        crate::game::ui::music::export_tunes(std::path::Path::new(&dir));

        return false;
    }

    if intro_skipped(&mut game.ui) == 0 {
        play_intro(game, video);
    }

    copy_protection_check();
    show_play_screen_background(game, video);

    if copy_protection_riddle() == 0 {
        return false;
    }

    delay(game, video, PLACARD_WAIT);
    start_first_brother(game, video);

    true
}

/// Blanks both pages.
fn clear_both_pages(screen: &mut Screen) {
    for page in [screen.viewing_bitmap_id(), screen.drawing_bitmap_id()] {
        screen.port_mut(PortId::Map).target = Some(page);
        screen.clear(PortId::Map, 0);
    }
}

/// Sets a colour of the page palette from 4-bit red, green and blue.
fn set_page_color(screen: &mut Screen, color: usize, rgb: (u32, u32, u32)) {
    screen.set_color(ViewPortId::Page, color, rgb);
}

/// The legal title text on a blue page, then the music and the sound effects are read in.
fn show_title_screen(game: &mut Game, video: &mut Video) {
    video.screen.vp_page.ras = Some(0);
    video.screen.pages[1].ras = 1;
    clear_both_pages(&mut video.screen);
    video.screen.current_port = PortId::Map;
    set_intro_zoom(game, video, ZOOM_NORMAL);
    set_page_color(&mut video.screen, BACKGROUND_COLOR, TITLE_BLUE);
    set_page_color(&mut video.screen, TEXT_COLOR, WHITE_RGB);

    let pen = video.screen.port(PortId::Map).pen.fg;
    video.screen.set_a_pen(PortId::Map, TITLE_PEN);
    print_positioned(video, TITLE_TEXT);
    video.screen.set_a_pen(PortId::Map, pen);

    delay(game, video, TITLE_WAIT);

    video.screen.current_port = PortId::Text;
    video.screen.port_mut(PortId::Text).target = Some(BitmapId::TextPlane0);
    video.screen.set_font_kind(PortId::Text, FontKind::Story);
    video.screen.set_a_pen(PortId::Text, STORY_PEN);
    video.screen.set_b_pen(PortId::Text, STORY_BACKGROUND_PEN);

    read_score();
    read_sample();

    delay(game, video, TITLE_WAIT);
}

/// Starts the intro tune, and flips the pages twice so both show the same thing.
fn prepare_intro(game: &mut Game, video: &mut Video) {
    video.screen.vp_page.ras = Some(0);

    playscore(INTRO_TRACKS.map(track));

    video.screen.load_colors(ViewPortId::Text, &black_colors);
    video.screen.pages[1].ras = 1;
    set_intro_zoom(game, video, 0);

    flip_pages(game, video); // "prime the pump"
    flip_pages(game, video);
}

/// Switches to the normal playing screen: loads the shadow masks and the panel picture, and shows
/// the copyright placard.
fn show_play_screen_background(game: &mut Game, video: &mut Video) {
    video.screen.pages[1].ras = 1;
    clear_both_pages(&mut video.screen);

    video.screen.load_colors(ViewPortId::Text, &black_colors);

    set_intro_zoom(game, video, ZOOM_NORMAL);
    set_page_color(&mut video.screen, BACKGROUND_COLOR, LOADING_BLUE);

    load_track_range(
        SHADOW_BLOCK.0,
        SHADOW_BLOCK.1,
        &mut video.graphics.shadow,
        0,
    );

    set_page_color(&mut video.screen, BACKGROUND_COLOR, TITLE_BLUE);
    load_picture("hiscreen", video.screen.bitmap_mut(BitmapId::Text), 0, 0);

    set_page_color(&mut video.screen, TEXT_COLOR, WHITE_RGB);

    video.screen.current_port = PortId::Map;
    video.screen.port_mut(PortId::Map).target = Some(video.screen.drawing_bitmap_id());
    show_still_screen(game, video);
    video.screen.set_a_pen(PortId::Map, TITLE_PEN);
    placard_text(video, PLACARD_COPYRIGHT);

    reset_key_buffer();
}

/// Starts the first brother and sets up the text page and the colours for play.
fn start_first_brother(game: &mut Game, video: &mut Video) {
    for ras in &mut video.screen.ras[..2] {
        ras.x = 0;
        ras.y = 0;
    }

    stopscore();
    revive(game, video, 1);

    video.screen.port_mut(PortId::Map).target = Some(video.screen.drawing_bitmap_id());

    video.screen.vp_text.y = PAGE_HEIGHT;
    video.screen.vp_text.height = TEXT_HEIGHT;
    video.screen.vp_page.height = PAGE_VIEW_HEIGHT;
    video.screen.vp_page.x = PAGE_VIEW_X;
    video.screen.vp_page.width = LOWRES_WIDTH;
    video.screen.vp_page.y = 0;
    video.screen.load_colors(ViewPortId::Text, &text_colors);

    for page in &mut video.screen.pages {
        page.obcount = 0;
        page.wflag = 0;
    }

    game.ui.view_status = VIEW_REDRAW;

    game.ui.current_menu = 0;
    print_options(game, video);
}

/// The story pictures with the zoom in and out (skipped if the player presses Space).
fn play_intro(game: &mut Game, video: &mut Video) {
    load_picture("page0", video.screen.bitmap_mut(BitmapId::PageB), 0, 0);
    copy_title_picture_to(&mut video.screen, BitmapId::Page1);
    copy_title_picture_to(&mut video.screen, BitmapId::Page2);

    video.screen.pages[1].ras = 0;

    for x in (0..=ZOOM_FULL).step_by(ZOOM_STEP) {
        set_intro_zoom(game, video, x);
    }

    video.screen.pages[1].ras = 1;

    if intro_skipped(&mut game.ui) == 0 {
        for (first, second, first_wait, second_wait) in STORY_PAGES {
            show_story_page(game, video, first, second, first_wait, second_wait);
        }

        if { game.ui.intro_skip_requested } == 0 {
            delay(game, video, INTRO_PAGES_WAIT);
        }
    }

    video.screen.pages[1].ras = 0;

    for x in (0..=ZOOM_NORMAL).rev().step_by(ZOOM_STEP) {
        set_intro_zoom(game, video, x);
    }
}

/// Copies the whole title picture (loaded into `PageB`) onto the bitmap of a display page.
fn copy_title_picture_to(screen: &mut Screen, page: BitmapId) {
    let whole = Rect {
        x: 0,
        y: 0,
        width: PAGE_SIZE.0,
        height: PAGE_SIZE.1,
    };

    screen.copy(BitmapId::PageB, whole, page, (0, 0), ALL_FIVE_PLANES);
}
