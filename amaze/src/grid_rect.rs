use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D};

/// An axis-aligned rectangle on a 2D grid, addressed by its top-left origin.
///
/// Coordinates are inclusive: a rectangle with `origin = (2, 3)`, `width = 4`,
/// `height = 2` covers `x in 2..=5` and `y in 3..=4`. Both dimensions are
/// expected to be at least `1`.
///
/// `GridRect` is the shared building block for placement-based generation
/// (rooms, corridors) and any other code that needs to reason about rectangular
/// regions of a grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GridRect {
    /// Top-left corner of the rectangle.
    pub origin: GridCoord2D,
    /// Width in cells (at least 1).
    pub width: usize,
    /// Height in cells (at least 1).
    pub height: usize,
}

impl GridRect {
    /// Create a new rectangle from its top-left origin and dimensions.
    #[inline]
    pub fn new(origin: GridCoord2D, width: usize, height: usize) -> Self {
        Self {
            origin,
            width,
            height,
        }
    }

    /// The smallest x coordinate covered by this rectangle.
    #[inline]
    pub fn left(&self) -> usize {
        self.origin.x
    }

    /// The smallest y coordinate covered by this rectangle.
    #[inline]
    pub fn top(&self) -> usize {
        self.origin.y
    }

    /// The largest x coordinate covered by this rectangle (inclusive).
    #[inline]
    pub fn right(&self) -> usize {
        self.origin.x + self.width - 1
    }

    /// The largest y coordinate covered by this rectangle (inclusive).
    #[inline]
    pub fn bottom(&self) -> usize {
        self.origin.y + self.height - 1
    }

    /// The top-left corner.
    #[inline]
    pub fn top_left(&self) -> GridCoord2D {
        self.origin
    }

    /// The bottom-right corner (inclusive).
    #[inline]
    pub fn bottom_right(&self) -> GridCoord2D {
        GridCoord2D::new(self.right(), self.bottom())
    }

    /// The (rounded-down) center cell of the rectangle.
    #[inline]
    pub fn center(&self) -> GridCoord2D {
        GridCoord2D::new(
            self.origin.x + self.width / 2,
            self.origin.y + self.height / 2,
        )
    }

    /// Returns true if the coordinate lies inside this rectangle.
    #[inline]
    pub fn contains(&self, coord: GridCoord2D) -> bool {
        coord.x >= self.left()
            && coord.x <= self.right()
            && coord.y >= self.top()
            && coord.y <= self.bottom()
    }

    /// Returns true if this rectangle fits entirely within the given bounds
    /// (i.e. `origin` is non-negative — always true for `usize` — and the
    /// far corner stays inside `width`/`height`).
    #[inline]
    pub fn fits_within(&self, bounds: &impl GetCoordinateBounds2D) -> bool {
        self.right() < bounds.width() && self.bottom() < bounds.height()
    }

    /// Returns true if the two rectangles share at least one cell.
    ///
    /// Touching edges or corners do **not** count as intersecting — use
    /// [`GridRect::collides`] for the stricter test that also rejects adjacency.
    #[inline]
    pub fn intersects(&self, other: &GridRect) -> bool {
        self.left() <= other.right()
            && other.left() <= self.right()
            && self.top() <= other.bottom()
            && other.top() <= self.bottom()
    }

    /// Returns true if the two rectangles overlap **or** touch (share an edge or
    /// corner with no gap between them).
    ///
    /// This is the test used during placement: leaving at least a one-cell gap
    /// between unrelated rooms guarantees a wall can be carved between them, so
    /// they never visually merge.
    #[inline]
    pub fn collides(&self, other: &GridRect) -> bool {
        // Separated by a gap of >= 1 cell in any direction => no collision.
        let gap = self.right() + 1 < other.left()
            || other.right() + 1 < self.left()
            || self.bottom() + 1 < other.top()
            || other.bottom() + 1 < self.top();
        !gap
    }

    /// Iterate over every cell covered by this rectangle, row by row.
    pub fn coords(&self) -> impl Iterator<Item = GridCoord2D> + '_ {
        let (left, top, right, bottom) = (self.left(), self.top(), self.right(), self.bottom());
        (top..=bottom).flat_map(move |y| (left..=right).map(move |x| GridCoord2D::new(x, y)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: usize, y: usize, w: usize, h: usize) -> GridRect {
        GridRect::new(GridCoord2D::new(x, y), w, h)
    }

    #[test]
    fn corners_and_center() {
        let r = rect(2, 3, 4, 2);
        assert_eq!(r.left(), 2);
        assert_eq!(r.top(), 3);
        assert_eq!(r.right(), 5);
        assert_eq!(r.bottom(), 4);
        assert_eq!(r.top_left(), GridCoord2D::new(2, 3));
        assert_eq!(r.bottom_right(), GridCoord2D::new(5, 4));
        assert_eq!(r.center(), GridCoord2D::new(4, 4));
    }

    #[test]
    fn contains_only_interior() {
        let r = rect(1, 1, 3, 3); // covers x,y in 1..=3
        assert!(r.contains(GridCoord2D::new(1, 1)));
        assert!(r.contains(GridCoord2D::new(3, 3)));
        assert!(!r.contains(GridCoord2D::new(0, 1)));
        assert!(!r.contains(GridCoord2D::new(4, 3)));
    }

    #[test]
    fn intersects_overlap_touch_disjoint() {
        let a = rect(0, 0, 3, 3); // x,y in 0..=2
        // Overlapping
        assert!(a.intersects(&rect(2, 2, 3, 3)));
        // Touching edge (a.right = 2, other.left = 3): not an intersection
        assert!(!a.intersects(&rect(3, 0, 2, 3)));
        // Disjoint
        assert!(!a.intersects(&rect(5, 5, 2, 2)));
    }

    #[test]
    fn collides_treats_touching_as_collision() {
        let a = rect(0, 0, 3, 3); // x,y in 0..=2
        // Overlapping
        assert!(a.collides(&rect(2, 2, 3, 3)));
        // Edge-adjacent (gap 0)
        assert!(a.collides(&rect(3, 0, 2, 3)));
        // Corner-adjacent (gap 0)
        assert!(a.collides(&rect(3, 3, 2, 2)));
        // Separated by a one-cell gap
        assert!(!a.collides(&rect(4, 0, 2, 3)));
    }

    #[test]
    fn fits_within_bounds() {
        struct Bounds(usize, usize);
        impl GetCoordinateBounds2D for Bounds {
            fn width(&self) -> usize {
                self.0
            }
            fn height(&self) -> usize {
                self.1
            }
        }
        let bounds = Bounds(10, 10);
        assert!(rect(0, 0, 10, 10).fits_within(&bounds));
        assert!(!rect(0, 0, 11, 5).fits_within(&bounds));
        assert!(!rect(8, 8, 3, 3).fits_within(&bounds));
    }

    #[test]
    fn coords_visits_every_cell() {
        let r = rect(1, 1, 2, 3); // 2 wide, 3 tall = 6 cells
        let cells: Vec<_> = r.coords().collect();
        assert_eq!(cells.len(), 6);
        assert_eq!(cells[0], GridCoord2D::new(1, 1));
        assert_eq!(cells[5], GridCoord2D::new(2, 3));
        // Every cell is contained by the rect
        assert!(cells.iter().all(|&c| r.contains(c)));
    }
}
