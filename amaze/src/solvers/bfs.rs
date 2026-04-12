use crate::grid_coord_2d::{GridCoord2D, LinearizeCoords2D};
use crate::solvers::{rebuild_path, MazeSolver};
use crate::wall4_grid::Wall4Grid;
use std::collections::VecDeque;

#[derive(Debug, Copy, Clone, Default)]
pub struct BfsSolver;

impl MazeSolver for BfsSolver {
    fn solve(
        &self,
        maze: &Wall4Grid,
        start: GridCoord2D,
        end: GridCoord2D,
    ) -> Option<crate::path::Path> {
        if maze.get(start).is_none() || maze.get(end).is_none() {
            return None;
        }

        let mut queue = VecDeque::new();
        let mut seen = vec![false; maze.width() * maze.height()];
        let mut parent = vec![None; maze.width() * maze.height()];

        seen[maze.linearize_coords(start)] = true;
        queue.push_back(start);

        while let Some(cell) = queue.pop_front() {
            if cell == end {
                return rebuild_path(&parent, maze, start, end);
            }

            for next in maze.open_neighbors(cell) {
                let idx = maze.linearize_coords(next);
                if !seen[idx] {
                    seen[idx] = true;
                    parent[idx] = Some(cell);
                    queue.push_back(next);
                }
            }
        }

        None
    }
}
