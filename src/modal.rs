use std::cmp;

#[derive(PartialEq)]
pub struct Modal {
    pub name: String,
    pub line: String,
    pub col: usize,
}

impl Modal {

    pub fn new(name: String) -> Modal {
        Modal{
            name,
            line: "".to_owned(),
            col: 0,
        }
    }

    pub fn move_cursor_left(&mut self, delta: usize) {
        if delta <= self.col {
            self.col -= delta;
        }
    }

    pub fn move_end_of_line(&mut self) {
        self.col = self.line.len();
    }

    pub fn move_start_of_line(&mut self) {
        self.col = 0;
    }

    pub fn move_cursor_right(&mut self, delta: usize) {
        self.col = cmp::min(self.col + delta, self.line.len());
    }

    pub fn insert_char(&mut self, ch: char) {
        self.line.insert(self.col, ch);
        self.col += 1;
    }

    pub fn insert(mut self, text: &str) {
        self.line.insert_str(self.col, text);
        self.col += text.len();
    }

    // Like a "delete", remove the character under the cursor.
    pub fn remove_at(&mut self) {
        self.line.remove(self.col);
    }

    // Like "backspace", remove the character before the cursor.
    pub fn remove_before(&mut self) {
        if self.col > 0 {
            self.line.remove(self.col - 1);
            self.col -= 1;
        }
    }
}
