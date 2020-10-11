use criterion::{black_box, criterion_group, criterion_main, Criterion, BatchSize};
use rand::random;
use rstext::text_buffer::piece_table::PieceTable;
use rstext::text_buffer::{TextBuffer};

const TEXT: &str = include_str!("large.txt");

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("line_content_start", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        b.iter(|| {
            piece_table.line_at(0);
        });
    });
    c.bench_function("line_content_mid", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        let mid = piece_table.line_count() / 2;
        b.iter(|| {
            piece_table.line_at(mid);
        });
    });
    c.bench_function("line_content_end", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        let end = piece_table.line_count() - 1;
        b.iter(|| {
            piece_table.line_at(end);
        });
    });
    c.bench_function("line_content_random", |b| {
        let text = TEXT.chars().collect::<Vec<char>>();
        let piece_table = &mut PieceTable::new(text);
        let line_count = piece_table.line_count();
        b.iter(|| {
            piece_table.line_at(random::<usize>() % line_count);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);