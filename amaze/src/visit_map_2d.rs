use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D, LinearizeCoords2D};
use crate::room4::Wall4;
use std::borrow::Borrow;
use std::ops::{Index, IndexMut};

#[derive(Debug, Default)]
pub struct VisitMap2D {
    width: usize,
    height: usize,
    cells: Vec<bool>,
}

impl VisitMap2D {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![false; width * height],
        }
    }

    pub fn new_like<T>(other: &T) -> Self
    where
        T: GetCoordinateBounds2D,
    {
        Self::new(other.width(), other.height())
    }

    pub fn get(&self, coords: GridCoord2D) -> Option<&bool> {
        if coords.x >= self.width || coords.y >= self.height {
            return None;
        }
        return Some(&self[coords]);
    }

    pub fn get_mut(&mut self, coords: GridCoord2D) -> Option<&mut bool> {
        if coords.x >= self.width || coords.y >= self.height {
            return None;
        }
        return Some(&mut self[coords]);
    }

    pub fn unvisited_neighbors(&self, coords: GridCoord2D) -> Vec<GridCoord2D> {
        let mut vec = Vec::with_capacity(4);
        if coords.x >= self.width || coords.y >= self.height {
            return vec;
        }

        self.push_if_unvisited(&mut vec, coords.up());
        self.push_if_unvisited(&mut vec, coords.right());
        self.push_if_unvisited(&mut vec, coords.down());
        self.push_if_unvisited(&mut vec, coords.left());
        vec
    }

    fn push_if_unvisited(&self, vec: &mut Vec<GridCoord2D>, coord: Option<GridCoord2D>) {
        if let Some(coord) = self.is_unvisited(coord) {
            vec.push(coord);
        }
    }

    fn is_unvisited(&self, coord: Option<GridCoord2D>) -> Option<GridCoord2D> {
        match coord {
            None => None,
            Some(value) => match self.get(value) {
                None => None,
                Some(true) => None,
                Some(false) => Some(value),
            },
        }
    }
}

impl GetCoordinateBounds2D for VisitMap2D {
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }
}

impl Index<GridCoord2D> for VisitMap2D {
    type Output = bool;

    fn index(&self, index: GridCoord2D) -> &Self::Output {
        let index = self.linearize_coords(index);
        &self.cells[index]
    }
}

impl IndexMut<GridCoord2D> for VisitMap2D {
    fn index_mut(&mut self, index: GridCoord2D) -> &mut Self::Output {
        let index = self.linearize_coords(index);
        &mut self.cells[index]
    }
}
