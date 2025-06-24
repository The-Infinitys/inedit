
// src/components/bottom_bar.rs

use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use crate::app::App; // App構造体を使用するためにインポート

/// Bottom Barを描画します。右寄せで現在のカーソル位置を表示します。
pub fn render_bottom_bar(f: &mut Frame, area: Rect, app: &App) {
    // カーソル位置の文字列をフォーマット (0-indexed なので +1 して表示)
    let cursor_info = format!("Ln {}, Col {}", app.editor.cursor.y + 1, app.editor.cursor.x + 1);

    let paragraph = Paragraph::new(Line::from(cursor_info).right_aligned())
        .block(Block::default().borders(Borders::TOP)) // 上線で区切る
        .style(Style::default().black().on_white()); // 黒文字、白背景

    f.render_widget(paragraph, area);
}
