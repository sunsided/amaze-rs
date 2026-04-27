use crate::direction6::Direction6;
use crate::generators::{CellSelector, MazeGenerator6D, NewestCell};
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
        if width == 0 || height == 0 {
            return Wall6Grid::new(width, height);
        }

        let mut grid = Wall6Grid::new(width, height);
        let mut visited = vec![false; width * height];
        let mut frontier = Vec::new();
        let mut rng = StdRng::seed_from_u64(self.rng_seed);

        let start = HexCoord::new(
            rng.random_range(0..width) as isize,
            rng.random_range(0..height) as isize,
        );

        let start_idx = (start.r as usize) * width + start.q as usize;
        visited[start_idx] = true;
        frontier.push(start);

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
            let next_idx = (next.r as usize) * width + next.q as usize;
            visited[next_idx] = true;
            frontier.push(next);
        }

        grid
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

    fn name(&self) -> &'static str {
        "hex-growing-tree"
    }

    fn description(&self) -> &'static str {
        "Growing tree maze generator for hexagonal grids with configurable frontier selection"
    }
}
