mod recursive_backtracker4;

use crate::wall4_grid::Wall4Grid;
pub use recursive_backtracker4::RecursiveBacktracker4;

pub trait MazeGenerator2D {
    /// Generates a new maze generator initialized by a random seed.
    fn new_random() -> Self;
    /// Generates a new maze generator initialized by the specified seed value.
    fn new_from_seed(rng_seed: u64) -> Self;
    /// Generates a new grid of the dimensions `width` by `height`.
    fn generate(&self, width: usize, height: usize) -> Wall4Grid;
}
