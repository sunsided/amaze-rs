use amaze::generators::RecursiveBacktracker4;
use amaze::preamble::GridCoord2D;
use amaze::solvers::{AStarSolver, BfsSolver, DeadEndFillingSolver, DfsSolver, MazeSolver};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn bench_solvers(c: &mut Criterion) {
    let sizes = [(16, "16x16"), (64, "64x64"), (256, "256x256")];

    let generator = RecursiveBacktracker4::new_from_seed(1337);

    let mut group = c.benchmark_group("solvers");

    for &(size, label) in &sizes {
        let maze = generator.generate(size, size);
        let start = GridCoord2D::default();
        let end = GridCoord2D::new(size - 1, size - 1);

        group.bench_with_input(
            BenchmarkId::new("bfs", label),
            &(&maze, start, end),
            |b, input| {
                let (maze, start, end) = *input;
                b.iter(|| BfsSolver.solve(maze, start, end))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("dfs", label),
            &(&maze, start, end),
            |b, input| {
                let (maze, start, end) = *input;
                b.iter(|| DfsSolver.solve(maze, start, end))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("astar", label),
            &(&maze, start, end),
            |b, input| {
                let (maze, start, end) = *input;
                b.iter(|| AStarSolver.solve(maze, start, end))
            },
        );
        group.bench_with_input(
            BenchmarkId::new("dead_end_filling", label),
            &(&maze, start, end),
            |b, input| {
                let (maze, start, end) = *input;
                b.iter(|| DeadEndFillingSolver.solve(maze, start, end))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_solvers);
criterion_main!(benches);
