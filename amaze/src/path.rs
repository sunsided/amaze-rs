use crate::grid_coord_2d::GridCoord2D;

/// A solution path through a maze.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Path {
    cells: Vec<GridCoord2D>,
    pub length: usize,
}

impl Path {
    pub fn new(cells: Vec<GridCoord2D>) -> Self {
        let length = cells.len();
        Self { cells, length }
    }

    pub fn cells(&self) -> &[GridCoord2D] {
        &self.cells
    }

    pub fn start(&self) -> Option<GridCoord2D> {
        self.cells.first().copied()
    }

    pub fn end(&self) -> Option<GridCoord2D> {
        self.cells.last().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl IntoIterator for Path {
    type Item = GridCoord2D;
    type IntoIter = std::vec::IntoIter<GridCoord2D>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

impl<'a> IntoIterator for &'a Path {
    type Item = &'a GridCoord2D;
    type IntoIter = std::slice::Iter<'a, GridCoord2D>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}
