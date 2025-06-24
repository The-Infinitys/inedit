use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style, Modifier},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use crate::app::App; // App構造体を使用するためにインポート

/// Top Bar を描画します。ファイル名、言語、変更状態などを表示します。
pub fn render_top_bar(f: &mut Frame, area: Rect, app: &App) {
    let filename = app.target_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("untitled");

    let modified_indicator = if app.has_unsaved_changes() { "*" } else { "" };

    let current_language = &app.current_syntax_name;
    let current_theme = &app.highlighter.current_theme_name;


    let text_content = format!(
        " {} {}{} | Language: {} | Theme: {} ",
        app.editor.search_query, // 検索クエリがあれば表示
        filename,
        modified_indicator,
        current_language,
        current_theme
    );

    let paragraph = Paragraph::new(Line::from(text_content))
        .block(Block::default().borders(Borders::BOTTOM)) // 下側に境界線
        .alignment(Alignment::Left) // 左寄せ
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)); // 明るい青色

    f.render_widget(paragraph, area);
}
