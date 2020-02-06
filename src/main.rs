extern crate termion;

use termion::raw::RawTerminal;
use std::io::{stdin, stdout, Stdout, Write};
use termion::clear;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

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
                            let current_line = &mut line_buffer.line_at(cursor.row).expect("Missing expected line.").character_buffer.clone();
                            let line_above = line_buffer.line_at_mut(cursor.row - 1).expect("Missing expected line.");
                            cursor.column = line_above.character_buffer.len();
                            line_above.character_buffer.append(current_line);
                            line_buffer.remove_line_at(cursor.row);
                            cursor.row -= 1;
                    } else if cursor.column > 0 {
                        let current_line = line_buffer.line_at_mut(cursor.row).expect("Missing expected line.");
                        cursor.column -= 1;
                        current_line.remove_character_at(cursor.column);
                    }
                }
                Key::Char(c) => {
                    if c == '\n' {
                        let current_line = line_buffer.line_at_mut(cursor.row).expect("Missing expected line.");
                        let mut new_line = Line::new();
                        let characters_to_eol = current_line.character_buffer.drain(cursor.column..);
                        new_line.character_buffer.extend(characters_to_eol);
                        cursor.column = 0;
                        cursor.row += 1;
                        line_buffer.insert_line_at(new_line, cursor.row);
                    } else {
                        let current_line = line_buffer.line_at_mut(cursor.row).expect("Missing expected line.");
                        current_line.insert_character_at(c, cursor.column);
                        cursor.column += 1;
                    }
                },
                Key::Up => {
                    if cursor.row > 0 {
                        cursor.row -= 1;
                    }
                },
                Key::Down => {
                    if cursor.row < line_buffer.lines.len() - 1 {
                        cursor.row += 1;
                    }
                },
                Key::Left => {
                    if cursor.column > 0 {
                        cursor.column -= 1;
                    } else if cursor.row > 0 {
                        cursor.row -= 1;
                        let current_line = line_buffer.line_at(cursor.row).expect("Missing expected line.");
                        cursor.column = current_line.character_buffer.len();
                    }
                },
                Key::Right => {
                    let current_line = line_buffer.line_at(cursor.row).expect("Missing expected line.");
                    if cursor.column < current_line.character_buffer.len() {
                        cursor.column += 1;
                    } else if cursor.row < line_buffer.lines.len() - 1 {
                        cursor.column = 0;
                        cursor.row += 1;
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
    for row_index in 0..(&line_buffer.lines).len() {
        write!(stdout, "{}", cursor::Goto(1, (row_index + 1) as u16));
        for character in &line_buffer.lines[row_index].character_buffer {
            print!("{}", character);
        }
        stdout.flush().unwrap();
    }
    write!(stdout, "{}", cursor::Goto((cursor.column + 1) as u16, (cursor.row + 1) as u16));
    stdout.flush().unwrap();
}
