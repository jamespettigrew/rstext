use crate::cursor::Cursor;
use crate::grapheme;
use crate::text_buffer;
use crate::window::Window;
use std::cmp::{Eq, PartialEq};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue, style,
    style::{ Color, style, StyledContent },
    terminal,
    terminal::{Clear, ClearType}
};
use grapheme::{Grapheme};
use std::io::Write;
use text_buffer::{ line::Line, TextBuffer };

struct TerminalCursorPosition {
    row: usize,
    column: usize,
}

fn calc_absolute_cursor_position(
    cursor: &Cursor,
    current_line_graphemes: &Vec<Grapheme>) -> TerminalCursorPosition {
    
    let column = current_line_graphemes.iter().take(cursor.character).map(|g| g.len()).sum();

    TerminalCursorPosition {
        row: cursor.line,
        column: column
    }
}

fn line_number_width(min_width: usize,  line_count: usize) -> u16 {
    // Number of columns the display of line numbers will require: max(3, num_digits) + 1 space
    let line_number_digits = line_count.to_string().len();
    (std::cmp::max(min_width, line_number_digits) + 1) as u16
}

fn renderable_lines(text_buffer: &impl TextBuffer, window: &Window) -> Vec<(usize, Line)> {
    let last_line = std::cmp::min(window.bottom(), text_buffer.line_count());
    let line_range = window.vertical_offset..last_line;
    line_range.map(|x| (x, text_buffer.line_at(x))).collect()
}

pub fn render(
    screen: &mut impl Write,
    text_buffer: &impl TextBuffer,
    cursor: &Cursor,
    window: &mut Window) {
    queue!(screen, Clear(ClearType::All), Hide);

    let (terminal_width, terminal_height) =
        terminal::size().expect("Failed to get terminal size.");
    let min_width_line_number = 3usize;
    let line_number_columns = line_number_width(min_width_line_number, text_buffer.line_count());
    window.resize(terminal_height, terminal_width - line_number_columns);

    let current_line = text_buffer.line_at(cursor.line);
    let graphemes = &Grapheme::from_line(&current_line);
    let absolute_cursor_position = &calc_absolute_cursor_position(cursor, graphemes);
    window.update_offsets(absolute_cursor_position.row, absolute_cursor_position.column);

    let mut line_count = 0;
    for (line_index, line) in renderable_lines(text_buffer, window) {
        let mut background_color = Color::Reset;

        if line_index == cursor.line {
            background_color = Color::Rgb { r: 59, g: 66, b: 82 };
            let characters = (0..terminal_width).map(|_| ' ').collect::<String>();
            let styled_characters = style(characters)
                .on(background_color);
            queue!(screen, MoveTo(0, line_count as u16), style::PrintStyledContent(styled_characters));
        }

        let characters = format!(
                "{:>min_width$}",
                line_index + 1,
                min_width = min_width_line_number
            );
        let styled_characters = style(characters).with(Color::Blue).on(background_color);
        queue!(
            screen,
            MoveTo(0, line_count as u16),
            style::PrintStyledContent(styled_characters),
            MoveTo(line_number_columns, line_count as u16)
        );

        let graphemes = &Grapheme::from_line(&line);
        let graphemes = grapheme::visible_in_window(graphemes, window);
        let styled_graphemes = graphemes.iter().map(|g| {
            match g.is_escaped {
                true => style(&g.content).with(Color::Yellow).on(background_color),
                false => style(&g.content).with(Color::White).on(background_color)
            }
        }).collect::<Vec<StyledContent<&String>>>();

        for styled in styled_graphemes {
            queue!(screen, style::PrintStyledContent(styled));
        }

        line_count += 1;
    }

    let relative_cursor_row = absolute_cursor_position.row - window.vertical_offset;
    let relative_cursor_column =
        line_number_columns + ((absolute_cursor_position.column - window.horizontal_offset) as u16);
    queue!(screen, MoveTo(relative_cursor_column, relative_cursor_row as u16), Show);
    screen.flush().unwrap();
}