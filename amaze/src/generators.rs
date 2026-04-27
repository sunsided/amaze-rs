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
mod prim4;
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
pub use prim4::Prim4;
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
