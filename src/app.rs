// src/app.rs
pub mod cursor;
pub mod editor;
pub mod features;
pub mod msg;
use crate::{emsg, msg};
use editor::Editor;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
/// UIに表示されるメッセージの種類を定義します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    Info,
    Error,
}

/// アプリケーション全体の状態を管理します。
pub struct App {
    pub editor: Editor,
    pub target_path: Option<PathBuf>, // 編集対象の元のファイルのパス
    pub temp_path: Option<PathBuf>,   // 編集中の内容を保存する一時ファイルのパス
    pub clipboard: Option<String>,    // アプリケーション内のクリップボードデータ
    pub messages: Vec<(MessageType, String)>, // UIに表示するメッセージのキュー (種類と内容)
}

impl Default for App {
    fn default() -> Self {
        Self {
            editor: Editor::new(String::new()),
            target_path: None,
            temp_path: None,
            clipboard: None,
            messages: Vec::new(), // メッセージキューを初期化
        }
    }
}

/// Appがスコープを抜ける際に一時ファイルを削除するためのDrop実装
impl Drop for App {
    fn drop(&mut self) {
        if let Some(path) = &self.temp_path {
            if path.exists() {
                if let Err(e) = fs::remove_file(path) {
                    // msg!マクロはここでは使えないため、eprintln!を使用
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

        // msg!/emsg!マクロをApp::init()で使うために、ここでAppをmutで渡し、スコープ内でのみ使う
        // あるいは、init()後にメッセージを追加するようにする
        // 今回はinit()内でも使えるように、マクロの定義をcomponents/msg.rsからapp.rsの前に移動するか、
        // main.rsでマクロをインポートしてapp.rsの前に置く、などの工夫が必要だが、
        // シンプルにするため、init()で発生するメッセージはまだeprintln!を使う。
        // event_handler.rsでの利用を優先する。
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
            let temp_path = PathBuf::from(".");
            let temp_path = original_path
                .parent()
                .unwrap_or_else(|| temp_path.as_path())
                .join(temp_filename);
            app.temp_path = Some(temp_path.clone());

            eprintln!("元のファイルパス: {:?}", original_path); // 後でmsg!/emsg!に置き換え
            eprintln!("一時ファイルパス: {:?}", temp_path); // 後でmsg!/emsg!に置き換え

            if temp_path.exists() {
                match app.editor.load_from_file(&temp_path) {
                    Ok(_) => {
                        eprintln!("一時ファイル {:?} から正常に読み込みました。", temp_path); // 後でmsg!
                        return app;
                    }
                    Err(e) => {
                        eprintln!(
                            "一時ファイル {:?} の読み込み中にエラーが発生しました: {}。元のファイルに戻ります。",
                            temp_path, e
                        ); // 後でemsg!
                    }
                }
            }

            if original_path.exists() {
                match app.editor.load_from_file(&original_path) {
                    Ok(_) => {
                        eprintln!("元のファイル {:?} を正常に読み込みました。", original_path); // 後でmsg!
                        if let Err(e) = app.editor.save_to_file(&temp_path) {
                            eprintln!(
                                "警告: 初期コンテンツを一時ファイル {:?} に書き込めませんでした: {}",
                                temp_path, e
                            ); // 後でemsg!
                        } else {
                            eprintln!(
                                "初期コンテンツを一時ファイル {:?} に書き込みました。",
                                temp_path
                            ); // 後でmsg!
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "元のファイル {:?} の読み込み中にエラーが発生しました: {}。空のバッファで開始します。",
                            original_path, e
                        ); // 後でemsg!
                        if let Err(e) = fs::write(&temp_path, "") {
                            eprintln!(
                                "警告: 空の一時ファイル {:?} を作成できませんでした: {}",
                                temp_path, e
                            ); // 後でemsg!
                        }
                    }
                }
            } else {
                eprintln!(
                    "元のファイルが存在しません: {:?}。新しいファイルバッファと一時ファイルを作成します。",
                    file_path_str
                ); // 後でmsg!
                if let Err(e) = fs::write(&temp_path, "") {
                    eprintln!(
                        "警告: 空の一時ファイル {:?} を作成できませんでした: {}",
                        temp_path, e
                    ); // 後でemsg!
                }
            }
        } else {
            eprintln!(
                "ファイルパスが指定されていません。空のバッファ（プレーンテキストモード）で開始します。一時ファイルは作成されません。"
            ); // 後でmsg!
        }
        app
    }

    /// 現在のファイルを保存します。元のファイルパスが設定されている必要があります。
    pub fn save_current_file(&mut self) -> io::Result<()> {
        // self を mutable に変更
        if let Some(original_path) = &self.target_path {
            self.editor.save_to_file(original_path)?;
            // eprintln!をmsg!に置き換え
            msg!(self, "ファイルは {:?} に保存されました。", original_path);

            if let Some(temp_path) = &self.temp_path {
                if temp_path.exists() {
                    if let Err(e) = fs::remove_file(temp_path) {
                        // eprintln!をemsg!に置き換え
                        emsg!(
                            self,
                            "警告: 一時ファイル {:?} を削除できませんでした: {}",
                            temp_path,
                            e
                        );
                    } else {
                        // eprintln!をmsg!に置き換え
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
            // eprintln!をemsg!に置き換え
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
    /// 指定された行数を超えた場合は古いメッセージを削除します。
    pub fn add_message(&mut self, message_type: MessageType, msg: String) {
        const MAX_MESSAGES: usize = 3; // 表示するメッセージの最大行数
        self.messages.push((message_type, msg));
        if self.messages.len() > MAX_MESSAGES {
            self.messages.remove(0); // 最も古いメッセージを削除
        }
    }
}
