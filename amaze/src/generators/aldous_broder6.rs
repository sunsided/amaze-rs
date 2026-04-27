use crate::direction6::Direction6;
use crate::generators::{
    HexGenerationStep, HexGenerationSteps, HexGenerationVisitor, MazeGenerator6D,
    VecHexGenerationVisitor,
};
use crate::hex_coord::HexCoord;
use crate::wall6_grid::Wall6Grid;
use rand::RngExt;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub struct AldousBroder6 {
    rng_seed: u64,
}

impl Default for AldousBroder6 {
    fn default() -> Self {
        Self::new_random()
    }
}

impl AldousBroder6 {
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

    pub fn generate(&self, width: usize, height: usize) -> Wall6Grid {
        self.generate_with_steps(width, height).0
    }

    pub fn generate_steps(&self, width: usize, height: usize) -> HexGenerationSteps {
        HexGenerationSteps::new(self.generate_with_steps(width, height).1)
    }

    fn generate_with_steps(
        &self,
        width: usize,
        height: usize,
    ) -> (Wall6Grid, Vec<HexGenerationStep>) {
        let mut grid = Wall6Grid::new(width, height);
        let mut visitor = VecHexGenerationVisitor::default();

        if width == 0 || height == 0 {
            visitor.on_step(&HexGenerationStep::Complete);
            return (grid, visitor.into_steps());
        }

        let mut visited = vec![false; width * height];
        let mut visited_count = 0usize;
        let total_cells = width * height;

        let mut rng = StdRng::seed_from_u64(self.rng_seed);
        let mut current = HexCoord::new(
            rng.random_range(0..width) as isize,
            rng.random_range(0..height) as isize,
        );

        let idx = (current.r as usize) * width + current.q as usize;
        if !visited[idx] {
            visited[idx] = true;
            visited_count += 1;
            visitor.on_step(&HexGenerationStep::Visit { cell: current });
        }

        while visited_count < total_cells {
            let neighbors = Self::neighbors(current, width, height);
            let next = neighbors[rng.random_range(0..neighbors.len())];
            let next_idx = (next.r as usize) * width + next.q as usize;

            if !visited[next_idx] {
                visited[next_idx] = true;
                visited_count += 1;
                grid.remove_wall_between(current, next);
                visitor.on_step(&HexGenerationStep::Carve {
                    from: current,
                    to: next,
                });
                visitor.on_step(&HexGenerationStep::Visit { cell: next });
            }

            current = next;
        }

        visitor.on_step(&HexGenerationStep::Complete);
        (grid, visitor.into_steps())
    }

    fn neighbors(cell: HexCoord, width: usize, height: usize) -> Vec<HexCoord> {
        let mut result = Vec::with_capacity(6);
        for &dir in &Direction6::CARDINALS {
            if let Some(n) = cell.try_neighbor(dir, width, height) {
                result.push(n);
            }
        }
        result
    }
}

impl MazeGenerator6D for AldousBroder6 {
    fn new_random() -> Self {
        AldousBroder6::new_random()
    }

    fn new_from_seed(rng_seed: u64) -> Self {
        AldousBroder6::new_from_seed(rng_seed)
    }

    fn generate(&self, width: usize, height: usize) -> Wall6Grid {
        self.generate(width, height)
    }

    fn generate_steps(&self, width: usize, height: usize) -> HexGenerationSteps {
        self.generate_steps(width, height)
    }

    fn name(&self) -> &'static str {
        "hex-aldous-broder"
    }

    fn description(&self) -> &'static str {
        "Aldous-Broder algorithm for hexagonal grids producing uniform spanning trees"
    }
}
