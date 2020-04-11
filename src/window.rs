
pub struct Window {
    pub height: u16,
    pub width: u16,
    pub vertical_offset: usize,
    pub horizontal_offset: usize,
}

impl Window {
    pub fn new(height: u16, width: u16, vertical_offset: usize, horizontal_offset: usize) -> Window {
        Window {
            height,
            width,
            vertical_offset: 0,
            horizontal_offset: 0,
        }
    }

    pub fn resize(&mut self, height: u16, width: u16) {
        self.height = height;
        self.width = width;
    }

    pub fn bottom(&self) -> usize {
        match self.vertical_offset.checked_add(self.height as usize) {
            Some(result) => result,
            None => usize::max_value(),
        }
    }

    pub fn right(&self) -> usize {
        match self.horizontal_offset.checked_add(self.width as usize) {
            Some(result) => result,
            None => usize::max_value(),
        }
    }

    pub fn update_offsets(&mut self, row: usize, column: usize) {
        if row < self.vertical_offset {
            self.vertical_offset = row;
        }
        if row >= self.bottom() {
            self.vertical_offset += row - self.bottom() + 1;
        }
        if column < self.horizontal_offset {
            self.horizontal_offset = column;
        }
        if column >= self.right() {
            self.horizontal_offset += column - self.right() + 1;
        }
    }
}
