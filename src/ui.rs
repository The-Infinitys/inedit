// src/ui.rs
use crate::{
    app::App,
    components::{
        bottom_bar::render_bottom_bar,
        // middle_block を削除
        top_bar::render_top_bar,
        message_display::render_message_display,
        middle_block::left_block::render_left_block,    // 新規
        middle_block::editor_block::render_editor_block, // 新規
        middle_block::right_block::render_right_block,   // 新規
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
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),    // Top Bar (タイトル)
                Constraint::Min(0),       // メインエディタ領域（Left + Editor + Right）
                Constraint::Length(1),    // Bottom Bar (カーソル位置)
            ]
            .as_ref(),
        )
        .split(size);

    // Top Bar の描画
    render_top_bar(f, main_chunks[0], app);

    // メインエディタ領域をさらに分割 (Left Block + Editor Block + Right Block)
    // 左ブロック (行番号と差分): 5文字固定 (例: "  999 +")
    // 右ブロック (スクロールバーと差分): 3文字固定 (例: " |#")
    // エディタ本体: 残りのスペース
    let editor_area_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(7),   // Left Block (行番号と差分シンボル、例: " 999 + ")
                Constraint::Min(0),      // Editor Block (エディタ本体)
                Constraint::Length(3),   // Right Block (スクロールバーと差分マーカー)
            ]
            .as_ref(),
        )
        .split(main_chunks[1]); // main_chunks[1] がエディタの親領域

    // Middle Block の描画前にスクロールオフセットを調整
    // エディタ本体の描画領域を adjust_viewport_offset に渡す
    app.editor.adjust_viewport_offset(editor_area_chunks[1]);

    // Left Block の描画
    render_left_block(f, editor_area_chunks[0], app);

    // Editor Block の描画
    render_editor_block(f, editor_area_chunks[1], app);

    // Right Block の描画
    render_right_block(f, editor_area_chunks[2], app);

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
