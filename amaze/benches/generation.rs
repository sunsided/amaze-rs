use amaze::generators::{
    BinaryTree4, Eller4, GrowingTree4, HuntAndKill4, Kruskal4, MazeGenerator2D, MixedCell, Prim4,
    RecursiveBacktracker4, Sidewinder4, Wilson4,
};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

#[cfg(feature = "generator-hex")]
use amaze::generators::{AldousBroder6, GrowingTree6, MazeGenerator6D, RecursiveBacktracker6};

fn bench_generators(c: &mut Criterion) {
    let sizes = [(16, "16x16"), (64, "64x64"), (256, "256x256")];

    let generators: Vec<(&str, Box<dyn MazeGenerator2D>)> = vec![
        (
            "recursive_backtracker",
            Box::new(RecursiveBacktracker4::new_from_seed(1337)) as Box<dyn MazeGenerator2D>,
        ),
        (
            "growing_tree",
            Box::new(GrowingTree4::<MixedCell>::new_from_seed(1337)) as Box<dyn MazeGenerator2D>,
        ),
        (
            "kruskal",
            Box::new(Kruskal4::new_from_seed(1337)) as Box<dyn MazeGenerator2D>,
        ),
        (
            "eller",
            Box::new(Eller4::new_from_seed(1337)) as Box<dyn MazeGenerator2D>,
        ),
        (
            "wilson",
            Box::new(Wilson4::new_from_seed(1337)) as Box<dyn MazeGenerator2D>,
        ),
        (
            "hunt_and_kill",
            Box::new(HuntAndKill4::new_from_seed(1337)) as Box<dyn MazeGenerator2D>,
        ),
        (
            "sidewinder",
            Box::new(Sidewinder4::new_from_seed(1337)) as Box<dyn MazeGenerator2D>,
        ),
        (
            "binary_tree",
            Box::new(BinaryTree4::new_from_seed(1337)) as Box<dyn MazeGenerator2D>,
        ),
        (
            "prim",
            Box::new(Prim4::new_from_seed(1337)) as Box<dyn MazeGenerator2D>,
        ),
    ];

    let mut group = c.benchmark_group("generators");
    group.sample_size(10);

    for (name, generator) in generators {
        for &(size, label) in &sizes {
            let id = BenchmarkId::new(name, label);
            group.bench_with_input(id, &size, |b, &size| {
                b.iter(|| generator.generate(size, size))
            });
        }
    }

    #[cfg(feature = "generator-hex")]
    {
        use amaze::generators::NewestCell;

        let hex_generators: Vec<(&str, Box<dyn MazeGenerator6D>)> = vec![
            (
                "hex_recursive_backtracker",
                Box::new(RecursiveBacktracker6::new_from_seed(1337)) as Box<dyn MazeGenerator6D>,
            ),
            (
                "hex_growing_tree",
                Box::new(GrowingTree6::<NewestCell>::new_from_seed(1337))
                    as Box<dyn MazeGenerator6D>,
            ),
            (
                "hex_aldous_broder",
                Box::new(AldousBroder6::new_from_seed(1337)) as Box<dyn MazeGenerator6D>,
            ),
        ];

        for (name, generator) in hex_generators {
            for &(size, label) in &sizes {
                let id = BenchmarkId::new(name, label);
                group.bench_with_input(id, &size, |b, &size| {
                    b.iter(|| generator.generate(size, size))
                });
            }
        }
    }

    group.finish();
}

criterion_group!(benches, bench_generators);
criterion_main!(benches);
