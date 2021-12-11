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
use std::rc::Rc;
use std::cell::RefCell;

struct Context {
    state: AppState,
    active_buf: Buffer,
    name: String,
}

fn draw_edit(ctx: &Context) -> crossterm::Result<()> {
    let line = ctx.active_buf.get_line();
    let cur = ctx.active_buf.get_cursor();

    execute!(
        stdout(),
        MoveTo(0, 3),
        Clear(ClearType::CurrentLine),
        Print(cur.line + 1),
        MoveTo(12, 3),
        Print(ctx.name.clone()),
        MoveTo(0, 0),
        Clear(ClearType::CurrentLine),
        Print(line),
        MoveTo(u16::try_from(cur.col).unwrap(), 0),
    )
}

fn draw_modal(ctx: &Context, modal: &Modal) -> crossterm::Result<()> {
    let cur = ctx.active_buf.get_cursor();

    execute!(
        stdout(),
        MoveTo(0, 3),
        Clear(ClearType::CurrentLine),
        Print(cur.line + 1),
        MoveTo(12, 3),
        Print(ctx.name.clone()),
        MoveTo(0, 0),
        Clear(ClearType::CurrentLine),
        Print(format!("{}: {}", modal.name, modal.line)),
        MoveTo(u16::try_from(modal.name.len() + 2 + modal.col).unwrap(), 0),
    )
}

fn draw_screen(ctx: &mut Context) -> crossterm::Result<()> {
    match &ctx.state {
        AppState::Edit => draw_edit(ctx),
        AppState::GoToLineModal(modal) => draw_modal(ctx, modal),
        AppState::Quit => Ok(())
    }
}

#[derive(PartialEq)]
enum AppState {
    Edit,
    GoToLineModal(Modal),
    Quit,
}


fn handle_edit_input(ctx: &mut Context, event: Event) {
    match event {
        Event::Key(event) => 
            match event.modifiers {
                KeyModifiers::CONTROL => {
                    match event.code {
                        KeyCode::Char('g') => {
                            ctx.state = AppState::GoToLineModal(Modal::new("Go to line".to_owned()))
                        },
                        _ => (),
                    }
                },
                KeyModifiers::NONE => {
                    match event.code {
                        KeyCode::Esc => {ctx.state = AppState::Quit},
                        KeyCode::Enter => ctx.active_buf.newline(),
                        KeyCode::Up => ctx.active_buf.move_cursor_up(1),
                        KeyCode::Down => ctx.active_buf.move_cursor_down(1),
                        KeyCode::Left => ctx.active_buf.move_cursor_left(1),
                        KeyCode::Right => ctx.active_buf.move_cursor_right(1),
                        KeyCode::Delete => ctx.active_buf.remove_at(),
                        KeyCode::Backspace => ctx.active_buf.remove_before(),
                        KeyCode::Home => ctx.active_buf.move_start_of_line(),
                        KeyCode::End => ctx.active_buf.move_end_of_line(),
                        KeyCode::Char(ch) => ctx.active_buf.insert_char(ch),
                        _ => (),
                    }
                }
                _ => (),
            }
        
        _ => (),
    };
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

fn handle_goto_line_modal_input(ctx: &mut Context, event: Event) {
    if let AppState::GoToLineModal(modal) = &mut ctx.state {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Esc => { ctx.state = AppState::Edit },
                KeyCode::Enter => {
                    let line_no: usize = modal.get_line().parse::<usize>().unwrap();
                    ctx.active_buf.go_to_line(line_no).unwrap();
                    ctx.state = AppState::Edit
                }
                _ => {
                    handle_modal_input(modal, event);
                },
            },
            _ => (),
        };
    }    
}

fn handle_input(ctx: &mut Context, event: Event) {
    match ctx.state {
        AppState::Edit => handle_edit_input(ctx, event),
        AppState::GoToLineModal(_) => handle_goto_line_modal_input(ctx, event),
        AppState::Quit => (),
    }
}

fn app_loop(ctx: &mut Context) -> crossterm::Result<()> {

    // Clean up the screen
    execute!(stdout(), Clear(ClearType::All))?;

    while ctx.state != AppState::Quit {
        draw_screen(ctx)?;
        
        handle_input(ctx, read()?)
    }

    Ok(())
}

fn main() {
    let here = std::fs::File::open("./src/main.rs").unwrap();

    let buffer = Buffer::new(here).unwrap();

    let mut ctx = Context {
        state: AppState::Edit,
        active_buf: buffer,
        name: String::from("[draft]"),
    };

    enable_raw_mode().unwrap();

    app_loop(&mut ctx).unwrap();
}
