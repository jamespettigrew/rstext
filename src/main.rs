extern crate termion;

use std::io::{stdin, stdout, Stdout, Write};
use termion::{ clear, color, cursor };
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;

struct Line {
    character_buffer: Vec<char>
}

impl Line {
    fn new() -> Line {
        Line { character_buffer: vec![] }
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

struct LineBuffer<> {
    lines: Vec<Line>
}

impl LineBuffer {
    fn new() -> LineBuffer {
        LineBuffer { lines: vec![Line::new()] }
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
    column: usize
}

impl Cursor {
    fn new() -> Cursor {
        Cursor { row: 0, column: 0 }
    }

    fn moved(&mut self, cursor_move: CursorMove) {
        match cursor_move.vertical {
            Some(VerticalMove::Up(rows)) => {
                match self.row.checked_sub(rows) {
                    Some(result) => self.row = result,
                    None => self.row = 0
                }
            },
            Some(VerticalMove::Down(rows)) => {
                match self.row.checked_add(rows) {
                    Some(result) => self.row = result,
                    None => self.row = usize::max_value()
                }
            },
            _ => ()
        }

        match cursor_move.horizontal {
            Some(HorizontalMove::Left(columns)) => {
                match self.column.checked_sub(columns) {
                    Some(result) => self.column = result,
                    None => self.column = 0
                }
            },
            Some(HorizontalMove::Right(columns)) => {
                match self.column.checked_add(columns) {
                    Some(result) => self.column = result,
                    None => self.column = usize::max_value()
                }
            },
            _ => ()
        }
    }
}

enum VerticalMove {
    Up(usize),
    Down(usize)
}

enum HorizontalMove {
    Left(usize),
    Right(usize)
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
    horizontal: Option<HorizontalMove>
}

struct Window {
    height: u16,
    width: u16,
    vertical_offset: usize,
    horizontal_offset: usize
}

impl Window {
    fn new(height: u16, width: u16, vertical_offset: usize, horizontal_offset: usize) -> Window {
        Window { height, width, vertical_offset: 0, horizontal_offset: 0 }
    }

    fn resize(&mut self, height: u16, width: u16) {
        self.height = height;
        self.width = width;
    }

    fn bottom(&self) -> usize {
        match self.vertical_offset.checked_add(self.height as usize) {
            Some(result) => result,
            None => usize::max_value()
        }
    }

    fn right(&self) -> usize {
        match self.horizontal_offset.checked_add(self.width as usize) {
            Some(result) => result,
            None => usize::max_value()
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

fn main() {
    let mut stdin = stdin().keys();
    let stdout = &mut stdout().into_raw_mode().expect("Failed to set tty to raw mode");
    let line_buffer = &mut LineBuffer::new();
    let cursor = &mut Cursor::new();
    
    loop {
        render(stdout, line_buffer, cursor);

        let input = stdin.next();
        if let Some(Ok(key)) = input
        {
            match key {
                Key::Ctrl('q') => break,
                Key::Backspace => {
                    if cursor.column == 0 && cursor.row > 0 {
                        let current_line = &mut line_buffer
                            .line_at(cursor.row)
                            .expect("Missing expected line.")
                            .character_buffer
                            .clone();
                        let line_above = line_buffer.line_at_mut(cursor.row - 1).expect("Missing expected line.");
                        let new_cursor_column= line_above.character_buffer.len();
                        line_above.character_buffer.append(current_line);
                        line_buffer.remove_line_at(cursor.row);

                        let column_delta = (new_cursor_column as isize) - (cursor.column as isize);
                        let cursor_move = CursorMove {
                            vertical: Some(VerticalMove::Up(1)),
                            horizontal: HorizontalMove::from_delta(column_delta)
                        };
                        cursor.moved(cursor_move);
                    } else if cursor.column > 0 {
                        let current_line = line_buffer.line_at_mut(cursor.row).expect("Missing expected line.");
                        cursor.moved(CursorMove { vertical: None, horizontal: Some(HorizontalMove::Left(1)) });
                        current_line.remove_character_at(cursor.column);
                    }
                }
                Key::Char(c) => {
                    if c == '\n' {
                        let current_line = line_buffer.line_at_mut(cursor.row).expect("Missing expected line.");
                        let mut new_line = Line::new();
                        let characters_to_eol = current_line.character_buffer.drain(cursor.column..);
                        new_line.character_buffer.extend(characters_to_eol);

                        let column_delta = 0 - (cursor.column as isize);
                        let cursor_move = CursorMove {
                            vertical: Some(VerticalMove::Down(1)),
                            horizontal: HorizontalMove::from_delta(column_delta)
                        };
                        cursor.moved(cursor_move);
                        line_buffer.insert_line_at(new_line, cursor.row);
                    } else {
                        let current_line = line_buffer.line_at_mut(cursor.row).expect("Missing expected line.");
                        current_line.insert_character_at(c, cursor.column);
                        let cursor_move = CursorMove {
                            vertical: None,
                            horizontal: Some(HorizontalMove::Right(1))
                        };
                        cursor.moved(cursor_move);
                    }
                },
                Key::Up => {
                    if cursor.row > 0 {
                        let line_above = line_buffer.line_at(cursor.row - 1).expect("Missing expected line.");
                        let mut column_delta = 0;
                        if line_above.character_buffer.len() == 0 ||
                                (line_above.character_buffer.len() - 1) < cursor.column {
                            column_delta = (line_above.character_buffer.len() as isize) - (cursor.column as isize)
                        }
                        
                        let cursor_move = CursorMove { 
                            vertical: Some(VerticalMove::Up(1)),
                            horizontal: HorizontalMove::from_delta(column_delta)
                        }; 
                        cursor.moved(cursor_move);
                    }
                },
                Key::Down => {
                    if cursor.row < line_buffer.lines.len() - 1 {
                        let line_below = line_buffer.line_at(cursor.row + 1).expect("Missing expected line.");
                        let mut column_delta = 0;
                        if line_below.character_buffer.len() == 0 ||
                                (line_below.character_buffer.len() - 1) < cursor.column {
                            column_delta = (line_below.character_buffer.len() as isize) - (cursor.column as isize)
                        }
                        
                        let cursor_move = CursorMove { 
                            vertical: Some(VerticalMove::Down(1)),
                            horizontal: HorizontalMove::from_delta(column_delta)
                        }; 
                        cursor.moved(cursor_move);
                    }
                },
                Key::Left => {
                    if cursor.column > 0 {
                        cursor.moved(CursorMove { vertical: None, horizontal: Some(HorizontalMove::Left(1)) });
                    } else if cursor.row > 0 {
                        let line_above = line_buffer.line_at(cursor.row - 1).expect("Missing expected line.");
                        let column_delta = (line_above.character_buffer.len() as isize) - (cursor.column as isize);
                        let cursor_move = CursorMove { 
                            vertical: Some(VerticalMove::Up(1)),
                            horizontal: HorizontalMove::from_delta(column_delta)
                        }; 
                        cursor.moved(cursor_move);
                    }
                },
                Key::Right => {
                    let current_line = line_buffer.line_at(cursor.row).expect("Missing expected line.");
                    if cursor.column < current_line.character_buffer.len() {
                        cursor.moved(CursorMove { vertical: None, horizontal: Some(HorizontalMove::Right(1)) });
                    } else if cursor.row < line_buffer.lines.len() - 1 {
                        let column_delta = 0 - ( cursor.column as isize);
                        let cursor_move = CursorMove { 
                            vertical: Some(VerticalMove::Down(1)),
                            horizontal: HorizontalMove::from_delta(column_delta)
                        }; 
                        cursor.moved(cursor_move);
                    }
                },
                _ => ()
            }
        }
    }

    stdout.suspend_raw_mode().expect("Failed to restore tty to original state.");
}

fn render(stdout: &mut RawTerminal<Stdout>, line_buffer: &LineBuffer, cursor: &Cursor) {
    write!(stdout, "{}", clear::All);

    let (terminal_width, terminal_height) = termion::terminal_size().expect("Failed to get terminal size.");
    let line_number_columns = (line_buffer.lines.len().to_string().len() + 1) as u16;
    let window = &mut Window::new(terminal_height, terminal_width - line_number_columns, 0, 0);
    window.update_offsets_for_cursor(cursor);
    
    let line_iter = line_buffer.lines.iter().enumerate().skip(window.vertical_offset).take(window.height as usize);
    let mut row_count = 0;
    for (row_index, line) in line_iter {
        write!(stdout, "{}", cursor::Goto(1, (row_count + 1) as u16));
        write!(stdout, "{}{}{}{}", color::Fg(color::Blue), row_index + 1, color::Fg(color::Reset), " ");

        write!(stdout, "{}", cursor::Goto(line_number_columns + 1, (row_count + 1) as u16));
        let character_iter = line.character_buffer.iter().skip(window.horizontal_offset).take(window.width as usize);
        for character in character_iter {
            write!(stdout, "{}", character);
        }

        row_count += 1;
    }

    let virtual_cursor_row = cursor.row - window.vertical_offset + 1;
    let virtual_cursor_column = line_number_columns + ((cursor.column - window.horizontal_offset + 1) as u16);
    write!(stdout, "{}", cursor::Goto(virtual_cursor_column, virtual_cursor_row as u16));
    stdout.flush().unwrap();
}
