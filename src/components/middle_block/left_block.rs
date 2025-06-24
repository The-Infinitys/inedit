use crate::app::{App, LineStatus};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
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

    // 折返しモード対応: エディタ本体のwrap幅でvisual_linesを取得
    let editor_area_width = get_editor_area_width(area);
    let wrap_width = if app.word_wrap_enabled {
        editor_area_width as usize
    } else {
        usize::MAX // wrapしない場合は非常に大きな値
    };
    let visual_lines = app.editor.get_visual_lines_with_width_word_wrap(wrap_width);
    let total_visual_lines = visual_lines.len();
    let start = app.editor.scroll_offset_y as usize;
    let end = (start + area.height as usize).min(total_visual_lines);
    let visible_lines = &visual_lines[start..end];

    // --- 修正: startまでに何個の論理行があったかを数える ---
    // ビューポートの先頭 visual line が属する論理行番号を計算
    let mut logical_line_counter = 1;
    if start > 0 {
        // startまでのvisual_linesでwrap_idx==0の数を数える
        logical_line_counter += visual_lines[..start]
            .iter()
            .filter(|(_, wrap_idx, _)| *wrap_idx == 0)
            .count();
    }
    // 折返し1行目ごとに論理行番号をインクリメントして表示し、折返し2行目以降は空白にする
    for (buf_idx, wrap_idx, _line_str) in visible_lines.iter() {
        if *wrap_idx == 0 {
            // 論理行の先頭 visual line のみ行番号を表示
            let line_number = logical_line_counter.to_string();
            let line_status = app.line_statuses.get(*buf_idx).copied().unwrap_or(LineStatus::Unchanged);
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
            logical_line_counter += 1;
        } else {
            // 折返し部分には行番号・差分とも空白
            let line_num_span = Span::styled("    ", Style::default().fg(Color::DarkGray));
            let diff_span = Span::styled("  ", Style::default().fg(Color::DarkGray));
            lines_to_display.push(Line::from(vec![line_num_span, diff_span]));
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
