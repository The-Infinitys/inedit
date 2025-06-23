use crate::app::App;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

// 右情報（スクロールバー＋diff色付き）
pub fn right_line<'a>(app: &App, height: usize, diff: Option<&[char]>) -> Paragraph<'a> {
    let total = app.buffer.lines().count();
    let visible = height;
    let scroll = app.scroll;
    let mut bar = vec![' '; total.max(visible)];
    if total > 0 && visible < total {
        let bar_height = (visible * visible / total).max(1);
        let bar_top = (scroll * visible / total).min(visible - bar_height);
        for i in bar_top..(bar_top + bar_height) {
            if i < bar.len() {
                bar[i] = '█';
            }
        }
    } else if total > 0 {
        for i in 0..total.min(visible) {
            bar[i] = '█';
        }
    }
    let lines: Vec<Line> = bar
        .iter()
        .enumerate()
        .take(visible)
        .map(|(i, &c)| {
            let mut span = if c == '█' {
                Span::styled(c.to_string(), Style::default().fg(Color::Blue))
            } else {
                Span::raw(" ")
            };
            // diff情報も色付きで表示
            if let Some(diff) = diff {
                if let Some(mark) = diff.get(i + scroll) {
                    let color = match mark {
                        '+' => Color::Green,
                        '-' => Color::Red,
                        _ => Color::Reset,
                    };
                    span = Span::styled(mark.to_string(), Style::default().fg(color));
                }
            }
            Line::from(span)
        })
        .collect();
    Paragraph::new(lines).block(Block::default().borders(Borders::LEFT))
}
