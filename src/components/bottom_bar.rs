use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
}; // App構造体を使用するためにインポート

/// Bottom Bar を描画します。カーソル位置、行数などを表示します。
pub fn render_bottom_bar(f: &mut Frame, area: Rect, app: &App) {
    let cursor_pos = app.editor.cursor.get_current_pos();
    let total_lines = app.editor.lines.len();
    let _col_count = if let Some(line) = app.editor.lines.get(cursor_pos.1 as usize) {
        // 現在の行の論理的な文字数
        line.chars().count()
    } else {
        0
    };

    // テーマの背景色と前景色を取得
    let theme_bg = app.highlighter.get_background_color();
    let theme_fg = app.highlighter.get_foreground_color();

    let text_content = format!(
        " Ln {}, Col {} | Total Lines: {} ",
        cursor_pos.1.saturating_add(1), // 1-indexed line number
        cursor_pos.0.saturating_add(1), // 1-indexed column number
        total_lines
    );

    let paragraph = Paragraph::new(Line::from(text_content))
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default().bg(theme_bg)),
        ) // テーマの背景色を適用
        .alignment(Alignment::Right) // 右寄せ
        .style(Style::default().fg(theme_fg).add_modifier(Modifier::BOLD)); // テーマの前景色を適用

    f.render_widget(paragraph, area);
}
