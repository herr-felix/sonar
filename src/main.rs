mod buffer;
mod modal;

use crate::buffer::Buffer;
use crate::modal::Modal;

use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{enable_raw_mode, Clear, ClearType};
use std::convert::TryFrom;
use std::io::stdout;


#[derive(PartialEq)]
enum App {
    Editor(Buffer),
    GoToLineModal(Buffer, Modal),
    Quit,
}

fn draw_edit(buf: &Buffer) -> crossterm::Result<()> {
    let line = buf.get_line();
    let cur = buf.get_cursor();

    execute!(
        stdout(),
        MoveTo(0, 3),
        Clear(ClearType::CurrentLine),
        Print(format!("{}, {}",cur.line + 1, cur.col + 1)),
        MoveTo(12, 3),
        Print(buf.name.clone()),
        MoveTo(0, 0),
        Clear(ClearType::CurrentLine),
        Print(line),
        MoveTo(u16::try_from(cur.col).unwrap(), 0),
    )
}

fn draw_modal(modal: &Modal) -> crossterm::Result<()> {
    execute!(
        stdout(),
        MoveTo(0, 0),
        Clear(ClearType::CurrentLine),
        Print(format!("{}: {}", modal.name, modal.line)),
        MoveTo(u16::try_from(modal.name.len() + 2 + modal.col).unwrap(), 0),
    )
}

fn draw_screen(app: &App) -> crossterm::Result<()> {
    match &app {
        App::Editor(buffer) => draw_edit(buffer),
        App::GoToLineModal(_, modal) => draw_modal(modal),
        App::Quit => Ok(()),
    }
}


fn handle_edit_input(mut buf: Buffer, event: Event) -> App {
    match event {
        Event::Key(event) => match event.modifiers {
            KeyModifiers::CONTROL => match event.code {
                // Go to line
                KeyCode::Char('g') => { 
                    return App::GoToLineModal(buf, Modal::new("Go to line".to_owned()))
                }
                // Undo
                KeyCode::Char('z') => buf.undo(),
                // Redo
                KeyCode::Char('y') => buf.redo(),
                _ => (),
            },
            _ => match event.code {
                KeyCode::Esc => return App::Quit,
                KeyCode::Enter => buf.newline(),
                KeyCode::Up => buf.move_cursor_up(1),
                KeyCode::Down => buf.move_cursor_down(1),
                KeyCode::Left => buf.move_cursor_left(1),
                KeyCode::Right => buf.move_cursor_right(1),
                KeyCode::Delete => buf.remove_at(),
                KeyCode::Backspace => buf.remove_before(),
                KeyCode::Home => buf.move_start_of_line(),
                KeyCode::End => buf.move_end_of_line(),
                KeyCode::Char(ch) => buf.insert_char(ch),
                _ => (),
            }
        },
        _ => (),
    };
    
    App::Editor(buf)
}

fn handle_modal_input(modal: &mut Modal, event: Event) {
    match event {
        Event::Key(event) => match event.code {
            KeyCode::Left => modal.move_cursor_left(1),
            KeyCode::Right => modal.move_cursor_right(1),
            KeyCode::Delete => modal.remove_at(),
            KeyCode::Backspace => modal.remove_before(),
            KeyCode::Home => modal.move_start_of_line(),
            KeyCode::End => modal.move_end_of_line(),
            KeyCode::Char(ch) => modal.insert_char(ch),
            _ => (),
        },
        _ => (),
    };
}

fn handle_goto_line_modal_input(mut buf: Buffer, mut modal: Modal, event: Event) -> App {
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Esc => return App::Editor(buf),
            KeyCode::Enter => {
                let line_no: usize = modal.line.parse::<usize>().unwrap();
                buf.go_to_line(line_no).unwrap();
                return App::Editor(buf)
            }
            _ => {
                handle_modal_input(&mut modal, event);
            }
        },
        _ => (),
    };

    App::GoToLineModal(buf, modal)
}

fn handle_input(mode: App, event: Event) -> App {
    if let Event::Resize(_, _) = event {
        execute!(stdout(), Clear(ClearType::All)).unwrap();
    }
    
    match mode {
        App::Editor(buf) => handle_edit_input(buf, event),
        App::GoToLineModal(buf, modal) => handle_goto_line_modal_input(buf, modal, event),
        App::Quit => App::Quit,
    }
}

fn app_loop(mut app: App) -> crossterm::Result<()> {
    // Clean up the screen
    execute!(stdout(), Clear(ClearType::All))?;

    while app != App::Quit {
        draw_screen(&app)?;

        app = handle_input(app, read()?);
    }

    Ok(())
}

fn main() {
    let here = std::fs::File::open("./src/main.rs").unwrap();

    let buffer = Buffer::new("[draft]".to_owned(), here).unwrap();

    let app = App::Editor(buffer);

    enable_raw_mode().unwrap();

    app_loop(app).unwrap();
}
