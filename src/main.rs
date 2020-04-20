mod cursor;
mod editor;
mod file;
mod grapheme;
mod renderer;
mod text_buffer;
mod window;

use editor::Editor;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path: Option<&str> = match &args[..] {
        [_, path] => Some(path),
        _ => None
    };

    let mut editor = Editor::new(file_path);
    editor.start();
}
