use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
}; // App構造体を使用するためにインポート

/// Top Bar を描画します。ファイル名、言語、変更状態などを表示します。
pub fn render_top_bar(f: &mut Frame, area: Rect, app: &App) {
    let filename = app
        .target_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("untitled");

    let modified_indicator = if app.has_unsaved_changes() { "*" } else { "" };

    let current_language = &app.current_syntax_name;
    let current_theme = &app.highlighter.current_theme_name;

    // テーマの背景色と前景色を取得
    let theme_bg = app.highlighter.get_background_color();
    let theme_fg = app.highlighter.get_foreground_color();

    let text_content = format!(
        " {} {}{} | Language: {} | Theme: {} ",
        app.editor.search_query, // 検索クエリがあれば表示
        filename,
        modified_indicator,
        current_language,
        current_theme
    );

    let paragraph = Paragraph::new(Line::from(text_content))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .style(Style::default().bg(theme_bg)),
        ) // テーマの背景色を適用
        .alignment(Alignment::Left) // 左寄せ
        .style(Style::default().fg(theme_fg).add_modifier(Modifier::BOLD)); // テーマの前景色を適用

    f.render_widget(paragraph, area);
}
