use crossterm::{
    event::{self, Event as CEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use inedit::app::{App, AppControlFlow, MessageType};
use inedit::event_handler;
use inedit::ui;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use std::panic;

// Helper struct to ensure terminal cleanup on drop
struct TerminalGuard<B: ratatui::backend::Backend + io::Write>(Terminal<B>);

impl<B: ratatui::backend::Backend + io::Write> Drop for TerminalGuard<B> {
    fn drop(&mut self) {
        // These operations can fail, so we ignore their results.
        let _ = disable_raw_mode();
        let _ = execute!(self.0.backend_mut(), LeaveAlternateScreen);
        let _ = self.0.show_cursor();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable raw mode and enter alternate screen
    enable_raw_mode()?;
    let stdout = io::stdout();
    execute!(io::stdout(), EnterAlternateScreen)?;

    // Create terminal and wrap it in a guard for automatic cleanup
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    let mut terminal_guard = TerminalGuard(terminal);

    let mut app = App::init();
    let tick_rate = Duration::from_millis(250);

    // Use catch_unwind to gracefully handle panics and ensure terminal cleanup
    let res = panic::catch_unwind(panic::AssertUnwindSafe(move || {
        run_app(&mut terminal_guard.0, &mut app, tick_rate)
    }));

    // `terminal_guard` will be dropped here, executing its `Drop` implementation
    // which restores the terminal to its original state, regardless of whether
    // `run_app` returned normally or panicked.

    match res {
        Ok(Ok(())) => Ok(()), // `run_app` completed successfully
        Ok(Err(e)) => Err(Box::new(e)), // `run_app` returned an `io::Error`
        Err(panic_payload) => {
            // A panic occurred inside `run_app`
            let panic_message = if let Some(s) = panic_payload.downcast_ref::<String>() {
                format!("Application panicked: {}", s)
            } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                format!("Application panicked: {}", s)
            } else {
                "Application panicked with an unknown error (payload type unknown)".to_string()
            };
            eprintln!("{}", panic_message);
            Err(panic_message.into()) // Convert String to Box<dyn Error>
        }
    }
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    _tick_rate: Duration,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw_ui(f, app))?;

        // --- ここを修正 ---
        // キーイベントを取得してhandle_eventに渡す
        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key_event) = event::read()? {
                let control_flow = event_handler::handle_event(app, &key_event)?;
                match control_flow {
                    AppControlFlow::Exit => {
                        return Ok(());
                    }
                    AppControlFlow::TriggerSaveAndExit => match app.save_current_file() {
                        Ok(_) => return Ok(()),
                        Err(e) => {
                            app.add_message(MessageType::Error, format!("保存エラー: {}", e));
                        }
                    },
                    AppControlFlow::TriggerDiscardAndExit => {
                        return Ok(());
                    }
                    AppControlFlow::Continue => {
                        // 何もしない
                    }
                    AppControlFlow::ShowExitPopup => {
                        // ポップアップ表示中はループ継続
                    }
                }
            }
        }
    }
}
