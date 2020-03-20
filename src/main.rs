mod piece_table;
use piece_table::{ PieceTable, TextBuffer };
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

    let text_buffer= &mut PieceTable::new(vec![]);
    let cursor = &mut Cursor::new();
    let window = &mut Window::new(0, 0, 0, 0);

    let screen = &mut stdout();

    loop {
        render(screen, text_buffer, cursor, window);

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
                        let line_above = text_buffer.line_at(cursor.row - 1);
                        let new_cursor_column = line_above.content.len();
                        text_buffer.remove_item_at(line_above.start_index + line_above.content.len());

                        let column_delta = (new_cursor_column as isize) - (cursor.column as isize);
                        let cursor_move = CursorMove {
                            vertical: Some(VerticalMove::Up(1)),
                            horizontal: HorizontalMove::from_delta(column_delta),
                        };
                        cursor.moved(cursor_move);
                    } else if cursor.column > 0 {
                        let current_line = text_buffer.line_at(cursor.row);
                        cursor.moved(CursorMove {
                            vertical: None,
                            horizontal: Some(HorizontalMove::Left(1)),
                        });
                        text_buffer.remove_item_at(current_line.start_index + cursor.column);
                    }
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: _,
                } => {
                    let current_line = text_buffer.line_at(cursor.row);
                    text_buffer.insert_item_at('\n', current_line.start_index + cursor.column);

                    let column_delta = 0 - (cursor.column as isize);
                    let cursor_move = CursorMove {
                        vertical: Some(VerticalMove::Down(1)),
                        horizontal: HorizontalMove::from_delta(column_delta),
                    };
                    cursor.moved(cursor_move);
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: _,
                } => {
                    let current_line = text_buffer.line_at(cursor.row);
                    text_buffer.insert_item_at(c, current_line.start_index + cursor.column);
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
                        let line_above = text_buffer.line_at(cursor.row - 1);
                        let mut column_delta = 0;
                        if line_above.content.len() == 0
                            || (line_above.content.len() - 1) < cursor.column
                        {
                            column_delta = (line_above.content.len() as isize)
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
                    if cursor.row < text_buffer.line_count() - 1 {
                        let line_below = text_buffer.line_at(cursor.row + 1);
                        let mut column_delta = 0;
                        if line_below.content.len() == 0
                            || (line_below.content.len() - 1) < cursor.column
                        {
                            column_delta = (line_below.content.len() as isize)
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
                        let line_above = text_buffer.line_at(cursor.row - 1);
                        let column_delta =
                            (line_above.content.len() as isize) - (cursor.column as isize);
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
                    let current_line = text_buffer.line_at(cursor.row);
                    if cursor.column < current_line.content.len() {
                        cursor.moved(CursorMove {
                            vertical: None,
                            horizontal: Some(HorizontalMove::Right(1)),
                        });
                    } else if cursor.row < text_buffer.line_count() - 1 {
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

fn render(screen: &mut impl Write, text_buffer: &impl TextBuffer, cursor: &Cursor, window: &mut Window) {
    queue!(screen, Clear(ClearType::All));

    let (terminal_width, terminal_height) =
        terminal::size().expect("Failed to get terminal size.");

    // Number of columns the display of line numbers will require: max(3, num_digits) + 1 space
    let min_line_number_columns = 3usize;
    let line_number_digits = text_buffer.line_count().to_string().len();
    let line_number_columns =
        (std::cmp::max(min_line_number_columns, line_number_digits) + 1) as u16;

    window.resize(terminal_height, terminal_width - line_number_columns);
    window.update_offsets_for_cursor(cursor);

    let last_line= std::cmp::min(window.bottom(), text_buffer.line_count());
    let line_range = window.vertical_offset..last_line;
    let line_iter = line_range.map(|x| (x, text_buffer.line_at(x)));
    
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
            .content
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
