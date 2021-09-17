use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SEED: u64 = 0; // fixed seed so all ASTs are the same
const DEPTH: usize = 7;
const MAX_WIDTH: usize = 100;
const SOURCE: &str = "source";

fn bench_ast_wrangling(c: &mut Criterion) {
    let mut group = c.benchmark_group("ast-wrangling");

    group.bench_function("<boxed-ast>", |b| {
        b.iter(|| {
            let mut stmt = experiments::ast_boxed::Stmt::build_random_tree(
                SOURCE,
                SEED,
                DEPTH,
                MAX_WIDTH,
                &(),
            );
            let mut state = std::collections::hash_map::DefaultHasher::new();
            stmt.wrangle(&(), &mut state);
            stmt
        })
    });

    group.bench_function("<bumpalo-ast>", |b| {
        b.iter(|| {
            let bump = bumpalo::Bump::new();

            let mut stmt = experiments::ast_bumped::Stmt::build_random_tree(
                SOURCE, SEED, DEPTH, MAX_WIDTH, &bump,
            );
            let mut state = std::collections::hash_map::DefaultHasher::new();
            stmt.wrangle(&bump, &mut state);

            std::mem::drop(black_box(stmt));
            bump
        })
    });

    group.bench_function("<bumpalo-ast-no-drop>", |b| {
        b.iter(|| {
            let bump = bumpalo::Bump::new();

            let mut stmt = experiments::ast_bumped::Stmt::build_random_tree(
                SOURCE, SEED, DEPTH, MAX_WIDTH, &bump,
            );
            // 4Head approach -- we mem::forget the tree root.
            let mut state = std::collections::hash_map::DefaultHasher::new();
            stmt.wrangle(&bump, &mut state);

            std::mem::forget(stmt);
            bump
        })
    });
}

criterion_group!(benches, bench_ast_wrangling);
criterion_main!(benches);
