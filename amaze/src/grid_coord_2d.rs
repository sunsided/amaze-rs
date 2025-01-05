use std::ops::{Add, Sub};

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
pub struct GridCoord2D {
    pub x: usize,
    pub y: usize,
}

impl GridCoord2D {
    #[inline]
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn up(&self) -> Option<Self> {
        self.y.checked_sub(1).map(|y| Self::new(self.x, y))
    }

    #[inline]
    pub fn down(&self) -> Option<Self> {
        self.y.checked_add(1).map(|y| Self::new(self.x, y))
    }

    #[inline]
    pub fn left(&self) -> Option<Self> {
        self.x.checked_sub(1).map(|x| Self::new(x, self.y))
    }

    #[inline]
    pub fn right(&self) -> Option<Self> {
        self.x.checked_add(1).map(|x| Self::new(x, self.y))
    }
}

impl Add<GridCoord2D> for GridCoord2D {
    type Output = Self;

    fn add(self, rhs: GridCoord2D) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<GridCoord2D> for GridCoord2D {
    type Output = Self;

    fn sub(self, rhs: GridCoord2D) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

pub trait LinearizeCoords2D {
    fn linearize_coords(&self, coords: GridCoord2D) -> usize;
}

pub trait GetCoordinateBounds2D {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

impl<T> LinearizeCoords2D for T
where
    T: GetCoordinateBounds2D,
{
    #[inline]
    fn linearize_coords(&self, coords: GridCoord2D) -> usize {
        let width = self.width();
        let height = self.height();

        let coords = coords.y * width + coords.x;
        assert!(coords < width * height, "Linear index out of bounds");
        coords
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wall4_grid::Wall4Grid;

    #[test]
    fn linearize_coords_correctly() {
        let grid = Wall4Grid::new(4, 4); // 4x4 grid
        assert_eq!(grid.linearize_coords(GridCoord2D::new(0, 0)), 0);
        assert_eq!(grid.linearize_coords(GridCoord2D::new(1, 0)), 1);
        assert_eq!(grid.linearize_coords(GridCoord2D::new(3, 0)), 3);
        assert_eq!(grid.linearize_coords(GridCoord2D::new(0, 1)), 4);
        assert_eq!(grid.linearize_coords(GridCoord2D::new(2, 2)), 10);
        assert_eq!(grid.linearize_coords(GridCoord2D::new(3, 3)), 15);
    }

    #[test]
    #[should_panic(expected = "Linear index out of bounds")]
    fn linearize_coords_out_of_bounds_panics() {
        let grid = Wall4Grid::new(3, 3); // 3x3 grid
                                         // This should panic as (3, 3) is out of bounds for 0-based indexing
        grid.linearize_coords(GridCoord2D::new(3, 3));
    }
}
