#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Buffer {
    Added,
    Original
}

#[derive(Debug, Eq, PartialEq)]
pub struct Piece {
    /// Associated PieceTable buffer.
    pub buffer: Buffer,
    /// Byte index of piece start within buffer.
    pub start: usize,
    /// Length (in bytes, from start index) of piece within buffer.
    pub length: usize,
    /// Index offsets (from start) of line breaks within buffer region spanned by this piece.
    pub line_break_offsets: Vec<usize>,
}

impl Piece {
    pub fn extend(&self, s: &str) -> Piece {
        let s_line_break_offsets = line_break_offsets(s)
            .iter()
            .map(|x| x + self.length)
            .collect::<Vec<usize>>();
        let new_line_break_offsets = self
            .line_break_offsets
            .iter()
            .map(|x| *x)
            .chain(s_line_break_offsets)
            .collect();

        Piece {
            buffer: self.buffer,
            start: self.start,
            length: self.length + s.len(),
            line_break_offsets: new_line_break_offsets
        }
    }

    pub fn split_at(&self, idx: usize) -> (Self, Self) {
        let left_line_break_offsets = self
            .line_break_offsets
            .iter()
            .map(|x| *x)
            .take_while(|x| *x < idx)
            .collect::<Vec<usize>>();
        let right_line_break_offsets = self
            .line_break_offsets[left_line_break_offsets.len()..]
            .iter()
            .map(|x| x.checked_sub(idx))
            .filter_map(|x| x)
            .collect::<Vec<usize>>();

        let left = Self {
            buffer: self.buffer,
            start: self.start,
            length: idx,
            line_break_offsets: left_line_break_offsets
        };
        let right = Self {
            buffer: self.buffer,
            start: self.start + idx,
            length: self.length - idx,
            line_break_offsets: right_line_break_offsets
        };

        (left, right)
    }

    pub fn truncate_left(&self, len: usize) -> Self {
        let line_break_offsets = self
            .line_break_offsets
            .iter()
            .map(|x| x.checked_sub(len))
            .filter_map(|x| x)
            .collect::<Vec<usize>>();

        Self {
            buffer: self.buffer,
            start: self.start + len,
            length: self.length - len,
            line_break_offsets,
        }
    }

    pub fn truncate_right(&self, len: usize) -> Self {
        let line_break_offsets = self
            .line_break_offsets
            .iter()
            .map(|x| *x)
            .take_while(|x| *x <= self.length - len)
            .collect::<Vec<usize>>();

        Self {
            buffer: self.buffer,
            start: self.start,
            length: self.length - len,
            line_break_offsets
        }
    }
}

pub fn line_break_offsets(s: &str) -> Vec<usize> {
    s.bytes()
        .enumerate()
        .filter_map(|(i, b)| match b {
            0x0A => Some(i),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extend()
    {
        let original = Piece
        {
            buffer: Buffer::Original,
            start: 3,
            length: 10,
            line_break_offsets: vec![2, 5]
        };

        let expected = Piece
        {
            buffer: Buffer::Original,
            start: 3,
            length: 13,
            line_break_offsets: vec![2, 5, 11]
        };
        assert_eq!(expected, original.extend("a\nb"));
    }

    #[test]
    fn line_break_offsets_correct() {
        let mut line = String::from("");
        let mut offsets = line_break_offsets(&line);
        assert_eq!(vec![0usize; 0], offsets);

        line = String::from("abc\ndef\nghijk\nl");
        offsets = line_break_offsets(&line);
        assert_eq!(vec![3, 7, 13], offsets);
    }

    #[test]
    fn split_at()
    {
        let original = Piece
        {
            buffer: Buffer::Original,
            start: 3,
            length: 17,
            line_break_offsets: vec![2, 5, 8, 13]
        };
        let (left, right) = original.split_at(7);

        let expected_left = Piece
        {
            buffer: Buffer::Original,
            start: 3,
            length: 7,
            line_break_offsets: vec![2, 5]
        };
        assert_eq!(expected_left, left);

        let expected_right = Piece
        {
            buffer: Buffer::Original,
            start: 10,
            length: 10,
            line_break_offsets: vec![1, 6]
        };
        assert_eq!(expected_right, right);
    }

    #[test]
    fn truncate_left()
    {
        let original = Piece
        {
            buffer: Buffer::Original,
            start: 3,
            length: 10,
            line_break_offsets: vec![2, 5]
        };

        let expected = Piece
        {
            buffer: Buffer::Original,
            start: 6,
            length: 7,
            line_break_offsets: vec![2]
        };
        assert_eq!(expected, original.truncate_left(3));
    }

    #[test]
    fn truncate_right()
    {
        let original = Piece
        {
            buffer: Buffer::Original,
            start: 3,
            length: 10,
            line_break_offsets: vec![2, 5, 8]
        };

        let expected = Piece
        {
            buffer: Buffer::Original,
            start: 3,
            length: 7,
            line_break_offsets: vec![2, 5]
        };
        assert_eq!(expected, original.truncate_right(3));
    }
}
