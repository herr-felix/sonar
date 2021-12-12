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

trait AppMode {
    fn draw(&self) -> crossterm::Result<()>;
    fn handle_input(self, event: Event) -> Option<AppState>;
}

#[derive(PartialEq)]
struct Editor {
    buf: Buffer,
}

#[derive(PartialEq)]
struct GoToLineModal {
    buf: Buffer,
    modal: Modal,
}

#[derive(PartialEq)]
struct App<S> {
    mode: S,
}

#[derive(PartialEq)]
enum AppState {
    Editor(App<Editor>),
    GoToLineModal(App<GoToLineModal>),
}

impl AppMode for App<Editor> {
    fn draw(&self) -> crossterm::Result<()> {
        let line = self.mode.buf.get_line();
        let cur = self.mode.buf.get_cursor();

        execute!(
            stdout(),
            MoveTo(0, 3),
            Clear(ClearType::CurrentLine),
            Print(format!("{}, {}", cur.line + 1, cur.col + 1)),
            MoveTo(12, 3),
            Print(self.mode.buf.name.clone()),
            MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            Print(line),
            MoveTo(u16::try_from(cur.col).unwrap(), 0),
        )
    }

    fn handle_input(mut self, event: Event) -> Option<AppState> {
        let buf = &mut self.mode.buf;

        match event {
            Event::Key(event) => match event.modifiers {
                KeyModifiers::CONTROL => match event.code {
                    // Go to line
                    KeyCode::Char('g') => return Some(AppState::GoToLineModal(self.into())),
                    // Undo
                    KeyCode::Char('z') => buf.undo(),
                    // Redo
                    KeyCode::Char('y') => buf.redo(),
                    _ => (),
                },
                _ => match event.code {
                    KeyCode::Esc => return None,
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
                },
            },
            _ => (),
        };

        Some(AppState::Editor(self))
    }
}

impl AppMode for App<GoToLineModal> {
    fn draw(&self) -> crossterm::Result<()> {
        let modal = &self.mode.modal;

        execute!(
            stdout(),
            MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            Print(format!("{}: {}", modal.name, modal.line)),
            MoveTo(u16::try_from(modal.name.len() + 2 + modal.col).unwrap(), 0),
        )
    }

    fn handle_input(mut self, event: Event) -> Option<AppState> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Esc => return Some(AppState::Editor(self.into())),
                KeyCode::Enter => {
                    let line_no: usize = self.mode.modal.line.parse::<usize>().unwrap();
                    self.mode.buf.go_to_line(line_no).unwrap();
                    return Some(AppState::Editor(self.into()));
                }
                _ => {
                    handle_modal_input(&mut self.mode.modal, event);
                }
            },
            _ => (),
        };

        Some(AppState::GoToLineModal(self))
    }
}

impl From<App<GoToLineModal>> for App<Editor> {
    fn from(val: App<GoToLineModal>) -> App<Editor> {
        App {
            mode: Editor { buf: val.mode.buf },
        }
    }
}

impl From<App<Editor>> for App<GoToLineModal> {
    fn from(val: App<Editor>) -> App<GoToLineModal> {
        App {
            mode: GoToLineModal {
                buf: val.mode.buf,
                modal: Modal::new("Go to line".to_owned()),
            },
        }
    }
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

fn draw_screen(state: &AppState) -> crossterm::Result<()> {
    match state {
        AppState::Editor(app) => app.draw(),
        AppState::GoToLineModal(app) => app.draw(),
    }
}

fn handle_input(state: AppState, event: Event) -> Option<AppState> {
    match state {
        AppState::Editor(app) => app.handle_input(event),
        AppState::GoToLineModal(app) => app.handle_input(event),
    }
}

fn clear_screen() -> crossterm::Result<()> {
    execute!(stdout(), Clear(ClearType::All))
}

fn get_event() -> crossterm::Result<Event> {
    let event = read()?;

    if let Event::Resize(_, _) = event {
        clear_screen()?;
    }

    Ok(event)
}

fn app_loop(mut state: AppState) -> crossterm::Result<()> {
    // Clean up the screen
    clear_screen()?;

    loop {
        draw_screen(&state)?;

        let event = get_event()?;

        if let Some(new_state) = handle_input(state, event) {
            state = new_state;
        } else {
            break;
        }
    }

    Ok(())
}

fn main() {
    let here = std::fs::File::open("./src/main.rs").unwrap();

    let buffer = Buffer::new("[draft]".to_owned(), here).unwrap();

    let app = AppState::Editor(App::<Editor> {
        mode: Editor { buf: buffer },
    });

    enable_raw_mode().unwrap();

    app_loop(app).unwrap();
}
