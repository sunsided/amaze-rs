use amaze::generators::RecursiveBacktracker4;
use amaze::stats::MazeStats;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn bench_stats(c: &mut Criterion) {
    let sizes = [(16, "16x16"), (64, "64x64"), (128, "128x128")];

    let generator = RecursiveBacktracker4::new_from_seed(1337);

    let mut group = c.benchmark_group("stats");
    group.sample_size(10);

    for &(size, label) in &sizes {
        let maze = generator.generate(size, size);
        let id = BenchmarkId::new("MazeStats::from_grid", label);
        group.bench_with_input(id, &size, |b, _| b.iter(|| MazeStats::from_grid(&maze)));
    }

    group.finish();
}

criterion_group!(benches, bench_stats);
criterion_main!(benches);
