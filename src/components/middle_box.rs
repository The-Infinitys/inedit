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

// middle_boxコンポーネント
pub fn middle_box<B: Backend>(f: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(4), // 左行番号
            Constraint::Min(10),   // エディタ本体
            Constraint::Length(6), // 右情報
        ])
        .split(area);

    f.render_widget(left_line(), chunks[0]);
    f.render_widget(editor(), chunks[1]);
    f.render_widget(right_line(), chunks[2]);
}
