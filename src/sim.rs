use crate::array2d::{Array2D, GridPos};

#[derive(Clone, Default, Copy)]
pub struct GridCell {
    prob: [f32; 9],
}

#[derive(Clone)]
pub struct Sim {
    pub grid: Array2D<GridCell>,
}

impl Sim {
    pub fn new(width: usize, height: usize) -> Self {
        Sim {
            grid: Array2D::new(width, height),
        }
    }

    pub fn step(&mut self, omega: f32) {}
}

