use crate::generators::{
    GenerationStep, GenerationSteps, GenerationVisitor, MazeGenerator2D, VecGenerationVisitor,
};
use crate::grid_coord_2d::GridCoord2D;
use crate::wall4_grid::Wall4Grid;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct Sidewinder4 {
    rng: StdRng,
}

impl Default for Sidewinder4 {
    fn default() -> Self {
        Self::new_random()
    }
}

impl Sidewinder4 {
    fn generate_with_steps(&self, width: usize, height: usize) -> (Wall4Grid, Vec<GenerationStep>) {
        let mut grid = Wall4Grid::new(width, height);
        let mut visitor = VecGenerationVisitor::default();
        if width == 0 || height == 0 {
            visitor.on_step(&GenerationStep::Complete);
            return (grid, visitor.into_steps());
        }

        let mut rng = self.rng.clone();
        for y in 0..height {
            let mut run_start = 0usize;
            for x in 0..width {
                let cell = GridCoord2D::new(x, y);
                visitor.on_step(&GenerationStep::Visit { cell });
                let at_east_boundary = x + 1 == width;
                let at_north_boundary = y == 0;
                let carve_east = !at_east_boundary && (at_north_boundary || rng.random_bool(0.5));

                if carve_east {
                    let east = GridCoord2D::new(x + 1, y);
                    grid.remove_wall_between(cell, east);
                    visitor.on_step(&GenerationStep::Carve {
                        from: cell,
                        to: east,
                    });
                } else if !at_north_boundary {
                    let carve_x = rng.random_range(run_start..=x);
                    let from = GridCoord2D::new(carve_x, y);
                    let to = GridCoord2D::new(carve_x, y - 1);
                    grid.remove_wall_between(from, to);
                    visitor.on_step(&GenerationStep::Carve { from, to });
                    run_start = x + 1;
                }
            }
        }

        visitor.on_step(&GenerationStep::Complete);
        (grid, visitor.into_steps())
    }
}

impl MazeGenerator2D for Sidewinder4 {
    fn new_random() -> Self {
        Self {
            rng: StdRng::from_os_rng(),
        }
    }

    fn new_from_seed(rng_seed: u64) -> Self {
        if rng_seed == 0 {
            Self::new_random()
        } else {
            Self {
                rng: StdRng::seed_from_u64(rng_seed),
            }
        }
    }

    fn generate(&self, width: usize, height: usize) -> Wall4Grid {
        self.generate_with_steps(width, height).0
    }

    fn generate_steps(&self, width: usize, height: usize) -> GenerationSteps {
        GenerationSteps::new(self.generate_with_steps(width, height).1)
    }

    fn name(&self) -> &'static str {
        "sidewinder"
    }

    fn description(&self) -> &'static str {
        "Fast row-wise generator with horizontal bias"
    }
}
