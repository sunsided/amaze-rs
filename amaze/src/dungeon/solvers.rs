//! Solver adapters for dungeons.
//!
//! Provides BFS and A* solvers that work with PassabilityGrid directly,
//! adapted from the Wall4Grid-based solvers.

use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D, LinearizeCoords2D};
use crate::path::Path;
use crate::representations::PassabilityGrid;
use std::collections::VecDeque;

/// Breadth-first search solver for PassabilityGrid.
pub fn solve_bfs(grid: &PassabilityGrid, start: GridCoord2D, end: GridCoord2D) -> Option<Path> {
    let start_pos = (start.x, start.y);
    let end_pos = (end.x, end.y);

    if !grid.is_passable(start_pos.0, start_pos.1) || !grid.is_passable(end_pos.0, end_pos.1) {
        return None;
    }

    let mut queue = VecDeque::new();
    let mut seen = vec![false; grid.width() * grid.height()];
    let mut parent = vec![None; grid.width() * grid.height()];

    seen[grid.linearize_coords(start)] = true;
    queue.push_back(start_pos);

    while let Some((x, y)) = queue.pop_front() {
        if (x, y) == end_pos {
            return rebuild_path_from_coords(&parent, grid, start_pos, end_pos);
        }

        // Check 4-connected neighbors
        for (dx, dy) in [(0isize, -1isize), (1, 0), (0, 1), (-1, 0)] {
            let nx = (x as isize + dx) as usize;
            let ny = (y as isize + dy) as usize;

            if nx < grid.width() && ny < grid.height() && grid.is_passable(nx, ny) {
                let next = GridCoord2D::new(nx, ny);
                let idx = grid.linearize_coords(next);
                if !seen[idx] {
                    seen[idx] = true;
                    parent[idx] = Some((x, y));
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    None
}

/// A* search solver for PassabilityGrid using Manhattan distance heuristic.
pub fn solve_astar(grid: &PassabilityGrid, start: GridCoord2D, end: GridCoord2D) -> Option<Path> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    let start_pos = (start.x, start.y);
    let end_pos = (end.x, end.y);

    if !grid.is_passable(start_pos.0, start_pos.1) || !grid.is_passable(end_pos.0, end_pos.1) {
        return None;
    }

    let manhattan =
        |a: (usize, usize), b: (usize, usize)| -> usize { a.0.abs_diff(b.0) + a.1.abs_diff(b.1) };

    let mut heap = BinaryHeap::new();
    let mut g_score = vec![usize::MAX; grid.width() * grid.height()];
    let mut parent = vec![None; grid.width() * grid.height()];

    let start_idx = grid.linearize_coords(start);
    g_score[start_idx] = 0;
    heap.push(Reverse((manhattan(start_pos, end_pos), start_pos)));

    while let Some(Reverse((_, (x, y)))) = heap.pop() {
        if (x, y) == end_pos {
            return rebuild_path_from_coords(&parent, grid, start_pos, end_pos);
        }

        let current = GridCoord2D::new(x, y);
        let current_idx = grid.linearize_coords(current);
        let current_g = g_score[current_idx];

        // Check 4-connected neighbors
        for (dx, dy) in [(0isize, -1isize), (1, 0), (0, 1), (-1, 0)] {
            let nx = (x as isize + dx) as usize;
            let ny = (y as isize + dy) as usize;

            if nx < grid.width() && ny < grid.height() && grid.is_passable(nx, ny) {
                let next = GridCoord2D::new(nx, ny);
                let next_idx = grid.linearize_coords(next);
                let tentative_g = current_g + 1;

                if tentative_g < g_score[next_idx] {
                    g_score[next_idx] = tentative_g;
                    parent[next_idx] = Some((x, y));
                    let f = tentative_g + manhattan((nx, ny), end_pos);
                    heap.push(Reverse((f, (nx, ny))));
                }
            }
        }
    }

    None
}

fn rebuild_path_from_coords(
    parent: &[Option<(usize, usize)>],
    grid: &PassabilityGrid,
    start: (usize, usize),
    end: (usize, usize),
) -> Option<Path> {
    let mut cur = end;
    let mut out = vec![GridCoord2D::new(end.0, end.1)];

    while cur != start {
        let idx = grid.linearize_coords(GridCoord2D::new(cur.0, cur.1));
        cur = parent.get(idx).copied().flatten()?;
        out.push(GridCoord2D::new(cur.0, cur.1));
    }

    out.reverse();
    Some(Path::new(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{DungeonType, DungeonWalkGenerator};

    #[test]
    fn test_bfs_finds_path() {
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Caverns, 42);
        let dungeon = generator.generate(20, 20, 80);
        let passability = PassabilityGrid::from(&dungeon);

        let start = GridCoord2D::new(
            passability.entrance_position().0,
            passability.entrance_position().1,
        );
        let end = GridCoord2D::new(passability.exit_position().0, passability.exit_position().1);

        let path = solve_bfs(&passability, start, end);
        assert!(path.is_some());
        assert!(path.unwrap().length > 0);
    }

    #[test]
    fn test_astar_finds_path() {
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Rooms, 123);
        let dungeon = generator.generate(20, 20, 100);
        let passability = PassabilityGrid::from(&dungeon);

        let start = GridCoord2D::new(
            passability.entrance_position().0,
            passability.entrance_position().1,
        );
        let end = GridCoord2D::new(passability.exit_position().0, passability.exit_position().1);

        let path = solve_astar(&passability, start, end);
        assert!(path.is_some());
        assert!(path.unwrap().length > 0);
    }

    #[test]
    fn test_bfs_astar_parity() {
        let generator = DungeonWalkGenerator::new_from_seed(DungeonType::Winding, 456);
        let dungeon = generator.generate(25, 25, 120);
        let passability = PassabilityGrid::from(&dungeon);

        let start = GridCoord2D::new(
            passability.entrance_position().0,
            passability.entrance_position().1,
        );
        let end = GridCoord2D::new(passability.exit_position().0, passability.exit_position().1);

        let bfs_path = solve_bfs(&passability, start, end);
        let astar_path = solve_astar(&passability, start, end);

        assert!(bfs_path.is_some());
        assert!(astar_path.is_some());

        assert_eq!(
            bfs_path.unwrap().length,
            astar_path.unwrap().length,
            "BFS and A* should find paths of equal length"
        );
    }
}
