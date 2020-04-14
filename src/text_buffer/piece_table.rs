use crate::text_buffer::{ Line, TextBuffer };
use std::iter::Iterator;
use std::ops::{Index, Range};

use Buffer::*;
use IndexLocation::*;

#[derive(Copy, Clone)]
enum Buffer {
    Original,
    Added,
}

enum IndexLocation {
    PieceHead(usize),
    PieceBody(usize, usize),
    PieceTail(usize),
    EOF,
}

struct Piece {
    buffer: Buffer,
    start: usize,
    length: usize,
    line_break_offsets: Vec<usize>
}

impl Piece {
    fn new(buffer: Buffer, start: usize, length: usize, piece_table: &PieceTable) -> Piece {
        let line_break_offsets = piece_table.line_breaks_in_buffer_range(
            buffer,
            start..start + length);
        let mut piece = Piece {
            buffer,
            start,
            length,
            line_break_offsets
        };

        piece
    }
}

pub struct PieceTable {
    original: Vec<char>,
    added: Vec<char>,
    pieces: Vec<Piece>,
    pub length: usize,
}

impl PieceTable {
    pub fn new(content: Vec<char>) -> PieceTable {
        let mut piece_table = PieceTable {
            length: content.len(),
            pieces: Vec::new(),
            original: content,
            added: Vec::new(),
        };
        piece_table.pieces = vec![Piece::new(Original, 0, piece_table.length, &piece_table)];

        piece_table
    }

    fn iter<'a>(&'a self) -> PieceTableIter<'a> {
        PieceTableIter {
            inner: self,
            current_piece_index: 0,
            current_piece_offset: 0,
            end_piece_index: self.pieces.len(),
            end_piece_offset: 0
        }
    }

    fn iter_range<'a>(&'a self, range: Range<usize>) -> PieceTableIter<'a> {
        if self.length == 0 || range.start >= range.end {
            return PieceTableIter {
                inner: self,
                current_piece_index: 0,
                current_piece_offset: 0,
                end_piece_index: 0,
                end_piece_offset: 0
            };
        }

        let start_location = self.index_location(range.start);
        let end_location = self.index_location(range.end.checked_sub(1).unwrap_or(0));

        let (start_piece_index, start_piece_offset) = match start_location {
            PieceHead(piece_index) => (piece_index, 0),
            PieceBody(piece_index, piece_offset) => (piece_index, piece_offset),
            PieceTail(piece_index) => (piece_index, self.pieces[piece_index].length - 1),
            EOF => panic!("Start index out of range")
        };

        let (end_piece_index, end_piece_offset) = match end_location {
            PieceHead(piece_index) => (piece_index, 1),
            PieceBody(piece_index, piece_offset) => (piece_index, piece_offset + 1),
            PieceTail(piece_index) => (piece_index + 1, 0),
            EOF => (self.pieces.len(), 0)
        };

        PieceTableIter {
            inner: self,
            current_piece_index: start_piece_index,
            current_piece_offset: start_piece_offset,
            end_piece_index,
            end_piece_offset,
        }
    }

    fn index_location(&self, index: usize) -> IndexLocation {
        let mut item_count = 0usize;
        for (piece_index, piece) in self.pieces.iter().enumerate() {
            if index >= item_count && index < item_count + piece.length {
                return match index {
                    index if index == item_count => PieceHead(piece_index),
                    index if index == item_count + piece.length - 1 => PieceTail(piece_index),
                    _ => PieceBody(piece_index, index - item_count),
                };
            }
            item_count += piece.length;
        }

        EOF
    }

    fn line_breaks_in_buffer_range(&self, buffer: Buffer, range: Range<usize>) -> Vec<usize> {
        let mut offsets = Vec::new();
        for (count, index) in range.enumerate() {
           let character = match buffer {
               Original => self.original[index],
               Added => self.added[index]
           };

           match character {
               '\n' => offsets.push(count),
               _ => ()
           }
        }

        offsets
    }
}

impl TextBuffer for PieceTable {
    fn insert_item_at(&mut self, item: char, index: usize) {
        self.insert_items_at(vec![item], index);
    }

    fn insert_items_at(&mut self, items: Vec<char>, index: usize) {
        let location = self.index_location(index);
        self.added.extend(&items);
        let new_piece = Piece::new(
            Buffer::Added,
            self.added.len() - items.len(),
            items.len(),
            self
        );
        self.length += items.len();

        match location {
            PieceHead(piece_index) => self.pieces.insert(piece_index, new_piece),
            PieceBody(piece_index, offset) => {
                let original_piece = &self.pieces[piece_index];
                let offcut_piece = Piece::new(
                    original_piece.buffer,
                    original_piece.start + offset,
                    original_piece.length - offset,
                    self
                );
                self.pieces[piece_index] = Piece::new(
                    original_piece.buffer,
                    original_piece.start,
                    offset,
                    self
                );
                self.pieces.insert(piece_index + 1, new_piece);
                self.pieces.insert(piece_index + 2, offcut_piece);
            }
            PieceTail(piece_index) => {
                let original_piece = &self.pieces[piece_index];
                let offcut_piece = Piece::new(
                    original_piece.buffer,
                    original_piece.start + original_piece.length - 1,
                    1,
                    self
                );
                self.pieces[piece_index] = Piece::new(
                    original_piece.buffer,
                    original_piece.start,
                    original_piece.length - 1,
                    self
                );
                self.pieces.insert(piece_index + 1, new_piece);
                self.pieces.insert(piece_index + 2, offcut_piece);
            }
            EOF => self.pieces.push(new_piece),
        }
    }

    fn all_content(&self) -> Vec<char> {
        self.iter().collect()
    }

    // TODO: Support different line endings
    fn line_at(&self, line_index: usize) -> Line {
        let mut line_start_index = 0;
        let mut line_end_index = None;
        let mut item_count = 0;
        let mut line_start_piece_index = 0;

        if line_index == 0 {
            // Line starts at index 0, ends at first line break found
            for piece in self.pieces.iter() {
                if !piece.line_break_offsets.is_empty() {
                    line_end_index = Some(item_count + piece.line_break_offsets[0]);
                    break;
                }
                item_count += piece.length;
            }
        }
        else {
            // Find start index
            let mut line_breaks_remaining = line_index;
            for (piece_index, piece) in self.pieces.iter().enumerate() {
                if line_breaks_remaining <= piece.line_break_offsets.len() {
                    // Line starts in this piece
                    let line_break_offset = piece.line_break_offsets[line_breaks_remaining - 1];
                    line_start_index = item_count + line_break_offset + 1;
                    line_start_piece_index = piece_index;

                    if line_breaks_remaining < piece.line_break_offsets.len() {
                        // Start of next line is also in this piece
                        let next_line_break_offset = piece.line_break_offsets[line_breaks_remaining];
                        line_end_index = Some(item_count + next_line_break_offset);
                    }
                    item_count += piece.length;
                    break;
                }
                line_breaks_remaining = line_breaks_remaining
                    .checked_sub(piece.line_break_offsets.len())
                    .unwrap_or(0);
                item_count += piece.length;
            }

            if line_end_index.is_none() {
                // Find end index by searching for first line break from line_start_index onwards
                for piece in self.pieces.iter().skip(line_start_piece_index + 1) {
                    if !piece.line_break_offsets.is_empty() {
                        line_end_index = Some(item_count + piece.line_break_offsets[0]);
                        break;
                    }
                    item_count += piece.length;
                }
            }
        }

        let content = self
            .iter_range(line_start_index..line_end_index.unwrap_or(self.length))
            .collect::<Vec<char>>();

        Line::new(line_start_index, content)
    }

    // TODO: Support different line endings
    fn line_count(&self) -> usize {
        self.pieces.iter().fold(1, |count, piece| piece.line_break_offsets.len() + count)
    }

    fn remove_item_at(&mut self, index: usize) {
        self.remove_items(index..index + 1);
    }

    fn remove_items(&mut self, range: Range<usize>) {
        if range.start >= range.end {
            return;
        }

        let start_location = self.index_location(range.start);
        let end_location = self.index_location(range.end - 1);

        let start_piece_index = match start_location {
            PieceHead(piece_index) => piece_index,
            PieceBody(piece_index, _) => piece_index,
            PieceTail(piece_index) => piece_index,
            EOF => panic!("Cannot remove from EOF"),
        };

        let mut end_piece_index = match end_location {
            PieceHead(piece_index) => piece_index,
            PieceBody(piece_index, _) => piece_index,
            PieceTail(piece_index) => piece_index,
            EOF => self.pieces.len(),
        };

        if start_piece_index < end_piece_index {
            let drained_length = self
                .pieces
                .drain(start_piece_index + 1..end_piece_index)
                .len();
            end_piece_index -= drained_length;

            match start_location {
                PieceHead(_) => {
                    self.pieces.remove(start_piece_index);
                    end_piece_index -= 1;
                }
                PieceBody(_, piece_offset) => {
                    let original_piece = &self.pieces[start_piece_index];
                    self.pieces[start_piece_index] = Piece::new(
                        original_piece.buffer,
                        original_piece.start,
                        piece_offset,
                        self
                    );
                }
                PieceTail(_) => {
                    let original_piece = &self.pieces[start_piece_index];
                    self.pieces[start_piece_index] = Piece::new(
                        original_piece.buffer,
                        original_piece.start,
                        original_piece.length - 1,
                        self
                    );
                }
                EOF => {}
            };

            match end_location {
                PieceHead(_) => {
                    let original_piece = &self.pieces[end_piece_index];
                    let new_piece = Piece::new(
                        original_piece.buffer,
                        original_piece.start + 1,
                        original_piece.length - 1,
                        self
                    );

                    match new_piece.length {
                        x if x >= 1 => { self.pieces[end_piece_index] = new_piece; },
                        _ => { self.pieces.remove(end_piece_index); },
                    }
                }
                PieceBody(_, piece_offset) => {
                    let original_piece = &self.pieces[end_piece_index];
                    self.pieces[end_piece_index] = Piece::new(
                        original_piece.buffer,
                        original_piece.start + piece_offset + 1,
                        original_piece.length - piece_offset - 1,
                        self
                    );
                }
                PieceTail(_) => {
                    self.pieces.remove(end_piece_index);
                }
                EOF => {}
            };
        } else if start_piece_index == end_piece_index {
            match (start_location, end_location) {
                (PieceHead(_), PieceHead(_)) => {
                    let original_piece = &self.pieces[start_piece_index];
                    let new_piece = Piece::new(
                        original_piece.buffer,
                        original_piece.start + 1,
                        original_piece.length - 1,
                        self
                    );

                    match new_piece.length {
                        x if x >= 1 => { self.pieces[end_piece_index] = new_piece; },
                        _ => { self.pieces.remove(end_piece_index); },
                    }
                }
                (PieceHead(_), PieceBody(_, piece_offset)) => {
                    let original_piece = &self.pieces[start_piece_index];
                    self.pieces[start_piece_index] = Piece::new(
                        original_piece.buffer,
                        original_piece.start + piece_offset + 1,
                        original_piece.length - piece_offset - 1,
                        self
                    );
                }
                (PieceBody(_, start_offset), PieceBody(_, end_offset)) => {
                    let left_piece = &self.pieces[start_piece_index];
                    let right_piece = Piece::new(
                        left_piece.buffer,
                        left_piece.start + end_offset + 1,
                        left_piece.length - end_offset - 1,
                        self
                    );
                    self.pieces[start_piece_index] = Piece::new(
                        left_piece.buffer,
                        left_piece.start,
                        start_offset,
                        self
                    );
                    self.pieces.insert(start_piece_index + 1, right_piece);
                }
                (PieceHead(_), PieceTail(_)) => {
                    self.pieces.remove(start_piece_index);
                }
                (PieceBody(_, start_offset), PieceTail(_)) => {
                    let original_piece = &self.pieces[start_piece_index];
                    self.pieces[start_piece_index] = Piece::new(
                        original_piece.buffer,
                        original_piece.start,
                        start_offset + 1,
                        self
                    );
                }
                (PieceTail(_), PieceTail(_)) => {
                    let original_piece = &self.pieces[start_piece_index];
                    if original_piece.length.checked_sub(1).is_some() {
                        self.pieces[start_piece_index] = Piece::new(
                            original_piece.buffer,
                            original_piece.start,
                            original_piece.length - 1,
                            self);
                    } else {
                        &self.pieces.remove(start_piece_index);
                    }
                }
                _ => panic!(),
            }
        }
        self.length = self.length.checked_sub(range.end - range.start).unwrap_or(0);
    }
}

struct PieceTableIter<'a> {
    inner: &'a PieceTable,
    current_piece_index: usize,
    current_piece_offset: usize,
    end_piece_index: usize,
    end_piece_offset: usize,
}

impl Index<usize> for PieceTable {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        let mut index_count = 0usize;
        for piece in &self.pieces {
            if index_count + piece.length > index {
                return match piece.buffer {
                    Original => &self.original[piece.start + index - index_count],
                    Added => &self.added[piece.start + index - index_count],
                };
            }
            index_count += piece.length;
        }

        panic!("Index out of range")
    }
}

impl<'a> Iterator for PieceTableIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_piece_index == self.end_piece_index
            && self.current_piece_offset >= self.end_piece_offset
        {
            return None;
        }

        if let Some(current_piece) = self.inner.pieces.get(self.current_piece_index) {
            if self.current_piece_offset >= current_piece.length {
                self.current_piece_index += 1;
                self.current_piece_offset = 0;
                return self.next();
            }

            let item = match current_piece.buffer {
                Original => self.inner.original[current_piece.start + self.current_piece_offset],
                Added => self.inner.added[current_piece.start + self.current_piece_offset],
            };
            self.current_piece_offset += 1;

            return Some(item);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index() {
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd']);
        pt.added = vec!['0', '1', '2', '3'];
        pt.pieces = vec![
            Piece {
                buffer: Buffer::Original,
                start: 0,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
                line_break_offsets: Vec::new(),
            },
        ];

        assert_eq!('a', pt[0]);
        assert_eq!('0', pt[2]);
        assert_eq!('d', pt[6]);
        assert_eq!('3', pt[7]);
    }

    #[test]
    fn insert_head() {
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd']);

        pt.insert_item_at('0', 0);
        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['0', 'a', 'b', 'c', 'd']
        );

        pt.insert_item_at('1', 1);
        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['0', '1', 'a', 'b', 'c', 'd']
        );

        pt.insert_item_at('2', 0);
        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['2', '0', '1', 'a', 'b', 'c', 'd']
        );
    }

    #[test]
    fn insert_body() {
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd']);
        pt.insert_items_at(vec!['0', '1', '2'], 2);

        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['a', 'b', '0', '1', '2', 'c', 'd']
        );

        pt.insert_item_at('3', 4);
        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['a', 'b', '0', '1', '3', '2', 'c', 'd']
        );
    }

    #[test]
    fn insert_end() {
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd']);
        pt.insert_items_at(vec!['0', '1', '2'], 4);

        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['a', 'b', 'c', 'd', '0', '1', '2']
        );

        pt.insert_item_at('3', 7);
        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['a', 'b', 'c', 'd', '0', '1', '2', '3']
        );
    }

    #[test]
    fn iter() {
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd']);
        pt.added = vec!['0', '1', '2', '3'];
        pt.pieces = vec![
            Piece {
                buffer: Buffer::Original,
                start: 0,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
                line_break_offsets: Vec::new(),
            },
        ];

        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['a', 'b', '0', '1', '2', 'c', 'd', '3']
        );
    }

    #[test]
    fn iter_range() {
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd']);
        pt.added = vec!['0', '1', '2', '3'];
        pt.pieces = vec![
            Piece {
                buffer: Buffer::Original,
                start: 0,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
                line_break_offsets: Vec::new(),
            },
        ];

        // vec!['a', 'b', '0', '1', '2', 'c', 'd', '3']
        assert_eq!(
            pt.iter_range(1..4).collect::<Vec<char>>(),
            vec!['b', '0', '1']
        );
        assert_eq!(
            pt.iter_range(0..5).collect::<Vec<char>>(),
            vec!['a', 'b', '0', '1', '2']
        );
        assert_eq!(
            pt.iter_range(4..23).collect::<Vec<char>>(),
            vec!['2', 'c', 'd', '3']
        );
    }

    #[test]
    fn line_at() {
        let pt = &mut PieceTable::new(vec!['a', 'b']);
        assert_eq!(vec!['a', 'b'], pt.line_at(0).characters);
        pt.insert_items_at(vec!['\n', 'd', '0', '\n', '2', '3', '4', '5', '6', '7', '\n', '8', '9'], 4);
        assert_eq!(vec!['d', '0'], pt.line_at(1).characters);
        assert_eq!(vec!['2', '3', '4', '5', '6', '7'], pt.line_at(2).characters);
        assert_eq!(vec!['8', '9'], pt.line_at(3).characters);

        pt.insert_item_at('\n', 14);
        assert_eq!(vec!['8'], pt.line_at(3).characters);

        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd']);
        pt.insert_item_at('\n', 2);
        assert_eq!(vec!['a', 'b'], pt.line_at(0).characters);
        assert_eq!(vec!['c', 'd'], pt.line_at(1).characters);
        pt.remove_item_at(2);
        pt.insert_item_at('\n', 2);
        assert_eq!(vec!['a', 'b'], pt.line_at(0).characters);
        assert_eq!(vec!['c', 'd'], pt.line_at(1).characters);

        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd']);
        pt.insert_item_at('\n', 2);
        pt.insert_item_at('c', 2);
        pt.insert_item_at('c', 3);
        assert_eq!(vec!['a', 'b', 'c', 'c'], pt.line_at(0).characters);
        assert_eq!(vec!['c', 'd'], pt.line_at(1).characters);

        // Single piece with lines
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd', '\n', 'e', 'f']);
        assert_eq!(vec!['a', 'b', 'c', 'd'], pt.line_at(0).characters);

        // Line not at index 0 where multiple pieces and start of next line in same piece
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd', '\n', 'e', 'f', '\n', 'h', 'i']);
        pt.insert_items_at(vec!['\n', 'j', 'k'], 20);
        assert_eq!(vec!['e', 'f'], pt.line_at(1).characters);
    }

    #[test]
    fn line_count() {
        let pt = &mut PieceTable::new(vec!['a', 'b', '\n', 'd']);
        pt.insert_items_at(vec!['0', '\n', '2'], 4);

        assert_eq!(3, pt.line_count());
    }

    #[test]
    fn remove_head() {
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd']);
        pt.added = vec!['0', '1', '2', '3'];
        pt.pieces = vec![
            Piece {
                buffer: Buffer::Original,
                start: 0,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
                line_break_offsets: Vec::new(),
            },
        ];
        pt.length = pt.added.len() + pt.original.len();
        
        pt.remove_item_at(0);
        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['b', '0', '1', '2', 'c', 'd', '3']
        );

        pt.remove_items(0..3);
        assert_eq!(pt.iter().collect::<Vec<char>>(), vec!['2', 'c', 'd', '3']);
    }

    #[test]
    fn remove_body() {
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd']);
        pt.added = vec!['0', '1', '2', '3'];
        pt.pieces = vec![
            Piece {
                buffer: Buffer::Original,
                start: 0,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
                line_break_offsets: Vec::new(),
            },
        ];
        pt.length = pt.added.len() + pt.original.len();

        pt.remove_item_at(3);
        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['a', 'b', '0', '2', 'c', 'd', '3']
        );

        pt.remove_items(1..6);
        assert_eq!(pt.iter().collect::<Vec<char>>(), vec!['a', '3']);
    }

    #[test]
    fn remove_tail() {
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd']);
        pt.added = vec!['0', '1', '2', '3'];
        pt.pieces = vec![
            Piece {
                buffer: Buffer::Original,
                start: 0,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
                line_break_offsets: Vec::new(),
            },
        ];
        pt.length = pt.added.len() + pt.original.len();

        pt.remove_item_at(1);
        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['a', '0', '1', '2', 'c', 'd', '3']
        );

        // Remove linebreak from tail
        let pt = &mut PieceTable::new(vec!['a', 'b', 'c', 'd', '\n']);
        pt.added = vec!['0', '1', '2', '3'];
        pt.pieces = vec![
            Piece {
                buffer: Buffer::Original,
                start: 0,
                length: 2,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 3,
                line_break_offsets: Vec::new(),
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
                line_break_offsets: Vec::new(),
            },
        ];
        pt.length = pt.added.len() + pt.original.len();

        pt.remove_item_at(7);
        assert_eq!(
            pt.line_at(0).characters,
            vec!['a', 'b', '0', '1', '2', 'c', 'd', '3']
        );
    }
}
