use crate::app::App;
use crate::components::{bottom_bar::bottom_bar, middle_box::middle_box, top_bar::top_bar};
use ratatui::prelude::*;

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top_bar
            Constraint::Min(1),    // editor
            Constraint::Length(1), // bottom_bar
        ])
        .split(size);

    f.render_widget(top_bar(app, size.width), chunks[0]);
    middle_box(f, chunks[1], app); // ← 修正
    f.render_widget(bottom_bar(), chunks[2]);

    let line = app.buffer.lines().nth(app.cursor.0).unwrap_or("");
    app.cursor.1 = app.cursor.1.min(line.chars().count());
}
