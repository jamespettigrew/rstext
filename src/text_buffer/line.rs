pub struct Line {
    pub start_index: usize,
    pub characters: Vec<char>,
}

impl Line {
    pub fn new(start_index: usize, characters: Vec<char>) -> Line {
        let string: String = characters.iter().collect();

        Line {
            start_index,
            characters,
        }
    }

    pub fn len(&self) -> usize {
        self.characters.len()
    }
}
