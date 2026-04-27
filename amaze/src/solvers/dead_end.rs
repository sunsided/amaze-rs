use crate::grid_coord_2d::{GridCoord2D, LinearizeCoords2D};
use crate::path::Path;
use crate::solvers::MazeSolver;
use crate::wall4_grid::Wall4Grid;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Copy, Clone, Default)]
pub struct DeadEndFillingSolver;

impl MazeSolver for DeadEndFillingSolver {
    fn solve(&self, maze: &Wall4Grid, start: GridCoord2D, end: GridCoord2D) -> Option<Path> {
        if maze.get(start).is_none() || maze.get(end).is_none() {
            return None;
        }

        let mut degree = vec![0usize; maze.width() * maze.height()];
        for cell in maze.coords() {
            degree[maze.linearize_coords(cell)] = maze.open_neighbors(cell).count();
        }

        let mut removed: HashSet<GridCoord2D> = HashSet::new();
        let mut q = VecDeque::new();
        for cell in maze.coords() {
            let idx = maze.linearize_coords(cell);
            if cell != start && cell != end && degree[idx] <= 1 {
                q.push_back(cell);
            }
        }

        while let Some(cell) = q.pop_front() {
            if !removed.insert(cell) {
                continue;
            }

            for n in maze.open_neighbors(cell) {
                if removed.contains(&n) {
                    continue;
                }
                let idx = maze.linearize_coords(n);
                if degree[idx] > 0 {
                    degree[idx] -= 1;
                    if n != start && n != end && degree[idx] <= 1 {
                        q.push_back(n);
                    }
                }
            }
        }

        let mut current = start;
        let mut path = vec![start];
        let mut prev: Option<GridCoord2D> = None;

        while current != end {
            let mut next_options = maze
                .open_neighbors(current)
                .filter(|n| Some(*n) != prev)
                .filter(|n| !removed.contains(n) || *n == end)
                .collect::<Vec<_>>();

            if next_options.is_empty() {
                return None;
            }

            let next = next_options.swap_remove(0);
            path.push(next);
            prev = Some(current);
            current = next;
        }

        Some(Path::new(path))
    }
}
