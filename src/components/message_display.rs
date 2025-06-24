// src/components/message_display.rs

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use crate::app::{App, MessageType}; // AppとMessageType構造体を使用するためにインポート

/// アプリケーションのメッセージキューを表示します。
pub fn render_message_display(f: &mut Frame, area: Rect, app: &App) {
    let messages: Vec<Line> = app.messages.iter()
        .map(|(msg_type, msg_content)| {
            let style = match msg_type {
                MessageType::Info => Style::default().fg(Color::Yellow), // 情報メッセージは黄色
                MessageType::Error => Style::default().fg(Color::Red),   // エラーメッセージは赤色
            };
            Line::from(msg_content.clone()).style(style)
        })
        .collect();

    let block = Block::default()
        .borders(Borders::TOP) // 上に境界線
        .title("Messages");

    let paragraph = Paragraph::new(messages)
        .block(block)
        .scroll((0, 0)); // スクロールはさせない（新しいメッセージが下に追加される）

    f.render_widget(paragraph, area);
}
