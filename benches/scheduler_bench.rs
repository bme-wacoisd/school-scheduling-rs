use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_scheduling(_c: &mut Criterion) {
    // TODO: Add benchmarks when needed
}

criterion_group!(benches, benchmark_scheduling);
criterion_main!(benches);
