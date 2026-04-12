use crate::generators::helpers::union_find::UnionFind;
use crate::generators::{
    GenerationStep, GenerationSteps, GenerationVisitor, MazeGenerator2D, VecGenerationVisitor,
};
use crate::grid_coord_2d::{GridCoord2D, LinearizeCoords2D};
use crate::wall4_grid::Wall4Grid;
use rand::SeedableRng;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;

pub struct Kruskal4 {
    rng: StdRng,
}

impl Default for Kruskal4 {
    fn default() -> Self {
        Self::new_random()
    }
}

impl Kruskal4 {
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

        let mut edges = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let cell = GridCoord2D::new(x, y);
                if x + 1 < width {
                    edges.push((cell, GridCoord2D::new(x + 1, y)));
                }
                if y + 1 < height {
                    edges.push((cell, GridCoord2D::new(x, y + 1)));
                }
            }
        }

        let mut rng = self.rng.clone();
        edges.shuffle(&mut rng);

        let mut uf = UnionFind::new(width * height);
        for (a, b) in edges {
            let ia = grid.linearize_coords(a);
            let ib = grid.linearize_coords(b);
            if uf.union(ia, ib) {
                grid.remove_wall_between(a, b);
                visitor.on_step(&GenerationStep::Carve { from: a, to: b });
            }
        }

        visitor.on_step(&GenerationStep::Complete);
        (grid, visitor.into_steps())
    }
}

impl MazeGenerator2D for Kruskal4 {
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
        "kruskal"
    }

    fn description(&self) -> &'static str {
        "Randomized Kruskal algorithm using union-find"
    }
}
