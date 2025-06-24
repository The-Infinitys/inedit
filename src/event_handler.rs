// src/event_handler.rs

use crate::app::{App, AppControlFlow, ExitPopupResult};
use crate::{emsg, msg}; // ← 修正: appモジュール経由でemsg, msgをインポート
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// イベントを処理し、アプリケーションの状態を更新します。
/// 戻り値はアプリケーションの次の制御フローを示します。
pub fn handle_event(app: &mut App, key: &KeyEvent) -> std::io::Result<AppControlFlow> {
    // キーの押下イベントのみを処理（繰り返しやリリースは無視）
    if key.kind == KeyEventKind::Press {
        let extend_selection = key.modifiers.contains(KeyModifiers::SHIFT);

        // 終了ポップアップが表示されている場合は、ポップアップのキーイベントを優先的に処理
        if let Some(popup_state) = app.exit_popup_state.as_mut() {
            // --- 入力モード時の処理 ---
            if popup_state.input_mode {
                match key.code {
                    KeyCode::Enter => {
                        if !popup_state.input_text.is_empty() {
                            app.target_path =
                                Some(std::path::PathBuf::from(&popup_state.input_text));
                            popup_state.input_mode = false;
                            popup_state.input_text.clear(); // Clear input after use
                            app.exit_popup_state = None; // Hide popup
                            return Ok(AppControlFlow::TriggerSaveAndExit);
                        } else {
                            // If Enter is pressed with empty input, just return to popup main state
                            popup_state.input_mode = false;
                            msg!(app, "ファイルパスが空です。");
                            return Ok(AppControlFlow::ShowExitPopup);
                        }
                    }
                    KeyCode::Esc => {
                        popup_state.input_mode = false;
                        popup_state.input_text.clear();
                        return Ok(AppControlFlow::ShowExitPopup);
                    }
                    KeyCode::Backspace => {
                        if popup_state.input_text.is_empty() {
                            popup_state.input_mode = false;
                            return Ok(AppControlFlow::ShowExitPopup);
                        } else {
                            popup_state.input_text.pop();
                        }
                    }
                    KeyCode::Char(c) => {
                        popup_state.input_text.push(c);
                    }
                    _ => {}
                }
                return Ok(AppControlFlow::ShowExitPopup); // 入力モード中はポップアップを表示し続ける
            }

            // --- 通常のポップアップ操作 ---
            // ここでpopup_stateの可変参照はもう使わないので、再度appを可変参照できる
            let popup_result = app.handle_exit_popup_key(key); // keyを渡す
            match popup_result {
                ExitPopupResult::SaveAndExit => {
                    if app.target_path.is_none() {
                        // 新規ファイルで保存する場合、パス入力モードへ
                        if let Some(popup_state) = app.exit_popup_state.as_mut() {
                            popup_state.input_mode = true;
                            popup_state.input_text.clear(); // 新規保存なのでテキストをクリア
                            return Ok(AppControlFlow::ShowExitPopup);
                        }
                    }
                    // target_pathがある場合はそのまま保存して終了
                    return Ok(AppControlFlow::TriggerSaveAndExit);
                }
                ExitPopupResult::DiscardAndExit => {
                    return Ok(AppControlFlow::TriggerDiscardAndExit);
                }
                ExitPopupResult::Cancel => return Ok(AppControlFlow::Continue), // ポップアップが閉じて続行
                ExitPopupResult::None => return Ok(AppControlFlow::ShowExitPopup), // ポップアップ内で選択中
            }
        }

        // ポップアップが表示されていない場合の通常のキーイベント処理
        let bindings = &app.config.key_bindings;

        // 終了コマンドのチェック
        if bindings.exit_1.matches(key)
            || bindings.exit_2.matches(key)
            || bindings.exit_3.matches(key)
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
        else if bindings.save_file.matches(key) {
            if app.target_path.is_none() {
                // 新規ファイルの場合、保存パス入力モードへ
                if app.exit_popup_state.is_none() {
                    app.exit_popup_state = Some(Default::default()); // ポップアップを初期化
                }
                if let Some(popup_state) = app.exit_popup_state.as_mut() {
                    popup_state.selected_option = crate::app::ExitPopupOption::SaveAndExit; // デフォルトでSaveを選択
                    popup_state.input_mode = true; // 入力モードを有効化
                    popup_state.input_text.clear(); // 入力フィールドをクリア
                }
                return Ok(AppControlFlow::ShowExitPopup); // ポップアップを表示して入力待ち
            }
            match app.save_current_file() {
                Ok(_) => msg!(app, "ファイルが正常に保存されました。"),
                Err(e) => emsg!(app, "ファイルの保存に失敗しました: {}", e),
            }
        }
        // コピー
        else if bindings.copy.matches(key) {
            if app.editor.copy_selection().is_some() {
                msg!(app, "選択範囲をクリップボードにコピーしました。");
            } else {
                msg!(app, "コピーする選択範囲がありません。");
            }
        }
        // カット
        else if bindings.cut.matches(key) {
            if let Some(cut_text) = app.editor.cut_selection() {
                app.clipboard = Some(cut_text);
                msg!(app, "選択範囲をクリップボードに切り取りました。");
            } else {
                msg!(app, "切り取る選択範囲がありません。");
            }
            app.calculate_diff_status();
        }
        // ペースト
        else if bindings.paste.matches(key) {
            if let Some(text_to_paste) = app.clipboard.clone() {
                // .clone() で所有権を移動させずに参照を使う
                app.editor.paste_text(&text_to_paste);
                app.calculate_diff_status();
                msg!(app, "クリップボードの内容をペーストしました。");
            } else {
                msg!(app, "クリップボードが空です。");
            }
        }
        // 全選択
        else if bindings.select_all.matches(key) {
            app.editor.select_all();
            msg!(app, "全選択しました。");
        }
        // 折り返し表示トグル
        else if bindings.toggle_word_wrap.matches(key) {
            app.word_wrap_enabled = !app.word_wrap_enabled;
            app.calculate_diff_status();
            if app.word_wrap_enabled {
                msg!(app, "折り返し表示モード: ON");
            } else {
                msg!(app, "折り返し表示モード: OFF");
            }
        }
        // 改行
        else if bindings.insert_newline.matches(key) {
            app.editor.insert_char('\n');
            app.calculate_diff_status();
        }
        // タブ
        else if bindings.insert_tab.matches(key) {
            app.editor.paste_text("    "); // paste_textは内部でcalculate_diff_statusを呼び出す
            app.calculate_diff_status(); // Tabでpaste_textを呼んだ場合も明示的にdiff再計算
        }
        // 前の文字を削除
        else if bindings.delete_previous_char.matches(key) {
            app.editor.delete_previous_char();
            app.calculate_diff_status();
        }
        // 現在の文字を削除
        else if bindings.delete_current_char.matches(key) {
            app.editor.delete_current_char();
            app.calculate_diff_status();
        }
        // カーソル移動 (Shiftキーの状態を考慮)
        else if bindings.move_left.matches(key) {
            app.editor.previous_char(extend_selection);
        } else if bindings.move_right.matches(key) {
            app.editor.next_char(extend_selection);
        } else if bindings.move_up.matches(key) {
            app.editor.previous_line(extend_selection);
        } else if bindings.move_down.matches(key) {
            app.editor.next_line(extend_selection);
        } else if bindings.move_line_start.matches(key) {
            app.editor.move_cursor_to_line_start(extend_selection);
        } else if bindings.move_line_end.matches(key) {
            app.editor.move_cursor_to_line_end(extend_selection);
        } else if bindings.move_document_start.matches(key) {
            app.editor.move_cursor_to_document_start(extend_selection);
        } else if bindings.move_document_end.matches(key) {
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
    Ok(AppControlFlow::Continue)
}
