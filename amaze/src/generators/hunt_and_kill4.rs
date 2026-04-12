use crate::generators::{
    GenerationStep, GenerationSteps, GenerationVisitor, MazeGenerator2D, VecGenerationVisitor,
};
use crate::grid_coord_2d::{GetCoordinateBounds2D, GridCoord2D};
use crate::visit_map_2d::VisitMap2D;
use crate::wall4_grid::Wall4Grid;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct HuntAndKill4 {
    rng: StdRng,
}

impl Default for HuntAndKill4 {
    fn default() -> Self {
        Self::new_random()
    }
}

impl HuntAndKill4 {
    fn visited_neighbors(visited: &VisitMap2D, cell: GridCoord2D) -> Vec<GridCoord2D> {
        let mut all = Vec::with_capacity(4);
        for c in [cell.up(), cell.right(), cell.down(), cell.left()]
            .into_iter()
            .flatten()
        {
            if c.x < visited.width() && c.y < visited.height() && visited[c] {
                all.push(c);
            }
        }
        all
    }

    fn generate_with_steps(&self, width: usize, height: usize) -> (Wall4Grid, Vec<GenerationStep>) {
        let mut grid = Wall4Grid::new(width, height);
        let mut visited = VisitMap2D::new_like(&grid);
        let mut visitor = VecGenerationVisitor::default();
        if width == 0 || height == 0 {
            visitor.on_step(&GenerationStep::Complete);
            return (grid, visitor.into_steps());
        }

        let mut rng = self.rng.clone();
        let mut current = Some(GridCoord2D::new(
            rng.gen_range(0..width),
            rng.gen_range(0..height),
        ));

        while let Some(cell) = current {
            visited[cell] = true;
            visitor.on_step(&GenerationStep::Visit { cell });

            let mut unvisited = visited.unvisited_neighbors(cell);
            if !unvisited.is_empty() {
                unvisited.shuffle(&mut rng);
                let next = unvisited[0];
                grid.remove_wall_between(cell, next);
                visitor.on_step(&GenerationStep::Carve {
                    from: cell,
                    to: next,
                });
                current = Some(next);
                continue;
            }

            current = None;
            'hunt: for y in 0..height {
                for x in 0..width {
                    let hunt = GridCoord2D::new(x, y);
                    if visited[hunt] {
                        continue;
                    }
                    let mut neighbors = Self::visited_neighbors(&visited, hunt);
                    if neighbors.is_empty() {
                        continue;
                    }
                    neighbors.shuffle(&mut rng);
                    let next = neighbors[0];
                    grid.remove_wall_between(hunt, next);
                    visitor.on_step(&GenerationStep::Carve {
                        from: hunt,
                        to: next,
                    });
                    current = Some(hunt);
                    visitor.on_step(&GenerationStep::AddToFrontier { cell: hunt });
                    break 'hunt;
                }
            }
        }

        visitor.on_step(&GenerationStep::Complete);
        (grid, visitor.into_steps())
    }
}

impl MazeGenerator2D for HuntAndKill4 {
    fn new_random() -> Self {
        Self {
            rng: StdRng::from_entropy(),
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
        "hunt-and-kill"
    }

    fn description(&self) -> &'static str {
        "Walk and hunt generator with no explicit recursion stack"
    }
}
