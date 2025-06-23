
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

// エディタ本体
pub fn editor<'a>() -> Paragraph<'a> {
    Paragraph::new("ここがエディタです。").block(Block::default().borders(Borders::NONE))
}