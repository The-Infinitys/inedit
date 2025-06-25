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
    let editor_area_width = crate::components::middle_block::left_block::get_editor_area_width(area);
    let wrap_width = if app.word_wrap_enabled {
        editor_area_width as usize
    } else {
        usize::MAX
    };
    let visual_lines = app.editor.get_visual_lines_with_width_word_wrap(wrap_width);
    let visual_lines_count = visual_lines.len() as u16;
    let viewport_height = area.height;

    let mut scrollbar_content: Vec<Line> = Vec::new();

    // テーマの背景色を取得
    let theme_bg = app.highlighter.get_background_color();

    // スクロールバーの「つまみ」の位置とサイズを計算
    let thumb_height = if visual_lines_count > 0 {
        ((viewport_height as f32 / visual_lines_count as f32) * viewport_height as f32) as u16
    } else {
        0
    };
    let thumb_height = thumb_height.max(1); // 最小1文字の高さ

    // つまみの上端位置
    let thumb_start_y = if visual_lines_count > 0 {
        ((app.editor.scroll_offset_y as f32 / visual_lines_count as f32) * viewport_height as f32) as u16
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
        let visual_idx = (app.editor.scroll_offset_y + y_on_screen) as usize;
        let buf_idx = if visual_idx < visual_lines.len() {
            visual_lines[visual_idx].0
        } else {
            usize::MAX
        };
        if buf_idx != usize::MAX {
            let status = app
                .line_statuses
                .get(buf_idx)
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
