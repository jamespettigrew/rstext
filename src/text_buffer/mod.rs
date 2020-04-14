pub mod line;
pub mod piece_table;

use line::Line;
use std::ops::Range;

pub trait TextBuffer {
    fn insert_item_at(&mut self, item: char, index: usize);
    fn insert_items_at(&mut self, items: Vec<char>, index: usize);
    fn all_content(&self) -> Vec<char>;
    fn line_at(&self, row: usize) -> Line;
    fn line_count(&self) -> usize;
    fn remove_item_at(&mut self, index: usize);
    fn remove_items(&mut self, range: Range<usize>);
}
