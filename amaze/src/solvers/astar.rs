use crate::grid_coord_2d::{GridCoord2D, LinearizeCoords2D};
use crate::solvers::{MazeSolver, rebuild_path};
use crate::wall4_grid::Wall4Grid;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Copy, Clone, Default)]
pub struct AStarSolver;

fn heuristic(a: GridCoord2D, b: GridCoord2D) -> usize {
    a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
}

impl MazeSolver for AStarSolver {
    fn solve(
        &self,
        maze: &Wall4Grid,
        start: GridCoord2D,
        end: GridCoord2D,
    ) -> Option<crate::path::Path> {
        if maze.get(start).is_none() || maze.get(end).is_none() {
            return None;
        }

        let total = maze.width() * maze.height();
        let mut g = vec![usize::MAX; total];
        let mut parent = vec![None; total];
        let mut open = BinaryHeap::new();

        let sidx = maze.linearize_coords(start);
        g[sidx] = 0;
        open.push((Reverse(heuristic(start, end)), start));

        while let Some((_, cell)) = open.pop() {
            if cell == end {
                return rebuild_path(&parent, maze, start, end);
            }

            let cell_cost = g[maze.linearize_coords(cell)];
            for next in maze.open_neighbors(cell) {
                let nidx = maze.linearize_coords(next);
                let tentative = cell_cost + 1;
                if tentative < g[nidx] {
                    g[nidx] = tentative;
                    parent[nidx] = Some(cell);
                    let f = tentative + heuristic(next, end);
                    open.push((Reverse(f), next));
                }
            }
        }

        None
    }
}
