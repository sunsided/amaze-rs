use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D, LinearizeCoords2D};
use crate::wall4_grid::Wall4Grid;
use std::ops::Index;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WeightedAdjacencyList {
    pub width: usize,
    pub height: usize,
    pub neighbors: Vec<Vec<(GridCoord2D, f32)>>,
}

impl WeightedAdjacencyList {
    pub fn from_wall_grid_with<F>(value: &Wall4Grid, mut weight_fn: F) -> Self
    where
        F: FnMut(GridCoord2D, GridCoord2D) -> f32,
    {
        let mut neighbors = vec![Vec::with_capacity(4); value.width() * value.height()];

        for from in value.coords() {
            let weighted_neighbors = value
                .open_neighbors(from)
                .into_iter()
                .map(|to| (to, weight_fn(from, to)))
                .collect();

            neighbors[value.linearize_coords(from)] = weighted_neighbors;
        }

        Self {
            width: value.width(),
            height: value.height(),
            neighbors,
        }
    }

    pub fn neighbors(&self, coord: GridCoord2D) -> &[(GridCoord2D, f32)] {
        &self.neighbors[self.linearize_coords(coord)]
    }

    pub fn get_neighbors(&self, coord: GridCoord2D) -> Option<&[(GridCoord2D, f32)]> {
        if coord.x >= self.width || coord.y >= self.height {
            return None;
        }
        Some(self.neighbors(coord))
    }
}

impl GetCoordinateBounds2D for WeightedAdjacencyList {
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }
}

impl Index<GridCoord2D> for WeightedAdjacencyList {
    type Output = [(GridCoord2D, f32)];

    fn index(&self, index: GridCoord2D) -> &Self::Output {
        self.neighbors(index)
    }
}

impl From<&Wall4Grid> for WeightedAdjacencyList {
    fn from(value: &Wall4Grid) -> Self {
        Self::from_wall_grid_with(value, |_, _| 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker4;
    use crate::representations::AdjacencyList;

    #[test]
    fn default_weights_are_one() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = WeightedAdjacencyList::from(&maze);

        assert!(list
            .neighbors
            .iter()
            .all(|neighbors| neighbors.iter().all(|(_, weight)| *weight == 1.0)));
    }

    #[test]
    fn custom_weight_fn_applied() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = WeightedAdjacencyList::from_wall_grid_with(&maze, |from, to| {
            (from.x + from.y + to.x + to.y) as f32 + 0.5
        });

        for from in maze.coords() {
            for (to, weight) in list.neighbors(from) {
                assert_eq!(*weight, (from.x + from.y + to.x + to.y) as f32 + 0.5);
            }
        }
    }

    #[test]
    fn get_neighbors_returns_none_out_of_bounds() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = WeightedAdjacencyList::from(&maze);

        assert!(list
            .get_neighbors(GridCoord2D::new(list.width, 0))
            .is_none());
        assert!(list
            .get_neighbors(GridCoord2D::new(0, list.height))
            .is_none());
    }

    #[test]
    fn index_trait_matches_neighbors_method() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = WeightedAdjacencyList::from(&maze);

        for coord in maze.coords() {
            assert_eq!(&list[coord], list.neighbors(coord));
        }
    }

    #[test]
    fn neighbor_count_matches_unweighted() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let weighted = WeightedAdjacencyList::from(&maze);
        let unweighted = AdjacencyList::from(&maze);

        for coord in maze.coords() {
            assert_eq!(
                weighted.neighbors(coord).len(),
                unweighted.neighbors(coord).len()
            );
        }
    }
}
