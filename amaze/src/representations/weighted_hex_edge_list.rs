use crate::hex_coord::HexCoord;
use crate::wall6_grid::Wall6Grid;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HexWeightedEdge {
    pub from: HexCoord,
    pub to: HexCoord,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WeightedHexEdgeList {
    pub width: usize,
    pub height: usize,
    pub edges: Vec<HexWeightedEdge>,
}

impl WeightedHexEdgeList {
    pub fn from_wall_grid_with<F>(value: &Wall6Grid, mut weight_fn: F) -> Self
    where
        F: FnMut(HexCoord, HexCoord) -> f32,
    {
        let mut edges = Vec::new();

        for from in value.coords() {
            for to in value.open_neighbors(from) {
                if from < to {
                    edges.push(HexWeightedEdge {
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
    pub fn iter_edges(&self) -> impl Iterator<Item = &HexWeightedEdge> {
        self.edges.iter()
    }
}

impl From<&Wall6Grid> for WeightedHexEdgeList {
    fn from(value: &Wall6Grid) -> Self {
        Self::from_wall_grid_with(value, |_, _| 1.0)
    }
}

#[cfg(all(test, feature = "generator-hex-recursive-backtracker"))]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker6;
    use crate::representations::HexEdgeList;

    fn make_hex_maze() -> Wall6Grid {
        RecursiveBacktracker6::new_from_seed(42).generate(5, 5)
    }

    #[test]
    fn default_weights_are_one() {
        let list = WeightedHexEdgeList::from(&make_hex_maze());
        assert!(list.edges.iter().all(|edge| edge.weight == 1.0));
    }

    #[test]
    fn custom_weight_fn_applied() {
        let maze = make_hex_maze();
        let list = WeightedHexEdgeList::from_wall_grid_with(&maze, |from, to| {
            (from.q + from.r + to.q + to.r) as f32 + 0.5
        });

        assert!(list.edges.iter().all(|edge| {
            edge.weight == (edge.from.q + edge.from.r + edge.to.q + edge.to.r) as f32 + 0.5
        }));
    }

    #[test]
    fn edge_count_matches_unweighted() {
        let maze = make_hex_maze();
        let weighted = WeightedHexEdgeList::from(&maze);
        let unweighted = HexEdgeList::from(&maze);

        assert_eq!(weighted.edges.len(), unweighted.edges.len());
    }

    #[test]
    fn iter_edges_yields_all() {
        let maze = make_hex_maze();
        let list = WeightedHexEdgeList::from(&maze);

        let from_iter: Vec<_> = list.iter_edges().copied().collect();
        assert_eq!(from_iter, list.edges);
    }
}
