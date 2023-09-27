use crate::array2d::{Array2D, GridPos};
use crate::cell::GridCell;
use crate::sim::{
    calc_equilibrium_predict, calc_total_avg_velocity, calc_total_density, weights, Sim,
};
use eframe::egui::{DragValue, Grid, Rgba, RichText, ScrollArea, Slider, Ui};
use egui::os::OperatingSystem;
use egui::{CentralPanel, Frame, Rect, Sense, Stroke};
use egui::{Color32, Rounding, SidePanel};
use glam::Vec2;
use rand::prelude::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TemplateApp {
    // Sim state
    sim: Sim,
    parts: Streamers,

    // Settings
    omega: f32,
    pause: bool,
    single_step: bool,
    show_settings_only: bool,
}

fn new_sim() -> Sim {
    Sim::new(60, 60)
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let sim = new_sim();
        let omega = 1.8;

        Self {
            parts: Streamers::new(5000, sim.grid()),
            sim,
            omega,
            pause: false,
            single_step: false,
            show_settings_only: false,
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        // Update
        for _ in 0..10 {
            if !self.pause || self.single_step {
                /*
                for k in 90..=110 {
                self.sim.bounds_mut()[(k, 105)] = true;
                }
                */
                //self.sim.grid_mut()[(100, 100)][(1, 1)] = -0.5;
                /*for k in 10..90 {
                self.sim.grid_mut()[(10, k)][(1, 1)] = 0.5;
                }*/
                for k in 28..=32 {
                    let point = (20, k);
                    self.sim.grid_mut()[point][(0, 1)] = 0.1;
                    //grid[point] = calc_equilibrium_predict(&grid[point]);
                    //grid[point] = force_unit_density(grid[point]);
                }

                for k in 10..50 {
                    self.sim.bounds_mut()[(40, k)] = true;
                }

                self.sim.grid_mut().data_mut()
                    .iter_mut()
                    .for_each(|cell| *cell = force_unit_density(*cell));

                bound_circle(self.sim.bounds_mut(), (20, 22), 5);
                bound_circle(self.sim.bounds_mut(), (30, 35), 5);
                bound_circle(self.sim.bounds_mut(), (40, 18), 5);

                self.sim.step(self.omega);
                self.parts.step(self.sim.grid(), self.sim.bounds());
                self.single_step = false;
            }
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
        let (rect, _response) =
            ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());

        let coords = CoordinateMapping::new(&self.sim.grid(), rect);

        let painter = ui.painter_at(rect);

        /*
        let square_size = coords.sim_to_egui_vect(Vec2::ONE);
        for j in 0..self.sim.grid().height() {
        for i in 0..self.sim.grid().width() {
        let min = Vec2::new(i as f32, j as f32);
        let min = coords.sim_to_egui(min);
        let rect = Rect::from_min_size(min, square_size.abs());
        let rect = rect.expand(1.);
        let coord = (i, j);

        let vel = 25600.0 * calc_total_avg_velocity(&self.sim.grid()[coord]);

        let color = Color32::from_rgb(vel.x.abs() as u8, vel.y.abs() as u8, 0);

        painter.rect_filled(rect, Rounding::none(), color);
        }
        }
        */

        let square_size = coords.sim_to_egui_vect(Vec2::ONE);
        for j in 0..self.sim.grid().height() {
            for i in 0..self.sim.grid().width() {
                let min = Vec2::new(i as f32, j as f32);
                let min = coords.sim_to_egui(min);
                let rect = Rect::from_min_size(min, square_size.abs());
                let rect = rect.expand(1.);
                let coord = (i, j);

                if self.sim.bounds()[coord] {
                    painter.rect_filled(rect, Rounding::none(), Color32::DARK_RED);
                }
            }
        }

        for part in &self.parts.particles {
            let pt = coords.sim_to_egui(*part);
            painter.circle_filled(pt, 1., Color32::WHITE);
        }
    }

    fn settings_gui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.pause, "Pause");
            self.single_step |= ui.button("Step").clicked();
        });
        ui.add(
            DragValue::new(&mut self.omega)
                .prefix("Omega: ")
                .clamp_range(0.0..=2.0)
                .speed(1e-2),
        );
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
        let area =
            Rect::from_min_size(area.min, egui::Vec2::splat(area.width().min(area.height())));

        Self {
            width: grid.width() as f32,
            height: grid.height() as f32,
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
            (pt.y / self.height) * self.area.height(),
        ) + self.area.left_top().to_vec2()
    }

    pub fn egui_to_sim(&self, pt: egui::Pos2) -> glam::Vec2 {
        let pt = pt - self.area.left_top().to_vec2();
        glam::Vec2::new(
            (pt.x / self.area.width()) * self.width,
            (pt.y / self.area.height()) * self.height,
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

fn bound_circle(arr: &mut Array2D<bool>, center: (i32, i32), radius: i32) {
    let (x, y) = center;

    for i in -radius..=radius {
        for j in -radius..=radius {
            if i * i + j * j < radius * radius {
                if let Some(coord) = bound_check((i + x, j + y), arr) {
                    arr[coord] = true;
                }
            }
        }
    }
}

fn bound_check<T>((x, y): (i32, i32), arr: &Array2D<T>) -> Option<(usize, usize)> {
    let in_bound = x >= 0 && y >= 0 && x < arr.width() as i32 && y < arr.height() as i32;
    in_bound.then(|| (x as usize, y as usize))
}

pub fn force_unit_density(mut cell: GridCell<f32>) -> GridCell<f32> {
    let total = calc_total_density(&cell);
    if total == 0. {
        weights()
    } else {
        cell.data_mut().iter_mut().for_each(|x| *x /= total);
        cell
    }
}

struct Streamers {
    pub particles: Vec<Vec2>,
}

impl Streamers {
    pub fn new(n: usize, grid: &Array2D<GridCell<f32>>) -> Self {
        let mut particles = vec![];
        for _ in 0..n {
            particles.push(random_particle(grid));
        }

        Self { particles }
    }

    pub fn step(&mut self, grid: &Array2D<GridCell<f32>>, bound: &Array2D<bool>) {
        *self.particles.choose_mut(&mut rand::thread_rng()).unwrap() = random_particle(grid);

        for part in &mut self.particles {
            let virt_pos = *part - Vec2::splat(0.5);

            let x = (virt_pos.x).max(0.) as usize;
            let y = (virt_pos.y).max(0.) as usize;
            if bound[(x, y)] {
                *part = random_particle(grid);
            } else {
                let tl = calc_total_avg_velocity(&grid[(x, y)]);
                let tr = calc_total_avg_velocity(&grid[(x + 1, y)]);
                let bl = calc_total_avg_velocity(&grid[(x, y + 1)]);
                let br = calc_total_avg_velocity(&grid[(x + 1, y + 1)]);
                let frac = virt_pos - virt_pos.floor();
                let top = tl.lerp(tr, frac.x);
                let bottom = bl.lerp(br, frac.x);
                let vel = top.lerp(bottom, frac.y);
                *part += vel;
            }
        }
    }
}

fn random_particle(grid: &Array2D<GridCell<f32>>) -> Vec2 {
    let mut rng = rand::thread_rng();
    Vec2::new(
        rng.gen_range(1.0..=grid.width() as f32 - 1.),
        rng.gen_range(1.0..=grid.height() as f32 - 1.),
    )
}
