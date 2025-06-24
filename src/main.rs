use crossterm::{
    event::{self, Event as CEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use inedit::app::{App, AppControlFlow, MessageType};
use inedit::event_handler;
use inedit::ui;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::init();
    let tick_rate = Duration::from_millis(250);

    let res = run_app(&mut terminal, &mut app, tick_rate);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
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
