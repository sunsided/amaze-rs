use crate::hex_coord::HexCoord;
use crate::wall6_grid::Wall6Grid;

#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HexAdjacencyList {
    pub width: usize,
    pub height: usize,
    pub neighbors: Vec<Vec<HexCoord>>,
}

impl HexAdjacencyList {
    pub fn neighbors(&self, coord: HexCoord) -> &[HexCoord] {
        &self.neighbors[self.linearize_coord(coord)]
    }

    pub fn get_neighbors(&self, coord: HexCoord) -> Option<&[HexCoord]> {
        if coord.q < 0
            || coord.q >= self.width as isize
            || coord.r < 0
            || coord.r >= self.height as isize
        {
            return None;
        }
        Some(self.neighbors(coord))
    }

    #[inline]
    fn linearize_coord(&self, coord: HexCoord) -> usize {
        (coord.r as usize) * self.width + (coord.q as usize)
    }
}

impl From<&Wall6Grid> for HexAdjacencyList {
    fn from(value: &Wall6Grid) -> Self {
        let mut neighbors = vec![Vec::with_capacity(6); value.width() * value.height()];

        for cell in value.coords() {
            let idx = (cell.r as usize) * value.width() + (cell.q as usize);
            neighbors[idx] = value.open_neighbors(cell).collect();
        }

        Self {
            width: value.width(),
            height: value.height(),
            neighbors,
        }
    }
}

#[cfg(all(test, feature = "generator-hex-recursive-backtracker"))]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker6;

    fn make_hex_maze() -> Wall6Grid {
        RecursiveBacktracker6::new_from_seed(42).generate(5, 5)
    }

    #[test]
    fn adjacency_matches_grid_open_neighbors() {
        let maze = make_hex_maze();
        let adjacency = HexAdjacencyList::from(&maze);

        for coord in maze.coords() {
            let expected: Vec<_> = maze.open_neighbors(coord).collect();
            assert_eq!(adjacency.neighbors(coord), expected);
        }
    }

    #[test]
    fn get_neighbors_returns_none_out_of_bounds() {
        let maze = make_hex_maze();
        let adjacency = HexAdjacencyList::from(&maze);

        assert!(
            adjacency
                .get_neighbors(HexCoord::new(adjacency.width as isize, 0))
                .is_none()
        );
        assert!(
            adjacency
                .get_neighbors(HexCoord::new(0, adjacency.height as isize))
                .is_none()
        );
        assert!(adjacency.get_neighbors(HexCoord::new(-1, 0)).is_none());
    }

    #[test]
    fn neighbor_symmetry() {
        let maze = make_hex_maze();
        let adjacency = HexAdjacencyList::from(&maze);

        for coord in maze.coords() {
            for neighbor in adjacency.neighbors(coord) {
                assert!(adjacency.neighbors(*neighbor).contains(&coord));
            }
        }
    }

    #[test]
    fn empty_grid_produces_empty_adjacency() {
        let maze = RecursiveBacktracker6::new_from_seed(42).generate(0, 0);
        let adjacency = HexAdjacencyList::from(&maze);

        assert_eq!(adjacency.width, 0);
        assert_eq!(adjacency.height, 0);
        assert!(adjacency.neighbors.is_empty());
    }
}
