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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::RecursiveBacktracker4;

    #[test]
    fn solvers_find_paths() {
        let maze = RecursiveBacktracker4::new_from_seed(1234).generate(20, 20);
        let start = GridCoord2D::new(0, 0);
        let end = GridCoord2D::new(19, 19);

        let bfs = BfsSolver.solve(&maze, start, end).expect("bfs path");
        let dfs = DfsSolver.solve(&maze, start, end).expect("dfs path");
        let astar = AStarSolver.solve(&maze, start, end).expect("astar path");
        let dead_end = DeadEndFillingSolver
            .solve(&maze, start, end)
            .expect("dead-end path");

        assert!(!bfs.is_empty());
        assert!(!dfs.is_empty());
        assert_eq!(bfs.length, astar.length);
        assert_eq!(bfs.length, dead_end.length);
    }
}
