// src/components/message_display.rs

use crate::app::{App, MessageType}; // AppとMessageType構造体を使用するためにインポート
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use std::time::Duration; // Durationをインポート

/// アプリケーションのメッセージキューを表示します。（通知スタイル）
pub fn render_message_display(f: &mut Frame, area: Rect, app: &App) {
    const MESSAGE_LIFETIME_SECS: u64 = 3; // メッセージの表示期間（秒）

    // 現在時刻から、表示期間内のメッセージのみをフィルタリング
    let now = std::time::Instant::now();
    let mut visible_messages: Vec<Line> = app
        .messages
        .iter()
        .filter(|(_, _, timestamp)| {
            now.duration_since(*timestamp) < Duration::from_secs(MESSAGE_LIFETIME_SECS)
        })
        .map(|(msg_type, msg_content, _)| {
            let style = match msg_type {
                MessageType::Info => Style::default().fg(Color::Yellow), // 情報メッセージは黄色
                MessageType::Error => Style::default().fg(Color::Red),   // エラーメッセージは赤色
            };
            Line::from(msg_content.clone()).style(style)
        })
        .collect();

    // 最新のメッセージが下に来るように逆順に並べ替え（画面下から上へ表示するため）
    visible_messages.reverse();

    // ui.rs で既に表示エリアの高さが調整されているため、ここでのtruncateは不要です。
    // Paragraphウィジェットは、渡されたTextの行数がarea.heightを超える場合、自動的にクリップします。

    // 表示するメッセージがない場合、または表示エリアの高さが0の場合は描画しない
    if visible_messages.is_empty() || area.height == 0 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL) // 境界線
        .style(Style::default().bg(Color::DarkGray)); // 暗い背景色で通知感を出す

    let paragraph = Paragraph::new(visible_messages)
        .block(block)
        .alignment(ratatui::layout::Alignment::Right) // 右寄せ
        .scroll((0, 0)); // スクロールはさせない

    f.render_widget(paragraph, area);
}
