use crate::grid_coord_2d::GridCoord2D;
use crate::wall4_grid::Wall4Grid;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WeightedEdge {
    pub from: GridCoord2D,
    pub to: GridCoord2D,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WeightedEdgeList {
    pub width: usize,
    pub height: usize,
    pub edges: Vec<WeightedEdge>,
}

impl WeightedEdgeList {
    pub fn from_wall_grid_with<F>(value: &Wall4Grid, mut weight_fn: F) -> Self
    where
        F: FnMut(GridCoord2D, GridCoord2D) -> f32,
    {
        let mut edges = Vec::new();

        for from in value.coords() {
            for to in value.open_neighbors(from) {
                if from < to {
                    edges.push(WeightedEdge {
                        from,
                        to,
                        weight: weight_fn(from, to),
                    });
                }
            }
        }

        Self {
            width: value.width(),
            height: value.height(),
            edges,
        }
    }

    #[inline]
    pub fn iter_edges(&self) -> impl Iterator<Item = &WeightedEdge> {
        self.edges.iter()
    }
}

impl From<&Wall4Grid> for WeightedEdgeList {
    fn from(value: &Wall4Grid) -> Self {
        Self::from_wall_grid_with(value, |_, _| 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker4;

    #[test]
    fn default_weights_are_one() {
        let maze = RecursiveBacktracker4::new_from_seed(7).generate(8, 8);
        let list = WeightedEdgeList::from(&maze);
        assert!(list.edges.iter().all(|edge| edge.weight == 1.0));
    }
}
