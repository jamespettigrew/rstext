use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use rstext::text_buffer::piece_table::PieceTable;

const TEXT_SMALL: &str = include_str!("small.txt");
const TEXT_MEDIUM: &str = include_str!("medium.txt");
const TEXT_LARGE: &str = include_str!("large.txt");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("create_small", |b| {
        let text_small = TEXT_SMALL.chars().collect::<Vec<char>>();
        b.iter_batched(|| text_small.clone(), |data| PieceTable::new(data), BatchSize::SmallInput);
    });

    c.bench_function("create_medium", |b| {
        let text = TEXT_MEDIUM.chars().collect::<Vec<char>>();
        b.iter_batched(|| text.clone(), |data| PieceTable::new(data), BatchSize::SmallInput);
    });

    c.bench_function("create_large", |b| {
        let text = TEXT_LARGE.chars().collect::<Vec<char>>();
        b.iter_batched(|| text.clone(), |data| PieceTable::new(data), BatchSize::SmallInput);
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);