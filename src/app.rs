// src/app.rs
pub mod cursor;
pub mod editor;
pub mod features;
pub mod msg;
use editor::Editor;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Instant;

// msg!とemsg!マクロをインポート
use crate::{emsg, msg};

/// UIに表示されるメッセージの種類を定義します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    Info,
    Error,
}

/// 各行の差分状態を定義します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStatus {
    Unchanged,
    Modified, // 変更された行
    Added,    // 新しく追加された行
              // Deletedは現在のバッファに行が存在しないため、このVecでは直接表現しません。
              // 左ブロックや右ブロックで、original_bufferと比較して空白または特殊記号で表現することを検討します。
}

/// アプリケーション全体の状態を管理します。
pub struct App {
    pub editor: Editor,
    pub target_path: Option<PathBuf>, // 編集対象の元のファイルのパス
    pub temp_path: Option<PathBuf>,   // 編集中の内容を保存する一時ファイルのパス
    pub clipboard: Option<String>,    // アプリケーション内のクリップボードデータ
    pub messages: Vec<(MessageType, String, Instant)>, // UIに表示するメッセージのキュー (種類, 内容, タイムスタンプ)
    pub original_buffer: String, // ファイル読み込み時のオリジナルコンテンツ（差分計算用）
    pub word_wrap_enabled: bool, // 折り返し表示モードのON/OFF
    pub line_statuses: Vec<LineStatus>, // 各行の差分状態
}

impl Default for App {
    fn default() -> Self {
        Self {
            editor: Editor::new(String::new()),
            target_path: None,
            temp_path: None,
            clipboard: None,
            messages: Vec::new(),
            original_buffer: String::new(), // 初期化
            word_wrap_enabled: true,        // デフォルトで折り返し表示を有効
            line_statuses: Vec::new(),      // 初期化
        }
    }
}

/// Appがスコープを抜ける際に一時ファイルを削除するためのDrop実装
impl Drop for App {
    fn drop(&mut self) {
        if let Some(path) = &self.temp_path {
            if path.exists() {
                if let Err(e) = fs::remove_file(path) {
                    eprintln!(
                        "一時ファイル {:?} の削除中にエラーが発生しました: {}",
                        path, e
                    );
                } else {
                    eprintln!("一時ファイル {:?} を削除しました。", path);
                }
            }
        }
    }
}

impl App {
    /// アプリケーションを初期化します。コマンドライン引数からファイルパスを読み込みます。
    pub fn init() -> Self {
        let mut app = Self::default();

        let args: Vec<String> = env::args().collect();
        let file_path_str_opt = args.get(1);

        if let Some(file_path_str) = file_path_str_opt {
            let original_path = PathBuf::from(file_path_str);
            app.target_path = Some(original_path.clone());

            let temp_filename = format!(
                ".{}.inedit",
                original_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("untitled")
            );
            let temp_file = PathBuf::from(".");
            let temp_path = original_path
                .parent()
                .unwrap_or_else(|| temp_file.as_path())
                .join(temp_filename);
            app.temp_path = Some(temp_path.clone());

            msg!(app, "元のファイルパス: {:?}", original_path);
            msg!(app, "一時ファイルパス: {:?}", temp_path);

            // まず一時ファイルからの読み込みを試みる
            if temp_path.exists() {
                match app.editor.load_from_file(&temp_path) {
                    Ok(_) => {
                        msg!(
                            app,
                            "一時ファイル {:?} から正常に読み込みました。",
                            temp_path
                        );
                        app.original_buffer = app.editor.buffer.clone(); // original_bufferも設定
                        app.calculate_diff_status();
                        return app;
                    }
                    Err(e) => {
                        emsg!(
                            app,
                            "一時ファイル {:?} の読み込み中にエラーが発生しました: {}。元のファイルに戻ります。",
                            temp_path,
                            e
                        );
                    }
                }
            }

            // 一時ファイルが存在しないか読み込みに失敗した場合、元のファイルを試す
            if original_path.exists() {
                match app.editor.load_from_file(&original_path) {
                    Ok(_) => {
                        msg!(
                            app,
                            "元のファイル {:?} を正常に読み込みました。",
                            original_path
                        );
                        app.original_buffer = app.editor.buffer.clone(); // original_bufferも設定
                        // 元のファイルを読み込んだら、その内容をすぐに一時ファイルに書き込む
                        if let Err(e) = app.editor.save_to_file(&temp_path) {
                            emsg!(
                                app,
                                "警告: 初期コンテンツを一時ファイル {:?} に書き込めませんでした: {}",
                                temp_path,
                                e
                            );
                        } else {
                            msg!(
                                app,
                                "初期コンテンツを一時ファイル {:?} に書き込みました。",
                                temp_path
                            );
                        }
                    }
                    Err(e) => {
                        emsg!(
                            app,
                            "元のファイル {:?} の読み込み中にエラーが発生しました: {}。空のバッファで開始します。",
                            original_path,
                            e
                        );
                        if let Err(e) = fs::write(&temp_path, "") {
                            emsg!(
                                app,
                                "警告: 空の一時ファイル {:?} を作成できませんでした: {}",
                                temp_path,
                                e
                            );
                        }
                    }
                }
            } else {
                msg!(
                    app,
                    "元のファイルが存在しません: {:?}。新しいファイルバッファと一時ファイルを作成します。",
                    file_path_str
                );
                if let Err(e) = fs::write(&temp_path, "") {
                    emsg!(
                        app,
                        "警告: 空の一時ファイル {:?} を作成できませんでした: {}",
                        temp_path,
                        e
                    );
                }
            }
        } else {
            msg!(
                app,
                "ファイルパスが指定されていません。空のバッファ（プレーンテキストモード）で開始します。一時ファイルは作成されません。"
            );
        }
        app.calculate_diff_status(); // 初期化時に差分状態を計算
        app
    }

    /// 現在のファイルを保存します。元のファイルパスが設定されている必要があります。
    pub fn save_current_file(&mut self) -> io::Result<()> {
        if let Some(original_path) = &self.target_path {
            self.editor.save_to_file(original_path)?;
            msg!(self, "ファイルは {:?} に保存されました。", original_path);
            self.original_buffer = self.editor.buffer.clone(); // 保存後、オリジナルバッファを更新
            self.calculate_diff_status(); // 差分状態を再計算

            if let Some(temp_path) = &self.temp_path {
                if temp_path.exists() {
                    if let Err(e) = fs::remove_file(temp_path) {
                        emsg!(
                            self,
                            "警告: 一時ファイル {:?} を削除できませんでした: {}",
                            temp_path,
                            e
                        );
                    } else {
                        msg!(
                            self,
                            "一時ファイル {:?} は正常な保存後に削除されました。",
                            temp_path
                        );
                    }
                }
            }
            Ok(())
        } else {
            emsg!(
                self,
                "ファイルを保存するターゲットパスが設定されていません。「名前を付けて保存」機能を使用してください。"
            );
            Err(io::Error::other(
                "ファイルを保存するターゲットパスが設定されていません。「名前を付けて保存」機能を使用してください。",
            ))
        }
    }

    /// メッセージキューに新しいメッセージを追加します。
    pub fn add_message(&mut self, message_type: MessageType, msg: String) {
        self.messages.push((message_type, msg, Instant::now()));
    }

    /// 現在のバッファとオリジナルバッファを比較し、各行の差分状態を計算します。
    /// （簡易的な行ごとの比較で、行の挿入・削除によるズレは考慮しません。）
    pub fn calculate_diff_status(&mut self) {
        self.line_statuses.clear();
        let original_lines: Vec<&str> = self.original_buffer.lines().collect();
        let current_lines: Vec<&str> = self.editor.buffer.lines().collect();

        for (i, current_line) in current_lines.iter().enumerate() {
            if let Some(original_line) = original_lines.get(i) {
                if current_line == original_line {
                    self.line_statuses.push(LineStatus::Unchanged);
                } else {
                    self.line_statuses.push(LineStatus::Modified);
                }
            } else {
                // original_linesよりもcurrent_linesが長い場合、これは追加された行
                self.line_statuses.push(LineStatus::Added);
            }
        }
        // ここでは、original_linesに存在しcurrent_linesに存在しない行（削除された行）は
        // line_statusesの長さには影響しません。UI側でoriginal_bufferの行数と比較して、
        // 削除された行のギャップを表現することを検討できますが、今回は簡易的なアプローチです。
    }
}
