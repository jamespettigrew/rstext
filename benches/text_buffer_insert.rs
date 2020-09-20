use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use rand::random;
use rstext::text_buffer::piece_table::PieceTable;
use rstext::text_buffer::{TextBuffer};

const INSERT_LARGE: &str = include_str!("small.txt");
const TEXT: &str = include_str!("large.txt");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("insert_random_char", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        b.iter(|| {
            piece_table.insert_item_at('a', random::<usize>() % piece_table.length);
        });
    });
    c.bench_function("insert_random_small_str", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        let items = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g'];
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert_items_at(items, random::<usize>() % piece_table.length);
            },
            BatchSize::SmallInput
        );
    });
    c.bench_function("insert_random_large_str", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        let items = INSERT_LARGE.chars().collect::<Vec<char>>();
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert_items_at(items, random::<usize>() % piece_table.length);
            },
            BatchSize::SmallInput
        );
    });

    c.bench_function("insert_start_char", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        b.iter(|| {
            piece_table.insert_item_at('a', 0);
        });
    });
    c.bench_function("insert_start_small_str", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        let items = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g'];
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert_items_at(items, 0);
            },
            BatchSize::SmallInput
        );
    });
    c.bench_function("insert_start_large_str", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        let items = INSERT_LARGE.chars().collect::<Vec<char>>();
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert_items_at(items, 0);
            },
            BatchSize::SmallInput
        );
    });

    c.bench_function("insert_middle_char", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        b.iter(|| {
            piece_table.insert_item_at('a', piece_table.length / 2);
        });
    });
    c.bench_function("insert_middle_small_str", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        let items = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g'];
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert_items_at(items, piece_table.length / 2);
            },
            BatchSize::SmallInput
        );
    });
    c.bench_function("insert_middle_large_str", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        let items = INSERT_LARGE.chars().collect::<Vec<char>>();
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert_items_at(items, piece_table.length / 2);
            },
            BatchSize::SmallInput
        );
    });

    c.bench_function("insert_end_char", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        b.iter(|| {
            piece_table.insert_item_at('a', piece_table.length);
        });
    });
    c.bench_function("insert_end_small_str", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        let items = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g'];
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert_items_at(items, piece_table.length);
            },
            BatchSize::SmallInput
        );
    });
    c.bench_function("insert_end_large_str", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        let items = INSERT_LARGE.chars().collect::<Vec<char>>();
        b.iter_batched(
            || items.clone(), 
            |items| {
                piece_table.insert_items_at(items, piece_table.length);
            },
            BatchSize::SmallInput
        );
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);