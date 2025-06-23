use crate::app::App;
use crate::components::middle_box::left_line::diff::get_diff_marks;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::*,
};
use sha2::{Sha256, Digest}; // Cargo.tomlにsha2 = "0.10" などを追加

pub mod editor;
pub mod left_line;
pub mod right_line;
use editor::editor;
use left_line::left_line;
use right_line::right_line;

// 仮: diff情報はNoneで渡す
pub fn middle_box(f: &mut Frame, area: Rect, app: &mut App) {
    let diff = get_diff_cached(app);
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

fn get_diff_cached(app: &mut App) -> Option<Vec<char>> {
    let buffer_hash = {
        let mut hasher = Sha256::new();
        hasher.update(app.buffer.as_bytes());
        format!("{:x}", hasher.finalize())
    };
    if let Some((ref last_hash, ref diff)) = app.diff_cache {
        if last_hash == &buffer_hash {
            return Some(diff.clone());
        }
    }
    if let Some(ref path) = app.file_path {
        if let Some(diff) = get_diff_marks(path, &app.buffer) {
            app.diff_cache = Some((buffer_hash, diff.clone()));
            return Some(diff);
        }
    }
    None
}
