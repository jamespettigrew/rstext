use std::iter::Iterator;
use std::ops::{Index, Range};

use Buffer::*;
use IndexLocation::*;

trait TextBuffer {
    fn insert_item_at(&mut self, item: char, index: usize);
    fn insert_items_at(&mut self, items: Vec<char>, index: usize);
    fn line_count(&self) -> usize;
    fn remove_item_at(&mut self, index: usize);
    fn remove_items(&mut self, range: Range<usize>);
}

struct PieceTable<T>
where
    T: Copy,
{
    original: Vec<T>,
    added: Vec<T>,
    pieces: Vec<Piece>,
    length: usize,
}

#[derive(Copy, Clone)]
enum Buffer {
    Original,
    Added,
}

#[derive(Copy, Clone)]
struct Piece {
    buffer: Buffer,
    start: usize,
    length: usize,
}

enum IndexLocation {
    PieceHead(usize),
    PieceBody(usize, usize),
    PieceTail(usize),
    EOF,
}

impl<T> PieceTable<T>
where
    T: Copy,
{
    fn new(content: Vec<T>) -> PieceTable<T> {
        PieceTable {
            length: content.len(),
            pieces: vec![Piece {
                buffer: Buffer::Original,
                start: 0,
                length: content.len(),
            }],
            original: content,
            added: Vec::new(),
        }
    }

    fn iter<'a>(&'a self) -> PieceTableIter<'a, T> {
        PieceTableIter {
            inner: self,
            current_piece_index: 0,
            current_piece_offset: 0,
            end_piece_index: self.pieces.len() - 1,
            end_piece_offset: self.pieces.last().map_or_else(|| 0, |p| p.length - 1),
        }
    }

    fn iter_range<'a>(&'a self, range: Range<usize>) -> PieceTableIter<'a, T> {
        let start_location = self.index_location(range.start);
        let end_location = self.index_location(range.end - 1);

        let (start_piece_index, start_piece_offset) = match start_location {
            PieceHead(piece_index) => (piece_index, 0),
            PieceBody(piece_index, piece_offset) => (piece_index, piece_offset),
            PieceTail(piece_index) => (piece_index, self.pieces[piece_index].length - 1),
            EOF => panic!("Start index of range")
        };

        let (end_piece_index, end_piece_offset) = match end_location {
            PieceHead(piece_index) => (piece_index, 0),
            PieceBody(piece_index, piece_offset) => (piece_index, piece_offset),
            PieceTail(piece_index) => (piece_index, self.pieces[piece_index].length - 1),
            EOF => (self.pieces.len() - 1, self.pieces.last().map_or_else(|| 0, |p| p.length - 1))
        };

        PieceTableIter {
            inner: self,
            current_piece_index: start_piece_index,
            current_piece_offset: start_piece_offset,
            end_piece_index: end_piece_index,
            end_piece_offset: end_piece_offset,
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
}

impl TextBuffer for PieceTable<char> {
    fn insert_item_at(&mut self, item: char, index: usize) {
        self.insert_items_at(vec![item], index);
    }

    fn insert_items_at(&mut self, items: Vec<char>, index: usize) {
        let location = self.index_location(index);
        let new_piece = Piece {
            buffer: Buffer::Added,
            start: self.added.len(),
            length: items.len(),
        };

        match location {
            PieceHead(piece_index) => self.pieces.insert(piece_index, new_piece),
            PieceBody(piece_index, offset) => {
                let original_piece = &mut self.pieces[piece_index];
                let offcut_piece = Piece {
                    buffer: original_piece.buffer,
                    start: original_piece.start + offset,
                    length: original_piece.length - offset,
                };
                original_piece.length = offset;
                self.pieces.insert(piece_index + 1, new_piece);
                self.pieces.insert(piece_index + 2, offcut_piece);
            }
            PieceTail(piece_index) => {
                let original_piece = &mut self.pieces[piece_index];
                let offcut_piece = Piece {
                    buffer: original_piece.buffer,
                    start: original_piece.start + original_piece.length - 1,
                    length: 1,
                };
                original_piece.length -= 1;
                self.pieces.insert(piece_index + 1, new_piece);
                self.pieces.insert(piece_index + 2, offcut_piece);
            }
            EOF => self.pieces.push(new_piece),
        }
        self.length += items.len();
        self.added.extend(items);
    }

    // TODO: Support different line endings
    fn line_count(&self) -> usize {
        let mut count = 1;
        for item in self.iter() {
            match item {
                '\n' => count += 1,
                _ => {}
            }
        }

        count
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
                    self.pieces[start_piece_index].length = piece_offset;
                }
                PieceTail(_) => {
                    self.pieces[start_piece_index].length -= 1;
                }
                EOF => {}
            };

            match end_location {
                PieceHead(_) => {
                    let piece = &mut self.pieces[end_piece_index];
                    piece.start += 1;
                    piece.length -= 1;
                    if piece.length == 0 {
                        self.pieces.remove(end_piece_index);
                    }
                }
                PieceBody(_, piece_offset) => {
                    let piece = &mut self.pieces[end_piece_index];
                    piece.start += piece_offset + 1;
                    piece.length -= piece_offset + 1;
                }
                PieceTail(_) => {
                    self.pieces.remove(end_piece_index);
                }
                EOF => {}
            };
        } else if start_piece_index == end_piece_index {
            match (start_location, end_location) {
                (PieceHead(_), PieceHead(_)) => {
                    let piece = &mut self.pieces[start_piece_index];
                    piece.start += 1;
                    piece.length -= 1;
                    if piece.length == 0 {
                        self.pieces.remove(end_piece_index);
                    }
                }
                (PieceHead(_), PieceBody(_, piece_offset)) => {
                    let piece = &mut self.pieces[start_piece_index];
                    piece.start += piece_offset + 1;
                    piece.length -= piece_offset + 1;
                }
                (PieceBody(_, start_offset), PieceBody(_, end_offset)) => {
                    let left_piece = &mut self.pieces[start_piece_index];
                    let right_piece = Piece {
                        buffer: left_piece.buffer,
                        start: left_piece.start + end_offset + 1,
                        length: left_piece.length - end_offset - 1,
                    };
                    left_piece.length = start_offset;
                    self.pieces.insert(start_piece_index + 1, right_piece);
                }
                (PieceHead(_), PieceTail(_)) => {
                    self.pieces.remove(start_piece_index);
                }
                (PieceBody(_, start_offset), PieceTail(_)) => {
                    let piece = &mut self.pieces[start_piece_index];
                    piece.length = start_offset + 1;
                }
                _ => panic!(),
            }
        }
        self.length -= (range.end - 1) - range.start;
    }
}

struct PieceTableIter<'a, T>
where
    T: Copy,
{
    inner: &'a PieceTable<T>,
    current_piece_index: usize,
    current_piece_offset: usize,
    end_piece_index: usize,
    end_piece_offset: usize,
}

impl<T> Index<usize> for PieceTable<T>
where
    T: Copy,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let mut index_count = 0usize;
        for piece in self.pieces.iter() {
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

impl<'a, T> Iterator for PieceTableIter<'a, T>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_piece_index == self.end_piece_index
            && self.current_piece_offset > self.end_piece_offset
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
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 2,
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
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
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 2,
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
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
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 2,
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
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
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 2,
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
            },
        ];

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
            },
            Piece {
                buffer: Buffer::Added,
                start: 0,
                length: 3,
            },
            Piece {
                buffer: Buffer::Original,
                start: 2,
                length: 2,
            },
            Piece {
                buffer: Buffer::Added,
                start: 3,
                length: 1,
            },
        ];

        pt.remove_item_at(3);
        assert_eq!(
            pt.iter().collect::<Vec<char>>(),
            vec!['a', 'b', '0', '2', 'c', 'd', '3']
        );

        pt.remove_items(1..6);
        assert_eq!(pt.iter().collect::<Vec<char>>(), vec!['a', '3']);
    }
}
