// src/ui.rs
use crate::{
    app::App,
    components::{
        bottom_bar::render_bottom_bar,
        middle_block::render_middle_block,
        top_bar::render_top_bar,
        message_display::render_message_display, // message_displayをインポート
    },
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

/// アプリケーションのUIを描画します。
pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // メインのレイアウトチャンクを定義
    // Top Bar: 1行
    // Middle Block: 残りのスペース
    // Message Display: 3行 (最大メッセージ数に対応)
    // Bottom Bar: 1行
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),    // Top Bar (タイトル)
                Constraint::Min(0),       // Middle Block (エディタ本体)
                Constraint::Length(3),    // Message Display (メッセージ、最大3行)
                Constraint::Length(1),    // Bottom Bar (カーソル位置)
            ]
            .as_ref(),
        )
        .split(size);

    // Top Bar の描画
    render_top_bar(f, chunks[0], app);

    // Middle Block の描画前にスクロールオフセットを調整
    app.editor.adjust_viewport_offset(chunks[1]);

    // Middle Block (エディタ本体) の描画
    render_middle_block(f, chunks[1], app);

    // Message Display の描画
    render_message_display(f, chunks[2], app);

    // Bottom Bar の描画
    render_bottom_bar(f, chunks[3], app);
}
