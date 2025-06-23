use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use inedit::{app, event_handler, ui};
use ratatui::prelude::*;
use std::env;
use std::fs;
use std::io::{self, Stdout, stdout};
use ui::ui;

fn main() -> io::Result<()> {
    // コマンドライン引数の取得
    let args: Vec<String> = env::args().collect();
    let buffer = if args.len() > 1 {
        // ファイルパスが指定されていれば読み込む
        fs::read_to_string(&args[1]).unwrap_or_else(|_| String::new())
    } else {
        // 指定がなければ空バッファ
        String::new()
    };

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App::new()にbufferを渡す設計に今後拡張可能
    let file_path = if args.len() > 1 { Some(args[1].clone()) } else { None };
    let mut app = App::new(buffer, file_path);

    while !app.should_quit {
        let mut editor_height = 0;
        terminal
            .draw(|f| {
                // レイアウトを再現してeditorの高さを取得
                let size = f.size();
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(1),
                        ratatui::layout::Constraint::Min(1),
                        ratatui::layout::Constraint::Length(1),
                    ])
                    .split(size);
                editor_height = chunks[1].height as usize;
                ui::<CrosstermBackend<Stdout>>(f, &mut app);
            })
            .unwrap();
        event_handler::handle_events(&mut app, editor_height)?;
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
