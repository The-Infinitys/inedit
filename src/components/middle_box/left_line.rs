use crate::app::App;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub mod diff;

// スクロール位置と高さを考慮して行番号を生成
fn make_left_line<'a>(
    buffer: &'a str,
    diff: Option<&[char]>,
    scroll: usize,
    height: usize,
) -> Vec<Line<'a>> {
    let mut result = Vec::new();
    let lines: Vec<&str> = buffer.lines().collect();
    let diff = diff.unwrap_or(&[]);
    let end = (scroll + height).min(lines.len());
    for i in scroll..end {
        let line_num = i + 1;
        let diff_mark = diff.get(i).copied().unwrap_or(' ');
        // 行番号の色
        let color = match diff_mark {
            '+' => Color::Green,
            '-' => Color::Red,
            '~' => Color::Yellow,
            _ => Color::Reset,
        };
        // 削除された行がこの行の直後にある場合、下線を付ける
        let underline = if diff.get(i + 1) == Some(&'-') {
            Modifier::UNDERLINED
        } else {
            Modifier::empty()
        };
        result.push(Line::from(vec![
            Span::styled(
                format!("{:>3}", line_num),
                Style::default().fg(color).add_modifier(underline),
            ),
            Span::styled(diff_mark.to_string(), Style::default().fg(color)),
        ]));
    }
    result
}

// 左行番号
pub fn left_line<'a>(app: &'a App, diff: Option<&[char]>, height: usize) -> Paragraph<'a> {
    let lines = make_left_line(&app.buffer, diff, app.scroll, height);
    Paragraph::new(lines).block(Block::default().borders(Borders::RIGHT))
}
