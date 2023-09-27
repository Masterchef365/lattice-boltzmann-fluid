use crate::array2d::{Array2D, GridPos};
use crate::cell::GridCell;
use eframe::egui::{DragValue, Grid, Rgba, RichText, ScrollArea, Slider, Ui};
use egui::os::OperatingSystem;
use egui::{CentralPanel, Frame, Rect, Sense, Stroke, TextEdit};
use egui::{Color32, Rounding, SidePanel};
use glam::Vec2;
use rand::prelude::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TemplateApp {
    source: String,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let source = include_str!("default.frag").to_string();

        Self { source }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        // Update continuously
        ctx.request_repaint();
        SidePanel::left("Source").width_range(300.0..=1000.0).show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| self.source_gui(ui))
        });

        CentralPanel::default().show(ctx, |ui| {
            Frame::canvas(ui.style()).show(ui, |ui| self.output_widget(ui))
        });
    }
}

impl TemplateApp {
    fn output_widget(&mut self, ui: &mut Ui) {
        let (rect, _response) =
            ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
    }

    fn source_gui(&mut self, ui: &mut Ui) {
        ui.separator();
        let resp = TextEdit::multiline(&mut self.source)
            .code_editor()
            .lock_focus(true)
            .show(ui);

        if let Some(cursor) = resp.state.ccursor_range() {
            //let begin = cursor.secondary.index.min(cursor.primary.index);
            let cursor_pos = cursor.primary.index;

            let begin_word = self.source[..cursor_pos]
                .char_indices()
                .rev()
                .find_map(|(idx, char)| char.is_whitespace().then(|| idx + 1))
                .unwrap_or(cursor_pos);

            let end_word = self.source[cursor_pos..]
                .char_indices()
                .find_map(|(idx, char)| char.is_whitespace().then(|| idx + cursor_pos))
                .unwrap_or(cursor_pos);

            dbg!(begin_word, end_word, self.source.len());

            let selected_text = &self.source[begin_word..end_word];
            ui.label(format!("\"{}\"", selected_text));

        }
    }
}
