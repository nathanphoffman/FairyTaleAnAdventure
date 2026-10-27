//! The title screen, the intro and getting the first brother started (from csrc/fmain.c), up to the
//! first frame of the game.

use crate::amiga::{BitMap, RastPort, TextFont, View, ViewPort};
use crate::game::files::loading::intro_skip_requested;
use crate::game::ui::menus::print_options;

use crate::game::actors::revive::revive;
use crate::game::data::globals::{main_view, story_font};
use crate::game::data::state::{the_memory, the_screen, the_ui};
use crate::game::data::tables::{black_colors, pointer_sprite, text_colors};
use crate::game::files::disk::{load_track_range, mark_last_read_unused, wait_for_last_read};
use crate::game::files::iff_picture::load_picture;
use crate::game::files::loading::{
    copy_protection_check, copy_protection_riddle, intro_skipped, read_sample, read_score,
    show_story_page,
};

use crate::game::render::display::flip_pages;
use crate::game::render::screen_fx::{set_intro_zoom, show_still_screen};
use crate::game::ui::text::{placard_text, print_positioned};

unsafe extern "C" {

    // AmigaOS calls the platform layer provides.
    fn SetBPen(current_rastport: *mut RastPort, pen: u32);

    fn SetAPen(current_rastport: *mut RastPort, pen: u32);

    fn SetRast(current_rastport: *mut RastPort, pen: u32);

    fn SetFont(current_rastport: *mut RastPort, f: *mut TextFont);

    fn MakeVPort(v: *mut View, current_viewport: *mut ViewPort) -> i32;

    fn Delay(ticks: i32);

    fn SetRGB4(current_viewport: *mut ViewPort, n: i32, r: u32, g: u32, b: u32);

    fn LoadRGB4(current_viewport: *mut ViewPort, colors: *const u16, count: i32);

    fn BltBitMap(
        src: *mut BitMap,
        sx: i32,
        sy: i32,
        dst: *mut BitMap,
        dx: i32,
        dy: i32,
        w: i32,
        h: i32,
        minterm: u32,
        mask: u32,
        temp: *mut u8,
    ) -> i32;

    // The game's own code that is still C, or lives in other files.
    fn stopscore();
}

// Memory the game allocates (csrc/fmain.c).
const PAGE_HEIGHT: i16 = 143;
const TEXT_HEIGHT: i32 = 57;
const LOWRES_WIDTH: i16 = 288;
const PAGE_VIEW_HEIGHT: i16 = 140;
const PAGE_VIEW_X: i16 = 16;

unsafe extern "C" {
    static titletext: [u8; 0];

    fn playscore(t1: *const u8, t2: *const u8, t3: *const u8, t4: *const u8);
}

// Screen modes (`view_status`).
const VIEW_REDRAW: i8 = 99;

/// Intro: how big the picture zooms (half the width in pixels), and the step.
const ZOOM_FULL: i32 = 160;
const ZOOM_NORMAL: i32 = 156;
const ZOOM_STEP: usize = 4;

/// Colours `LoadRGB4` sets for the page and the text.
const PAGE_COLORS: i32 = 32;
const TEXT_COLORS: i32 = 20;

/// Pens: the title text, and the colour of the flashing pointer.
const TITLE_PEN: u32 = 1;

/// Waits, in fiftieths of a second.
const TITLE_WAIT: i32 = 50;
const INTRO_PAGES_WAIT: i32 = 190;
const PLACARD_WAIT: i32 = 20;

/// The placard shown while the copy protection question is asked.
const PLACARD_COPYRIGHT: i32 = 19;

/// Where the picture planes in `image_mem` start: five planes, 8000 bytes apart.
const IMAGE_PLANE_BYTES: usize = 8000;
const PAGE_HALF_BYTES: usize = 40000;

/// Disk track range holding the shadow masks.
const SHADOW_BLOCK: (i16, i16) = (896, 24);

/// Returns false if the player failed the copy protection.
pub(super) unsafe fn start_up() -> bool {
    unsafe {
        stopscore();
        show_title_screen();
        prepare_intro_pages_and_music();

        if let Some(dir) = std::env::var_os("FTA_MUSIC_EXPORT") {
            crate::game::ui::music::export_tunes(std::path::Path::new(&dir));

            return false;
        }

        if intro_skipped() == 0 {
            play_intro();
        }

        copy_protection_check();
        show_play_screen_background();

        if copy_protection_riddle() == 0 {
            return false;
        }

        Delay(PLACARD_WAIT);
        start_first_brother();

        true
    }
}

/// Blanks both pages.
unsafe fn clear_both_pages() {
    unsafe {
        the_screen.rp_map.BitMap = (*(*the_screen.viewing_page).ri_page).BitMap;
        SetRast(&raw mut the_screen.rp_map, 0);
        the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;
        SetRast(&raw mut the_screen.rp_map, 0);
    }
}

/// The legal title text on a blue page, then the music and the sound effects are read in.
unsafe fn show_title_screen() {
    unsafe {
        the_screen.vp_page.RasInfo = &raw mut the_screen.ri_page1;
        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page2;
        clear_both_pages();
        the_screen.current_rastport = &raw mut the_screen.rp_map;
        set_intro_zoom(ZOOM_NORMAL);
        SetRGB4(&raw mut the_screen.vp_page, 0, 0, 0, 6);
        SetRGB4(&raw mut the_screen.vp_page, 1, 15, 15, 15);

        let pen = (*the_screen.current_rastport).FgPen as u32;
        SetAPen(the_screen.current_rastport, TITLE_PEN);
        print_positioned(titletext.as_ptr());
        SetAPen(the_screen.current_rastport, pen);

        Delay(TITLE_WAIT);

        the_screen.current_rastport = &raw mut the_screen.rp_text;
        the_screen.rp_text.BitMap = &raw mut the_screen.bm_scroll;
        SetFont(the_screen.current_rastport, story_font);
        SetAPen(the_screen.current_rastport, 10);
        SetBPen(the_screen.current_rastport, 11);

        read_score();
        read_sample();

        Delay(TITLE_WAIT);
    }
}

/// Points the picture loader at the page memory, starts the intro tune, and flips the pages twice
/// so both show the same thing.
unsafe fn prepare_intro_pages_and_music() {
    unsafe {
        the_screen.current_viewport = &raw mut the_screen.vp_page; // so that the IFF routines can communicate
        the_screen.vp_page.RasInfo = &raw mut the_screen.ri_page1;

        for i in 0..5 {
            the_screen.pageb.Planes[i] = the_memory.image_mem.add(i * IMAGE_PLANE_BYTES);
            the_screen.pagea.Planes[i] = the_screen.pageb.Planes[i].add(PAGE_HALF_BYTES);
        }

        (*the_screen.bm_lim).Planes[0] = the_memory.sector_mem;

        playscore(
            the_memory.track[12],
            the_memory.track[13],
            the_memory.track[14],
            the_memory.track[15],
        );

        LoadRGB4(
            &raw mut the_screen.vp_text,
            (&raw const black_colors).cast(),
            PAGE_COLORS,
        );

        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page2;
        set_intro_zoom(0);

        flip_pages(); // "prime the pump"
        flip_pages();
    }
}

/// Switches to the normal playing screen: loads the shadow masks and the panel picture, and shows
/// the copyright placard.
unsafe fn show_play_screen_background() {
    unsafe {
        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page2;
        clear_both_pages();

        LoadRGB4(
            &raw mut the_screen.vp_text,
            (&raw const black_colors).cast(),
            PAGE_COLORS,
        );

        set_intro_zoom(ZOOM_NORMAL);
        SetRGB4(&raw mut the_screen.vp_page, 0, 0, 0, 3);
        load_track_range(
            SHADOW_BLOCK.0,
            SHADOW_BLOCK.1,
            the_memory.shadow_mem.cast(),
            0,
        );

        wait_for_last_read();
        mark_last_read_unused();

        SetRGB4(&raw mut the_screen.vp_page, 0, 0, 0, 6);
        load_picture(c"hiscreen".as_ptr(), the_screen.bm_text, 0, 0);

        SetRGB4(&raw mut the_screen.vp_page, 1, 15, 15, 15);

        the_screen.current_rastport = &raw mut the_screen.rp_map;
        the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;
        show_still_screen();
        SetAPen(the_screen.current_rastport, TITLE_PEN);
        placard_text(PLACARD_COPYRIGHT);
        the_ui.handler_data.laydown = 0;
        the_ui.handler_data.pickup = 0;
    }
}

/// Starts the first brother and sets up the text page and the colours for play.
unsafe fn start_first_brother() {
    unsafe {
        the_screen.ri_page1.RxOffset = 0;
        the_screen.ri_page2.RxOffset = 0;
        the_screen.ri_page1.RyOffset = 0;
        the_screen.ri_page2.RyOffset = 0;

        stopscore();
        revive(1);
        the_screen.rp_map.BitMap = (*(*the_screen.drawing_page).ri_page).BitMap;

        the_screen.vp_text.DyOffset = PAGE_HEIGHT;
        the_screen.vp_text.DHeight = TEXT_HEIGHT as i16;
        the_screen.vp_page.DHeight = PAGE_VIEW_HEIGHT;
        the_screen.vp_page.DxOffset = PAGE_VIEW_X;
        the_screen.vp_page.DWidth = LOWRES_WIDTH;
        the_screen.vp_page.DyOffset = 0;
        MakeVPort(&raw mut main_view, &raw mut the_screen.vp_text);
        LoadRGB4(
            &raw mut the_screen.vp_text,
            (&raw const text_colors).cast(),
            TEXT_COLORS,
        );

        the_ui.handler_data.pbase = (&raw mut pointer_sprite).cast();

        the_screen.fp_page1.obcount = 0;
        the_screen.fp_page2.obcount = 0;
        the_screen.fp_page1.wflag = 0;
        the_screen.fp_page2.wflag = 0;

        the_screen.view_status = VIEW_REDRAW;

        the_ui.current_menu = 0;
        print_options();
    }
}

/// The story pictures with the zoom in and out (skipped if the player presses Space).
unsafe fn play_intro() {
    unsafe {
        load_picture(c"page0".as_ptr(), &raw mut the_screen.pageb, 0, 0);
        copy_title_picture_to(the_screen.bm_page1);
        copy_title_picture_to(the_screen.bm_page2);

        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page1;

        for x in (0..=ZOOM_FULL).step_by(ZOOM_STEP) {
            set_intro_zoom(x);
        }

        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page2;

        if intro_skipped() == 0 {
            show_story_page(c"p1a".as_ptr(), c"p1b".as_ptr(), 21, 29);
            show_story_page(c"p2a".as_ptr(), c"p2b".as_ptr(), 20, 29);
            show_story_page(c"p3a".as_ptr(), c"p3b".as_ptr(), 20, 33);

            if intro_skip_requested == 0 {
                Delay(INTRO_PAGES_WAIT);
            }
        }

        the_screen.fp_page2.ri_page = &raw mut the_screen.ri_page1;

        for x in (0..=ZOOM_NORMAL).rev().step_by(ZOOM_STEP) {
            set_intro_zoom(x);
        }
    }
}

/// Copies the whole title picture (loaded into `pageb`) onto the bitmap of a display page.
unsafe fn copy_title_picture_to(bitmap: *mut BitMap) {
    unsafe {
        BltBitMap(
            &raw mut the_screen.pageb,
            0,
            0,
            bitmap,
            0,
            0,
            320,
            200,
            0xC0,
            0x1f,
            std::ptr::null_mut(),
        );
    }
}
