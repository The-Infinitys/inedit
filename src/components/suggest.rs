use crate::app::App;
use crate::app::InputOverlay;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// サジェストバーや検索・置換バーなどのオーバーレイUIを描画します。
pub fn render_overlay(f: &mut Frame, app: &App) {
    match &app.input_overlay {
        InputOverlay::Search { query, .. } => {
            let area = Rect {
                x: 0,
                y: f.area().height - 2,
                width: f.area().width,
                height: 2,
            };
            let text = format!("🔍 検索: {}", query);
            let para = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
            f.render_widget(para, area);
        }
        InputOverlay::Replace {
            query,
            replace,
            focus_replace,
            ..
        } => {
            let area = Rect {
                x: 0,
                y: f.area().height - 3,
                width: f.area().width,
                height: 3,
            };
            let text = format!(
                "🔍 検索: {}  ⬇ 置換: {}{}",
                query,
                replace,
                if *focus_replace { " ←" } else { "" }
            );
            let para = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
            f.render_widget(para, area);
        }
        InputOverlay::Suggest {
            suggestions,
            selected,
            ..
        } => {
            let area = Rect {
                x: 10,
                y: f.area().height - 5,
                width: 30,
                height: suggestions.len().min(5) as u16 + 2,
            };
            let mut lines = vec![];
            for (i, s) in suggestions.iter().enumerate() {
                if i == *selected {
                    lines.push(Line::from(Span::styled(
                        s,
                        Style::default().bg(Color::Blue),
                    )));
                } else {
                    lines.push(Line::from(s.as_str()));
                }
            }
            let para = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));
            f.render_widget(para, area);
        }
        InputOverlay::None => {}
    }
}
