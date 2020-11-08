use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use rand::random;
use rstext::text_buffer::piece_table::PieceTable;
use rstext::text_buffer::{TextBuffer};

const INSERT_LARGE: &str = include_str!("small.txt");
const TEXT: &str = include_str!("large.txt");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("insert_random_char", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        b.iter(|| {
            piece_table.insert("a", random::<usize>() % piece_table.length);
        });
    });
    c.bench_function("insert_random_small_str", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        let items = "abcdefg";
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert(items, random::<usize>() % piece_table.length);
            },
            BatchSize::SmallInput
        );
    });
    c.bench_function("insert_random_large_str", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        b.iter(|| {
            piece_table.insert(INSERT_LARGE, random::<usize>() % piece_table.length)
        })
    });

    c.bench_function("insert_start_char", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        b.iter(|| {
            piece_table.insert("a", 0);
        });
    });
    c.bench_function("insert_start_small_str", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        let items = "abcdefg";
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert(items, 0);
            },
            BatchSize::SmallInput
        );
    });
    c.bench_function("insert_start_large_str", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        b.iter(|| {
            piece_table.insert(INSERT_LARGE, 0);
        })
    });

    c.bench_function("insert_middle_char", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        b.iter(|| {
            piece_table.insert("a", piece_table.length / 2);
        });
    });
    c.bench_function("insert_middle_small_str", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        let items = "abcdefg";
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert(items, piece_table.length / 2);
            },
            BatchSize::SmallInput
        );
    });
    c.bench_function("insert_middle_large_str", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        b.iter(|| {
            piece_table.insert(INSERT_LARGE, piece_table.length / 2);
        })
    });

    c.bench_function("insert_end_char", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        b.iter(|| {
            piece_table.insert("a", piece_table.length);
        });
    });
    c.bench_function("insert_end_small_str", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        let items = "abcdefg";
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert(items, piece_table.length);
            },
            BatchSize::SmallInput
        );
    });
    c.bench_function("insert_end_large_str", |b| {
        let piece_table = &mut PieceTable::new(TEXT.to_string());
        b.iter(|| {
            piece_table.insert(INSERT_LARGE, piece_table.length);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);