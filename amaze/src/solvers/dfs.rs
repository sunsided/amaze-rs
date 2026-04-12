use crate::grid_coord_2d::{GridCoord2D, LinearizeCoords2D};
use crate::solvers::{MazeSolver, rebuild_path};
use crate::wall4_grid::Wall4Grid;

#[derive(Debug, Copy, Clone, Default)]
pub struct DfsSolver;

impl MazeSolver for DfsSolver {
    fn solve(
        &self,
        maze: &Wall4Grid,
        start: GridCoord2D,
        end: GridCoord2D,
    ) -> Option<crate::path::Path> {
        if maze.get(start).is_none() || maze.get(end).is_none() {
            return None;
        }

        let mut stack = vec![start];
        let mut seen = vec![false; maze.width() * maze.height()];
        let mut parent = vec![None; maze.width() * maze.height()];
        seen[maze.linearize_coords(start)] = true;

        while let Some(cell) = stack.pop() {
            if cell == end {
                return rebuild_path(&parent, maze, start, end);
            }

            for next in maze.open_neighbors(cell) {
                let idx = maze.linearize_coords(next);
                if !seen[idx] {
                    seen[idx] = true;
                    parent[idx] = Some(cell);
                    stack.push(next);
                }
            }
        }

        None
    }
}
