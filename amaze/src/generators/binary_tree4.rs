use crate::generators::{
    GenerationStep, GenerationSteps, GenerationVisitor, MazeGenerator2D, VecGenerationVisitor,
};
use crate::grid_coord_2d::GridCoord2D;
use crate::wall4_grid::Wall4Grid;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct BinaryTree4 {
    rng: StdRng,
}

impl Default for BinaryTree4 {
    fn default() -> Self {
        Self::new_random()
    }
}

impl BinaryTree4 {
    fn generate_with_steps(&self, width: usize, height: usize) -> (Wall4Grid, Vec<GenerationStep>) {
        let mut grid = Wall4Grid::new(width, height);
        let mut visitor = VecGenerationVisitor::default();
        if width == 0 || height == 0 {
            visitor.on_step(&GenerationStep::Complete);
            return (grid, visitor.into_steps());
        }

        let mut rng = self.rng.clone();
        for y in 0..height {
            for x in 0..width {
                let cell = GridCoord2D::new(x, y);
                visitor.on_step(&GenerationStep::Visit { cell });
                let can_north = y > 0;
                let can_east = x + 1 < width;

                let target = match (can_north, can_east) {
                    (true, true) => {
                        if rng.gen_bool(0.5) {
                            Some(GridCoord2D::new(x, y - 1))
                        } else {
                            Some(GridCoord2D::new(x + 1, y))
                        }
                    }
                    (true, false) => Some(GridCoord2D::new(x, y - 1)),
                    (false, true) => Some(GridCoord2D::new(x + 1, y)),
                    (false, false) => None,
                };

                if let Some(next) = target {
                    grid.remove_wall_between(cell, next);
                    visitor.on_step(&GenerationStep::Carve {
                        from: cell,
                        to: next,
                    });
                }
            }
        }

        visitor.on_step(&GenerationStep::Complete);
        (grid, visitor.into_steps())
    }
}

impl MazeGenerator2D for BinaryTree4 {
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
        "binary-tree"
    }

    fn description(&self) -> &'static str {
        "Very fast simple generator with directional bias"
    }
}
