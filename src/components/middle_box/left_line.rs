use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

// 左行番号
pub fn left_line<'a>() -> Paragraph<'a> {
    Paragraph::new("1\n2\n3").block(Block::default().borders(Borders::RIGHT))
}
