use crate::app::{App, LineStatus};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Right Block を描画します。スクロールバーと差分マーカーを表示します。
pub fn render_right_block(f: &mut Frame, area: Rect, app: &App) {
    // --- 修正: visual line数ベースで計算 ---
    let editor_content_width = crate::components::middle_block::left_block::get_editor_area_width(area);
    let mut total_visual_lines: u16 = 0;
    for logical_line_idx in 0..app.editor.lines.len() {
        total_visual_lines += app.editor.get_visual_lines_for_logical_line(
            logical_line_idx,
            editor_content_width,
        ).len() as u16;
    }
    let viewport_height = area.height;

    let mut scrollbar_content: Vec<Line> = Vec::new();

    // テーマの背景色を取得
    let theme_bg = app.highlighter.get_background_color();

    // スクロールバーの「つまみ」の位置とサイズを計算
    let thumb_height = if total_visual_lines > 0 {
        ((viewport_height as f32 / total_visual_lines as f32) * viewport_height as f32) as u16
    } else {
        0
    };
    let thumb_height = thumb_height.max(1); // 最小1文字の高さ

    // つまみの上端位置
    let thumb_start_y = if total_visual_lines > 0 {
        ((app.editor.scroll_offset_visual_y as f32 / total_visual_lines as f32) * viewport_height as f32) as u16
    } else {
        0
    };
    let thumb_start_y = thumb_start_y.min(viewport_height.saturating_sub(thumb_height));

    for y_on_screen in 0..viewport_height {
        let mut spans: Vec<Span> = Vec::new();
        let mut line_content = String::new();

        // スクロールバーの描画
        if y_on_screen >= thumb_start_y && y_on_screen < thumb_start_y + thumb_height {
            // つまみの色
            line_content.push('#');
        } else {
            // レール部分の色
            line_content.push('|');
        }
        spans.push(Span::styled(
            line_content,
            Style::default().fg(Color::DarkGray),
        )); // スクロールバーの色は固定

        // 差分マーカーの描画 (visual lineに対応する論理行で判定)
        let current_visual_y_on_screen = app.editor.scroll_offset_visual_y + y_on_screen;
        let mut logical_line_for_marker: Option<usize> = None;
        let mut visual_line_counter_for_marker: u16 = 0;

        'find_logical_line: for logical_idx in 0..app.editor.lines.len() {
            let wrapped_segments = app.editor.get_visual_lines_for_logical_line(
                logical_idx,
                editor_content_width,
            );
            for (wrap_idx, _segment) in wrapped_segments.iter().enumerate() {
                if visual_line_counter_for_marker == current_visual_y_on_screen {
                    // Only show diff marker for the first visual line of a logical line
                    if wrap_idx == 0 {
                        logical_line_for_marker = Some(logical_idx);
                    }
                    break 'find_logical_line;
                }
                visual_line_counter_for_marker += 1;
            }
        }

        if let Some(logical_idx) = logical_line_for_marker {
            let status = app
                .line_statuses
                .get(logical_idx)
                .copied()
                .unwrap_or(LineStatus::Unchanged);

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
            spans.push(Span::raw(" "));
        }
        scrollbar_content.push(Line::from(spans));
    }

    let block = Block::default()
        .borders(Borders::LEFT) // 左側に境界線
        .style(Style::default().bg(theme_bg)); // テーマの背景色を適用

    let paragraph = Paragraph::new(scrollbar_content)
        .block(block)
        .alignment(Alignment::Left); // 左寄せ

    f.render_widget(paragraph, area);
}
