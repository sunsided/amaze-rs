use crate::generators::{
    GenerationStep, GenerationSteps, GenerationVisitor, MazeGenerator2D, VecGenerationVisitor,
};
use crate::grid_coord_2d::GridCoord2D;
use crate::visit_map_2d::VisitMap2D;
use crate::wall4_grid::Wall4Grid;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};

pub struct Wilson4 {
    rng: StdRng,
}

impl Default for Wilson4 {
    fn default() -> Self {
        Self::new_random()
    }
}

impl Wilson4 {
    fn random_neighbor<R: Rng>(rng: &mut R, grid: &Wall4Grid, cell: GridCoord2D) -> GridCoord2D {
        let neighbors: Vec<_> = grid.neighbors(cell).collect();
        neighbors[rng.random_range(0..neighbors.len())]
    }

    fn generate_with_steps(&self, width: usize, height: usize) -> (Wall4Grid, Vec<GenerationStep>) {
        let mut grid = Wall4Grid::new(width, height);
        let mut visitor = VecGenerationVisitor::default();
        if width == 0 || height == 0 {
            visitor.on_step(&GenerationStep::Complete);
            return (grid, visitor.into_steps());
        }

        let mut rng = self.rng.clone();
        let all_cells: Vec<_> = grid.coords().collect();
        let mut in_maze = VisitMap2D::new_like(&grid);

        let first = all_cells[rng.random_range(0..all_cells.len())];
        in_maze[first] = true;
        visitor.on_step(&GenerationStep::Visit { cell: first });

        while all_cells.iter().copied().any(|c| !in_maze[c]) {
            let mut start = all_cells[rng.random_range(0..all_cells.len())];
            while in_maze[start] {
                start = all_cells[rng.random_range(0..all_cells.len())];
            }

            let mut walk = vec![start];
            let mut pos: HashMap<GridCoord2D, usize> = HashMap::new();
            pos.insert(start, 0);
            let mut current = start;

            while !in_maze[current] {
                let next = Self::random_neighbor(&mut rng, &grid, current);
                if let Some(&at) = pos.get(&next) {
                    walk.truncate(at + 1);
                    pos = walk
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(i, c)| (c, i))
                        .collect();
                    current = next;
                } else {
                    walk.push(next);
                    pos.insert(next, walk.len() - 1);
                    current = next;
                }
            }

            let mut emitted: HashSet<GridCoord2D> = HashSet::new();
            for pair in walk.windows(2) {
                let from = pair[0];
                let to = pair[1];
                grid.remove_wall_between(from, to);
                visitor.on_step(&GenerationStep::Carve { from, to });
                if emitted.insert(from) {
                    visitor.on_step(&GenerationStep::Visit { cell: from });
                }
                in_maze[from] = true;
                in_maze[to] = true;
            }
        }

        visitor.on_step(&GenerationStep::Complete);
        (grid, visitor.into_steps())
    }
}

impl MazeGenerator2D for Wilson4 {
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
        "wilson"
    }

    fn description(&self) -> &'static str {
        "Unbiased loop-erased random walk spanning tree generator"
    }
}
