use crate::direction6::Direction6;
use crate::generators::{
    CellSelector, HexGenerationStep, HexGenerationSteps, HexGenerationVisitor, MazeGenerator6D,
    NewestCell, VecHexGenerationVisitor,
};
use crate::hex_coord::HexCoord;
use crate::wall6_grid::Wall6Grid;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

pub struct GrowingTree6<S: CellSelector = NewestCell> {
    rng_seed: u64,
    selector: S,
}

impl<S> GrowingTree6<S>
where
    S: CellSelector,
{
    pub fn with_selector(selector: S) -> Self {
        Self {
            rng_seed: rand::random(),
            selector,
        }
    }

    pub fn new_from_seed_with_selector(rng_seed: u64, selector: S) -> Self {
        let rng_seed = if rng_seed == 0 {
            rand::random()
        } else {
            rng_seed
        };

        Self { rng_seed, selector }
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
        let mut frontier: Vec<HexCoord> = Vec::new();
        let mut rng = StdRng::seed_from_u64(self.rng_seed);

        let start = HexCoord::new(
            rng.random_range(0..width) as isize,
            rng.random_range(0..height) as isize,
        );

        let start_idx = (start.r as usize) * width + start.q as usize;
        visited[start_idx] = true;
        visitor.on_step(&HexGenerationStep::Visit { cell: start });
        frontier.push(start);
        visitor.on_step(&HexGenerationStep::AddToFrontier { cell: start });

        while !frontier.is_empty() {
            let idx = self.selector.select(&mut rng, frontier.len());
            let cell = frontier[idx];
            let mut candidates = Self::unvisited_neighbors(&visited, cell, width, height);

            if candidates.is_empty() {
                frontier.swap_remove(idx);
                continue;
            }

            candidates.shuffle(&mut rng);
            let next = candidates[0];
            grid.remove_wall_between(cell, next);
            visitor.on_step(&HexGenerationStep::Carve {
                from: cell,
                to: next,
            });
            let next_idx = (next.r as usize) * width + next.q as usize;
            visited[next_idx] = true;
            visitor.on_step(&HexGenerationStep::Visit { cell: next });
            frontier.push(next);
            visitor.on_step(&HexGenerationStep::AddToFrontier { cell: next });
        }

        visitor.on_step(&HexGenerationStep::Complete);
        (grid, visitor.into_steps())
    }

    fn unvisited_neighbors(
        visited: &[bool],
        cell: HexCoord,
        width: usize,
        height: usize,
    ) -> Vec<HexCoord> {
        let mut result = Vec::with_capacity(6);
        for &dir in &Direction6::CARDINALS {
            if let Some(n) = cell.try_neighbor(dir, width, height) {
                let idx = (n.r as usize) * width + n.q as usize;
                if !visited[idx] {
                    result.push(n);
                }
            }
        }
        result
    }
}

impl<S> MazeGenerator6D for GrowingTree6<S>
where
    S: CellSelector + Default,
{
    fn new_random() -> Self {
        Self {
            rng_seed: rand::random(),
            selector: S::default(),
        }
    }

    fn new_from_seed(rng_seed: u64) -> Self {
        if rng_seed == 0 {
            Self::new_random()
        } else {
            Self {
                rng_seed,
                selector: S::default(),
            }
        }
    }

    fn generate(&self, width: usize, height: usize) -> Wall6Grid {
        self.generate(width, height)
    }

    fn generate_steps(&self, width: usize, height: usize) -> HexGenerationSteps {
        self.generate_steps(width, height)
    }

    fn name(&self) -> &'static str {
        "hex-growing-tree"
    }

    fn description(&self) -> &'static str {
        "Growing tree maze generator for hexagonal grids with configurable frontier selection"
    }
}
