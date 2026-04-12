use crate::generators::{
    CellSelector, GenerationStep, GenerationSteps, GenerationVisitor, MazeGenerator2D, NewestCell,
    VecGenerationVisitor,
};
use crate::grid_coord_2d::GridCoord2D;
use crate::visit_map_2d::VisitMap2D;
use crate::wall4_grid::Wall4Grid;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct GrowingTree4<S: CellSelector = NewestCell> {
    rng: StdRng,
    selector: S,
}

impl<S> GrowingTree4<S>
where
    S: CellSelector,
{
    pub fn with_selector(selector: S) -> Self {
        Self {
            rng: StdRng::from_os_rng(),
            selector,
        }
    }

    pub fn new_from_seed_with_selector(rng_seed: u64, selector: S) -> Self {
        let rng = if rng_seed == 0 {
            StdRng::from_os_rng()
        } else {
            StdRng::seed_from_u64(rng_seed)
        };

        Self { rng, selector }
    }

    fn generate_with_steps(&self, width: usize, height: usize) -> (Wall4Grid, Vec<GenerationStep>) {
        let mut grid = Wall4Grid::new(width, height);
        let mut visited = VisitMap2D::new_like(&grid);
        let mut frontier = Vec::new();
        let mut visitor = VecGenerationVisitor::default();

        if width == 0 || height == 0 {
            visitor.on_step(&GenerationStep::Complete);
            return (grid, visitor.into_steps());
        }

        let mut rng = self.rng.clone();
        let start = GridCoord2D::new(rng.gen_range(0..width), rng.gen_range(0..height));

        visited[start] = true;
        frontier.push(start);
        visitor.on_step(&GenerationStep::Visit { cell: start });
        visitor.on_step(&GenerationStep::AddToFrontier { cell: start });

        while !frontier.is_empty() {
            let idx = self.selector.select(&mut rng, frontier.len());
            let cell = frontier[idx];
            let mut candidates = visited.unvisited_neighbors(cell);
            if candidates.is_empty() {
                frontier.swap_remove(idx);
                visitor.on_step(&GenerationStep::Backtrack { to: cell });
                continue;
            }

            candidates.shuffle(&mut rng);
            let next = candidates[0];
            grid.remove_wall_between(cell, next);
            visited[next] = true;
            frontier.push(next);

            visitor.on_step(&GenerationStep::Carve {
                from: cell,
                to: next,
            });
            visitor.on_step(&GenerationStep::Visit { cell: next });
            visitor.on_step(&GenerationStep::AddToFrontier { cell: next });
        }

        visitor.on_step(&GenerationStep::Complete);
        (grid, visitor.into_steps())
    }
}

impl<S> MazeGenerator2D for GrowingTree4<S>
where
    S: CellSelector + Default,
{
    fn new_random() -> Self {
        Self {
            rng: StdRng::from_os_rng(),
            selector: S::default(),
        }
    }

    fn new_from_seed(rng_seed: u64) -> Self {
        if rng_seed == 0 {
            Self::new_random()
        } else {
            Self {
                rng: StdRng::seed_from_u64(rng_seed),
                selector: S::default(),
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
        "growing-tree"
    }

    fn description(&self) -> &'static str {
        "Growing tree maze generator with configurable frontier selection"
    }
}
