use crate::app::App;
use crossterm::event::{self, Event, KeyCode};
use std::io;
pub fn handle_events(app: &mut App) -> io::Result<()> {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('w') => app.should_quit = true,
                _ => {}
            }
        }
    }
    Ok(())
}
