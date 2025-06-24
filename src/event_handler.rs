// src/event_handler.rs

use crate::app::App;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

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
                        return Ok(true); // Ctrl+Qで終了
                    }
                    KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(true); // Ctrl+Wで終了
                    }
                    KeyCode::Esc => {
                        return Ok(true); // Escキーで終了
                    }

                    // 編集コマンド
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+S で保存
                        if let Err(e) = app.save_current_file() {
                            eprintln!("ファイルの保存に失敗しました: {}", e);
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+C でコピー
                        if let Some(selected_text) = app.editor.copy_selection() {
                            app.clipboard = Some(selected_text);
                        }
                    }
                    KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+X で切り取り
                        if let Some(cut_text) = app.editor.cut_selection() {
                            app.clipboard = Some(cut_text);
                        }
                    }
                    KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+V でペースト
                        if let Some(text_to_paste) = &app.clipboard {
                            app.editor.paste_text(text_to_paste);
                        }
                    }
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+A で全て選択
                        app.editor.select_all();
                    }

                    // テキスト挿入
                    KeyCode::Char(c) => {
                        // Ctrlキーが押されていない通常の文字入力
                        // Modifiers::ALT、Modifiers::CONTROLは除外して通常の文字入力とみなす
                        if !key.modifiers.contains(KeyModifiers::CONTROL)
                            && !key.modifiers.contains(KeyModifiers::ALT)
                        {
                            app.editor.insert_char(c);
                        }
                    }
                    KeyCode::Backspace => {
                        // Backspaceキー
                        app.editor.delete_previous_char();
                    }
                    KeyCode::Delete => {
                        // Deleteキー
                        app.editor.delete_current_char();
                    }
                    KeyCode::Enter => {
                        // Enterキー (改行)
                        app.editor.insert_char('\n');
                    }
                    KeyCode::Tab => {
                        // Tabキー (簡易的にスペース4つを挿入)
                        app.editor.paste_text("    ");
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
