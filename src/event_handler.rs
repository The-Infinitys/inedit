use crate::app::App;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io;
pub fn handle_events(app: &mut App, view_height: usize) -> io::Result<()> {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            let shift = key.modifiers.contains(KeyModifiers::SHIFT);
            match key.code {
                KeyCode::Char('w') => app.should_quit = true,
                KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::ALT) => {
                    app.fold_mode = !app.fold_mode;
                }
                KeyCode::Up => {
                    if app.cursor.0 > 0 { app.cursor.0 -= 1; }
                    if app.cursor.0 < app.scroll { app.scroll = app.cursor.0; }
                    if shift {
                        let sel_start = app.selection.map(|(s, _)| s).unwrap_or(app.cursor);
                        app.selection = Some((sel_start, app.cursor));
                    } else {
                        app.selection = None;
                    }
                }
                KeyCode::Down => {
                    let max = app.buffer.lines().count().saturating_sub(1);
                    if app.cursor.0 < max { app.cursor.0 += 1; }
                    if app.cursor.0 >= app.scroll + view_height {
                        app.scroll = app.cursor.0 + 1 - view_height;
                    }
                    if shift {
                        let sel_start = app.selection.map(|(s, _)| s).unwrap_or(app.cursor);
                        app.selection = Some((sel_start, app.cursor));
                    } else {
                        app.selection = None;
                    }
                }
                KeyCode::Left => {
                    if app.cursor.1 > 0 { app.cursor.1 -= 1; }
                    if shift {
                        let sel_start = app.selection.map(|(s, _)| s).unwrap_or(app.cursor);
                        app.selection = Some((sel_start, app.cursor));
                    } else {
                        app.selection = None;
                    }
                }
                KeyCode::Right => {
                    let line = app.buffer.lines().nth(app.cursor.0).unwrap_or("");
                    if app.cursor.1 < line.chars().count() { app.cursor.1 += 1; }
                    if shift {
                        let sel_start = app.selection.map(|(s, _)| s).unwrap_or(app.cursor);
                        app.selection = Some((sel_start, app.cursor));
                    } else {
                        app.selection = None;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
