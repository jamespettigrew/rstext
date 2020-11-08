pub mod line;
pub mod piece;
pub mod piece_table;

use line::Line;
use std::ops::Range;

pub trait TextBuffer {
    fn insert(&mut self, s: &str, offset: usize);
    fn all_content(&self) -> String;
    fn line_at(&self, idx: usize) -> Line;
    fn line_count(&self) -> usize;
    fn remove(&mut self, range: Range<usize>);
}
