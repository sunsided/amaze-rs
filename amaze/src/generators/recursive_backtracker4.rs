use crate::generators::{
    GenerationStep, GenerationSteps, GenerationVisitor, MazeGenerator2D, VecGenerationVisitor,
};
use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D};
use crate::visit_map_2d::VisitMap2D;
use crate::wall4_grid::Wall4Grid;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;

/// A maze generator that implements the Recursive Backtracking algorithm.
pub struct RecursiveBacktracker4 {
    rng: StdRng,
}

impl Default for RecursiveBacktracker4 {
    fn default() -> Self {
        Self::new_random()
    }
}

impl RecursiveBacktracker4 {
    pub fn new_random() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    pub fn new_from_seed(rng_seed: u64) -> Self {
        if rng_seed == 0 {
            Self::new_random()
        } else {
            Self {
                rng: StdRng::seed_from_u64(rng_seed),
            }
        }
    }

    pub fn generate(&self, width: usize, height: usize) -> Wall4Grid {
        self.generate_with_steps(width, height).0
    }

    pub fn generate_steps(&self, width: usize, height: usize) -> GenerationSteps {
        GenerationSteps::new(self.generate_with_steps(width, height).1)
    }

    fn generate_with_steps(&self, width: usize, height: usize) -> (Wall4Grid, Vec<GenerationStep>) {
        let mut cells = Wall4Grid::new(width, height);
        let mut visit_map = VisitMap2D::new_like(&cells);
        let start_coordinate = GridCoord2D::default();
        let mut backtrace = Vec::new();
        let mut visitor = VecGenerationVisitor::default();

        if width == 0 || height == 0 {
            visitor.on_step(&GenerationStep::Complete);
            return (cells, visitor.into_steps());
        }

        let rng = self.rng.clone();
        Self::backtrack(
            rng,
            &mut cells,
            &mut visit_map,
            start_coordinate,
            &mut backtrace,
            &mut visitor,
        );
        visitor.on_step(&GenerationStep::Complete);

        (cells, visitor.into_steps())
    }

    fn backtrack<V: GenerationVisitor>(
        mut rng: StdRng,
        cells: &mut Wall4Grid,
        visit_map: &mut VisitMap2D,
        mut current_cell: GridCoord2D,
        backtrace: &mut Vec<GridCoord2D>,
        visitor: &mut V,
    ) {
        debug_assert_eq!(cells.width(), visit_map.width());
        debug_assert_eq!(cells.height(), visit_map.height());

        loop {
            if !visit_map[current_cell] {
                visitor.on_step(&GenerationStep::Visit { cell: current_cell });
            }
            visit_map[current_cell] = true;

            if let Some(selected_cell) =
                Self::select_random_unvisited_neighbor(&mut rng, visit_map, current_cell)
            {
                backtrace.push(current_cell);
                visitor.on_step(&GenerationStep::AddToFrontier { cell: current_cell });
                cells.remove_wall_between(current_cell, selected_cell);
                visitor.on_step(&GenerationStep::Carve {
                    from: current_cell,
                    to: selected_cell,
                });
                current_cell = selected_cell;
                continue;
            }

            if let Some(cell) = backtrace.pop() {
                current_cell = cell;
                visitor.on_step(&GenerationStep::Backtrack { to: cell });
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

        let list = visit_map.unvisited_neighbors(current_cell);
        if list.is_empty() {
            return None;
        }

        list.choose(rng).copied()
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

    fn generate_steps(&self, width: usize, height: usize) -> GenerationSteps {
        self.generate_steps(width, height)
    }

    fn name(&self) -> &'static str {
        "recursive-backtracker"
    }

    fn description(&self) -> &'static str {
        "Depth-first backtracking generator that creates long corridors"
    }
}

#[cfg(all(test, feature = "unicode-renderer"))]
mod tests {
    use super::*;
    use crate::renderers::{UnicodeRenderStyle, UnicodeRenderer};

    #[test]
    fn it_works() {
        let generator = RecursiveBacktracker4::new_from_seed(42);
        let grid = generator.generate(16, 16);
        assert_eq!(grid.width(), 16);
        assert_eq!(grid.height(), 16);

        let renderer = UnicodeRenderer::new(UnicodeRenderStyle::Heavy, true);
        let maze = renderer.render(&grid);
        assert!(!maze.is_empty());
    }
}
