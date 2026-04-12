use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D, LinearizeCoords2D};
use crate::wall4_grid::Wall4Grid;
use std::ops::Index;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AdjacencyList {
    pub width: usize,
    pub height: usize,
    pub neighbors: Vec<Vec<GridCoord2D>>,
}

impl AdjacencyList {
    pub fn neighbors(&self, coord: GridCoord2D) -> &[GridCoord2D] {
        &self.neighbors[self.linearize_coords(coord)]
    }

    pub fn get_neighbors(&self, coord: GridCoord2D) -> Option<&[GridCoord2D]> {
        if coord.x >= self.width || coord.y >= self.height {
            return None;
        }
        Some(self.neighbors(coord))
    }
}

impl GetCoordinateBounds2D for AdjacencyList {
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }
}

impl Index<GridCoord2D> for AdjacencyList {
    type Output = [GridCoord2D];

    fn index(&self, index: GridCoord2D) -> &Self::Output {
        self.neighbors(index)
    }
}

impl From<&Wall4Grid> for AdjacencyList {
    fn from(value: &Wall4Grid) -> Self {
        let mut neighbors = vec![Vec::with_capacity(4); value.width() * value.height()];

        for cell in value.coords() {
            neighbors[value.linearize_coords(cell)] = value.open_neighbors(cell);
        }

        Self {
            width: value.width(),
            height: value.height(),
            neighbors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker4;

    #[test]
    fn adjacency_matches_grid_open_neighbors() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let adjacency = AdjacencyList::from(&maze);

        for coord in maze.coords() {
            assert_eq!(adjacency.neighbors(coord), maze.open_neighbors(coord));
        }
    }

    #[test]
    fn get_neighbors_returns_none_out_of_bounds() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let adjacency = AdjacencyList::from(&maze);

        assert!(
            adjacency
                .get_neighbors(GridCoord2D::new(adjacency.width, 0))
                .is_none()
        );
        assert!(
            adjacency
                .get_neighbors(GridCoord2D::new(0, adjacency.height))
                .is_none()
        );
    }

    #[test]
    fn index_trait_matches_neighbors_method() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let adjacency = AdjacencyList::from(&maze);

        for coord in maze.coords() {
            assert_eq!(&adjacency[coord], adjacency.neighbors(coord));
        }
    }

    #[test]
    fn neighbor_symmetry() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let adjacency = AdjacencyList::from(&maze);

        for coord in maze.coords() {
            for neighbor in adjacency.neighbors(coord) {
                assert!(adjacency.neighbors(*neighbor).contains(&coord));
            }
        }
    }

    #[test]
    fn empty_grid_produces_empty_adjacency() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(0, 0);
        let adjacency = AdjacencyList::from(&maze);

        assert_eq!(adjacency.width, 0);
        assert_eq!(adjacency.height, 0);
        assert!(adjacency.neighbors.is_empty());
    }
}
