use crate::editor::Editor;
use crate::file;
use crate::text_buffer::TextBuffer;

pub fn exit(editor: &mut Editor) {
    editor.running = false;
}

pub fn save(editor: &mut Editor) {
    if let Some(path) = &editor.file_path {
        file::save(path, editor.text_buffer.all_content());
    }
}
