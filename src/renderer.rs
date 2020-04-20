use crate::cursor::Cursor;
use crate::grapheme;
use crate::text_buffer;
use crate::window::Window;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    queue, style,
    style::{style, Color, StyledContent},
    terminal,
    terminal::{Clear, ClearType},
};
use grapheme::Grapheme;
use std::io::Write;
use text_buffer::{line::Line, TextBuffer};

const MIN_WIDTH_LINE_NUMBER: u16 = 3;

struct TerminalCursorPosition {
    row: usize,
    column: usize,
}

fn calc_absolute_cursor_position(
    cursor: &Cursor,
    current_line_graphemes: &Vec<Grapheme>,
) -> TerminalCursorPosition {
    let column = current_line_graphemes
        .iter()
        .take(cursor.character)
        .map(|g| g.len())
        .sum();

    TerminalCursorPosition {
        row: cursor.line,
        column: column,
    }
}

fn line_number_width(line_count: usize) -> u16 {
    // Number of columns the display of line numbers will require: max(3, num_digits) + 1 space
    let line_number_digits = line_count.to_string().len();
    (std::cmp::max(3, line_number_digits) + 1) as u16
}

fn renderable_lines(text_buffer: &dyn TextBuffer, window: &Window) -> Vec<(usize, Line)> {
    let last_line = std::cmp::min(window.bottom(), text_buffer.line_count());
    let line_range = window.vertical_offset..last_line;
    line_range.map(|x| (x, text_buffer.line_at(x))).collect()
}

fn get_cursor_position_info(
    cursor: &Cursor,
    absolute_cursor_position: &TerminalCursorPosition,
) -> String {
    if cursor.character == absolute_cursor_position.column {
        format!("Ln {}, Col {}", cursor.line + 1, cursor.character + 1)
    } else {
        format!(
            "Ln {}, Col {}-{}",
            cursor.line + 1,
            cursor.character + 1,
            absolute_cursor_position.column + 1
        )
    }
}

pub fn render(
    screen: &mut impl Write,
    text_buffer: &dyn TextBuffer,
    cursor: &Cursor,
    window: &mut Window,
) {
    queue!(screen, Clear(ClearType::All), Hide);

    let (terminal_width, terminal_height) = terminal::size().expect("Failed to get terminal size.");
    let line_number_columns = line_number_width(text_buffer.line_count());
    window.resize(terminal_height - 1, terminal_width - line_number_columns);

    let current_line = text_buffer.line_at(cursor.line);
    let graphemes = &Grapheme::from_line(&current_line);
    let absolute_cursor_position = &calc_absolute_cursor_position(cursor, graphemes);
    window.update_offsets(
        absolute_cursor_position.row,
        absolute_cursor_position.column,
    );

    let mut line_count = 0;
    for (line_index, line) in renderable_lines(text_buffer, window) {
        let mut background_color = Color::Reset;

        if line_index == cursor.line {
            background_color = Color::Rgb {
                r: 59,
                g: 66,
                b: 82,
            };
            let characters = (0..terminal_width).map(|_| ' ').collect::<String>();
            let styled_characters = style(characters).on(background_color);
            queue!(
                screen,
                MoveTo(0, line_count as u16),
                style::PrintStyledContent(styled_characters)
            );
        }

        let characters = format!(
            "{:>min_width$}",
            line_index + 1,
            min_width = MIN_WIDTH_LINE_NUMBER as usize
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
        let styled_graphemes = graphemes
            .iter()
            .map(|g| match g.is_escaped {
                true => style(&g.content).with(Color::Yellow).on(background_color),
                false => style(&g.content).with(Color::White).on(background_color),
            })
            .collect::<Vec<StyledContent<&String>>>();

        for styled in styled_graphemes {
            queue!(screen, style::PrintStyledContent(styled));
        }

        line_count += 1;
    }

    let relative_cursor_row = absolute_cursor_position.row - window.vertical_offset;
    let relative_cursor_column =
        line_number_columns + ((absolute_cursor_position.column - window.horizontal_offset) as u16);

    let cursor_position_info = get_cursor_position_info(cursor, absolute_cursor_position);
    let print_column_start =
        match terminal_width.checked_sub(cursor_position_info.chars().count() as u16) {
            Some(x) => x,
            None => 0,
        };
    queue!(
        screen,
        MoveTo(print_column_start, terminal_height - 1),
        style::Print(cursor_position_info)
    );

    queue!(
        screen,
        MoveTo(relative_cursor_column, relative_cursor_row as u16),
        Show
    );
    screen.flush().unwrap();
}
