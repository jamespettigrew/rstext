use std::fs::File;
use std::io;
use std::io::prelude::{Read, Write};

pub fn load(path: &str) -> io::Result<String> {
    let file_contents = match File::open(path) {
        Ok(mut f) => {
            let mut contents = String::new();
            f.read_to_string(&mut contents)?;
            contents
        }
        _ => String::new(),
    };

    Ok(file_contents)
}

pub fn save(path: &str, content: String) -> io::Result<()> {
    let file = &mut File::create(path)?;
    Ok(for c in content.chars() {
        let buf = &mut [0u8; 4];
        let subslice = c.encode_utf8(buf);
        file.write(subslice.as_bytes())?;
    })
}
