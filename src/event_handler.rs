// src/event_handler.rs

use crate::app::{App, AppControlFlow, ExitPopupResult};
use crate::{emsg, msg};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

/// イベントを処理し、アプリケーションの状態を更新します。
/// 戻り値はアプリケーションの次の制御フローを示します。
pub fn handle_event(app: &mut App) -> std::io::Result<AppControlFlow> {
    // 100ミリ秒間イベントをポーリング
    if event::poll(std::time::Duration::from_millis(100))? {
        // キーイベントのみを処理
        if let Event::Key(key) = event::read()? {
            // キーの押下イベントのみを処理（繰り返しやリリースは無視）
            if key.kind == KeyEventKind::Press {
                let extend_selection = key.modifiers.contains(KeyModifiers::SHIFT);

                // 終了ポップアップが表示されている場合は、ポップアップのキーイベントを優先的に処理
                if app.exit_popup_state.is_some() {
                    let popup_result = app.handle_exit_popup_key(&key);
                    match popup_result {
                        ExitPopupResult::SaveAndExit => {
                            return Ok(AppControlFlow::TriggerSaveAndExit);
                        }
                        ExitPopupResult::DiscardAndExit => {
                            return Ok(AppControlFlow::TriggerDiscardAndExit);
                        }
                        ExitPopupResult::Cancel => return Ok(AppControlFlow::Continue),
                        ExitPopupResult::None => return Ok(AppControlFlow::ShowExitPopup),
                    }
                }

                // ポップアップが表示されていない場合の通常のキーイベント処理
                let bindings = &app.config.key_bindings;

                // 終了コマンドのチェック
                if bindings.exit_1.matches(&key)
                    || bindings.exit_2.matches(&key)
                    || bindings.exit_3.matches(&key)
                {
                    app.trigger_exit_popup_if_needed();
                    if app.exit_popup_state.is_some() {
                        return Ok(AppControlFlow::ShowExitPopup);
                    } else {
                        msg!(app, "アプリケーションを終了します。");
                        return Ok(AppControlFlow::Exit);
                    }
                }
                // ファイル保存
                else if bindings.save_file.matches(&key) {
                    match app.save_current_file() {
                        Ok(_) => msg!(app, "ファイルが正常に保存されました。"),
                        Err(e) => emsg!(app, "ファイルの保存に失敗しました: {}", e),
                    }
                }
                // コピー
                else if bindings.copy.matches(&key) {
                    if app.editor.copy_selection().is_some() {
                        msg!(app, "選択範囲をクリップボードにコピーしました。");
                    } else {
                        msg!(app, "コピーする選択範囲がありません。");
                    }
                }
                // カット
                else if bindings.cut.matches(&key) {
                    if let Some(cut_text) = app.editor.cut_selection() {
                        app.clipboard = Some(cut_text);
                        msg!(app, "選択範囲をクリップボードに切り取りました。");
                    } else {
                        msg!(app, "切り取る選択範囲がありません。");
                    }
                    app.calculate_diff_status();
                }
                // ペースト
                else if bindings.paste.matches(&key) {
                    if let Some(text_to_paste) = app.clipboard.clone() {
                        app.editor.paste_text(&text_to_paste);
                        app.calculate_diff_status();
                        msg!(app, "クリップボードの内容をペーストしました。");
                    } else {
                        msg!(app, "クリップボードが空です。");
                    }
                }
                // 全選択
                else if bindings.select_all.matches(&key) {
                    app.editor.select_all();
                    msg!(app, "全選択しました。");
                }
                // 折り返し表示トグル
                else if bindings.toggle_word_wrap.matches(&key) {
                    app.word_wrap_enabled = !app.word_wrap_enabled;
                    app.calculate_diff_status();
                    if app.word_wrap_enabled {
                        msg!(app, "折り返し表示モード: ON");
                    } else {
                        msg!(app, "折り返し表示モード: OFF");
                    }
                }
                // 改行
                else if bindings.insert_newline.matches(&key) {
                    app.editor.insert_char('\n');
                    app.calculate_diff_status();
                }
                // タブ
                else if bindings.insert_tab.matches(&key) {
                    app.editor.paste_text("    ");
                    app.calculate_diff_status();
                }
                // 前の文字を削除
                else if bindings.delete_previous_char.matches(&key) {
                    app.editor.delete_previous_char();
                    app.calculate_diff_status();
                }
                // 現在の文字を削除
                else if bindings.delete_current_char.matches(&key) {
                    app.editor.delete_current_char();
                    app.calculate_diff_status();
                }
                // カーソル移動 (Shiftキーの状態を考慮)
                else if bindings.move_left.matches(&key) {
                    app.editor.previous_char(extend_selection);
                } else if bindings.move_right.matches(&key) {
                    app.editor.next_char(extend_selection);
                } else if bindings.move_up.matches(&key) {
                    app.editor.previous_line(extend_selection);
                } else if bindings.move_down.matches(&key) {
                    app.editor.next_line(extend_selection);
                } else if bindings.move_line_start.matches(&key) {
                    app.editor.move_cursor_to_line_start(extend_selection);
                } else if bindings.move_line_end.matches(&key) {
                    app.editor.move_cursor_to_line_end(extend_selection);
                } else if bindings.move_document_start.matches(&key) {
                    app.editor.move_cursor_to_document_start(extend_selection);
                } else if bindings.move_document_end.matches(&key) {
                    app.editor.move_cursor_to_document_end(extend_selection);
                }
                // 通常の文字入力
                else if let KeyCode::Char(c) = key.code {
                    if !key.modifiers.contains(KeyModifiers::CONTROL)
                        && !key.modifiers.contains(KeyModifiers::ALT)
                    {
                        app.editor.insert_char(c);
                        app.calculate_diff_status();
                    }
                }
            }
        }
    }
    Ok(AppControlFlow::Continue)
}
