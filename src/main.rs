use rstext::editor::Editor;
use std::env;
use std::ffi::OsString;
use std::path::{ Path, PathBuf};

fn main() {
    let args: Vec<OsString> = env::args_os().collect();
    let file_path: Option<PathBuf> = match &args[..] {
        [_, path] => Some(PathBuf::from(path.clone())),
        _ => None,
    };

    let mut editor = Editor::new(file_path);
    editor.start();
}
