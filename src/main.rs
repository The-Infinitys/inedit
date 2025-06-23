use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use inedit::{app, event_handler, ui};
use ratatui::prelude::*;
use std::env;
use std::fs;
use std::io::{self, Stdout, Write, stdout};
use std::path::PathBuf;
use ui::ui;
use uuid::Uuid;

fn generate_tmp_path(original_path: Option<&String>) -> Option<PathBuf> {
    if let Some(orig) = original_path {
        let mut p = PathBuf::from(orig);
        p.set_file_name(format!(
            ".inedit_tmp_{}_{}",
            p.file_name()?.to_string_lossy(),
            Uuid::new_v4()
        ));
        Some(p)
    } else {
        Some(PathBuf::from(format!(".inedit_tmp_{}", Uuid::new_v4())))
    }
}

fn create_tmp_file(original_path: Option<&String>, buffer: &str) -> Option<String> {
    let tmp_path = generate_tmp_path(original_path)?;
    fs::write(&tmp_path, buffer).ok()?;
    Some(tmp_path.to_string_lossy().to_string())
}

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
    let file_path = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };
    let tmp_file_path = generate_tmp_path(file_path.as_ref());

    // 仮ファイルが存在する場合、復元するか確認
    let mut buffer = buffer;
    if let Some(ref tmp_path) = tmp_file_path {
        if std::path::Path::new(tmp_path).exists() {
            print!("編集中の仮ファイルが見つかりました。復元しますか？ (y/n): ");
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            if input.trim().eq_ignore_ascii_case("y") {
                buffer = std::fs::read_to_string(tmp_path).unwrap_or(buffer);
                println!("仮ファイルから復元しました。");
            } else {
                println!("仮ファイルは無視されます。");
            }
        }
    }

    let tmp_file_path = create_tmp_file(file_path.as_ref(), &buffer);
    let mut app = App::new(buffer, file_path, tmp_file_path);

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
