use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn editor(f: &mut Frame, area: Rect, app: &App, height: usize) {
    use ratatui::style::{Color, Style};
    let lines: Vec<String> = app.buffer.lines().map(|l| l.to_string()).collect();
    let start = app.scroll;
    let mut display_lines = Vec::new();
    let selection = app.selection;

    for (i, line) in lines.iter().enumerate().skip(start).take(height) {
        let global_line = i; // バッファ全体での行番号
        if let Some((start_sel, end_sel)) = selection {
            let (sy, sx) = start_sel;
            let (ey, ex) = end_sel;
            let (sel_start_line, sel_start_col, sel_end_line, sel_end_col) = if (sy, sx) <= (ey, ex)
            {
                (sy, sx, ey, ex)
            } else {
                (ey, ex, sy, sx)
            };
            if global_line >= sel_start_line && global_line <= sel_end_line {
                let mut spans = Vec::new();
                let len = line.chars().count();
                let (sel_start, sel_end) = if sel_start_line == sel_end_line {
                    (sel_start_col.min(len), sel_end_col.min(len))
                } else if global_line == sel_start_line {
                    (sel_start_col.min(len), len)
                } else if global_line == sel_end_line {
                    (0, sel_end_col.min(len))
                } else {
                    (0, len)
                };
                for (j, c) in line.chars().enumerate() {
                    if j >= sel_start && j < sel_end {
                        spans.push(Span::styled(
                            c.to_string(),
                            Style::default().bg(Color::Blue),
                        ));
                    } else {
                        spans.push(Span::raw(c.to_string()));
                    }
                }
                display_lines.push(Line::from(spans));
                continue;
            }
        }
        display_lines.push(Line::from(Span::raw(line.clone())));
    }
    let para = Paragraph::new(display_lines).block(Block::default().borders(Borders::NONE));
    f.render_widget(para, area);

    // カーソル位置を設定（行末を超えないように）
    let cursor_y = app.cursor.0.saturating_sub(app.scroll);
    let line = lines
        .get(app.cursor.0)
        .map(|l| l.chars().count())
        .unwrap_or(0);
    let cursor_x = app.cursor.1.min(line);
    if cursor_y < height {
        f.set_cursor_position((area.x + cursor_x as u16, area.y + cursor_y as u16));
    }
}
