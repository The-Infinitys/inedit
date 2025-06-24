use crate::app::{App, LineStatus};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
}; // AppとLineStatus構造体を使用するためにインポート

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
        ((app.editor.scroll_offset_y as f32 / editor_lines_count as f32) * viewport_height as f32)
            as u16
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
        spans.push(Span::raw(line_content)); // スクロールバー部分

        // 差分マーカーの描画 (該当する行がある場合のみ)
        let corresponding_editor_line_idx = (app.editor.scroll_offset_y + y_on_screen) as usize;
        let total_editor_lines = app.editor.buffer.lines().count(); // バッファの実際の総行数

        if corresponding_editor_line_idx < total_editor_lines {
            // 該当する行が存在する場合のみ差分マーカーを表示
            let status = app
                .line_statuses
                .get(corresponding_editor_line_idx)
                .copied()
                .unwrap_or(LineStatus::Unchanged); // 安全のため

            let diff_style = match status {
                LineStatus::Modified => Style::default().fg(Color::Yellow),
                LineStatus::Added => Style::default().fg(Color::Green),
                LineStatus::Unchanged => Style::default().fg(Color::DarkGray),
            };
            let marker_char = match status {
                LineStatus::Modified => '~',
                LineStatus::Added => '+',
                LineStatus::Unchanged => ' ',
            };
            spans.push(Span::styled(marker_char.to_string(), diff_style));
        } else {
            // 該当する行がない場合は空のSpanを追加し、スペースを確保（可視文字は描画しない）
            spans.push(Span::raw(" "));
        }
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
