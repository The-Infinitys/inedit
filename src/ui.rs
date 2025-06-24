// src/ui.rs
use crate::{
    app::App,
    components::{
        bottom_bar::render_bottom_bar,
        middle_block::render_middle_block,
        top_bar::render_top_bar,
        message_display::render_message_display,
    },
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

/// アプリケーションのUIを描画します。
pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // メインのレイアウトチャンクを定義
    // Top Bar: 1行
    // Middle Block: 残りのスペース
    // Bottom Bar: 1行
    let main_chunks = Layout::default()
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
    render_top_bar(f, main_chunks[0], app);

    // Middle Block の描画前にスクロールオフセットを調整
    app.editor.adjust_viewport_offset(main_chunks[1]);

    // Middle Block (エディタ本体) の描画
    render_middle_block(f, main_chunks[1], app);

    // Bottom Bar の描画
    render_bottom_bar(f, main_chunks[2], app);

    // メッセージ通知エリアを計算 (画面全体の右下)
    const MESSAGE_HEIGHT: u16 = 3; // メッセージ表示の最大行数
    const MESSAGE_WIDTH: u16 = 40; // メッセージ表示の幅
    const MESSAGE_MARGIN: u16 = 1; // 画面端からのマージン

    let msg_area = Rect {
        x: size.width.saturating_sub(MESSAGE_WIDTH).saturating_sub(MESSAGE_MARGIN),
        y: size.height.saturating_sub(MESSAGE_HEIGHT).saturating_sub(MESSAGE_MARGIN),
        width: MESSAGE_WIDTH,
        height: MESSAGE_HEIGHT,
    };

    // メッセージ通知の描画
    render_message_display(f, msg_area, app);
}
