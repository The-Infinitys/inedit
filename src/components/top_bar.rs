// src/components/top_bar.rs

use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
}; // App構造体を使用するためにインポート

/// Top Barを描画します。中央寄せでタイトルを表示します。
pub fn render_top_bar(f: &mut Frame, area: Rect, app: &App) {
    // タイトルを決定
    let title = if let Some(path) = &app.target_path {
        format!("{} - InEdit", path.display()) // ファイルパスがあればそれを表示
    } else {
        "Untitled - InEdit".to_string() // なければ「Untitled」
    };

    let paragraph = Paragraph::new(Line::from(title).centered())
        .block(Block::default().borders(Borders::BOTTOM)) // 下線で区切る
        .style(Style::default().add_modifier(Modifier::BOLD)); // 太字にする

    f.render_widget(paragraph, area);
}
