use ratatui::widgets::{Block, Borders, Paragraph};

// 右情報
pub fn right_line<'a>() -> Paragraph<'a> {
    Paragraph::new("info").block(Block::default().borders(Borders::LEFT))
}
