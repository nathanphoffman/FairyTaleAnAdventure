//! An `Output` that draws nothing and writes down what the rules asked for, so the rules can run
//! (and be tested) without a screen.

use crate::game::data::state::{Hero, MapData, World};
use crate::game::output::Output;

/// One thing the rules asked an `Output` to do.
#[derive(Debug, Clone, PartialEq)]
pub enum Call {
    PrintBytes(Vec<u8>),
    PrintContBytes(Vec<u8>),
    PrintNumber(i32, i32),
    LoadPageColors(Vec<u16>),
    SwapPages,
    Present,
    WaitFrame,
    ForgetSprites,
    RewindSpritesTo(u8),
    StartSpriteSheet(u8),
    LoadSpriteFile(i32),
    FinishSpriteLoad(i16),
    LoadImageBank(usize, i16),
    SetTextPosition(i32, i32),
    UseStoryFont,
    UsePanelFont,
    SelectMessageArea,
    ClearPlacard,
    DrawPlacardBorder,
    DrawMenuEntry {
        slot: usize,
        label: Vec<u8>,
        text_pen: u8,
        box_pen: u8,
    },
    BlankMenuEntry(usize),
    DrawCompass(i16),
    DrawItemList,
    DrawOverviewMap(i32, i32),
    MarkOverviewMap(Option<(i32, i32)>),
    ClearDrawingPage(u8),
    ResetPageScroll,
    PrepareMessagePage(u8),
    SetPanelVisible(bool),
    SetZoomSize(i32, i32),
    LoadPictureToDrawingPage(String),
    LoadPanelColors(Vec<u16>),
}

/// Records every call, and counts a tick for each `wait_frame`.
pub struct Recorder {
    pub calls: Vec<Call>,
    ticks: u32,
}

impl Default for Recorder {
    fn default() -> Recorder {
        Recorder::new()
    }
}

impl Recorder {
    pub fn new() -> Recorder {
        Recorder {
            calls: Vec::new(),
            ticks: 0,
        }
    }

    /// The lines started with `print_bytes`, each with what `print_cont_bytes` added.
    pub fn printed_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();

        for call in &self.calls {
            match call {
                Call::PrintBytes(text) => lines.push(String::from_utf8_lossy(text).into_owned()),
                Call::PrintContBytes(text) => {
                    if let Some(last) = lines.last_mut() {
                        last.push_str(&String::from_utf8_lossy(text));
                    }
                }
                _ => {}
            }
        }

        lines
    }
}

impl Output for Recorder {
    fn print_bytes(&mut self, text: &[u8]) {
        self.calls.push(Call::PrintBytes(text.to_vec()));
    }

    fn print_cont_bytes(&mut self, text: &[u8]) {
        self.calls.push(Call::PrintContBytes(text.to_vec()));
    }

    fn print_number(&mut self, value: i32, len: i32) {
        self.calls.push(Call::PrintNumber(value, len));
    }

    fn load_page_colors(&mut self, colors: &[u16]) {
        self.calls.push(Call::LoadPageColors(colors.to_vec()));
    }

    fn swap_pages(&mut self) {
        self.calls.push(Call::SwapPages);
    }

    fn present(&mut self) {
        self.calls.push(Call::Present);
    }

    fn wait_frame(&mut self) {
        self.ticks += 1;
        self.calls.push(Call::WaitFrame);
    }

    fn ticks(&self) -> u32 {
        self.ticks
    }

    fn forget_sprites(&mut self) {
        self.calls.push(Call::ForgetSprites);
    }

    fn rewind_sprites_to(&mut self, slot: u8) {
        self.calls.push(Call::RewindSpritesTo(slot));
    }

    fn start_sprite_sheet(&mut self, slot: u8) {
        self.calls.push(Call::StartSpriteSheet(slot));
    }

    fn load_sprite_file(&mut self, file: i32) {
        self.calls.push(Call::LoadSpriteFile(file));
    }

    fn finish_sprite_load(&mut self, slot: i16) {
        self.calls.push(Call::FinishSpriteLoad(slot));
    }

    fn load_image_bank(&mut self, bank: usize, first_block: i16) {
        self.calls.push(Call::LoadImageBank(bank, first_block));
    }

    fn set_text_position(&mut self, x: i32, y: i32) {
        self.calls.push(Call::SetTextPosition(x, y));
    }

    fn use_story_font(&mut self) {
        self.calls.push(Call::UseStoryFont);
    }

    fn use_panel_font(&mut self) {
        self.calls.push(Call::UsePanelFont);
    }

    fn select_message_area(&mut self) {
        self.calls.push(Call::SelectMessageArea);
    }

    fn clear_placard(&mut self) {
        self.calls.push(Call::ClearPlacard);
    }

    fn draw_placard_border(&mut self) {
        self.calls.push(Call::DrawPlacardBorder);
    }

    fn draw_menu_entry(&mut self, slot: usize, label: &[u8], text_pen: u8, box_pen: u8) {
        self.calls.push(Call::DrawMenuEntry {
            slot,
            label: label.to_vec(),
            text_pen,
            box_pen,
        });
    }

    fn blank_menu_entry(&mut self, slot: usize) {
        self.calls.push(Call::BlankMenuEntry(slot));
    }

    fn draw_compass(&mut self, dir: i16) {
        self.calls.push(Call::DrawCompass(dir));
    }

    fn draw_item_list(&mut self, _hero: &mut Hero) {
        self.calls.push(Call::DrawItemList);
    }

    fn draw_overview_map(&mut self, _map: &MapData, _world: &mut World, mx: i32, my: i32) {
        self.calls.push(Call::DrawOverviewMap(mx, my));
    }

    fn mark_overview_map(&mut self, at: Option<(i32, i32)>) {
        self.calls.push(Call::MarkOverviewMap(at));
    }

    fn clear_drawing_page(&mut self, pen: u8) {
        self.calls.push(Call::ClearDrawingPage(pen));
    }

    fn reset_page_scroll(&mut self) {
        self.calls.push(Call::ResetPageScroll);
    }

    fn prepare_message_page(&mut self, pen: u8) {
        self.calls.push(Call::PrepareMessagePage(pen));
    }

    fn set_panel_visible(&mut self, visible: bool) {
        self.calls.push(Call::SetPanelVisible(visible));
    }

    fn set_zoom_size(&mut self, x: i32, y: i32) {
        self.calls.push(Call::SetZoomSize(x, y));
    }

    fn load_picture_to_drawing_page(&mut self, name: &str) {
        self.calls
            .push(Call::LoadPictureToDrawingPage(name.to_string()));
    }

    fn load_panel_colors(&mut self, colors: &[u16]) {
        self.calls.push(Call::LoadPanelColors(colors.to_vec()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::data::state::Game;
    use crate::game::render::screen_fx::map_message;
    use crate::game::ui::menus::print_options;
    use crate::game::ui::text::{show_error, show_event};

    #[test]
    fn show_error_prints_the_error_number_at_a_fixed_place() {
        let mut out = Recorder::new();
        show_error(&mut out, 5);

        assert_eq!(
            out.calls,
            vec![
                Call::SetTextPosition(200, 90),
                Call::PrintContBytes(b"ERROR ".to_vec()),
                Call::PrintNumber(5, 2),
            ]
        );
    }

    #[test]
    fn show_error_without_graphics_prints_nothing() {
        let mut out = Recorder::new();
        show_error(&mut out, 2);

        assert!(out.calls.is_empty());
    }

    #[test]
    fn a_menu_prints_one_entry_per_slot_and_blanks_the_rest() {
        let mut game = Game::new();
        let mut out = Recorder::new();
        print_options(&mut game, &mut out);

        let drawn = out
            .calls
            .iter()
            .filter(|c| matches!(c, Call::DrawMenuEntry { .. }))
            .count();
        let blank = out
            .calls
            .iter()
            .filter(|c| matches!(c, Call::BlankMenuEntry(_)))
            .count();
        assert_eq!(drawn + blank, 12);
    }

    #[test]
    fn an_event_message_is_printed_as_text_lines() {
        let mut game = Game::new();
        game.hero.brother = 1;
        let mut out = Recorder::new();
        show_event(&mut game, &mut out, 0);

        assert!(!out.printed_lines().is_empty());
    }

    #[test]
    fn a_map_message_fades_out_switches_page_and_shows_again() {
        let mut game = Game::new();
        let mut out = Recorder::new();
        map_message(&mut game, &mut out);

        let prepare = out
            .calls
            .iter()
            .position(|c| matches!(c, Call::PrepareMessagePage(_)));
        let hide = out
            .calls
            .iter()
            .position(|c| *c == Call::SetPanelVisible(false));
        assert!(prepare.is_some() && hide > prepare);

        assert!(out.ticks() > 0, "the fade waits for frames");
        assert_eq!(game.ui.view_status, 2);
    }
}
