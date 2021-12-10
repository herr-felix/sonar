use std::cmp;

#[derive(Clone, Copy)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
}

pub struct Buffer {
    lines: Vec<String>,
    cursor: Cursor,
}


impl Buffer {
    pub fn new() -> Buffer {
        Buffer{
            lines: vec![String::from("")],
            cursor: Cursor{line: 0, col: 0},
        }
    }

    pub fn move_cursor_up(&mut self, delta: usize) {
        if delta <= self.cursor.line {
            self.cursor.line -= delta;
            self.cursor.col = cmp::min(self.cursor.col, self.lines[self.cursor.line].len());
        }
    }

    pub fn move_cursor_down(&mut self, delta: usize) {
        self.cursor.line = cmp::min(self.cursor.line + delta, self.lines.len() - 1);
        self.cursor.col = cmp::min(self.cursor.col, self.lines[self.cursor.line].len());
    }

    pub fn move_cursor_left(&mut self, delta: usize) {
        if delta <= self.cursor.col {
            self.cursor.col -= delta;
        }
    }

    pub fn move_cursor_right(&mut self, delta: usize) {
        self.cursor.col = cmp::min(self.cursor.col + 1, self.lines[self.cursor.line].len());
    }

    pub fn newline(&mut self) {
        let new_line = self.lines[self.cursor.line].split_off(self.cursor.col);

        self.cursor.line += 1;
        self.cursor.col = 0;

        self.lines.insert(self.cursor.line, new_line);
    }

    pub fn get_cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn get_line(&self) -> String {
        self.lines[self.cursor.line].to_owned()
    }

    pub fn insert_char(&mut self, ch: char) {
        if let Some(line) = self.lines.get_mut(self.cursor.line) {
            if self.cursor.col == line.len() {
                line.push(ch);
            } else {
                line.insert(self.cursor.col, ch);
            }
            self.cursor.col += 1;
        }
    }

    pub fn insert(mut self, text: &str) {

        if let Some(line) = self.lines.get_mut(self.cursor.line) {
            if self.cursor.col == line.len() {
                line.push_str(text);
            } else {
                line.insert_str(self.cursor.col, text);
            }

            self.cursor.col += text.len();
        }
        else { // Should never happen, but just in case
            self.lines.push(String::from(text));
            self.cursor = Cursor{
                line: self.lines.len() - 1,
                col: text.len(),
            };
        }
    }

    // Like a "delete", remove the character under the cursor.
    // Append the next line to the current line if the cursor
    // is at the end of the line.
    pub fn remove_at(&mut self) {

        if self.cursor.col < self.lines[self.cursor.line].len() { 
            self.lines[self.cursor.line].remove(self.cursor.col);
        }
        else { // End of line
            if self.cursor.line < (self.lines.len() - 1) { // Not end of file
                let next_line = self.lines.remove(self.cursor.line + 1);
                self.lines[self.cursor.line].push_str(next_line.as_str());
            }
        }
    }

    // Like "backspace", remove the character before the cursor.
    // Where removing the first character of a line, moves the
    // line to the end of the previous line.
    pub fn remove_before(&mut self) {
        if self.cursor.col > 0 { 
            self.lines[self.cursor.line].remove(self.cursor.col - 1);
            self.cursor.col -= 1;
        }
        else { // Start of line
            if self.cursor.line > 0 { // Not first line
                let line = self.lines.remove(self.cursor.line);

                self.cursor.line -= 1;
                self.cursor.col = self.lines[self.cursor.line].len();

                self.lines[self.cursor.line].push_str(line.as_str());
            }
        }
    }
}

