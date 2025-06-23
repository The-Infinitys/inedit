use crate::app::App;
use crossterm::event::{self, Event, KeyCode};

pub fn handle_event(app: &mut App) -> std::io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('w') => return Ok(true),
                KeyCode::Char(c) => app.editor.buffer.push(c),
                KeyCode::Backspace => {
                    app.editor.buffer.pop();
                }
                _ => {}
            }
        }
    }
    Ok(false)
}
