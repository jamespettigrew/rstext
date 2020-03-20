pub struct Cursor {
    pub row: usize,
    pub column: usize,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor { row: 0, column: 0 }
    }

    pub fn moved(&mut self, cursor_move: CursorMove) {
        match cursor_move.vertical {
            Some(VerticalMove::Up(rows)) => match self.row.checked_sub(rows) {
                Some(result) => self.row = result,
                None => self.row = 0,
            },
            Some(VerticalMove::Down(rows)) => match self.row.checked_add(rows) {
                Some(result) => self.row = result,
                None => self.row = usize::max_value(),
            },
            _ => (),
        }

        match cursor_move.horizontal {
            Some(HorizontalMove::Left(columns)) => match self.column.checked_sub(columns) {
                Some(result) => self.column = result,
                None => self.column = 0,
            },
            Some(HorizontalMove::Right(columns)) => match self.column.checked_add(columns) {
                Some(result) => self.column = result,
                None => self.column = usize::max_value(),
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
