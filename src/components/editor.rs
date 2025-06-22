use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
// Editorコンポーネント
pub fn editor<'a>() -> Paragraph<'a> {
    Paragraph::new("ここがメインスペースです。").block(Block::default().borders(Borders::ALL))
}