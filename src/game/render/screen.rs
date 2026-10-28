//! The screen: the bitmaps the game draws on, the pens that draw on them, the two view ports that
//! make up the picture, and the two display pages that are drawn and shown in turn.
//!
//! The play field is a lores picture of 5 bit planes (32 colours) on top; the text panel is a hires
//! picture of 4 bit planes below it. The game draws on one page while the other is on screen, then
//! swaps them (`flip_pages`).

use crate::game::data::types::FPage;
use crate::gfx::Font;
use crate::gfx::bitmap::{Bitmap, Rect, copy_rect};
use crate::gfx::draw::{self, Pen};
use crate::gfx::scene::{Pointer, Scene, ViewPort};

/// The bitmaps the game keeps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BitmapId {
    /// The two display pages.
    Page1,
    Page2,
    /// The text panel, all 4 planes.
    Text,
    /// The text panel, but only its first plane (the message area is drawn in one colour).
    TextPlane0,
    /// The two pages of the story book, which are also work areas for the picture loader.
    PageA,
    PageB,
    /// The pictures of the things the hero carries, one long strip, for the item list.
    Items,
}

impl BitmapId {
    fn index(self) -> usize {
        match self {
            BitmapId::Page1 => 0,
            BitmapId::Page2 => 1,
            BitmapId::Text | BitmapId::TextPlane0 => 2,
            BitmapId::PageA => 3,
            BitmapId::PageB => 4,
            BitmapId::Items => 5,
        }
    }

    /// How many planes drawing on this bitmap touches.
    fn planes(self) -> usize {
        match self {
            BitmapId::TextPlane0 => 1,
            _ => usize::MAX,
        }
    }
}

/// The three places text and graphics are drawn from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PortId {
    /// The play field.
    Map,
    /// The message area of the text panel.
    Text,
    /// The menu area of the text panel.
    Text2,
}

impl PortId {
    fn index(self) -> usize {
        match self {
            PortId::Map => 0,
            PortId::Text => 1,
            PortId::Text2 => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FontKind {
    /// The small font of the panel.
    Panel,
    /// The story font.
    Story,
}

/// A pen, the bitmap it draws on and the font it writes in.
pub struct Port {
    pub pen: Pen,
    pub target: Option<BitmapId>,
    pub font: FontKind,
}

impl Port {
    const fn new() -> Port {
        Port {
            pen: Pen::new(),
            target: None,
            font: FontKind::Panel,
        }
    }
}

/// Which part of which bitmap a view port shows (`ras` is the index into `Screen::ras`).
#[derive(Clone, Copy)]
pub struct RasInfo {
    pub bitmap: BitmapId,
    pub x: i16,
    pub y: i16,
}

/// A rectangle of the screen that shows a bitmap with its own colours.
pub struct ViewPortState {
    /// The index into `Screen::ras` of what it shows, once it shows anything.
    pub ras: Option<usize>,
    pub width: i16,
    pub height: i16,
    pub x: i16,
    pub y: i16,
    pub modes: u16,
    pub colors: Vec<u16>,
}

impl ViewPortState {
    const fn new() -> ViewPortState {
        ViewPortState {
            ras: None,
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            modes: 0,
            colors: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewPortId {
    /// The play field.
    Page,
    /// The text panel.
    Text,
}

/// View port mode bits.
pub const HIRES: u16 = 0x8000;
pub const SPRITES: u16 = 0x4000;
pub const VP_HIDE: u16 = 0x2000;

// Sizes of the bitmaps. The play field is 320 wide with room to scroll, 5 planes deep; the text
// panel is hires, 4 planes deep.
const PAGE_WIDTH: usize = 320;
const PAGE_ROWS: usize = 200;
const PAGE_DEPTH: usize = 5;
const TEXT_WIDTH: usize = 640;
const TEXT_ROWS: usize = 57;
const TEXT_DEPTH: usize = 4;
const COMPASS_WIDTH: usize = 64;
const COMPASS_ROWS: usize = 24;

// Where the two view ports sit on the screen.
const LOWRES_WIDTH: i16 = 288;
const HIRES_WIDTH: i16 = 640;
const PAGE_VIEW_HEIGHT: i16 = 140;
const PAGE_VIEW_X: i16 = 16;
const PAGE_HEIGHT: i16 = 143;
const PAGE_COLORS: usize = 32;
const TEXT_COLORS: usize = 20;

/// Bytes of work memory for the sprite blitter's queue (and, the same again, its mask).
const BLITTER_REGION: usize = 6880 + 256;

// A colour is 12 bits: 4 bits each of red, green and blue.
const COMPONENT_MASK: u32 = 15;
const RED_SHIFT: u32 = 8;
const GREEN_SHIFT: u32 = 4;

pub struct Screen {
    bitmaps: [Bitmap; 6],
    /// The compass picture: lit up, and not.
    pub compass: [Bitmap; 2],
    pub ras: [RasInfo; 3],
    pub pages: [FPage; 2],
    /// Which of the two pages is being drawn on and which is on screen.
    pub drawing: usize,
    pub viewing: usize,
    ports: [Port; 3],
    pub current_port: PortId,
    fonts: [Font; 2],
    pub vp_page: ViewPortState,
    pub vp_text: ViewPortState,
    /// The bitmap the map and the sprites are being drawn on.
    pub draw_target: BitmapId,
    /// The mouse pointer's picture, once it is set up.
    pub pointer: Option<&'static [u32]>,
    /// The sprite blitter's work memory: saved backgrounds, and the scenery mask.
    pub queue_region: Vec<u8>,
    pub mask_region: Vec<u8>,
    pub planesize: i16,
    pub view_status: i8,
    pub flasher: i8,
}

impl Screen {
    pub const fn new() -> Screen {
        Screen {
            bitmaps: [
                Bitmap::empty(),
                Bitmap::empty(),
                Bitmap::empty(),
                Bitmap::empty(),
                Bitmap::empty(),
                Bitmap::empty(),
            ],
            compass: [Bitmap::empty(), Bitmap::empty()],
            ras: [RasInfo {
                bitmap: BitmapId::Page1,
                x: 0,
                y: 0,
            }; 3],
            pages: [FPage::new(), FPage::new()],
            drawing: 0,
            viewing: 1,
            ports: [Port::new(), Port::new(), Port::new()],
            current_port: PortId::Text,
            fonts: [Font::Builtin, Font::Builtin],
            vp_page: ViewPortState::new(),
            vp_text: ViewPortState::new(),
            draw_target: BitmapId::Page1,
            pointer: None,
            queue_region: Vec::new(),
            mask_region: Vec::new(),
            planesize: 0,
            view_status: 0,
            flasher: 0,
        }
    }

    /// Makes the bitmaps, pens and view ports, and the memory the sprite blitter works in.
    pub fn set_up(&mut self, compass: [&[u8]; 2]) {
        self.make_bitmaps(compass);
        self.make_rasters_and_ports();
        self.make_view_ports();

        self.queue_region = vec![0; BLITTER_REGION];
        self.mask_region = vec![0; BLITTER_REGION];
    }

    /// The pages, the text panel and the compass pictures.
    fn make_bitmaps(&mut self, compass: [&[u8]; 2]) {
        let page = || Bitmap::new(PAGE_WIDTH, PAGE_ROWS, PAGE_DEPTH);

        self.set_bitmap(BitmapId::Page1, page());
        self.set_bitmap(BitmapId::Page2, page());
        self.set_bitmap(
            BitmapId::Text,
            Bitmap::new(TEXT_WIDTH, TEXT_ROWS, TEXT_DEPTH),
        );
        self.set_bitmap(BitmapId::PageA, page());
        self.set_bitmap(BitmapId::PageB, page());
        self.compass = compass.map(|data| Bitmap::from_plane(COMPASS_WIDTH, COMPASS_ROWS, data));
    }

    /// Which bitmap each raster shows, which page is drawn and shown, and where each port draws.
    fn make_rasters_and_ports(&mut self) {
        let at_origin = |bitmap| RasInfo { bitmap, x: 0, y: 0 };

        self.ras = [
            at_origin(BitmapId::Page1),
            at_origin(BitmapId::Page2),
            at_origin(BitmapId::Text),
        ];

        // Both pages start on the first bitmap; the intro gives the second page its own.
        self.pages[0].ras = 0;
        self.pages[1].ras = 0;
        self.drawing = 0;
        self.viewing = 1;

        self.ports[PortId::Map.index()].target = None;
        self.ports[PortId::Text.index()].target = Some(BitmapId::Text);
        self.ports[PortId::Text2.index()].target = Some(BitmapId::Text);
        self.current_port = PortId::Text;
    }

    /// The play field on top and the text panel below it.
    fn make_view_ports(&mut self) {
        self.vp_page = ViewPortState {
            ras: None,
            width: LOWRES_WIDTH,
            height: PAGE_VIEW_HEIGHT,
            x: PAGE_VIEW_X,
            y: 0,
            modes: 0,
            colors: vec![0; PAGE_COLORS],
        };

        self.vp_text = ViewPortState {
            ras: Some(2),
            width: HIRES_WIDTH,
            height: TEXT_ROWS as i16,
            x: 0,
            y: PAGE_HEIGHT,
            modes: HIRES | SPRITES | VP_HIDE,
            colors: vec![0; TEXT_COLORS],
        };
    }

    // ---- bitmaps ----

    pub fn bitmap(&self, id: BitmapId) -> &Bitmap {
        &self.bitmaps[id.index()]
    }

    pub fn bitmap_mut(&mut self, id: BitmapId) -> &mut Bitmap {
        &mut self.bitmaps[id.index()]
    }

    pub fn set_bitmap(&mut self, id: BitmapId, bitmap: Bitmap) {
        self.bitmaps[id.index()] = bitmap;
    }

    /// The bitmap the map and sprites are being drawn on.
    /// What the sprite blitter works on: the bitmap being drawn on and its two work regions.
    pub fn blitter_memory(&mut self) -> (&mut Bitmap, &mut [u8], &mut [u8]) {
        (
            &mut self.bitmaps[self.draw_target.index()],
            &mut self.queue_region,
            &mut self.mask_region,
        )
    }

    pub fn draw_bitmap_mut(&mut self) -> &mut Bitmap {
        let id = self.draw_target;

        self.bitmap_mut(id)
    }

    /// The bitmap of display page `n` (0 or 1) as it is wired up now.
    pub fn page_bitmap(&self, n: usize) -> BitmapId {
        self.ras[self.pages[n].ras].bitmap
    }

    /// The page being drawn on.
    pub fn drawing_page(&self) -> &FPage {
        &self.pages[self.drawing]
    }

    pub fn drawing_page_mut(&mut self) -> &mut FPage {
        &mut self.pages[self.drawing]
    }

    /// The bitmap of the page being drawn on / being shown.
    pub fn drawing_bitmap_id(&self) -> BitmapId {
        self.page_bitmap(self.drawing)
    }

    pub fn viewing_bitmap_id(&self) -> BitmapId {
        self.page_bitmap(self.viewing)
    }

    // ---- ports ----

    pub fn port(&self, id: PortId) -> &Port {
        &self.ports[id.index()]
    }

    pub fn port_mut(&mut self, id: PortId) -> &mut Port {
        &mut self.ports[id.index()]
    }

    pub fn set_font_kind(&mut self, id: PortId, kind: FontKind) {
        self.port_mut(id).font = kind;
    }

    pub fn set_font(&mut self, kind: FontKind, font: Font) {
        self.fonts[kind as usize] = font;
    }

    pub fn set_a_pen(&mut self, id: PortId, pen: u8) {
        self.port_mut(id).pen.fg = pen;
    }

    pub fn set_b_pen(&mut self, id: PortId, pen: u8) {
        self.port_mut(id).pen.bg = pen;
    }

    pub fn set_draw_mode(&mut self, id: PortId, mode: u8) {
        self.port_mut(id).pen.mode = mode;
    }

    pub fn move_to(&mut self, id: PortId, x: i32, y: i32) {
        let pen = &mut self.port_mut(id).pen;
        pen.x = x;
        pen.y = y;
    }

    /// Splits a port from the bitmap it draws on and its font.
    fn target(&mut self, id: PortId) -> Option<(&mut Bitmap, &mut Pen, &Font, usize)> {
        let port = &mut self.ports[id.index()];
        let target = port.target?;

        Some((
            &mut self.bitmaps[target.index()],
            &mut port.pen,
            &self.fonts[port.font as usize],
            target.planes(),
        ))
    }

    /// Writes the bytes at the pen of the port.
    pub fn text(&mut self, id: PortId, bytes: &[u8]) {
        if let Some((bitmap, pen, font, planes)) = self.target(id) {
            draw::text(bitmap, pen, font, bytes, planes);
        }
    }

    /// Draws a line from the port's pen to (`x`, `y`).
    pub fn line_to(&mut self, id: PortId, x: i32, y: i32) {
        if let Some((bitmap, pen, _, planes)) = self.target(id) {
            draw::line_to(bitmap, pen, (x, y), planes);
        }
    }

    pub fn rect_fill(&mut self, id: PortId, from: (i32, i32), to: (i32, i32)) {
        if let Some((bitmap, pen, _, planes)) = self.target(id) {
            draw::rect_fill(bitmap, pen, from, to, planes);
        }
    }

    pub fn fill_polygon(&mut self, id: PortId, points: &[(i32, i32)], clip: Rect) {
        if let Some((bitmap, pen, _, planes)) = self.target(id) {
            draw::fill_polygon(bitmap, pen, points, clip, planes);
        }
    }

    /// Fills the whole bitmap of the port with colour `pen`.
    pub fn clear(&mut self, id: PortId, pen: u8) {
        if let Some((bitmap, _, _, planes)) = self.target(id) {
            bitmap.clear(pen, planes);
        }
    }

    /// Scrolls an area of the port's bitmap; what scrolls in gets the port's background colour.
    pub fn scroll(&mut self, id: PortId, area: Rect, by: (i32, i32)) {
        if let Some((bitmap, pen, _, planes)) = self.target(id) {
            crate::gfx::bitmap::scroll(bitmap, area, by, pen.bg, planes);
        }
    }

    /// Copies a rectangle of one bitmap to another (not the same one), for the planes in `mask`.
    pub fn copy(&mut self, from: BitmapId, rect: Rect, to: BitmapId, at: (i32, i32), mask: u32) {
        let (src, dst) = self.two_bitmaps(from, to);

        copy_rect(src, dst, rect, at, mask);
    }

    /// The compass picture `which` (0 normal, 1 lit) copied into plane `plane` of the text panel.
    pub fn copy_compass(&mut self, which: usize, rect: Rect, at: (i32, i32), plane: usize) {
        let dst = &mut self.bitmaps[BitmapId::Text.index()];

        crate::gfx::bitmap::copy_plane_rect(&self.compass[which], 0, dst, plane, rect, at);
    }

    /// Two different bitmaps, the first to read and the second to write.
    pub fn two_bitmaps(&mut self, from: BitmapId, to: BitmapId) -> (&Bitmap, &mut Bitmap) {
        let (a, b) = (from.index(), to.index());
        assert_ne!(a, b, "a copy needs two different bitmaps");

        if a < b {
            let (left, right) = self.bitmaps.split_at_mut(b);

            (&left[a], &mut right[0])
        } else {
            let (left, right) = self.bitmaps.split_at_mut(a);

            (&right[0], &mut left[b])
        }
    }

    // ---- view ports ----

    pub fn view_port_mut(&mut self, id: ViewPortId) -> &mut ViewPortState {
        match id {
            ViewPortId::Page => &mut self.vp_page,
            ViewPortId::Text => &mut self.vp_text,
        }
    }

    /// Loads the first colours of a view port's table from `colors` (12 bit colours, 0x0RGB).
    pub fn load_colors(&mut self, id: ViewPortId, colors: &[u16]) {
        let table = &mut self.view_port_mut(id).colors;
        let n = table.len().min(colors.len());
        table[..n].copy_from_slice(&colors[..n]);
    }

    /// Sets one colour of a view port from its red, green and blue (4 bits each).
    pub fn set_color(&mut self, id: ViewPortId, n: usize, (r, g, b): (u32, u32, u32)) {
        if let Some(color) = self.view_port_mut(id).colors.get_mut(n) {
            *color = ((r & COMPONENT_MASK) << RED_SHIFT
                | (g & COMPONENT_MASK) << GREEN_SHIFT
                | (b & COMPONENT_MASK)) as u16;
        }
    }

    // ---- showing ----

    fn view_port<'a>(&'a self, state: &'a ViewPortState, ras: usize) -> ViewPort<'a> {
        let ras = &self.ras[ras];

        ViewPort {
            bitmap: self.bitmap(ras.bitmap),
            scroll: (ras.x as i32, ras.y as i32),
            position: (state.x as i32, state.y as i32),
            size: (state.width as i32, state.height as i32),
            hires: state.modes & HIRES != 0,
            colors: &state.colors,
            shows_pointer: state.modes & SPRITES != 0,
        }
    }

    /// What is on screen now: the text panel on top of the play field, unless hidden.
    pub fn scene(&self) -> Scene<'_> {
        let mut view_ports = Vec::new();

        for state in [&self.vp_text, &self.vp_page] {
            if let Some(ras) = state.ras
                && state.modes & VP_HIDE == 0
                && !state.colors.is_empty()
            {
                view_ports.push(self.view_port(state, ras));
            }
        }

        Scene {
            view_ports,
            pointer: self.pointer.map(|image| Pointer {
                image,
                position: crate::platform::input::pointer(),
            }),
        }
    }
}

impl Default for Screen {
    fn default() -> Screen {
        Screen::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> Screen {
        let mut s = Screen::new();
        s.set_bitmap(BitmapId::Page1, Bitmap::new(320, 200, 5));
        s.set_bitmap(BitmapId::Page2, Bitmap::new(320, 200, 5));
        s.set_bitmap(BitmapId::Text, Bitmap::new(640, 57, 4));
        s.port_mut(PortId::Text2).target = Some(BitmapId::Text);
        s.port_mut(PortId::Text).target = Some(BitmapId::TextPlane0);

        s
    }

    #[test]
    fn the_message_port_draws_only_on_the_first_plane() {
        let mut s = screen();
        s.set_a_pen(PortId::Text, 0b1111);
        s.move_to(PortId::Text, 0, 8);
        s.text(PortId::Text, b"A");

        assert!((0..8).any(|y| (0..8).any(|x| s.bitmap(BitmapId::Text).pixel(x, y) == 1)));
        assert!((0..8).all(|y| (0..8).all(|x| s.bitmap(BitmapId::Text).pixel(x, y) <= 1)));
    }

    #[test]
    fn the_menu_port_uses_all_planes() {
        let mut s = screen();
        s.set_a_pen(PortId::Text2, 0b1010);
        s.rect_fill(PortId::Text2, (0, 0), (3, 3));

        assert_eq!(s.bitmap(BitmapId::Text).pixel(1, 1), 0b1010);
    }

    #[test]
    fn two_bitmaps_come_out_in_the_order_asked() {
        let mut s = screen();
        s.bitmap_mut(BitmapId::Page2).plot(0, 0, 1, false, 1);
        let rect = Rect {
            x: 0,
            y: 0,
            width: 2,
            height: 2,
        };

        s.copy(BitmapId::Page2, rect, BitmapId::Page1, (4, 4), 1);

        assert_eq!(s.bitmap(BitmapId::Page1).pixel(4, 4), 1);
    }
}
