use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use rstext::text_buffer::piece_table::PieceTable;

const TEXT_SMALL: &str = include_str!("small.txt");
const TEXT_MEDIUM: &str = include_str!("medium.txt");
const TEXT_LARGE: &str = include_str!("large.txt");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("create_small", |b| {
        b.iter_batched(|| TEXT_SMALL.to_string(), |data| PieceTable::new(data), BatchSize::SmallInput);
    });

    c.bench_function("create_medium", |b| {
        b.iter_batched(|| TEXT_MEDIUM.to_string(), |data| PieceTable::new(data), BatchSize::SmallInput);
    });

    c.bench_function("create_large", |b| {
        b.iter_batched(|| TEXT_LARGE.to_string(), |data| PieceTable::new(data), BatchSize::SmallInput);
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);