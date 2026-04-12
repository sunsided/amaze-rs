//! Maze solving algorithms for [`Wall4Grid`].
//!
//! Solvers return a [`Path`] from start to end when one exists.

mod astar;
mod bfs;
mod dead_end;
mod dfs;

use crate::grid_coord_2d::GridCoord2D;
use crate::path::Path;
use crate::wall4_grid::Wall4Grid;

pub use astar::AStarSolver;
pub use bfs::BfsSolver;
pub use dead_end::DeadEndFillingSolver;
pub use dfs::DfsSolver;

pub trait MazeSolver {
    fn solve(&self, maze: &Wall4Grid, start: GridCoord2D, end: GridCoord2D) -> Option<Path>;
}

fn rebuild_path(
    parent: &[Option<GridCoord2D>],
    maze: &Wall4Grid,
    start: GridCoord2D,
    end: GridCoord2D,
) -> Option<Path> {
    use crate::grid_coord_2d::LinearizeCoords2D;
    let mut cur = end;
    let mut out = vec![end];

    while cur != start {
        let idx = maze.linearize_coords(cur);
        cur = parent.get(idx).copied().flatten()?;
        out.push(cur);
    }

    out.reverse();
    Some(Path::new(out))
}
