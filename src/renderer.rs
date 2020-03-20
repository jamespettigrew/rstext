use crate::cursor::Cursor;
use crate:: text_buffer;

use crossterm::{
    cursor::MoveTo,
    queue, style,
    style::{ Color, style },
    terminal,
    terminal::{Clear, ClearType}
};
use std::io::Write;
use text_buffer::TextBuffer;

pub struct Window {
    height: u16,
    width: u16,
    vertical_offset: usize,
    horizontal_offset: usize,
}

impl Window {
    pub fn new(height: u16, width: u16, vertical_offset: usize, horizontal_offset: usize) -> Window {
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

pub fn render(screen: &mut impl Write, text_buffer: &impl TextBuffer, cursor: &Cursor, window: &mut Window) {
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
        let mut background_color = Color::Reset;

        if row_index == cursor.row {
            background_color = Color::Rgb { r: 59, g: 66, b: 82 };
            let characters = (0..terminal_width).map(|_| ' ').collect::<String>();
            let styled_characters = style(characters)
                .on(background_color);
            queue!(screen, MoveTo(0, row_count as u16), style::PrintStyledContent(styled_characters));
        }

        let characters = format!(
                "{:>min_width$}",
                row_index + 1,
                min_width = min_line_number_columns
            );
        let styled_characters = style(characters).with(Color::Blue).on(background_color);
        queue!(
            screen,
            MoveTo(0, row_count as u16),
            style::PrintStyledContent(styled_characters),
            MoveTo(line_number_columns, row_count as u16)
        );

        let characters = line
            .content
            .iter()
            .skip(window.horizontal_offset)
            .take(window.width as usize)
            .collect::<String>();
        let styled_characters = style(characters)
            .with(Color::Yellow)
            .on(background_color);
        queue!(screen, style::PrintStyledContent(styled_characters));
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