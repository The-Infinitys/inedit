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
    let editor_lines: Vec<&str> = app.editor.buffer.lines().collect();

    let start_line_idx = app.editor.scroll_offset_y as usize;
    let end_line_idx_in_buffer =
        (editor_lines.len() as u16).min(app.editor.scroll_offset_y + area.height) as usize;

    // テーマの背景色と前景色を取得
    let theme_bg = app.highlighter.get_background_color();
    let theme_fg = app.highlighter.get_foreground_color();

    // 表示する行の範囲で行番号と差分ステータスを生成
    for i in start_line_idx..end_line_idx_in_buffer {
        let line_number = (i + 1).to_string(); // 1-indexed
        let line_status = app
            .line_statuses
            .get(i)
            .copied()
            .unwrap_or(LineStatus::Unchanged);

        let diff_symbol_style = match line_status {
            LineStatus::Modified => Style::default()
                .fg(Color::Yellow)
                .add_modifier(ratatui::style::Modifier::BOLD),
            LineStatus::Added => Style::default()
                .fg(Color::Green)
                .add_modifier(ratatui::style::Modifier::BOLD),
            LineStatus::Unchanged => Style::default().fg(Color::DarkGray), // 変更なしは暗い灰色
        };
        let diff_symbol = match line_status {
            LineStatus::Modified => "~",
            LineStatus::Added => "+",
            LineStatus::Unchanged => " ",
        };

        // 行番号を右寄せ、差分シンボルをその左に配置
        let line_num_span = Span::styled(
            format!("{:>4}", line_number), // 行番号を4桁に右寄せ
            theme_fg,                      // テーマの前景色を適用
        );
        let diff_span = Span::styled(
            format!("{} ", diff_symbol), // シンボルとスペース
            diff_symbol_style,
        );

        lines_to_display.push(Line::from(vec![line_num_span, diff_span]));
    }

    // バッファの実際の行数よりもビューポートの高さが大きい場合、残りの領域を空行で埋める
    while lines_to_display.len() < area.height as usize {
        lines_to_display.push(Line::from(vec![Span::styled(
            "      ",
            Style::default().fg(Color::DarkGray), // 空行の色
        )]));
    }

    let block = Block::default()
        .borders(Borders::RIGHT) // 右側に境界線
        .style(Style::default().bg(theme_bg)); // テーマの背景色を適用

    let paragraph = Paragraph::new(lines_to_display)
        .block(block)
        .alignment(Alignment::Right); // 全体を右寄せにする

    f.render_widget(paragraph, area);
}
