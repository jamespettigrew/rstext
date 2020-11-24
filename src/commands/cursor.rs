use crate::editor::Editor;
use crate::str_utils;
use crate::text_buffer::piece_table::PieceTable;
use crate::text_buffer::TextBuffer;

pub fn cursor_backward(editor: &mut Editor) {
    let current_line = editor.text_buffer.line_at(editor.cursor.line);
    let previous_char_idx = str_utils::prev_char_idx(&current_line.content, editor.cursor.byte_offset);
    match previous_char_idx {
        Some(i) => {
            editor.cursor.byte_offset = i;
            editor.cursor.character -= 1;
        }
        None => {
            if editor.cursor.line > 0 {
                let line_above = editor.text_buffer.line_at(editor.cursor.line - 1);
                editor.cursor.byte_offset = line_above.content.len();
                editor.cursor.character = line_above.content.chars().count();
                editor.cursor.line -= 1;
            }
        }
    }
}

pub fn cursor_down(editor: &mut Editor) {
    if editor.cursor.line < editor.text_buffer.line_count() - 1 {
        let line_below = editor.text_buffer.line_at(editor.cursor.line + 1);
        if line_below.len() < editor.cursor.byte_offset
        {
            editor.cursor.byte_offset = line_below.len();
            editor.cursor.character = line_below.content.chars().count();
        }
        editor.cursor.line += 1;
    }
}

pub fn cursor_forward(editor: &mut Editor) {
    let current_line = editor.text_buffer.line_at(editor.cursor.line);

    let next_char_idx = str_utils::next_char_idx(&current_line.content, editor.cursor.byte_offset);
    match next_char_idx {
        Some(i) => {
            editor.cursor.byte_offset = i;
            editor.cursor.character += 1;
        }
        None => {
            if editor.cursor.byte_offset < current_line.len() {
                editor.cursor.byte_offset = current_line.len();
                editor.cursor.character += 1;
            } else if editor.cursor.line < editor.text_buffer.line_count() - 1 {
                editor.cursor.byte_offset = 0;
                editor.cursor.character = 0;
                editor.cursor.line += 1;
            }
        }
    }
}

pub fn cursor_up(editor: &mut Editor) {
    if editor.cursor.line > 0 {
        let line_above = editor.text_buffer.line_at(editor.cursor.line - 1);
        if line_above.len() < editor.cursor.byte_offset
        {
            editor.cursor.byte_offset = line_above.len();
            editor.cursor.character = line_above.content.chars().count();
        }
        editor.cursor.line -= 1;
    }
}
