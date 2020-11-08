use crate::{config::{EditorConfig, IndentationPreference}, str_utils};
use crate::cursor::Cursor;
use crate::file;
use crate::renderer;
use crate::text_buffer::piece_table::PieceTable;
use crate::text_buffer::TextBuffer;
use crate::window::Window;
use std::io::{stdout, Stdout, Write};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, terminal,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

pub struct Editor<'a> {
    config: EditorConfig,
    cursor: Cursor,
    file_path: Option<&'a str>,
    running: bool,
    screen: Stdout,
    text_buffer: PieceTable,
    window: Window,
}

impl<'a> Editor<'a> {
    pub fn new(file_path: Option<&'a str>) -> Editor<'a> {
        let file_contents = match file_path {
            Some(path) => file::load(path).unwrap_or(String::new()),
            _ => String::new(),
        };

        let text_buffer = PieceTable::new(file_contents);
        let config = EditorConfig {
            tab_width: 4,
            indentation: IndentationPreference::Tabs,
        };
        let cursor = Cursor::new();
        let window = Window::new(0, 0, 0, 0);

        Editor {
            config,
            cursor,
            file_path,
            running: false,
            screen: stdout(),
            text_buffer,
            window,
        }
    }

    pub fn start(&mut self) {
        self.running = true;

        execute!(self.screen, EnterAlternateScreen);
        terminal::enable_raw_mode();

        while self.running {
            renderer::render(
                &mut self.screen,
                &mut self.text_buffer,
                &mut self.cursor,
                &mut self.window,
                &self.config,
            );

            if let Ok(Event::Key(event)) = event::read() {
                let command = self.map_key_to_command(event);
                if let Some(command) = command {
                    self.execute_command(command);
                }
            }
        }

        execute!(self.screen, LeaveAlternateScreen);
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
            Command::InsertTab => self.insert_tab(),
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
            (KeyCode::Tab, _) => Some(Command::InsertTab),
            _ => None,
        }
    }

    fn cursor_backward(&mut self) {
        let current_line = self.text_buffer.line_at(self.cursor.line);
        let previous_char_idx = str_utils::prev_char_idx(&current_line.content, self.cursor.byte_offset);
        match previous_char_idx {
            Some(i) => {
                self.cursor.byte_offset = i;
                self.cursor.character -= 1;
            }
            None => {
                if self.cursor.line > 0 {
                    let line_above = self.text_buffer.line_at(self.cursor.line - 1);
                    self.cursor.byte_offset = line_above.content.len();
                    self.cursor.character = line_above.content.chars().count();
                    self.cursor.line -= 1;
                }
            }
        }
    }

    fn cursor_down(&mut self) {
        if self.cursor.line < self.text_buffer.line_count() - 1 {
            let line_below = self.text_buffer.line_at(self.cursor.line + 1);
            if line_below.len() < self.cursor.byte_offset
            {
                self.cursor.byte_offset = line_below.len();
                self.cursor.character = line_below.content.chars().count();
            }
            self.cursor.line += 1;
        }
    }

    fn cursor_forward(&mut self) {
        let current_line = self.text_buffer.line_at(self.cursor.line);

        let next_char_idx = str_utils::next_char_idx(&current_line.content, self.cursor.byte_offset);
        match next_char_idx {
            Some(i) => {
                self.cursor.byte_offset = i;
                self.cursor.character += 1;
            }
            None => {
                if self.cursor.byte_offset < current_line.len() {
                    self.cursor.byte_offset = current_line.len();
                    self.cursor.character += 1;
                } else if self.cursor.line < self.text_buffer.line_count() - 1 {
                    self.cursor.byte_offset = 0;
                    self.cursor.character = 0;
                    self.cursor.line += 1;
                }
            }
        }
    }

    fn cursor_up(&mut self) {
        if self.cursor.line > 0 {
            let line_above = self.text_buffer.line_at(self.cursor.line - 1);
            if line_above.len() < self.cursor.byte_offset
            {
                self.cursor.byte_offset = line_above.len();
                self.cursor.character = line_above.content.chars().count();
            }
            self.cursor.line -= 1;
        }
    }

    fn delete_backward(&mut self) {
        if self.cursor.byte_offset > 0 {
            let current_line = self.text_buffer.line_at(self.cursor.line);
            let prev_char_idx = str_utils::prev_char_idx(&current_line.content, self.cursor.byte_offset);
            match prev_char_idx {
                Some(i) => {
                    self.text_buffer.remove(current_line.start_index + i..current_line.start_index + self.cursor.byte_offset);
                    self.cursor.byte_offset = i;
                    self.cursor.character -= 1;
                },
                None => {
                    self.text_buffer.remove(current_line.start_index..current_line.start_index + self.cursor.byte_offset);
                    self.cursor.byte_offset = 0;
                    self.cursor.character = 0;
                }
            }
        } else if self.cursor.line > 0 {
            let line_above = self.text_buffer.line_at(self.cursor.line - 1);
            self.text_buffer.remove(line_above.start_index + line_above.len()..line_above.start_index + line_above.len() + 1);
            self.cursor.byte_offset = line_above.len();
            self.cursor.character = line_above.content.chars().count();
            self.cursor.line -= 1;
        }
    }

    fn exit(&mut self) {
        self.running = false;
    }

    fn insert_character(&mut self, c: char) {
        let current_line = self.text_buffer.line_at(self.cursor.line);
        self.text_buffer.insert(&c.to_string(), current_line.start_index + self.cursor.byte_offset);
        self.cursor.byte_offset += c.len_utf8();
        self.cursor.character += 1;
    }

    fn insert_newline(&mut self) {
        let current_line = self.text_buffer.line_at(self.cursor.line);
        self.text_buffer.insert("\n", current_line.start_index + self.cursor.byte_offset);
        self.cursor.byte_offset = 0;
        self.cursor.character = 0;
        self.cursor.line += 1;
    }

    fn insert_tab(&mut self) {
        let current_line = self.text_buffer.line_at(self.cursor.line);
        let to_insert = match self.config.indentation {
            IndentationPreference::Tabs => String::from("\t"),
            IndentationPreference::Spaces => vec![' '; self.config.tab_width as usize].into_iter().collect()
        };

        self.text_buffer
            .insert(&to_insert, current_line.start_index + self.cursor.character);
        self.cursor.byte_offset += to_insert.len();
        self.cursor.character += to_insert.chars().count();
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
    InsertTab,
    Save,
}
