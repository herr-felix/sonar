
mod buffer;
use crate::buffer::Buffer;

use std::convert::TryFrom;
use std::io::{stdout, Write};
use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal::{ScrollUp, SetSize, size, enable_raw_mode, Clear, ClearType};
use crossterm::style::{Print};
use crossterm::{execute, Result};
use crossterm::cursor::{MoveTo};

fn print_events(mut active_buffer: Buffer) -> crossterm::Result<()> {

    execute!(
        stdout(),
        Clear(ClearType::All)
    )?;

    loop {
        let line = active_buffer.get_line();
        let cur = active_buffer.get_cursor();

        execute!(
            stdout(),
            MoveTo(0,0),
            Clear(ClearType::CurrentLine),
            Print(line),
            MoveTo(0,3),
            Clear(ClearType::CurrentLine),
            Print(cur.line + 1),
            MoveTo(u16::try_from(cur.col).unwrap(), 0),
        )?;

        match read()? {
            Event::Key(event) => {
                match event.code {
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        active_buffer.newline();
                    },
                    KeyCode::Up => {
                        active_buffer.move_cursor_up(1);
                    }
                    KeyCode::Down => {
                        active_buffer.move_cursor_down(1);
                    }
                    KeyCode::Left => {
                        active_buffer.move_cursor_left(1);
                    }
                    KeyCode::Right => {
                        active_buffer.move_cursor_right(1);
                    },
                    KeyCode::Delete => {
                        active_buffer.remove_at();
                    },
                    KeyCode::Backspace => {
                        active_buffer.remove_before();
                    },
                    KeyCode::Char(ch) => {
                        active_buffer.insert_char(ch);
                    },
                    _ =>  println!("{:?}", event),
                }
            },
            _ => ()
        }

    }
    Ok(())
}


fn main() {

    let buffer = Buffer::new();
    
    enable_raw_mode();
    print_events(buffer).unwrap();

}
