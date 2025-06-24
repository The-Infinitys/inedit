use crate::app::{App, LineStatus};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Left Block を描画します。行番号と差分を表示します。
pub fn render_left_block(f: &mut Frame, area: Rect, app: &App) {
    let mut lines_to_display: Vec<Line> = Vec::new();
    let theme_bg = app.highlighter.get_background_color();
    let theme_fg = app.highlighter.get_foreground_color();

    // 折返しモード対応: 画面上の各行に対してバッファ行番号・折返し行番号を取得
    let visual_lines = if app.word_wrap_enabled {
        // スクロールオフセットからarea.height分だけ詰めて表示
        let all = app.editor.get_visual_lines_with_width(area.width as usize);
        let start = app.editor.scroll_offset_y as usize;
        let end = (start + area.height as usize).min(all.len());
        all[start..end].to_vec()
    } else {
        let all: Vec<_> = app.editor.buffer.lines().enumerate().map(|(i, line)| (i, 0, line.to_string())).collect();
        let start = app.editor.scroll_offset_y as usize;
        let end = (start + area.height as usize).min(all.len());
        all[start..end].to_vec()
    };

    // 表示する行数分だけlines_to_displayを構築
    for (buf_idx, wrap_idx, _line_str) in visual_lines.iter() {
        if *wrap_idx == 0 {
            let line_number = (*buf_idx + 1).to_string();
            let line_status = app
                .line_statuses
                .get(*buf_idx)
                .copied()
                .unwrap_or(LineStatus::Unchanged);
            let diff_symbol_style = match line_status {
                LineStatus::Modified => Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD),
                LineStatus::Added => Style::default().fg(Color::Green).add_modifier(ratatui::style::Modifier::BOLD),
                LineStatus::Unchanged => Style::default().fg(Color::DarkGray),
            };
            let diff_symbol = match line_status {
                LineStatus::Modified => "~",
                LineStatus::Added => "+",
                LineStatus::Unchanged => " ",
            };
            let line_num_span = Span::styled(format!("{:>4}", line_number), theme_fg);
            let diff_span = Span::styled(format!("{} ", diff_symbol), diff_symbol_style);
            lines_to_display.push(Line::from(vec![line_num_span, diff_span]));
        } else {
            // 折返し2行目以降は行番号・差分を空白で埋める
            lines_to_display.push(Line::from(vec![Span::styled("      ", Style::default().fg(Color::DarkGray))]));
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
