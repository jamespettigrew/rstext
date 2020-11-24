use crate::{config::{EditorConfig, IndentationPreference}, str_utils};
use crate::commands;
use crate::cursor::Cursor;
use crate::file;
use crate::renderer;
use crate::text_buffer::piece_table::PieceTable;
use crate::text_buffer::TextBuffer;
use crate::window::Window;
use std::io::{stdout, Stdout, Write};
use std::path::PathBuf;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, terminal,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

pub struct Editor {
    pub config: EditorConfig,
    pub cursor: Cursor,
    pub file_path: Option<PathBuf>,
    pub running: bool,
    screen: Stdout,
    pub text_buffer: PieceTable,
    pub window: Window,
}

impl Editor {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let file_contents = match &file_path {
            Some(path) => file::load(&path).unwrap_or(String::new()),
            _ => String::new(),
        };

        let text_buffer = PieceTable::new(file_contents);
        let config = EditorConfig {
            tab_width: 4,
            indentation: IndentationPreference::Tabs,
        };
        let cursor = Cursor::new();
        let window = Window::new(0, 0, 0, 0);

        Self {
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
                self.handle_key_event(event);
            }
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => commands::app::exit(self),
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => commands::app::save(self),
            (KeyCode::Char(c), _) => commands::edit::insert_character(self, c),
            (KeyCode::Backspace, _) => commands::edit::delete_backward(self),
            (KeyCode::Enter, _) => commands::edit::insert_newline(self),
            (KeyCode::Tab, _) => commands::edit::insert_tab(self),
            (KeyCode::Left, _) => commands::cursor::cursor_backward(self),
            (KeyCode::Right, _) => commands::cursor::cursor_forward(self),
            (KeyCode::Up, _) => commands::cursor::cursor_up(self),
            (KeyCode::Down, _) => commands::cursor::cursor_down(self),
            _ => ()
        };
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        execute!(self.screen, LeaveAlternateScreen);
        terminal::disable_raw_mode();
    }
}
