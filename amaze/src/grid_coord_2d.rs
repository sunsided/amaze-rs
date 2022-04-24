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
        match self.y.checked_sub(1) {
            Some(y) => Some(Self::new(self.x, y)),
            None => None,
        }
    }

    #[inline]
    pub fn down(&self) -> Option<Self> {
        match self.y.checked_add(1) {
            Some(y) => Some(Self::new(self.x, y)),
            None => None,
        }
    }

    #[inline]
    pub fn left(&self) -> Option<Self> {
        match self.x.checked_sub(1) {
            Some(x) => Some(Self::new(x, self.y)),
            None => None,
        }
    }

    #[inline]
    pub fn right(&self) -> Option<Self> {
        match self.x.checked_add(1) {
            Some(x) => Some(Self::new(x, self.y)),
            None => None,
        }
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

        let coords = coords.x * width + coords.y;
        debug_assert!(coords < width * height);
        coords
    }
}
