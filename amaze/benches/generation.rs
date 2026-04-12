use amaze::generators::RecursiveBacktracker4;
use criterion::{criterion_group, criterion_main, Criterion};

fn recursive_backtracker_bench(c: &mut Criterion) {
    let generator = RecursiveBacktracker4::new_from_seed(1337);
    c.bench_function("recursive_backtracker_64x64", |b| {
        b.iter(|| generator.generate(64, 64))
    });
}

criterion_group!(benches, recursive_backtracker_bench);
criterion_main!(benches);
