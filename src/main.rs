mod cursor;
mod text_buffer;
mod renderer;

use cursor::{ Cursor, CursorMove, HorizontalMove, VerticalMove };
use renderer::Window;
use std::io::{stdout, Write};
use text_buffer::TextBuffer;
use text_buffer::piece_table::PieceTable;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    Result,
};

fn main() -> Result<()> {
    execute!(stdout(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let text_buffer= &mut PieceTable::new(vec![]);
    let cursor = &mut Cursor::new();
    let window = &mut Window::new(0, 0, 0, 0);
    let screen = &mut stdout();

    loop {
        renderer::render(screen, text_buffer, cursor, window);

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
