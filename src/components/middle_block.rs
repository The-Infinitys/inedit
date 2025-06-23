use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};
use crate::app::App;

pub fn render_middle_block(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().title("Buffer").borders(Borders::ALL);
    let paragraph = Paragraph::new(app.buffer.as_str()).block(block);
    f.render_widget(paragraph, area);
}