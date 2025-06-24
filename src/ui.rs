// src/ui.rs
use crate::{
    app::App,
    components::{
        bottom_bar::render_bottom_bar, // bottom_barのインポートを追加
        middle_block::render_middle_block,
        top_bar::render_top_bar,     // top_barのインポートを追加
    },
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

/// アプリケーションのUIを描画します。
pub fn draw_ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // メインのレイアウトチャンクを定義
    // Top Bar: 1行
    // Middle Block: 残りのスペース
    // Bottom Bar: 1行
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),    // Top Bar (タイトル)
                Constraint::Min(0),       // Middle Block (エディタ本体)
                Constraint::Length(1),    // Bottom Bar (カーソル位置)
            ]
            .as_ref(),
        )
        .split(size);

    // Top Bar の描画
    render_top_bar(f, chunks[0], app);

    // Middle Block (エディタ本体) の描画
    render_middle_block(f, chunks[1], app);

    // Bottom Bar の描画
    render_bottom_bar(f, chunks[2], app);
}
