use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use inedit::{app, event_handler, ui};
use ratatui::prelude::*;
use std::io::{self, Stdout, stdout};
use ui::ui; // ← ui関数をインポート

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    while !app.should_quit {
        terminal
            .draw(|f| ui::<CrosstermBackend<Stdout>>(f, &app))
            .unwrap();
        event_handler::handle_events(&mut app)?;
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
