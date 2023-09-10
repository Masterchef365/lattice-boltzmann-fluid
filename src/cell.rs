use crate::array2d::GridPos;

const CELL_SIZE: usize = 3;

#[derive(Clone, Debug, Default, Copy)]
pub struct GridCell<T> {
    data: [T; CELL_SIZE * CELL_SIZE],
}

impl<T> GridCell<T> {
    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    fn calc_index(&self, (x, y): GridPos) -> usize {
        debug_assert!(x < CELL_SIZE);
        debug_assert!(y < CELL_SIZE);
        x + y * CELL_SIZE
    }

    pub fn width(&self) -> usize {
        CELL_SIZE
    }

    pub fn height(&self) -> usize {
        CELL_SIZE
    }
}

impl<T> std::ops::Index<GridPos> for GridCell<T> {
    type Output = T;
    fn index(&self, pos: GridPos) -> &T {
        &self.data[self.calc_index(pos)]
    }
}

impl<T> std::ops::IndexMut<GridPos> for GridCell<T> {
    fn index_mut(&mut self, pos: GridPos) -> &mut T {
        let idx = self.calc_index(pos);
        &mut self.data[idx]
    }
}

impl<T> From<[T; CELL_SIZE*CELL_SIZE]> for GridCell<T> {
    fn from(value: [T; CELL_SIZE*CELL_SIZE]) -> Self {
        Self { data: value }
    }
}
