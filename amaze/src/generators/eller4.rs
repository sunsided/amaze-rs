use crate::generators::{
    GenerationStep, GenerationSteps, GenerationVisitor, MazeGenerator2D, VecGenerationVisitor,
};
use crate::grid_coord_2d::GridCoord2D;
use crate::wall4_grid::Wall4Grid;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

pub struct Eller4 {
    rng: StdRng,
}

impl Default for Eller4 {
    fn default() -> Self {
        Self::new_random()
    }
}

impl Eller4 {
    fn generate_with_steps(&self, width: usize, height: usize) -> (Wall4Grid, Vec<GenerationStep>) {
        let mut grid = Wall4Grid::new(width, height);
        let mut visitor = VecGenerationVisitor::default();
        if width == 0 || height == 0 {
            visitor.on_step(&GenerationStep::Complete);
            return (grid, visitor.into_steps());
        }

        for cell in grid.coords() {
            visitor.on_step(&GenerationStep::Visit { cell });
        }

        let mut rng = self.rng.clone();
        let mut set_ids: Vec<usize> = (0..width).collect();
        let mut next_set_id = width;

        for y in 0..height {
            for x in 0..width.saturating_sub(1) {
                let join = y + 1 == height || rng.random_bool(0.5);
                if join && set_ids[x] != set_ids[x + 1] {
                    let a = GridCoord2D::new(x, y);
                    let b = GridCoord2D::new(x + 1, y);
                    grid.remove_wall_between(a, b);
                    visitor.on_step(&GenerationStep::Carve { from: a, to: b });
                    let from = set_ids[x + 1];
                    let to = set_ids[x];
                    for id in &mut set_ids {
                        if *id == from {
                            *id = to;
                        }
                    }
                }
            }

            if y + 1 == height {
                continue;
            }

            let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
            for (x, set_id) in set_ids.iter().copied().enumerate() {
                groups.entry(set_id).or_default().push(x);
            }

            let mut carry_down = vec![false; width];
            for indices in groups.values() {
                let forced = indices[rng.random_range(0..indices.len())];
                carry_down[forced] = true;
                for &x in indices {
                    if rng.random_bool(0.35) {
                        carry_down[x] = true;
                    }
                }
            }

            let mut next_row = vec![0usize; width];
            for x in 0..width {
                if carry_down[x] {
                    let a = GridCoord2D::new(x, y);
                    let b = GridCoord2D::new(x, y + 1);
                    grid.remove_wall_between(a, b);
                    visitor.on_step(&GenerationStep::Carve { from: a, to: b });
                    next_row[x] = set_ids[x];
                } else {
                    next_row[x] = next_set_id;
                    next_set_id += 1;
                }
            }

            set_ids = next_row;
        }

        visitor.on_step(&GenerationStep::Complete);
        (grid, visitor.into_steps())
    }
}

impl MazeGenerator2D for Eller4 {
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
        "eller"
    }

    fn description(&self) -> &'static str {
        "Row-by-row Eller's algorithm with O(width) memory"
    }
}
