use crate::grid_coord_2d::{GridCoord2D, LinearizeCoords2D};
use crate::wall4_grid::Wall4Grid;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MazeStats {
    pub dead_ends: usize,
    pub corridors: usize,
    pub junctions: usize,
    pub longest_path: usize,
    pub average_path_length: f64,
}

impl MazeStats {
    pub fn from_grid(grid: &Wall4Grid) -> Self {
        let mut dead_ends = 0usize;
        let mut corridors = 0usize;
        let mut junctions = 0usize;

        for cell in grid.coords() {
            let degree = grid.open_neighbors(cell).count();
            match degree {
                0 | 1 => dead_ends += 1,
                2 => corridors += 1,
                _ => junctions += 1,
            }
        }

        let avg = average_shortest_path_length(grid);
        let longest_path = maze_diameter(grid);

        Self {
            dead_ends,
            corridors,
            junctions,
            longest_path,
            average_path_length: avg,
        }
    }
}

fn maze_diameter(grid: &Wall4Grid) -> usize {
    if grid.width() == 0 || grid.height() == 0 {
        return 0;
    }

    let start = GridCoord2D::default();
    let first = farthest_from(grid, start).0;
    farthest_from(grid, first).1
}

fn farthest_from(grid: &Wall4Grid, start: GridCoord2D) -> (GridCoord2D, usize) {
    let dist = grid.bfs_distances(start);
    let mut farthest = (start, 0usize);

    for cell in grid.coords() {
        if let Some(d) = dist[grid.linearize_coords(cell)] {
            if d > farthest.1 {
                farthest = (cell, d);
            }
        }
    }

    farthest
}

fn average_shortest_path_length(grid: &Wall4Grid) -> f64 {
    if grid.width() == 0 || grid.height() == 0 {
        return 0.0;
    }

    let cells: Vec<_> = grid.coords().collect();
    let mut sum = 0usize;
    let mut count = 0usize;

    for (i, start) in cells.iter().copied().enumerate() {
        let dist = grid.bfs_distances(start);
        for end in cells.iter().copied().skip(i + 1) {
            if let Some(d) = dist[grid.linearize_coords(end)] {
                sum += d;
                count += 1;
            }
        }
    }

    if count == 0 {
        0.0
    } else {
        sum as f64 / count as f64
    }
}
