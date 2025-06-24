
use ratatui::{
    Frame,
    layout::{Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use crate::app::{App, LineStatus}; // AppとLineStatus構造体を使用するためにインポート

/// Right Block を描画します。スクロールバーと差分マーカーを表示します。
pub fn render_right_block(f: &mut Frame, area: Rect, app: &App) {
    let editor_lines_count = app.editor.buffer.lines().count() as u16;
    let viewport_height = area.height;

    let mut scrollbar_content: Vec<Line> = Vec::new();

    // スクロールバーの「つまみ」の位置とサイズを計算
    // つまみの高さはビューポートの高さと全体の高さの比率に比例
    let thumb_height = if editor_lines_count > 0 {
        ((viewport_height as f32 / editor_lines_count as f32) * viewport_height as f32) as u16
    } else {
        0
    };
    let thumb_height = thumb_height.max(1); // 最小1文字の高さ

    // つまみの上端位置
    let thumb_start_y = if editor_lines_count > 0 {
        ((app.editor.scroll_offset_y as f32 / editor_lines_count as f32) * viewport_height as f32) as u16
    } else {
        0
    };
    let thumb_start_y = thumb_start_y.min(viewport_height.saturating_sub(thumb_height));


    for y_on_screen in 0..viewport_height {
        let mut spans: Vec<Span> = Vec::new();
        let mut line_content = String::new();

        // スクロールバーの描画
        if y_on_screen >= thumb_start_y && y_on_screen < thumb_start_y + thumb_height {
            line_content.push('#'); // つまみの部分
        } else {
            line_content.push('|'); // レール部分
        }

        // 差分マーカーの描画
        let corresponding_editor_line_idx = (app.editor.scroll_offset_y + y_on_screen) as usize;
        let diff_marker = if let Some(&status) = app.line_statuses.get(corresponding_editor_line_idx) {
            match status {
                LineStatus::Modified => Style::default().fg(Color::Yellow),
                LineStatus::Added => Style::default().fg(Color::Green),
                LineStatus::Unchanged => Style::default().fg(Color::DarkGray),
            }
        } else {
            Style::default().fg(Color::DarkGray) // 該当する行がない場合
        };

        // マーカーの文字 (' ', '~', '+')
        let marker_char = if let Some(&status) = app.line_statuses.get(corresponding_editor_line_idx) {
            match status {
                LineStatus::Modified => '~',
                LineStatus::Added => '+',
                LineStatus::Unchanged => ' ',
            }
        } else {
            ' ' // 該当する行がない場合
        };

        spans.push(Span::raw(line_content)); // スクロールバー部分
        spans.push(Span::styled(marker_char.to_string(), diff_marker)); // 差分マーカー

        scrollbar_content.push(Line::from(spans));
    }


    let block = Block::default()
        .borders(Borders::LEFT) // 左側に境界線
        .style(Style::default().bg(Color::Rgb(30, 30, 30))); // 暗い背景色

    let paragraph = Paragraph::new(scrollbar_content)
        .block(block)
        .alignment(Alignment::Left); // 左寄せ

    f.render_widget(paragraph, area);
}
