use crate::app::{App, LineStatus};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style}, // Modifierをインポート
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// エディタ本体の描画幅（Rect.width）を取得する関数
pub fn get_editor_area_width(area: Rect) -> u16 {
    // left_block, right_block, 余白などを考慮してエディタ本体の幅を計算
    // ここでは仮に、全体エリアからleft_block(6)とright_block(3)を引いた幅とする
    // 必要に応じて正確な値に調整してください
    area.width.saturating_sub(6 + 3)
}

/// Left Block を描画します。行番号と差分を表示します。
pub fn render_left_block(f: &mut Frame, area: Rect, app: &App) {
    let mut lines_to_display: Vec<Line> = Vec::new();
    let theme_bg = app.highlighter.get_background_color();
    let theme_fg = app.highlighter.get_foreground_color();

    let editor_content_width = get_editor_area_width(area); // This is the width for text wrapping
    let mut visual_line_y_counter: u16 = 0; // Global visual line counter
    let mut drawn_lines_count: u16 = 0; // Lines drawn on screen

    // --- 修正: startまでに何個の論理行があったかを数える ---
    // ビューポートの先頭 visual line が属する論理行番号を計算
    'outer: for logical_line_idx in 0..app.editor.lines.len() {
        let wrapped_segments = app.editor.get_visual_lines_for_logical_line(
            logical_line_idx,
            editor_content_width, // Use the width of the editor content area for wrapping
        );

        for (wrap_idx, _segment) in wrapped_segments.iter().enumerate() {
            if visual_line_y_counter >= app.editor.scroll_offset_visual_y {
                let screen_y = area.y + drawn_lines_count;
                if screen_y >= area.bottom() {
                    break 'outer;
                }

                let mut spans: Vec<Span> = Vec::new();
                if wrap_idx == 0 {
                    // Only show line number and diff for the first visual line of a logical line
                    let line_number = (logical_line_idx + 1).to_string();
                    let line_status = app.line_statuses.get(logical_line_idx).copied().unwrap_or(LineStatus::Unchanged);

                    let diff_symbol_style = match line_status {
                        LineStatus::Modified => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                        LineStatus::Added => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                        LineStatus::Unchanged => Style::default().fg(Color::DarkGray),
                    };
                    let diff_symbol = match line_status {
                        LineStatus::Modified => "~",
                        LineStatus::Added => "+",
                        LineStatus::Unchanged => " ",
                    };

                    spans.push(Span::styled(format!("{:>4}", line_number), theme_fg));
                    spans.push(Span::styled(format!("{} ", diff_symbol), diff_symbol_style));
                } else {
                    // For wrapped lines, just show padding
                    spans.push(Span::styled("      ", Style::default().fg(Color::DarkGray)));
                }
                lines_to_display.push(Line::from(spans));
                drawn_lines_count += 1;
            }
            visual_line_y_counter += 1;
        }
    }
    // ビューポートの高さに満たない場合は空行で埋める
    while lines_to_display.len() < area.height as usize {
        lines_to_display.push(Line::from(vec![Span::styled(
            "      ",
            Style::default().fg(Color::DarkGray),
        )]));
    }

    let block = Block::default()
        .borders(Borders::RIGHT)
        .style(Style::default().bg(theme_bg));
    let paragraph = Paragraph::new(lines_to_display)
        .block(block)
        .alignment(Alignment::Right);
    f.render_widget(paragraph, area);
}
