use crate::app::App;
use ratatui::{
    style::{Color, Style, Modifier},
    text::{Span, Line},
    widgets::Paragraph,
};
use std::fs;

pub mod about;

// 仮ファイルと本体ファイルの差分行数を計算
fn count_diff_lines(app: &App) -> usize {
    if let (Some(tmp_path), Some(file_path)) = (&app.tmp_file_path, &app.file_path) {
        if let (Ok(tmp), Ok(orig)) = (
            fs::read_to_string(tmp_path),
            fs::read_to_string(file_path),
        ) {
            let tmp_lines: Vec<&str> = tmp.lines().collect();
            let orig_lines: Vec<&str> = orig.lines().collect();
            tmp_lines.iter().zip(orig_lines.iter())
                .filter(|(a, b)| a != b)
                .count()
                + tmp_lines.len().max(orig_lines.len()) - tmp_lines.len().min(orig_lines.len())
        } else {
            0
        }
    } else {
        0
    }
}

// TopBarコンポーネント
pub fn top_bar<'a>(app: &App, width: u16) -> Paragraph<'a> {
    let fg = Color::Black;
    let bg = Color::White;

    // ファイル名
    let file_name = app.file_path.as_ref()
        .map(|p| std::path::Path::new(p).file_name().unwrap_or_default().to_string_lossy())
        .unwrap_or_else(|| "Untitled".into());

    // セーブ状態
    let saved = if app.is_saved { "保存済" } else { "未保存" };

    // 仮ファイルと本体の差分行数
    let changed = count_diff_lines(app);

    // ファイル名を中央寄せ
    let file_name_str = format!(" {} ", file_name);
    let left_pad = ((width as isize - file_name_str.len() as isize) / 2).max(0) as usize;
    let mut line = String::new();
    line.push_str(&" ".repeat(left_pad));
    line.push_str(&file_name_str);

    // 右端に状態を表示
    let right_info = format!(" {} | 差分: {}行 ", saved, changed);
    let total_len = line.len() + right_info.len();
    if total_len < width as usize {
        line.push_str(&" ".repeat(width as usize - total_len));
    }
    line.push_str(&right_info);

    Paragraph::new(Line::from(vec![
        Span::styled(line, Style::default().fg(bg).bg(fg).add_modifier(Modifier::BOLD)),
    ]))
}
