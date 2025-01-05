use crate::generators::MazeGenerator2D;
use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D};
use crate::visit_map_2d::VisitMap2D;
use crate::wall4_grid::Wall4Grid;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

/// A maze generator that implements the Recursive Backtracking algorithm.
pub struct RecursiveBacktracker4 {
    rng: StdRng,
}

impl Default for RecursiveBacktracker4 {
    fn default() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }
}

impl RecursiveBacktracker4 {
    /// See [MazeGenerator2D::new_random].
    pub fn new_random() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    /// See [MazeGenerator2D::new_from_seed].
    pub fn new_from_seed(rng_seed: u64) -> Self {
        if rng_seed == 0 {
            Self::new_random()
        } else {
            Self {
                rng: StdRng::seed_from_u64(rng_seed),
            }
        }
    }

    /// See [MazeGenerator2D::generate].
    pub fn generate(&self, width: usize, height: usize) -> Wall4Grid {
        let mut cells = Wall4Grid::new(width, height);
        let mut visit_map = VisitMap2D::new_like(&cells);

        let start_coordinate = GridCoord2D::default();
        let mut backtrace = Vec::default();

        let rng = self.rng.clone();

        Self::backtrack(
            rng,
            &mut cells,
            &mut visit_map,
            start_coordinate,
            &mut backtrace,
        );

        cells
    }

    fn backtrack(
        mut rng: StdRng,
        cells: &mut Wall4Grid,
        visit_map: &mut VisitMap2D,
        mut current_cell: GridCoord2D,
        backtrace: &mut Vec<GridCoord2D>,
    ) {
        debug_assert_eq!(cells.width(), visit_map.width());
        debug_assert_eq!(cells.height(), visit_map.height());

        loop {
            // 1. Mark current cell as visited.
            visit_map[current_cell] = true;

            // 2. If the current cell has any neighbors which have not been visited,
            //  2.1 Choose randomly one of the unvisited neighbors.
            if let Some(selected_cell) =
                Self::select_random_unvisited_neighbor(&mut rng, visit_map, current_cell)
            {
                // 2.2 Add the current cell to the stack
                backtrace.push(current_cell);

                // 2.3 Remove the wall between the current cell and teh chosen cell.
                cells.remove_wall_between(current_cell, selected_cell);

                // 2.4 Make the chosen cell the current cell
                current_cell = selected_cell;

                // 2.5 Recursively call this function.
                continue;
            }

            // 3. else
            //  3.1 remove the last current cell from the stack
            if let Some(cell) = backtrace.pop() {
                // 3.2 Backtrack to the previous execution of this function.
                current_cell = cell;
            } else {
                break;
            }
        }
    }

    fn select_random_unvisited_neighbor(
        rng: &mut StdRng,
        visit_map: &VisitMap2D,
        current_cell: GridCoord2D,
    ) -> Option<GridCoord2D> {
        debug_assert!(current_cell.x < visit_map.width());
        debug_assert!(current_cell.y < visit_map.height());

        // Obtain a base value.
        let value = rng.next_u64() as usize;

        // Obtain neighbors.
        let list = visit_map.unvisited_neighbors(current_cell);
        if list.is_empty() {
            return None;
        }

        let selected = list[value % list.len()];
        Some(selected)
    }
}

impl MazeGenerator2D for RecursiveBacktracker4 {
    fn new_random() -> Self {
        RecursiveBacktracker4::new_random()
    }
    fn new_from_seed(rng_seed: u64) -> Self {
        RecursiveBacktracker4::new_from_seed(rng_seed)
    }
    fn generate(&self, width: usize, height: usize) -> Wall4Grid {
        self.generate(width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderers::{UnicodeRenderStyle, UnicodeRenderer};

    #[test]
    fn it_works() {
        let gen = RecursiveBacktracker4::default();
        let grid = gen.generate(16, 16);
        assert_eq!(grid.width(), 16);
        assert_eq!(grid.height(), 16);

        let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Heavy, true);
        let str = renderer.render(&grid);
        println!("{}", str);
    }
}
