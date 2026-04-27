use crate::generators::{
    GenerationStep, GenerationSteps, GenerationVisitor, MazeGenerator2D, VecGenerationVisitor,
};
use crate::grid_coord_2d::GridCoord2D;
use crate::visit_map_2d::VisitMap2D;
use crate::wall4_grid::Wall4Grid;
use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub struct Prim4 {
    rng_seed: u64,
}

impl Default for Prim4 {
    fn default() -> Self {
        Self::new_random()
    }
}

impl Prim4 {
    pub fn new_random() -> Self {
        Self {
            rng_seed: rand::random(),
        }
    }

    pub fn new_from_seed(rng_seed: u64) -> Self {
        if rng_seed == 0 {
            Self::new_random()
        } else {
            Self { rng_seed }
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
        let mut visitor = VecGenerationVisitor::default();

        if width == 0 || height == 0 {
            visitor.on_step(&GenerationStep::Complete);
            return (cells, visitor.into_steps());
        }

        let mut rng = StdRng::seed_from_u64(self.rng_seed);
        let start = GridCoord2D::new(rng.random_range(0..width), rng.random_range(0..height));

        visit_map[start] = true;
        visitor.on_step(&GenerationStep::Visit { cell: start });

        let mut frontier: Vec<(GridCoord2D, GridCoord2D)> = Vec::new();
        Self::add_frontier_walls(
            &mut frontier,
            &visit_map,
            start,
            width,
            height,
            &mut visitor,
        );

        while !frontier.is_empty() {
            let idx = rng.random_range(0..frontier.len());
            let (from_cell, to_cell) = frontier.swap_remove(idx);

            if visit_map[to_cell] {
                continue;
            }

            visit_map[to_cell] = true;
            cells.remove_wall_between(from_cell, to_cell);

            visitor.on_step(&GenerationStep::Visit { cell: to_cell });
            visitor.on_step(&GenerationStep::Carve {
                from: from_cell,
                to: to_cell,
            });

            Self::add_frontier_walls(
                &mut frontier,
                &visit_map,
                to_cell,
                width,
                height,
                &mut visitor,
            );
        }

        visitor.on_step(&GenerationStep::Complete);
        (cells, visitor.into_steps())
    }

    fn add_frontier_walls<V: GenerationVisitor>(
        frontier: &mut Vec<(GridCoord2D, GridCoord2D)>,
        visit_map: &VisitMap2D,
        cell: GridCoord2D,
        width: usize,
        height: usize,
        visitor: &mut V,
    ) {
        let neighbors = [cell.up(), cell.right(), cell.down(), cell.left()];
        for neighbor in neighbors.into_iter().flatten() {
            if neighbor.x < width && neighbor.y < height && !visit_map[neighbor] {
                frontier.push((cell, neighbor));
                visitor.on_step(&GenerationStep::AddToFrontier { cell: neighbor });
            }
        }
    }
}

impl MazeGenerator2D for Prim4 {
    fn new_random() -> Self {
        Prim4::new_random()
    }

    fn new_from_seed(rng_seed: u64) -> Self {
        Prim4::new_from_seed(rng_seed)
    }

    fn generate(&self, width: usize, height: usize) -> Wall4Grid {
        self.generate(width, height)
    }

    fn generate_steps(&self, width: usize, height: usize) -> GenerationSteps {
        self.generate_steps(width, height)
    }

    fn name(&self) -> &'static str {
        "prim"
    }

    fn description(&self) -> &'static str {
        "Randomized Prim's algorithm producing mazes with many short corridors and branches"
    }
}
