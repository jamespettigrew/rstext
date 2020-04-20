use crate::cursor::{Cursor, CursorMove, HorizontalMove, VerticalMove};
use crate::file;
use crate::renderer;
use crate::text_buffer::piece_table::PieceTable;
use crate::text_buffer::TextBuffer;
use crate::window::Window;
use std::io::{stdout, Write};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, terminal,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

pub struct Editor<'a> {
    cursor: Cursor,
    file_path: Option<&'a str>,
    running: bool,
    text_buffer: PieceTable,
    window: Window,
}

impl<'a> Editor<'a> {
    pub fn new(file_path: Option<&'a str>) -> Editor<'a> {
        let file_contents = match file_path {
            Some(path) => file::load(path).unwrap_or(Vec::new()),
            _ => Vec::new(),
        };

        let text_buffer = PieceTable::new(file_contents);
        let cursor = Cursor::new();
        let window = Window::new(0, 0, 0, 0);
        let screen = stdout();

        Editor {
            cursor,
            file_path,
            running: false,
            text_buffer,
            window,
        }
    }

    pub fn start(&mut self) {
        self.running = true;

        execute!(stdout(), EnterAlternateScreen);
        terminal::enable_raw_mode();

        while self.running {
            renderer::render(
                &mut stdout(),
                &mut self.text_buffer,
                &mut self.cursor,
                &mut self.window,
            );

            if let Ok(Event::Key(event)) = event::read() {
                let command = self.map_key_to_command(event);
                if let Some(command) = command {
                    self.execute_command(command);
                }
            }
        }

        execute!(stdout(), LeaveAlternateScreen);
        terminal::disable_raw_mode();
    }

    fn execute_command(&mut self, command: Command) {
        match command {
            Command::CursorBackward => self.cursor_backward(),
            Command::CursorDown => self.cursor_down(),
            Command::CursorForward => self.cursor_forward(),
            Command::CursorUp => self.cursor_up(),
            Command::DeleteBackward => self.delete_backward(),
            Command::Exit => self.exit(),
            Command::InsertCharacter(c) => self.insert_character(c),
            Command::InsertNewLine => self.insert_newline(),
            Command::Save => self.save(),
        }
    }

    fn map_key_to_command(&self, key_event: KeyEvent) -> Option<Command> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => Some(Command::Exit),
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => Some(Command::Save),
            (KeyCode::Char(c), _) => Some(Command::InsertCharacter(c)),
            (KeyCode::Backspace, _) => Some(Command::DeleteBackward),
            (KeyCode::Down, _) => Some(Command::CursorDown),
            (KeyCode::Enter, _) => Some(Command::InsertNewLine),
            (KeyCode::Left, _) => Some(Command::CursorBackward),
            (KeyCode::Right, _) => Some(Command::CursorForward),
            (KeyCode::Up, _) => Some(Command::CursorUp),
            _ => None,
        }
    }

    fn cursor_backward(&mut self) {
        if self.cursor.character > 0 {
            self.cursor.moved(CursorMove {
                vertical: None,
                horizontal: Some(HorizontalMove::Left(1)),
            });
        } else if self.cursor.line > 0 {
            let line_above = self.text_buffer.line_at(self.cursor.line - 1);
            let column_delta =
                (line_above.characters.len() as isize) - (self.cursor.character as isize);
            let cursor_move = CursorMove {
                vertical: Some(VerticalMove::Up(1)),
                horizontal: HorizontalMove::from_delta(column_delta),
            };
            self.cursor.moved(cursor_move);
        }
    }

    fn cursor_down(&mut self) {
        if self.cursor.line < self.text_buffer.line_count() - 1 {
            let line_below = self.text_buffer.line_at(self.cursor.line + 1);
            let mut column_delta = 0;
            if line_below.characters.len() == 0
                || (line_below.characters.len() - 1) < self.cursor.character
            {
                column_delta =
                    (line_below.characters.len() as isize) - (self.cursor.character as isize)
            }

            let cursor_move = CursorMove {
                vertical: Some(VerticalMove::Down(1)),
                horizontal: HorizontalMove::from_delta(column_delta),
            };
            self.cursor.moved(cursor_move);
        }
    }

    fn cursor_forward(&mut self) {
        let current_line = self.text_buffer.line_at(self.cursor.line);
        if self.cursor.character < current_line.characters.len() {
            self.cursor.moved(CursorMove {
                vertical: None,
                horizontal: Some(HorizontalMove::Right(1)),
            });
        } else if self.cursor.line < self.text_buffer.line_count() - 1 {
            let column_delta = 0 - (self.cursor.character as isize);
            let cursor_move = CursorMove {
                vertical: Some(VerticalMove::Down(1)),
                horizontal: HorizontalMove::from_delta(column_delta),
            };
            self.cursor.moved(cursor_move);
        }
    }

    fn cursor_up(&mut self) {
        if self.cursor.line > 0 {
            let line_above = self.text_buffer.line_at(self.cursor.line - 1);
            let mut column_delta = 0;
            if line_above.characters.len() == 0
                || (line_above.characters.len() - 1) < self.cursor.character
            {
                column_delta =
                    (line_above.characters.len() as isize) - (self.cursor.character as isize)
            }

            let cursor_move = CursorMove {
                vertical: Some(VerticalMove::Up(1)),
                horizontal: HorizontalMove::from_delta(column_delta),
            };
            self.cursor.moved(cursor_move);
        }
    }

    fn delete_backward(&mut self) {
        if self.cursor.character == 0 && self.cursor.line > 0 {
            let line_above = self.text_buffer.line_at(self.cursor.line - 1);
            let new_cursor_column = line_above.characters.len();
            self.text_buffer
                .remove_item_at(line_above.start_index + new_cursor_column);

            let column_delta = (new_cursor_column as isize) - (self.cursor.character as isize);
            let cursor_move = CursorMove {
                vertical: Some(VerticalMove::Up(1)),
                horizontal: HorizontalMove::from_delta(column_delta),
            };
            self.cursor.moved(cursor_move);
        } else if self.cursor.character > 0 {
            let current_line = self.text_buffer.line_at(self.cursor.line);
            self.cursor.moved(CursorMove {
                vertical: None,
                horizontal: Some(HorizontalMove::Left(1)),
            });
            self.text_buffer
                .remove_item_at(current_line.start_index + self.cursor.character);
        }
    }

    fn exit(&mut self) {
        self.running = false;
    }

    fn insert_character(&mut self, c: char) {
        let current_line = self.text_buffer.line_at(self.cursor.line);
        self.text_buffer
            .insert_item_at(c, current_line.start_index + self.cursor.character);
        let cursor_move = CursorMove {
            vertical: None,
            horizontal: Some(HorizontalMove::Right(1)),
        };
        self.cursor.moved(cursor_move);
    }

    fn insert_newline(&mut self) {
        let current_line = self.text_buffer.line_at(self.cursor.line);
        self.text_buffer
            .insert_item_at('\n', current_line.start_index + self.cursor.character);

        let column_delta = 0 - (self.cursor.character as isize);
        let cursor_move = CursorMove {
            vertical: Some(VerticalMove::Down(1)),
            horizontal: HorizontalMove::from_delta(column_delta),
        };
        self.cursor.moved(cursor_move);
    }

    fn save(&mut self) {
        if let Some(path) = self.file_path {
            file::save(path, self.text_buffer.all_content());
        }
    }
}

enum Command {
    CursorBackward,
    CursorDown,
    CursorUp,
    CursorForward,
    DeleteBackward,
    Exit,
    InsertCharacter(char),
    InsertNewLine,
    Save,
}
