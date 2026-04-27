use crate::hex_coord::HexCoord;
use crate::wall6_grid::Wall6Grid;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WeightedHexAdjacencyList {
    pub width: usize,
    pub height: usize,
    pub neighbors: Vec<Vec<(HexCoord, f32)>>,
}

impl WeightedHexAdjacencyList {
    pub fn from_wall_grid_with<F>(value: &Wall6Grid, mut weight_fn: F) -> Self
    where
        F: FnMut(HexCoord, HexCoord) -> f32,
    {
        let mut neighbors = vec![Vec::with_capacity(6); value.width() * value.height()];

        for from in value.coords() {
            let idx = (from.r as usize) * value.width() + (from.q as usize);
            let weighted_neighbors = value
                .open_neighbors(from)
                .map(|to| (to, weight_fn(from, to)))
                .collect();

            neighbors[idx] = weighted_neighbors;
        }

        Self {
            width: value.width(),
            height: value.height(),
            neighbors,
        }
    }

    pub fn neighbors(&self, coord: HexCoord) -> &[(HexCoord, f32)] {
        &self.neighbors[self.linearize_coord(coord)]
    }

    pub fn get_neighbors(&self, coord: HexCoord) -> Option<&[(HexCoord, f32)]> {
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

impl From<&Wall6Grid> for WeightedHexAdjacencyList {
    fn from(value: &Wall6Grid) -> Self {
        Self::from_wall_grid_with(value, |_, _| 1.0)
    }
}

#[cfg(all(test, feature = "generator-hex-recursive-backtracker"))]
mod tests {
    use super::*;
    use crate::generators::{MazeGenerator6D, RecursiveBacktracker6};
    use crate::representations::HexAdjacencyList;

    fn make_hex_maze() -> Wall6Grid {
        RecursiveBacktracker6::new_from_seed(42).generate(5, 5)
    }

    #[test]
    fn default_weights_are_one() {
        let list = WeightedHexAdjacencyList::from(&make_hex_maze());

        assert!(
            list.neighbors
                .iter()
                .all(|neighbors| neighbors.iter().all(|(_, weight)| *weight == 1.0))
        );
    }

    #[test]
    fn custom_weight_fn_applied() {
        let maze = make_hex_maze();
        let list = WeightedHexAdjacencyList::from_wall_grid_with(&maze, |from, to| {
            (from.q + from.r + to.q + to.r) as f32 + 0.5
        });

        for from in maze.coords() {
            for (to, weight) in list.neighbors(from) {
                assert_eq!(*weight, (from.q + from.r + to.q + to.r) as f32 + 0.5);
            }
        }
    }

    #[test]
    fn get_neighbors_returns_none_out_of_bounds() {
        let maze = make_hex_maze();
        let list = WeightedHexAdjacencyList::from(&maze);

        assert!(
            list.get_neighbors(HexCoord::new(list.width as isize, 0))
                .is_none()
        );
        assert!(
            list.get_neighbors(HexCoord::new(0, list.height as isize))
                .is_none()
        );
    }

    #[test]
    fn neighbor_count_matches_unweighted() {
        let maze = make_hex_maze();
        let weighted = WeightedHexAdjacencyList::from(&maze);
        let unweighted = HexAdjacencyList::from(&maze);

        for coord in maze.coords() {
            assert_eq!(
                weighted.neighbors(coord).len(),
                unweighted.neighbors(coord).len()
            );
        }
    }
}
