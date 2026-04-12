//! Maze generation algorithms for rectangular 4-connected grids.
//!
//! Algorithms are organized around the [`MazeGenerator2D`] trait, which supports
//! full generation and step-by-step event streams for animation.

mod binary_tree4;
mod cell_selector;
mod eller4;
mod growing_tree4;
mod helpers;
mod hunt_and_kill4;
mod kruskal4;
mod recursive_backtracker4;
mod sidewinder4;
mod wilson4;

use crate::grid_coord_2d::GridCoord2D;
use crate::wall4_grid::Wall4Grid;

pub use binary_tree4::BinaryTree4;
pub use cell_selector::{CellSelector, MixedCell, NewestCell, OldestCell, RandomCell};
pub use eller4::Eller4;
pub use growing_tree4::GrowingTree4;
pub use hunt_and_kill4::HuntAndKill4;
pub use kruskal4::Kruskal4;
pub use recursive_backtracker4::RecursiveBacktracker4;
pub use sidewinder4::Sidewinder4;
pub use wilson4::Wilson4;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GenerationStep {
    Visit { cell: GridCoord2D },
    Carve { from: GridCoord2D, to: GridCoord2D },
    Backtrack { to: GridCoord2D },
    AddToFrontier { cell: GridCoord2D },
    Complete,
}

pub trait GenerationVisitor {
    fn on_step(&mut self, step: &GenerationStep);
}

#[derive(Default)]
pub struct VecGenerationVisitor {
    steps: Vec<GenerationStep>,
}

impl VecGenerationVisitor {
    pub fn into_steps(self) -> Vec<GenerationStep> {
        self.steps
    }
}

impl GenerationVisitor for VecGenerationVisitor {
    fn on_step(&mut self, step: &GenerationStep) {
        self.steps.push(step.clone());
    }
}

pub struct GenerationSteps {
    inner: std::vec::IntoIter<GenerationStep>,
}

impl GenerationSteps {
    pub fn new(steps: Vec<GenerationStep>) -> Self {
        Self {
            inner: steps.into_iter(),
        }
    }
}

impl Iterator for GenerationSteps {
    type Item = GenerationStep;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub trait MazeGenerator2D {
    fn new_random() -> Self
    where
        Self: Sized;
    fn new_from_seed(rng_seed: u64) -> Self
    where
        Self: Sized;
    fn generate(&self, width: usize, height: usize) -> Wall4Grid;

    fn generate_steps(&self, width: usize, height: usize) -> GenerationSteps {
        let _ = self.generate(width, height);
        GenerationSteps::new(vec![GenerationStep::Complete])
    }

    fn name(&self) -> &'static str {
        "unknown"
    }

    fn description(&self) -> &'static str {
        "maze generator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid_coord_2d::GridCoord2D;
    use std::collections::VecDeque;

    fn assert_connected_tree(grid: &Wall4Grid) {
        let width = grid.width();
        let height = grid.height();
        let total = width * height;
        let mut seen = vec![false; total];
        let mut q = VecDeque::new();
        q.push_back(GridCoord2D::new(0, 0));
        seen[0] = true;
        let mut edge_count = 0usize;

        while let Some(cell) = q.pop_front() {
            let neighbors = grid.open_neighbors(cell);
            edge_count += neighbors.len();
            for n in neighbors {
                let idx = n.y * width + n.x;
                if !seen[idx] {
                    seen[idx] = true;
                    q.push_back(n);
                }
            }
        }

        assert_eq!(seen.into_iter().filter(|v| *v).count(), total);
        assert_eq!(edge_count / 2, total.saturating_sub(1));
    }

    #[test]
    fn generators_produce_spanning_trees() {
        let size = (12, 12);
        let mut grids = Vec::new();
        grids.push(RecursiveBacktracker4::new_from_seed(42).generate(size.0, size.1));
        grids.push(<GrowingTree4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
        grids.push(<Kruskal4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
        grids.push(<Eller4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
        grids.push(<Wilson4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
        grids.push(<HuntAndKill4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
        grids.push(<Sidewinder4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));
        grids.push(<BinaryTree4 as MazeGenerator2D>::new_from_seed(42).generate(size.0, size.1));

        for grid in grids {
            assert_connected_tree(&grid);
        }
    }
}
