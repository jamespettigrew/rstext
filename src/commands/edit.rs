use crate::config::IndentationPreference;
use crate::editor::Editor;
use crate::str_utils;
use crate::text_buffer::piece_table::PieceTable;
use crate::text_buffer::TextBuffer;

pub fn delete_backward(editor: &mut Editor) {
    if editor.cursor.byte_offset > 0 {
        let current_line = editor.text_buffer.line_at(editor.cursor.line);
        let prev_char_idx = str_utils::prev_char_idx(&current_line.content, editor.cursor.byte_offset);
        match prev_char_idx {
            Some(i) => {
                editor.text_buffer.remove(current_line.start_index + i..current_line.start_index + editor.cursor.byte_offset);
                editor.cursor.byte_offset = i;
                editor.cursor.character -= 1;
            },
            None => {
                editor.text_buffer.remove(current_line.start_index..current_line.start_index + editor.cursor.byte_offset);
                editor.cursor.byte_offset = 0;
                editor.cursor.character = 0;
            }
        }
    } else if editor.cursor.line > 0 {
        let line_above = editor.text_buffer.line_at(editor.cursor.line - 1);
        editor.text_buffer.remove(line_above.start_index + line_above.len()..line_above.start_index + line_above.len() + 1);
        editor.cursor.byte_offset = line_above.len();
        editor.cursor.character = line_above.content.chars().count();
        editor.cursor.line -= 1;
    }
}

pub fn insert_character(editor: &mut Editor, c: char) {
    let current_line = editor.text_buffer.line_at(editor.cursor.line);
    editor.text_buffer.insert(&c.to_string(), current_line.start_index + editor.cursor.byte_offset);
    editor.cursor.byte_offset += c.len_utf8();
    editor.cursor.character += 1;
}

pub fn insert_newline(editor: &mut Editor) {
    let current_line = editor.text_buffer.line_at(editor.cursor.line);
    editor.text_buffer.insert("\n", current_line.start_index + editor.cursor.byte_offset);
    editor.cursor.byte_offset = 0;
    editor.cursor.character = 0;
    editor.cursor.line += 1;
}

pub fn insert_tab(editor: &mut Editor) {
    let current_line = editor.text_buffer.line_at(editor.cursor.line);
    let to_insert = match editor.config.indentation {
        IndentationPreference::Tabs => String::from("\t"),
        IndentationPreference::Spaces => vec![' '; editor.config.tab_width as usize].into_iter().collect()
    };

    editor.text_buffer
        .insert(&to_insert, current_line.start_index + editor.cursor.character);
    editor.cursor.byte_offset += to_insert.len();
    editor.cursor.character += to_insert.chars().count();
}
