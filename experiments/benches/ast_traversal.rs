use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_ast_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");
    group.bench_function("<system-alloc-ast>", move |b| {});
}

criterion_group!(benches, bench_ast_traversal);
criterion_main!(benches);
