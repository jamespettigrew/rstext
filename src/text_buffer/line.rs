pub struct Line {
    pub start_index: usize,
    pub content: String,
}

impl Line {
    pub fn new(start_index: usize, content: String) -> Line {
        Line {
            start_index,
            content,
        }
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }
}
