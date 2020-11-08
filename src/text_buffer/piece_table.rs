use crate::str_utils;
use crate::text_buffer::{Line, TextBuffer};
use crate::text_buffer::piece::{ Buffer, Piece};
use std::iter::Iterator;
use std::ops::{Index, Range};

enum PiecePosition {
    Head(usize),
    Body(usize, usize),
    EOF,
}

struct ChangeRecord {
    offset: usize,
    piece_index: usize,
}

pub struct PieceTable {
    original: String,
    added: String,
    pieces: Vec<Piece>,
    pub length: usize,
    last_insert: Option<ChangeRecord>,
    last_remove: Option<ChangeRecord>,
}

impl PieceTable {
    pub fn new(content: String) -> Self {
        let mut pt = Self {
            length: content.len(),
            pieces: Vec::new(),
            original: content,
            added: String::new(),
            last_insert: None,
            last_remove: None,
        };
        if !pt.original.is_empty() {
            pt.pieces.push(pt.create_piece(Buffer::Original, 0, pt.length));
        }

        pt
    }

    fn create_piece(&self, buffer: Buffer, start: usize, length: usize) -> Piece {
        let buffer_contents = match buffer {
            Buffer::Added => &self.added,
            Buffer::Original => &self.original,
        };
        let piece_contents = &buffer_contents[start..start + length];
        let line_break_offsets = str_utils::line_break_offsets(piece_contents);

        let piece = Piece {
            buffer,
            start,
            length,
            line_break_offsets,
        };

        piece
    }

    fn iter(&self) -> PieceTableIter {
        PieceTableIter {
            inner: self,
            current_piece_index: 0,
            current_piece_offset: 0,
            end_piece_index: self.pieces.len(),
            end_piece_offset: 0,
        }
    }

    fn iter_range(&self, range: Range<usize>) -> PieceTableIter {
        if self.length == 0 || range.start >= range.end {
            return PieceTableIter {
                inner: self,
                current_piece_index: 0,
                current_piece_offset: 0,
                end_piece_index: 0,
                end_piece_offset: 0,
            };
        }

        let start_location = self.offset_to_piece_position(range.start);
        let end_location = self.offset_to_piece_position(range.end.checked_sub(1).unwrap_or(0));

        let (start_piece_index, start_piece_offset) = match start_location {
            PiecePosition::Head(piece_index) => (piece_index, 0),
            PiecePosition::Body(piece_index, piece_offset) => (piece_index, piece_offset),
            PiecePosition::EOF => panic!("Start index out of range"),
        };

        let (end_piece_index, end_piece_offset) = match end_location {
            PiecePosition::Head(piece_index) => (piece_index, 1),
            PiecePosition::Body(piece_index, piece_offset) => (piece_index, piece_offset + 1),
            PiecePosition::EOF => (self.pieces.len(), 0),
        };

        PieceTableIter {
            inner: self,
            current_piece_index: start_piece_index,
            current_piece_offset: start_piece_offset,
            end_piece_index,
            end_piece_offset,
        }
    }

    fn offset_to_piece_position(&self, offset: usize) -> PiecePosition {
        let mut item_count = 0usize;
        for (piece_index, piece) in self.pieces.iter().enumerate() {
            if offset >= item_count && offset < item_count + piece.length {
                return match offset {
                    offset if offset == item_count => PiecePosition::Head(piece_index),
                    _ => PiecePosition::Body(piece_index, offset - item_count),
                };
            }
            item_count += piece.length;
        }

        PiecePosition::EOF
    }

    fn raw_insert(&mut self, s: &str, offset: usize) {
        let location = self.offset_to_piece_position(offset);
        let new_piece = self.create_piece(
            Buffer::Added,
            self.added.len() - s.len(),
            s.len()
        );

        match location {
            PiecePosition::Head(piece_index) => {
                self.pieces.insert(piece_index, new_piece);
                self.last_insert = Some(ChangeRecord {
                    offset: offset + s.len(),
                    piece_index,
                });
            }
            PiecePosition::Body(piece_index, offset) => {
                let (left, right) = self.pieces[piece_index].split_at(offset);
                self.pieces[piece_index] = left;
                self.pieces.insert(piece_index + 1, new_piece);
                self.pieces.insert(piece_index + 2, right);
                self.last_insert = Some(ChangeRecord {
                    offset: offset + s.len(),
                    piece_index: piece_index + 1,
                });
            }
            PiecePosition::EOF => {
                self.pieces.push(new_piece);
                self.last_insert = Some(ChangeRecord {
                    offset: offset + s.len(),
                    piece_index: self.pieces.len() - 1,
                });
            },
        }
    }

    fn raw_remove(&mut self, offset: usize) {
        let location = self.offset_to_piece_position(offset);
        let cached_piece_index: Option<usize> = match location {
            PiecePosition::Head(piece_index) => {
                let original_piece = &self.pieces[piece_index];

                if original_piece.length <= 1 {
                    self.pieces.remove(piece_index);
                    piece_index.checked_sub(1)
                } else {
                    self.pieces[piece_index] = original_piece.truncate_left(1);
                    None
                }
            }
            PiecePosition::Body(piece_index, piece_offset) => {
                let (left, right) = self.pieces[piece_index].split_at(piece_offset);
                self.pieces[piece_index] = left;
                self.pieces.insert(piece_index + 1, right.truncate_left(1));
                Some(piece_index)
            }
            PiecePosition::EOF => panic!("Attempted to remove from EOF"),
        };

        self.last_remove = cached_piece_index.map(|i| {
            ChangeRecord {
                offset: offset.checked_sub(1).unwrap_or(0),
                piece_index: i,
            }
        });
    }
}

impl TextBuffer for PieceTable {
    fn insert(&mut self, to_insert: &str, offset: usize) {
        self.added.push_str(to_insert);
        self.length += to_insert.len();
        self.last_remove = None;

        match self.last_insert {
            Some(ChangeRecord { offset: last_offset, piece_index }) if last_offset == offset => {
                self.pieces[piece_index] = self.pieces[piece_index].extend(to_insert);
                self.last_insert = Some(ChangeRecord {
                    offset: offset + to_insert.len(),
                    piece_index,
                });
            }
            _ => {
                self.raw_insert(to_insert, offset);
            }
        }
    }

    fn all_content(&self) -> String {
        self.iter().collect()
    }

    fn line_at(&self, idx: usize) -> Line {
        let mut line_start_index = 0;
        let mut line_end_index = None;
        let mut item_count = 0;
        let mut line_start_piece_index = 0;

        if idx == 0 {
            // Line starts at index 0, ends at first line break found
            for piece in self.pieces.iter() {
                if !piece.line_break_offsets.is_empty() {
                    line_end_index = Some(item_count + piece.line_break_offsets[0]);
                    break;
                }
                item_count += piece.length;
            }
        } else {
            // Find start index
            let mut line_breaks_remaining = idx;
            for (piece_index, piece) in self.pieces.iter().enumerate() {
                if line_breaks_remaining <= piece.line_break_offsets.len() {
                    // Line starts in this piece
                    let line_break_offset = piece.line_break_offsets[line_breaks_remaining - 1];
                    line_start_index = item_count + line_break_offset + 1;
                    line_start_piece_index = piece_index;

                    if line_breaks_remaining < piece.line_break_offsets.len() {
                        // Start of next line is also in this piece
                        let next_line_break_offset =
                            piece.line_break_offsets[line_breaks_remaining];
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
            .collect::<String>();

        Line::new(line_start_index, content)
    }

    fn line_count(&self) -> usize {
        self.pieces
            .iter()
            .fold(1, |count, piece| piece.line_break_offsets.len() + count)
    }

    fn remove(&mut self, range: Range<usize>) {
        if range.start >= range.end {
            return;
        }

        self.last_insert = None;
        let removed_len = range.len();

        for offset in range.rev() {
            match self.last_remove {
                Some(ChangeRecord { offset: last_offset, piece_index }) if offset == last_offset => {
                    if self.pieces[piece_index].length <= 1 {
                        self.pieces.remove(piece_index);
                        self.last_remove = piece_index.checked_sub(1).map(|i| {
                            ChangeRecord {
                                offset: offset.checked_sub(1).unwrap_or(0),
                                piece_index: i,
                            }
                        });
                    } else {
                        self.pieces[piece_index] = self.pieces[piece_index].truncate_right(1);
                        self.last_remove = Some(ChangeRecord {
                            offset: offset.checked_sub(1).unwrap_or(0),
                            piece_index,
                        });
                    }
                }
                _ => self.raw_remove(offset)
            }

        }

        self.length = self.length.checked_sub(removed_len).unwrap_or(0);
    }
}

struct PieceTableIter<'a> {
    inner: &'a PieceTable,
    current_piece_index: usize,
    current_piece_offset: usize,
    end_piece_index: usize,
    end_piece_offset: usize,
}

impl<'a> Iterator for PieceTableIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_piece_index == self.end_piece_index
            && self.current_piece_offset >= self.end_piece_offset
        {
            return None;
        }

        match self.inner.pieces.get(self.current_piece_index) {
            Some(current_piece) => {
                if self.current_piece_offset >= current_piece.length {
                    self.current_piece_index += 1;
                    self.current_piece_offset = 0;
                    return self.next();
                }

                let buffer = match current_piece.buffer {
                    Buffer::Original => &self.inner.original,
                    Buffer::Added => &self.inner.added
                };

                let character = buffer[current_piece.start + self.current_piece_offset..]
                    .chars()
                    .next();

                if let Some(character) = character {
                    self.current_piece_offset += character.len_utf8();
                };

                character
            }
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_insertion() {
        let pt = &mut PieceTable::new(String::from("abcd"));

        pt.insert("0", 3);
        pt.insert("1", 4);
        pt.insert("2", 6);

        assert_eq!(pt.iter().collect::<String>(), "abc01d2");
    }

    #[test]
    fn insert_head() {
        let pt = &mut PieceTable::new(String::from("abcd"));

        pt.insert("0", 0);
        assert_eq!(pt.iter().collect::<String>(), "0abcd");

        pt.insert("1", 1);
        assert_eq!(pt.iter().collect::<String>(), "01abcd");

        pt.insert("2", 0);
        assert_eq!(pt.iter().collect::<String>(), "201abcd");
    }

    #[test]
    fn insert_body() {
        let pt = &mut PieceTable::new(String::from("abcd"));
        pt.insert("012", 2);
        assert_eq!(pt.iter().collect::<String>(), "ab012cd");

        pt.insert("3", 4);
        assert_eq!(pt.iter().collect::<String>(), "ab0132cd");
    }

    #[test]
    fn insert_end() {
        let pt = &mut PieceTable::new(String::from("abcd"));
        pt.insert("012", 4);
        assert_eq!(pt.iter().collect::<String>(), "abcd012");

        pt.insert("3", 7);
        assert_eq!(pt.iter().collect::<String>(), "abcd0123");
    }

    #[test]
    fn iter() {
        let pt = &mut PieceTable::new(String::from("abcd"));
        pt.added = String::from("0123");
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

        assert_eq!(pt.iter().collect::<String>(), "ab012cd3");
    }

    #[test]
    fn iter_range() {
        let pt = &mut PieceTable::new(String::from("abcd"));
        pt.added = String::from("0123");
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

        // ab012cd3
        assert_eq!(pt.iter_range(1..4).collect::<String>(), "b01");
        assert_eq!(pt.iter_range(0..5).collect::<String>(), "ab012");
        assert_eq!(pt.iter_range(4..23).collect::<String>(), "2cd3");
    }

    #[test]
    fn line_at() {
        let pt = &mut PieceTable::new(String::from("ab"));
        assert_eq!("ab", pt.line_at(0).content);
        pt.insert("\nd0\n234567\n89", 4);
        assert_eq!("d0", pt.line_at(1).content);
        assert_eq!("234567", pt.line_at(2).content);
        assert_eq!("89", pt.line_at(3).content);

        pt.insert("\n", 14);
        assert_eq!("8", pt.line_at(3).content);

        let pt = &mut PieceTable::new(String::from("abcd"));
        pt.insert("\n", 2);
        assert_eq!("ab", pt.line_at(0).content);
        assert_eq!("cd", pt.line_at(1).content);
        pt.remove(2..3);
        pt.insert("\n", 2);
        assert_eq!("ab", pt.line_at(0).content);
        assert_eq!("cd", pt.line_at(1).content);

        let pt = &mut PieceTable::new(String::from("abcd"));
        pt.insert("\n", 2);
        pt.insert("c", 2);
        pt.insert("c", 3);
        assert_eq!("abcc", pt.line_at(0).content);
        assert_eq!("cd", pt.line_at(1).content);

        // Single piece with lines
        let pt = &mut PieceTable::new(String::from("abcd\nef"));
        assert_eq!("abcd", pt.line_at(0).content);

        // Line not at index 0 where multiple pieces and start of next line in same piece
        let pt = &mut PieceTable::new(String::from("abcd\nef\nhi"));
        pt.insert("\njk", 20);
        assert_eq!("ef", pt.line_at(1).content);
    }

    #[test]
    fn line_count() {
        let pt = &mut PieceTable::new(String::from("ab\nd"));
        pt.insert("0\n2",4);

        assert_eq!(3, pt.line_count());
    }

    #[test]
    fn remove_head() {
        let pt = &mut PieceTable::new(String::from("abcd"));
        pt.added = String::from("0123");
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

        pt.remove(0..1);
        assert_eq!(pt.iter().collect::<String>(), "b012cd3");

        pt.remove(0..3);
        assert_eq!(pt.iter().collect::<String>(), "2cd3");
    }

    #[test]
    fn remove_body() {
        let pt = &mut PieceTable::new(String::from("abcd"));
        pt.added = String::from("0123");
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

        pt.remove(3..4);
        assert_eq!(pt.iter().collect::<String>(), "ab02cd3");

        pt.remove(1..6);
        assert_eq!(pt.iter().collect::<String>(), "a3");
    }

    #[test]
    fn remove_tail() {
        let pt = &mut PieceTable::new(String::from("abcd"));
        pt.added = String::from("0123");
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

        pt.remove(1..2);
        assert_eq!(pt.iter().collect::<String>(), "a012cd3");

        // Remove linebreak from tail
        let pt = &mut PieceTable::new(String::from("abcd\n"));
        pt.added = String::from("0123");
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

        pt.remove(7..8);
        assert_eq!(pt.line_at(0).content, "ab012cd3");
    }
}
