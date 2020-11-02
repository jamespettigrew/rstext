pub struct Cursor {
    pub line: usize,
    pub character: usize,
    pub byte_offset: usize
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            line: 0,
            character: 0,
            byte_offset: 0
        }
    }
}
