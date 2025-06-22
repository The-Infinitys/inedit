use ratatui::prelude::*;
use crate::components::{bottom_bar::bottom_bar, editor::editor, top_bar::top_bar};
use crate::app::App;

pub fn ui<B: Backend>(f: &mut Frame<>, _app: &App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top_bar
            Constraint::Min(1),    // editor
            Constraint::Length(1), // bottom_bar
        ])
        .split(size);

    f.render_widget(top_bar(), chunks[0]);
    f.render_widget(editor(), chunks[1]);
    f.render_widget(bottom_bar(), chunks[2]);
}