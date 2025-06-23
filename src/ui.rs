use crate::{app::App, components::middle_block::render_middle_block};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

pub fn draw_ui(f: &mut Frame, app: &App) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(size);
    render_middle_block(f, chunks[0], app);
}
