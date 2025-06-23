use crate::app::App;
use crate::components::middle_box::left_line::diff::get_diff_marks;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::*,
};

pub mod editor;
pub mod left_line;
pub mod right_line;
use editor::editor;
use left_line::left_line;
use right_line::right_line;

// 仮: diff情報はNoneで渡す
pub fn middle_box(f: &mut Frame, area: Rect, app: &App) {
    let diff = if let Some(path) = &app.file_path {
        get_diff_marks(path, &app.buffer)
    } else {
        None
    };
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(6),
        ])
        .split(area);

    let editor_height = chunks[1].height as usize;
    f.render_widget(left_line(app, diff.as_deref(), editor_height), chunks[0]);
    editor(f, chunks[1], app, editor_height);
    f.render_widget(right_line(app, editor_height, diff.as_deref()), chunks[2]);
}
