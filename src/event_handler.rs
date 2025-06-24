// src/event_handler.rs

use crate::app::App;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crate::{msg, emsg};

/// イベントを処理し、アプリケーションの状態を更新します。
/// 終了が要求された場合はtrueを返します。
pub fn handle_event(app: &mut App) -> std::io::Result<bool> {
    // 100ミリ秒間イベントをポーリング
    if event::poll(std::time::Duration::from_millis(100))? {
        // キーイベントのみを処理
        if let Event::Key(key) = event::read()? {
            // キーの押下イベントのみを処理（繰り返しやリリースは無視）
            if key.kind == KeyEventKind::Press {
                let extend_selection = key.modifiers.contains(KeyModifiers::SHIFT);

                match key.code {
                    // アプリケーション終了コマンド
                    KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        msg!(app, "アプリケーションを終了します。");
                        return Ok(true); // Ctrl+Qで終了
                    }
                    KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        msg!(app, "アプリケーションを終了します。");
                        return Ok(true); // Ctrl+Wで終了
                    }
                    KeyCode::Esc => {
                        msg!(app, "アプリケーションを終了します。");
                        return Ok(true); // Escキーで終了
                    }

                    // 編集コマンド
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+S で保存
                        match app.save_current_file() {
                            Ok(_) => msg!(app, "ファイルが正常に保存されました。"),
                            Err(e) => emsg!(app, "ファイルの保存に失敗しました: {}", e),
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+C でコピー
                        if app.editor.copy_selection().is_some() {
                             msg!(app, "選択範囲をクリップボードにコピーしました。");
                        } else {
                            msg!(app, "コピーする選択範囲がありません。");
                        }
                    }
                    KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+X で切り取り
                        if let Some(cut_text) = app.editor.cut_selection() {
                            app.clipboard = Some(cut_text);
                             msg!(app, "選択範囲をクリップボードに切り取りました。");
                        } else {
                            msg!(app, "切り取る選択範囲がありません。");
                        }
                    }
                    KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+V でペースト
                        if let Some(text_to_paste) = &app.clipboard {
                            app.editor.paste_text(text_to_paste);
                            // ペースト後、バッファ内容が変わるので差分を再計算
                            app.calculate_diff_status();
                            msg!(app, "クリップボードの内容をペーストしました。");
                        } else {
                            msg!(app, "クリップボードが空です。");
                        }
                    }
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+A で全て選択
                        app.editor.select_all();
                        msg!(app, "全選択しました。");
                    }
                    KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::ALT) => {
                        // Alt+Z で折り返し表示モードをトグル
                        app.word_wrap_enabled = !app.word_wrap_enabled;
                        app.calculate_diff_status(); // 折り返しモード変更でもDiff再計算（状態が変化したため）
                        if app.word_wrap_enabled {
                            msg!(app, "折り返し表示モード: ON");
                        } else {
                            msg!(app, "折り返し表示モード: OFF");
                        }
                    }

                    // テキスト挿入
                    KeyCode::Char(c) => {
                        // Ctrlキーが押されていない通常の文字入力
                        if !key.modifiers.contains(KeyModifiers::CONTROL)
                            && !key.modifiers.contains(KeyModifiers::ALT)
                        {
                            app.editor.insert_char(c);
                            app.calculate_diff_status(); // 文字入力後、バッファ内容が変わるので差分を再計算
                        }
                    }
                    KeyCode::Backspace => {
                        // Backspaceキー
                        app.editor.delete_previous_char();
                        app.calculate_diff_status(); // 削除後、バッファ内容が変わるので差分を再計算
                    }
                    KeyCode::Delete => {
                        // Deleteキー
                        app.editor.delete_current_char();
                        app.calculate_diff_status(); // 削除後、バッファ内容が変わるので差分を再計算
                    }
                    KeyCode::Enter => {
                        // Enterキー (改行)
                        app.editor.insert_char('\n');
                        app.calculate_diff_status(); // 改行後、バッファ内容が変わるので差分を再計算
                    }
                    KeyCode::Tab => {
                        // Tabキー (簡易的にスペース4つを挿入)
                        app.editor.paste_text("    "); // paste_textは内部でcalculate_diff_statusを呼び出す
                    }

                    // カーソル移動
                    KeyCode::Left => {
                        app.editor.previous_char(extend_selection);
                    }
                    KeyCode::Right => {
                        app.editor.next_char(extend_selection);
                    }
                    KeyCode::Up => {
                        app.editor.previous_line(extend_selection);
                    }
                    KeyCode::Down => {
                        app.editor.next_line(extend_selection);
                    }
                    KeyCode::Home => {
                        app.editor.move_cursor_to_line_start(extend_selection);
                    }
                    KeyCode::End => {
                        app.editor.move_cursor_to_line_end(extend_selection);
                    }

                    // その他のキーは現時点では無視
                    _ => {}
                }
            }
        }
    }
    Ok(false) // 終了が要求されていない場合はfalseを返す
}
