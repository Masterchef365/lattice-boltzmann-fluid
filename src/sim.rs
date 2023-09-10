use glam::Vec2;

use crate::{
    array2d::{Array2D, GridPos},
    cell::GridCell,
};

#[derive(Clone)]
pub struct Sim {
    read: Array2D<GridCell<f32>>,
    write: Array2D<GridCell<f32>>,
    bounds: Array2D<bool>,
}

impl Sim {
    pub fn new(width: usize, height: usize) -> Self {
        let read = Array2D::new(width, height);

        Sim {
            write: read.clone(),
            read,
            bounds: default_bounds(width, height),
        }
    }

    pub fn step(&mut self, omega: f32) {
        // Collision step
        for y in 0..self.read.height() {
            for x in 0..self.read.width() {
                let coord = (x, y);
                let eq = calc_equilibrium_predict(&self.read[coord]);
                self.read[coord]
                    .data_mut()
                    .iter_mut()
                    .zip(eq.data())
                    .for_each(|(read, eq)| {
                        *read = *read + omega * (*eq - *read);
                    });
            }
        }

        // Clear the write buffer
        self.write.data_mut().fill(GridCell::default());

        // Streaming step
        for y in 1..self.read.height() - 1 {
            for x in 1..self.read.width() - 1 {
                for uv in velocities_i32().data() {
                    let adj_coord = relative_index((x, y), *uv);
                    let inner_idx = relative_index((1, 1), *uv);
                    if self.bounds[adj_coord] {
                        // Boundary has been hit, reverse direction and stay within the same cell
                        let (u, v) = uv;
                        let rev_idx = relative_index((1, 1), (-u, -v));
                        self.write[(x, y)][rev_idx] += self.read[(x, y)][inner_idx];
                    } else {
                        // No boundary, use fluid transport
                        self.write[adj_coord][inner_idx] += self.read[(x, y)][inner_idx];
                    }
                }
            }
        }

        std::mem::swap(&mut self.read, &mut self.write);

        println!("########################################");
        for y in 0..self.read.height() {
            for x in 0..self.read.width() {
                let vel = calc_total_avg_velocity(&self.read[(x, y)]);
                println!("({x}, {y}): {vel}");
            }
        }
    }

    pub fn grid(&self) -> &Array2D<GridCell<f32>> {
        &self.read
    }

    pub fn grid_mut(&mut self) -> &mut Array2D<GridCell<f32>> {
        // This is counterintuitive, but the READ buffer will be read by the next sim step in order
        // to write to the WRITE buffer. So we want to modify which thing will be read.
        &mut self.read
    }

    pub fn bounds_mut(&mut self) -> &mut Array2D<bool> {
        &mut self.bounds
    }
}

fn relative_index(xy: (usize, usize), uv: (i32, i32)) -> (usize, usize) {
    let (x, y) = xy;
    let (u, v) = uv;
    (
        (x as i32 + u) as usize,
        (y as i32 + v) as usize,
    )
}

/// D2Q9 weights
fn weights() -> GridCell<f32> {
    let diag = 1. / 36.;
    let adj = 1. / 9.;
    let center = 4. / 9.;
    GridCell::from([
        diag, adj, diag, //.
        adj, center, adj, //.
        diag, adj, diag, //.
    ])
}

fn velocities_i32() -> GridCell<(i32, i32)> {
    let mut cell = GridCell::default();
    for j in 0..3 {
        for i in 0..3 {
            cell[(i, j)] = (i as i32 - 1, j as i32 - 1);
        }
    }
    cell
}

fn velocities() -> GridCell<Vec2> {
    let mut cell = GridCell::default();
    cell.data_mut()
        .iter_mut()
        .zip(velocities_i32().data())
        .for_each(|(out, &(vx, vy))| *out = Vec2::new(vx as f32, vy as f32));
    cell
}

fn calc_total_density(cell: &GridCell<f32>) -> f32 {
    cell.data()
        .iter()
        .zip(weights().data())
        .map(|(prob, weight)| prob * weight)
        .sum()
}

pub fn calc_total_avg_velocity(cell: &GridCell<f32>) -> Vec2 {
    cell.data()
        .iter()
        .zip(velocities().data())
        .map(|(vel, weight)| *weight * *vel)
        .sum()
}

fn calc_equilibrium_predict(cell: &GridCell<f32>) -> GridCell<f32> {
    let total_vel = calc_total_avg_velocity(cell);
    let total_vel_sq = total_vel.length_squared();
    let total_density = calc_total_density(cell);
    let mut output = GridCell::default();
    output
        .data_mut()
        .iter_mut()
        .zip(velocities().data())
        .zip(weights().data())
        .for_each(|((out, vel_vect), weight)| {
            let dot = vel_vect.dot(total_vel);
            *out = weight * total_density * (1. + 3. * dot + 4.5 * dot.powi(2) - 1.5 * total_vel_sq)
        });
    output
}

fn default_bounds(width: usize, height: usize) -> Array2D<bool> {
    let mut bounds = Array2D::new(width, height);

    for i in 0..width {
        bounds[(i, 0)] = true;
        bounds[(i, height - 1)] = true;
    }

    for i in 0..height {
        bounds[(0, i)] = true;
        bounds[(width - 1, i)] = true;
    }

    bounds
}
