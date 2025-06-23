use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use inedit::{app, event_handler, ui};
use ratatui::prelude::*;
use std::env;
use std::fs;
use std::io::{self, Write, stdout};
use ui::ui;
use uuid::Uuid;
use uuid::uuid;
fn generate_tmp_path(original_path: Option<&String>) -> Option<String> {
    use std::path::PathBuf;
    if let Some(orig) = original_path {
        let abs_path = if PathBuf::from(orig).is_absolute() {
            PathBuf::from(orig)
        } else {
            std::env::current_dir().ok()?.join(orig)
        };
        let abs_str = abs_path.to_string_lossy();
        let uuid = Uuid::new_v5(
            &uuid!("6ba7b811-9dad-11d1-80b4-00c04fd430c8"),
            abs_str.as_bytes(),
        );
        // file_nameがNoneなら"untitled"を使う
        let file_stem = abs_path.file_name()
            .map(|s| s.to_string_lossy())
            .unwrap_or_else(|| "untitled".into());
        let mut p = abs_path.clone();
        p.set_file_name(format!(".inedit_tmp_{}_{}", file_stem, uuid));
        Some(p.to_string_lossy().to_string())
    } else {
        Some(format!(".inedit_tmp_{}", Uuid::new_v4()))
    }
}

fn create_tmp_file(original_path: Option<&String>, buffer: &str) -> Option<String> {
    let tmp_path = generate_tmp_path(original_path)?;
    fs::write(&tmp_path, buffer).ok()?;
    Some(tmp_path)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    // ファイル引数がなければ Untitled-{number}.txt を生成
    let file_path = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        // 既存の Untitled-*.txt を探して重複しない番号を付与
        let mut n = 1;
        let file_name = loop {
            let name = format!("Untitled-{}.txt", n);
            if !std::path::Path::new(&name).exists() {
                break name;
            }
            n += 1;
        };
        Some(file_name)
    };

    let mut buffer = if let Some(ref path) = file_path {
        if std::path::Path::new(path).exists() {
            fs::read_to_string(path).unwrap_or_default()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let tmp_file_path = generate_tmp_path(file_path.as_ref());

    // 仮ファイルが存在する場合、復元するか確認
    let mut use_tmp_file_path = tmp_file_path.clone();
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
                // 仮ファイルを新規作成
                use_tmp_file_path = create_tmp_file(file_path.as_ref(), &buffer);
            }
        } else {
            // 仮ファイルがなければ新規作成
            use_tmp_file_path = create_tmp_file(file_path.as_ref(), &buffer);
        }
    }

    let mut app = App::new(buffer, file_path, use_tmp_file_path);

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    while !app.should_quit {
        let mut editor_height = 0;
        terminal
            .draw(|f| {
                let size = f.area();
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(1),
                        ratatui::layout::Constraint::Min(1),
                        ratatui::layout::Constraint::Length(1),
                    ])
                    .split(size);
                editor_height = chunks[1].height as usize;
                ui(f, &mut app);
            })
            .unwrap();
        event_handler::handle_events(&mut app, editor_height)?;
    }

    // 終了時に一時ファイルを削除
    if let Some(ref tmp_path) = app.tmp_file_path {
        let _ = fs::remove_file(tmp_path);
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}