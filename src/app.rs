use crate::array2d::{Array2D, GridPos};
use crate::cell::GridCell;
use crate::sim::{Sim, calc_total_avg_velocity};
use eframe::egui::{DragValue, Grid, Rgba, RichText, ScrollArea, Slider, Ui};
use egui::os::OperatingSystem;
use egui::{CentralPanel, Frame, Rect, Sense};
use egui::{Color32, Rounding, SidePanel};
use glam::Vec2;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TemplateApp {
    // Sim state
    sim: Sim,

    // Settings
    omega: f32,
    pause: bool,
    single_step: bool,
    show_settings_only: bool,
}

fn new_sim() -> Sim {
    Sim::new(20, 20)
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let sim = new_sim();
        let omega = 1.;

        Self {
            sim,
            omega,
            pause: true,
            single_step: false,
            show_settings_only: false,
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        // Update
        if !self.pause || self.single_step {
            self.sim.grid_mut()[(10, 10)][(0, 0)] = 10.;
            self.sim.step(self.omega);
            self.single_step = false;
        }

        // Update continuously
        ctx.request_repaint();
        if is_mobile(ctx) {
            CentralPanel::default().show(ctx, |ui| {
                ui.checkbox(&mut self.show_settings_only, "Show settings");
                if self.show_settings_only {
                    ScrollArea::both().show(ui, |ui| self.settings_gui(ui));
                } else {
                    Frame::canvas(ui.style()).show(ui, |ui| self.sim_widget(ui));
                }
            });
        } else {
            SidePanel::left("Settings").show(ctx, |ui| {
                ScrollArea::both().show(ui, |ui| self.settings_gui(ui))
            });
            CentralPanel::default().show(ctx, |ui| {
                Frame::canvas(ui.style()).show(ui, |ui| self.sim_widget(ui))
            });
        }
    }
}

impl TemplateApp {
    fn sim_widget(&mut self, ui: &mut Ui) {
        let (rect, _response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());

        let coords = CoordinateMapping::new(&self.sim.grid(), rect);

        let painter = ui.painter_at(rect);
        let square_size = coords.sim_to_egui_vect(Vec2::ONE);
        for j in 0..self.sim.grid().height() {
            for i in 0..self.sim.grid().width() {
                let min = Vec2::new(i as f32, j as f32);
                let min = coords.sim_to_egui(min);
                let rect = Rect::from_min_size(min, square_size.abs());
                let coord = (i, j);

                let vel = calc_total_avg_velocity(&self.sim.grid()[coord]);

                let color = Color32::from_rgb(vel.x.abs() as u8, vel.y.abs() as u8, 0);
                painter.rect_filled(rect, Rounding::none(), color);
            }
        }
    }

    fn settings_gui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.pause, "Pause");
            self.single_step |= ui.button("Step").clicked();
        });
        ui.add(DragValue::new(&mut self.omega).prefix("Omega: ").clamp_range(0.0..=2.0).speed(1e-2));
        if ui.button("Reset").clicked() {
            self.sim = new_sim();
        }
    }
}

/// Maps sim coordinates to/from egui coordinates
struct CoordinateMapping {
    width: f32,
    height: f32,
    area: Rect,
}

impl CoordinateMapping {
    pub fn new(grid: &Array2D<GridCell<f32>>, area: Rect) -> Self {
        //let area = Rect::from_center_size(area.center(), egui::Vec2::splat(area.width().min(area.height())));
        let area = Rect::from_min_size(area.min, egui::Vec2::splat(area.width().min(area.height())));

        Self {
            width: grid.width() as f32 - 1.,
            height: grid.height() as f32 - 1.,
            area,
        }
    }

    pub fn sim_to_egui_vect(&self, pt: glam::Vec2) -> egui::Vec2 {
        egui::Vec2::new(
            (pt.x / self.width) * self.area.width(),
            (-pt.y / self.height) * self.area.height(),
        )
    }

    pub fn sim_to_egui(&self, pt: glam::Vec2) -> egui::Pos2 {
        egui::Pos2::new(
            (pt.x / self.width) * self.area.width(),
            (1. - pt.y / self.height) * self.area.height(),
        ) + self.area.left_top().to_vec2()
    }

    pub fn egui_to_sim(&self, pt: egui::Pos2) -> glam::Vec2 {
        let pt = pt - self.area.left_top().to_vec2();
        glam::Vec2::new(
            (pt.x / self.area.width()) * self.width,
            (1. - pt.y / self.area.height()) * self.height,
        )
    }

    pub fn egui_to_sim_vector(&self, pt: egui::Vec2) -> glam::Vec2 {
        glam::Vec2::new(
            (pt.x / self.area.width()) * self.width,
            (-pt.y / self.area.height()) * self.height,
        )
    }
}

fn is_mobile(ctx: &egui::Context) -> bool {
    matches!(ctx.os(), OperatingSystem::Android | OperatingSystem::IOS)
}
