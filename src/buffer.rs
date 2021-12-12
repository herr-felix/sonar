use std::cmp::{self, Ordering};
use std::io::{self, BufRead, BufReader, Read};
use BufferOp::*;

#[derive(Clone, Copy, PartialEq)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
}

impl PartialOrd for Cursor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.line < other.line {
            Some(Ordering::Less)
        } else if self.line > other.line {
            Some(Ordering::Greater)
        } else if self.col > other.col {
            Some(Ordering::Greater)
        } else if self.col < other.col {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Equal)
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum LineCombination {
    FromStart, // Combined from the start of a line. Probably by pressing backspace.
    FromEnd,   // Combined from the end of a line. Probably by pressing delete.
}

#[derive(PartialEq)]
enum BufferOp {
    InsertChar(Cursor, char),
    RemoveChar(Cursor, char, bool),
    SplitLine(Cursor),
    CombineLine(Cursor, LineCombination),
    NoOp,
}

#[derive(PartialEq)]
pub struct Buffer {
    pub name: String,
    lines: Vec<String>,
    cursor: Cursor,
    undos: Vec<BufferOp>,
    redos: Vec<BufferOp>,
}

impl Buffer {
    pub fn empty() -> Buffer {
        Buffer {
            name: "[draft]".to_owned(),
            lines: vec![String::from("")],
            cursor: Cursor { line: 0, col: 0 },
            undos: Vec::new(),
            redos: Vec::new(),
        }
    }

    pub fn new<T: Read>(name: String, read: T) -> io::Result<Buffer> {
        let reader = BufReader::new(read);

        let lines = reader.lines().collect::<io::Result<Vec<String>>>()?;

        Ok(Buffer {
            name,
            lines,
            cursor: Cursor { line: 0, col: 0 },
            undos: Vec::new(),
            redos: Vec::new(),
        })
    }

    fn record_op(&mut self, op: BufferOp) {
        if op != NoOp {
            self.undos.push(op);
            self.redos.clear();
        }
    }

    pub fn undo(&mut self) {
        if let Some(op) = self.undos.pop() {
            match op {
                InsertChar(cur, _) => {
                    self.cursor = cur;
                    self.op_remove_at();
                }
                RemoveChar(cur, ch, at) => {
                    self.cursor = cur;
                    self.op_insert_char(ch);
                    if at {
                        self.cursor = cur;
                    }
                }
                SplitLine(cur) => {
                    self.cursor = cur;
                    self.op_remove_at();
                }
                CombineLine(cur, from) => {
                    self.cursor = cur;
                    self.op_newline();
                    if from == LineCombination::FromEnd {
                        self.cursor = cur;
                    }
                }
                NoOp => (),
            }
            self.redos.push(op);
        }
    }

    pub fn redo(&mut self) {
        if let Some(op) = self.redos.pop() {
            match op {
                InsertChar(cur, ch) => {
                    self.cursor = cur;
                    self.op_insert_char(ch);
                }
                RemoveChar(cur, _, at) => {
                    self.cursor = cur;
                    if at {
                        self.op_remove_at();
                    } else {
                        self.op_remove_before();
                    }
                }
                SplitLine(cur) => {
                    self.cursor = cur;
                    self.op_newline();
                }
                CombineLine(cur, from) => {
                    self.cursor = cur;
                    match from {
                        LineCombination::FromStart => self.op_remove_before(),
                        LineCombination::FromEnd => self.op_remove_at(),
                    };
                }
                NoOp => (),
            }
            self.undos.push(op);
        }
    }

    // MOVING AROUND

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

    pub fn move_end_of_line(&mut self) {
        self.cursor.col = self.lines[self.cursor.line].len();
    }

    pub fn move_start_of_line(&mut self) {
        self.cursor.col = 0;
    }

    pub fn move_cursor_right(&mut self, delta: usize) {
        self.cursor.col = cmp::min(self.cursor.col + delta, self.lines[self.cursor.line].len());
    }

    pub fn go_to_line(&mut self, line: usize) -> Result<(), String> {
        if line < self.lines.len() && line > 0 {
            self.cursor.line = line - 1;
            Ok(())
        } else {
            Err(format!("Error: Line {} is out of bound", line).to_owned())
        }
    }

    // GETTING DATA

    pub fn get_cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn get_line(&self) -> String {
        self.lines[self.cursor.line].to_owned()
    }

    // MUTATION OPERATIONS

    pub fn newline(&mut self) {
        let op = self.op_newline();
        self.record_op(op);
    }

    fn op_newline(&mut self) -> BufferOp {
        let new_line = self.lines[self.cursor.line].split_off(self.cursor.col);
        let op = SplitLine(self.cursor);

        self.cursor.line += 1;
        self.cursor.col = 0;

        self.lines.insert(self.cursor.line, new_line);

        op
    }

    pub fn insert_char(&mut self, ch: char) {
        let op = self.op_insert_char(ch);
        self.record_op(op);
    }

    fn op_insert_char(&mut self, ch: char) -> BufferOp {
        let cur = self.cursor;

        self.lines[cur.line].insert(cur.col, ch);
        self.cursor.col += 1;

        InsertChar(cur, ch)
    }

    // Like a "delete", remove the character under the cursor.
    // Append the next line to the current line if the cursor
    // is at the end of the line.
    pub fn remove_at(&mut self) {
        let op = self.op_remove_at();
        self.record_op(op);
    }

    fn op_remove_at(&mut self) -> BufferOp {
        // Not end of line
        if self.cursor.col < self.lines[self.cursor.line].len() {
            let ch = self.lines[self.cursor.line].remove(self.cursor.col);
            RemoveChar(self.cursor, ch, true)
        } else {
            // End of line
            if self.cursor.line < (self.lines.len() - 1) {
                // Not end of file
                let next_line = self.lines.remove(self.cursor.line + 1);
                self.lines[self.cursor.line].push_str(next_line.as_str());

                CombineLine(self.cursor, LineCombination::FromEnd)
            } else {
                // Del at EOF does nothing
                NoOp
            }
        }
    }

    // Like "backspace", remove the character before the cursor.
    // Where removing the first character of a line, moves the
    // line to the end of the previous line.
    pub fn remove_before(&mut self) {
        let op = self.op_remove_before();
        self.record_op(op);
    }

    fn op_remove_before(&mut self) -> BufferOp {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
            let ch = self.lines[self.cursor.line].remove(self.cursor.col);

            RemoveChar(self.cursor, ch, false)
        } else {
            // Start of line
            if self.cursor.line > 0 {
                // Not first line
                let line = self.lines.remove(self.cursor.line);

                self.cursor.line -= 1;
                self.cursor.col = self.lines[self.cursor.line].len();

                self.lines[self.cursor.line].push_str(line.as_str());

                CombineLine(self.cursor, LineCombination::FromStart)
            } else {
                // Backspace at 0,0 Does nothing
                NoOp
            }
        }
    }
}
