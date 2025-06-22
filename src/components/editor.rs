use ratatui::widgets::{Block, Borders, Paragraph};
// Editorコンポーネント
pub fn editor<'a>() -> Paragraph<'a> {
    Paragraph::new("ここがメインスペースです。").block(Block::default().borders(Borders::NONE))
}