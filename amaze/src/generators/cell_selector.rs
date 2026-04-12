use rand::Rng;

pub trait CellSelector {
    fn select<R: Rng>(&self, rng: &mut R, frontier_len: usize) -> usize;
}

#[derive(Debug, Copy, Clone, Default)]
pub struct NewestCell;

impl CellSelector for NewestCell {
    fn select<R: Rng>(&self, _rng: &mut R, frontier_len: usize) -> usize {
        frontier_len - 1
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct OldestCell;

impl CellSelector for OldestCell {
    fn select<R: Rng>(&self, _rng: &mut R, _frontier_len: usize) -> usize {
        0
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct RandomCell;

impl CellSelector for RandomCell {
    fn select<R: Rng>(&self, rng: &mut R, frontier_len: usize) -> usize {
        rng.random_range(0..frontier_len)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MixedCell {
    pub newest_probability: f64,
}

impl Default for MixedCell {
    fn default() -> Self {
        Self {
            newest_probability: 0.5,
        }
    }
}

impl CellSelector for MixedCell {
    fn select<R: Rng>(&self, rng: &mut R, frontier_len: usize) -> usize {
        if rng.random_bool(self.newest_probability.clamp(0.0, 1.0)) {
            frontier_len - 1
        } else {
            rng.random_range(0..frontier_len)
        }
    }
}
