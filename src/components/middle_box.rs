use ratatui::widgets::{Block, Borders, Paragraph};

pub mod editor;
// Editorコンポーネント
pub fn middle_box<'a>() -> Paragraph<'a> {
    Paragraph::new("ここがメインスペースです。").block(Block::default().borders(Borders::NONE))
}
