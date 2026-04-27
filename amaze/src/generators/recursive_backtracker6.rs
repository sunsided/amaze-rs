use crate::direction6::Direction6;
use crate::generators::MazeGenerator6D;
use crate::hex_coord::HexCoord;
use crate::wall6_grid::Wall6Grid;
use rand::SeedableRng;
use rand::prelude::IndexedRandom;
use rand::rngs::StdRng;

pub struct RecursiveBacktracker6 {
    rng_seed: u64,
}

impl Default for RecursiveBacktracker6 {
    fn default() -> Self {
        Self::new_random()
    }
}

impl RecursiveBacktracker6 {
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
        if width == 0 || height == 0 {
            return Wall6Grid::new(width, height);
        }

        let mut cells = Wall6Grid::new(width, height);
        let mut visit_map = vec![false; width * height];
        let start = HexCoord::new(0, 0);
        let mut backtrace = Vec::new();
        let mut rng = StdRng::seed_from_u64(self.rng_seed);

        let mut current = start;
        loop {
            visit_map[(current.r as usize) * width + current.q as usize] = true;

            if let Some(next) =
                Self::select_random_unvisited_neighbor(&mut rng, &visit_map, current, width, height)
            {
                backtrace.push(current);
                cells.remove_wall_between(current, next);
                current = next;
                continue;
            }

            if let Some(cell) = backtrace.pop() {
                current = cell;
            } else {
                break;
            }
        }

        cells
    }

    fn select_random_unvisited_neighbor(
        rng: &mut StdRng,
        visit_map: &[bool],
        current: HexCoord,
        width: usize,
        height: usize,
    ) -> Option<HexCoord> {
        let mut candidates = Vec::with_capacity(6);
        for &dir in &Direction6::CARDINALS {
            if let Some(n) = current.try_neighbor(dir, width, height) {
                let idx = (n.r as usize) * width + n.q as usize;
                if !visit_map[idx] {
                    candidates.push(n);
                }
            }
        }

        if candidates.is_empty() {
            return None;
        }

        candidates.choose(rng).copied()
    }
}

impl MazeGenerator6D for RecursiveBacktracker6 {
    fn new_random() -> Self {
        RecursiveBacktracker6::new_random()
    }

    fn new_from_seed(rng_seed: u64) -> Self {
        RecursiveBacktracker6::new_from_seed(rng_seed)
    }

    fn generate(&self, width: usize, height: usize) -> Wall6Grid {
        self.generate(width, height)
    }

    fn name(&self) -> &'static str {
        "hex-recursive-backtracker"
    }

    fn description(&self) -> &'static str {
        "Depth-first backtracking generator for hexagonal grids creating long corridors"
    }
}
