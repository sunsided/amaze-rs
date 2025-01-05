use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D, LinearizeCoords2D};
use crate::room4::Wall4;
use std::ops::{Index, IndexMut};

#[derive(Debug, Default)]
pub struct Wall4Grid {
    width: usize,
    height: usize,
    walls: Vec<Wall4>,
}

impl Wall4Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            walls: vec![Wall4::ALL; width * height],
        }
    }

    pub fn get(&self, coords: GridCoord2D) -> Option<&Wall4> {
        if coords.x >= self.width || coords.y >= self.height {
            return None;
        }
        Some(&self[coords])
    }

    pub fn get_mut(&mut self, coords: GridCoord2D) -> Option<&mut Wall4> {
        if coords.x >= self.width || coords.y >= self.height {
            return None;
        }
        Some(&mut self[coords])
    }

    pub fn remove_wall_between(&mut self, current: GridCoord2D, selected: GridCoord2D) {
        let c = self.linearize_coords(current);
        let s = self.linearize_coords(selected);

        if current.x > selected.x {
            self.walls[s] -= Wall4::EAST;
            self.walls[c] -= Wall4::WEST;
        } else if current.x < selected.x {
            self.walls[s] -= Wall4::WEST;
            self.walls[c] -= Wall4::EAST;
        }

        if current.y > selected.y {
            self.walls[s] -= Wall4::SOUTH;
            self.walls[c] -= Wall4::NORTH;
        } else if current.y < selected.y {
            self.walls[s] -= Wall4::NORTH;
            self.walls[c] -= Wall4::SOUTH;
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl GetCoordinateBounds2D for Wall4Grid {
    #[inline]
    fn width(&self) -> usize {
        self.width()
    }

    #[inline]
    fn height(&self) -> usize {
        self.height()
    }
}

impl Index<GridCoord2D> for Wall4Grid {
    type Output = Wall4;

    fn index(&self, index: GridCoord2D) -> &Self::Output {
        let index = self.linearize_coords(index);
        &self.walls[index]
    }
}

impl IndexMut<GridCoord2D> for Wall4Grid {
    fn index_mut(&mut self, index: GridCoord2D) -> &mut Self::Output {
        let index = self.linearize_coords(index);
        &mut self.walls[index]
    }
}
