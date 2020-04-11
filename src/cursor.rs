pub struct Cursor {
    pub line: usize,
    pub character: usize,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor { line: 0, character: 0 }
    }

    pub fn moved(&mut self, cursor_move: CursorMove) {
        match cursor_move.vertical {
            Some(VerticalMove::Up(rows)) => match self.line.checked_sub(rows) {
                Some(result) => self.line = result,
                None => self.line = 0,
            },
            Some(VerticalMove::Down(rows)) => match self.line.checked_add(rows) {
                Some(result) => self.line = result,
                None => self.line = usize::max_value(),
            },
            _ => (),
        }

        match cursor_move.horizontal {
            Some(HorizontalMove::Left(columns)) => match self.character.checked_sub(columns) {
                Some(result) => self.character = result,
                None => self.character = 0,
            },
            Some(HorizontalMove::Right(columns)) => match self.character.checked_add(columns) {
                Some(result) => self.character = result,
                None => self.character = usize::max_value(),
            },
            _ => (),
        }
    }
}

pub enum VerticalMove {
    Up(usize),
    Down(usize),
}

pub enum HorizontalMove {
    Left(usize),
    Right(usize),
}

impl HorizontalMove {
    pub fn from_delta(delta: isize) -> Option<HorizontalMove> {
        if delta < 0 {
            Some(HorizontalMove::Left(delta.abs() as usize))
        } else if delta > 0 {
            Some(HorizontalMove::Right(delta.abs() as usize))
        } else {
            None
        }
    }
}

pub struct CursorMove {
    pub vertical: Option<VerticalMove>,
    pub horizontal: Option<HorizontalMove>,
}
