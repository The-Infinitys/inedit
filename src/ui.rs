// src/ui.rs
use crate::{
    app::App,
    components::{
        bottom_bar::render_bottom_bar,
        message_display::render_message_display,
        middle_block::editor_block::render_editor_block,
        middle_block::left_block::render_left_block,
        middle_block::right_block::render_right_block,
        popup::{PopupKind, render_popup}, // ← 修正
        top_bar::render_top_bar,
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
                Constraint::Length(1), // Top Bar (タイトル)
                Constraint::Min(0),    // メインエディタ領域（Left + Editor + Right）
                Constraint::Length(1), // Bottom Bar (カーソル位置)
            ]
            .as_ref(),
        )
        .split(size);

    // Top Bar の描画
    render_top_bar(f, main_chunks[0], app);

    // メインエディタ領域をさらに分割 (Left Block + Editor Block + Right Block)
    let editor_area_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(7), // Left Block (行番号と差分シンボル、例: " 999 + ")
                Constraint::Min(0),    // Editor Block (エディタ本体)
                Constraint::Length(3), // Right Block (スクロールバーと差分マーカー)
            ]
            .as_ref(),
        )
        .split(main_chunks[1]); // main_chunks[1] がエディタの親領域

    // Middle Block の描画前にスクロールオフセットを調整
    // ここで `editor_area_chunks[1]` (Editor Block の領域) を渡すのが適切です。
    app.editor.adjust_viewport_offset(editor_area_chunks[1], app.word_wrap_enabled);

    // Left Block の描画
    render_left_block(f, editor_area_chunks[0], app);

    // Editor Block の描画
    render_editor_block(f, editor_area_chunks[1], app);

    // Right Block の描画
    render_right_block(f, editor_area_chunks[2], app);

    // Bottom Bar の描画
    render_bottom_bar(f, main_chunks[2], app);

    // メッセージ通知エリアを計算 (画面全体の右下)
    const MAX_MESSAGE_HEIGHT: u16 = 5;
    const MESSAGE_WIDTH: u16 = 40;
    const MESSAGE_MARGIN: u16 = 1;

    let actual_message_lines = app.get_visible_message_count().min(MAX_MESSAGE_HEIGHT);

    let msg_area = if actual_message_lines > 0 {
        Rect {
            x: size
                .width
                .saturating_sub(MESSAGE_WIDTH)
                .saturating_sub(MESSAGE_MARGIN),
            y: size
                .height
                .saturating_sub(actual_message_lines)
                .saturating_sub(MESSAGE_MARGIN),
            width: MESSAGE_WIDTH,
            height: actual_message_lines,
        }
    } else {
        Rect::new(0, 0, 0, 0)
    };

    // メッセージ通知の描画
    render_message_display(f, msg_area, app);

    // 終了ポップアップが表示されている場合は描画
    if let Some(exit_popup_state) = &app.exit_popup_state {
        if exit_popup_state.input_mode {
            render_popup(
                f,
                size,
                PopupKind::Input {
                    message: "保存先ファイル名を入力してください",
                    input: &exit_popup_state.input_text,
                },
                exit_popup_state,
            );
        } else {
            render_popup(f, size, PopupKind::Exit, exit_popup_state);
        }
    }
}
