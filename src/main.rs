use std::io::{stdout, Write};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue, style,
    style::Color,
    terminal,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

struct Line {
    character_buffer: Vec<char>,
}

impl Line {
    fn new() -> Line {
        Line {
            character_buffer: vec![],
        }
    }

    fn insert_character_at(&mut self, character: char, column: usize) {
        let column = column as usize;

        if column <= self.character_buffer.len() {
            self.character_buffer.insert(column, character);
        }
    }

    fn remove_character_at(&mut self, column: usize) {
        if column < self.character_buffer.len() {
            self.character_buffer.remove(column);
        }
    }
}

struct LineBuffer {
    lines: Vec<Line>,
}

impl LineBuffer {
    fn new() -> LineBuffer {
        LineBuffer {
            lines: vec![Line::new()],
        }
    }

    fn insert_line_at(&mut self, line: Line, row: usize) {
        if row <= self.lines.len() {
            self.lines.insert(row, line);
        }
    }

    fn line_at(&self, row: usize) -> Option<&Line> {
        if row < self.lines.len() {
            return Some(&self.lines[row]);
        }

        None
    }

    fn line_at_mut(&mut self, row: usize) -> Option<&mut Line> {
        if row < self.lines.len() {
            return Some(&mut self.lines[row]);
        }

        None
    }

    fn remove_line_at(&mut self, row: usize) {
        if row > 0 && row < self.lines.len() {
            self.lines.remove(row);
        }
    }
}

struct Cursor {
    row: usize,
    column: usize,
}

impl Cursor {
    fn new() -> Cursor {
        Cursor { row: 0, column: 0 }
    }

    fn moved(&mut self, cursor_move: CursorMove) {
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

enum VerticalMove {
    Up(usize),
    Down(usize),
}

enum HorizontalMove {
    Left(usize),
    Right(usize),
}

impl HorizontalMove {
    fn from_delta(delta: isize) -> Option<HorizontalMove> {
        if delta < 0 {
            Some(HorizontalMove::Left(delta.abs() as usize))
        } else if delta > 0 {
            Some(HorizontalMove::Right(delta.abs() as usize))
        } else {
            None
        }
    }
}

struct CursorMove {
    vertical: Option<VerticalMove>,
    horizontal: Option<HorizontalMove>,
}

struct Window {
    height: u16,
    width: u16,
    vertical_offset: usize,
    horizontal_offset: usize,
}

impl Window {
    fn new(height: u16, width: u16, vertical_offset: usize, horizontal_offset: usize) -> Window {
        Window {
            height,
            width,
            vertical_offset: 0,
            horizontal_offset: 0,
        }
    }

    fn resize(&mut self, height: u16, width: u16) {
        self.height = height;
        self.width = width;
    }

    fn bottom(&self) -> usize {
        match self.vertical_offset.checked_add(self.height as usize) {
            Some(result) => result,
            None => usize::max_value(),
        }
    }

    fn right(&self) -> usize {
        match self.horizontal_offset.checked_add(self.width as usize) {
            Some(result) => result,
            None => usize::max_value(),
        }
    }

    fn update_offsets_for_cursor(&mut self, cursor: &Cursor) {
        if cursor.row < self.vertical_offset {
            self.vertical_offset = cursor.row;
        }
        if cursor.row >= self.bottom() {
            self.vertical_offset += cursor.row - self.bottom() + 1;
        }
        if cursor.column < self.horizontal_offset {
            self.horizontal_offset = cursor.column;
        }
        if cursor.column >= self.right() {
            self.horizontal_offset += cursor.column - self.right() + 1;
        }
    }
}

fn main() -> Result<()> {
    execute!(stdout(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let line_buffer = &mut LineBuffer::new();
    let cursor = &mut Cursor::new();
    let window = &mut Window::new(0, 0, 0, 0);

    let screen = &mut stdout();

    loop {
        render(screen, line_buffer, cursor, window);

        if let Ok(Event::Key(event)) = event::read() {
            match event {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                } => break,
                KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: _,
                } => {
                    if cursor.column == 0 && cursor.row > 0 {
                        let current_line = &mut line_buffer
                            .line_at(cursor.row)
                            .expect("Missing expected line.")
                            .character_buffer
                            .clone();
                        let line_above = line_buffer
                            .line_at_mut(cursor.row - 1)
                            .expect("Missing expected line.");
                        let new_cursor_column = line_above.character_buffer.len();
                        line_above.character_buffer.append(current_line);
                        line_buffer.remove_line_at(cursor.row);

                        let column_delta = (new_cursor_column as isize) - (cursor.column as isize);
                        let cursor_move = CursorMove {
                            vertical: Some(VerticalMove::Up(1)),
                            horizontal: HorizontalMove::from_delta(column_delta),
                        };
                        cursor.moved(cursor_move);
                    } else if cursor.column > 0 {
                        let current_line = line_buffer
                            .line_at_mut(cursor.row)
                            .expect("Missing expected line.");
                        cursor.moved(CursorMove {
                            vertical: None,
                            horizontal: Some(HorizontalMove::Left(1)),
                        });
                        current_line.remove_character_at(cursor.column);
                    }
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: _,
                } => {
                    let current_line = line_buffer
                        .line_at_mut(cursor.row)
                        .expect("Missing expected line.");
                    let mut new_line = Line::new();
                    let characters_to_eol = current_line.character_buffer.drain(cursor.column..);
                    new_line.character_buffer.extend(characters_to_eol);

                    let column_delta = 0 - (cursor.column as isize);
                    let cursor_move = CursorMove {
                        vertical: Some(VerticalMove::Down(1)),
                        horizontal: HorizontalMove::from_delta(column_delta),
                    };
                    cursor.moved(cursor_move);
                    line_buffer.insert_line_at(new_line, cursor.row);
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: _,
                } => {
                    let current_line = line_buffer
                        .line_at_mut(cursor.row)
                        .expect("Missing expected line.");
                    current_line.insert_character_at(c, cursor.column);
                    let cursor_move = CursorMove {
                        vertical: None,
                        horizontal: Some(HorizontalMove::Right(1)),
                    };
                    cursor.moved(cursor_move);
                }
                KeyEvent {
                    code: KeyCode::Up,
                    modifiers: _,
                } => {
                    if cursor.row > 0 {
                        let line_above = line_buffer
                            .line_at(cursor.row - 1)
                            .expect("Missing expected line.");
                        let mut column_delta = 0;
                        if line_above.character_buffer.len() == 0
                            || (line_above.character_buffer.len() - 1) < cursor.column
                        {
                            column_delta = (line_above.character_buffer.len() as isize)
                                - (cursor.column as isize)
                        }

                        let cursor_move = CursorMove {
                            vertical: Some(VerticalMove::Up(1)),
                            horizontal: HorizontalMove::from_delta(column_delta),
                        };
                        cursor.moved(cursor_move);
                    }
                }
                KeyEvent {
                    code: KeyCode::Down,
                    modifiers: _,
                } => {
                    if cursor.row < line_buffer.lines.len() - 1 {
                        let line_below = line_buffer
                            .line_at(cursor.row + 1)
                            .expect("Missing expected line.");
                        let mut column_delta = 0;
                        if line_below.character_buffer.len() == 0
                            || (line_below.character_buffer.len() - 1) < cursor.column
                        {
                            column_delta = (line_below.character_buffer.len() as isize)
                                - (cursor.column as isize)
                        }

                        let cursor_move = CursorMove {
                            vertical: Some(VerticalMove::Down(1)),
                            horizontal: HorizontalMove::from_delta(column_delta),
                        };
                        cursor.moved(cursor_move);
                    }
                }
                KeyEvent {
                    code: KeyCode::Left,
                    modifiers: _,
                } => {
                    if cursor.column > 0 {
                        cursor.moved(CursorMove {
                            vertical: None,
                            horizontal: Some(HorizontalMove::Left(1)),
                        });
                    } else if cursor.row > 0 {
                        let line_above = line_buffer
                            .line_at(cursor.row - 1)
                            .expect("Missing expected line.");
                        let column_delta =
                            (line_above.character_buffer.len() as isize) - (cursor.column as isize);
                        let cursor_move = CursorMove {
                            vertical: Some(VerticalMove::Up(1)),
                            horizontal: HorizontalMove::from_delta(column_delta),
                        };
                        cursor.moved(cursor_move);
                    }
                }
                KeyEvent {
                    code: KeyCode::Right,
                    modifiers: _,
                } => {
                    let current_line = line_buffer
                        .line_at(cursor.row)
                        .expect("Missing expected line.");
                    if cursor.column < current_line.character_buffer.len() {
                        cursor.moved(CursorMove {
                            vertical: None,
                            horizontal: Some(HorizontalMove::Right(1)),
                        });
                    } else if cursor.row < line_buffer.lines.len() - 1 {
                        let column_delta = 0 - (cursor.column as isize);
                        let cursor_move = CursorMove {
                            vertical: Some(VerticalMove::Down(1)),
                            horizontal: HorizontalMove::from_delta(column_delta),
                        };
                        cursor.moved(cursor_move);
                    }
                }
                _ => (),
            }
        }
    }

    execute!(stdout(), LeaveAlternateScreen)?;
    terminal::disable_raw_mode()
}

fn render(screen: &mut impl Write, line_buffer: &LineBuffer, cursor: &Cursor, window: &mut Window) {
    queue!(screen, Clear(ClearType::All));

    let (terminal_width, terminal_height) =
        terminal::size().expect("Failed to get terminal size.");

    // Number of columns the display of line numbers will require: max(3, num_digits) + 1 space
    let min_line_number_columns = 3usize;
    let line_number_digits = line_buffer.lines.len().to_string().len();
    let line_number_columns =
        (std::cmp::max(min_line_number_columns, line_number_digits) + 1) as u16;

    window.resize(terminal_height, terminal_width - line_number_columns);
    window.update_offsets_for_cursor(cursor);

    let line_iter = line_buffer
        .lines
        .iter()
        .enumerate()
        .skip(window.vertical_offset)
        .take(window.height as usize);

    let mut row_count = 0;
    for (row_index, line) in line_iter {
        queue!(
            screen,
            MoveTo(0, row_count as u16),
            style::SetForegroundColor(Color::Blue),
            style::Print(format!(
                "{:>min_width$}",
                row_index + 1,
                min_width = min_line_number_columns
            )),
            style::ResetColor,
            MoveTo(line_number_columns, row_count as u16)
        );

        let character_iter = line
            .character_buffer
            .iter()
            .skip(window.horizontal_offset)
            .take(window.width as usize);
        for character in character_iter {
            queue!(screen, style::Print(character));
        }

        row_count += 1;
    }

    let virtual_cursor_row = cursor.row - window.vertical_offset;
    let virtual_cursor_column =
        line_number_columns + ((cursor.column - window.horizontal_offset) as u16);
    queue!(
        screen,
        MoveTo(virtual_cursor_column, virtual_cursor_row as u16)
    );
    screen.flush().unwrap();
}
