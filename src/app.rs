// src/app.rs
pub mod cursor;
pub mod editor;
pub mod features;
use editor::Editor;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf; // ファイル操作と一時ファイル削除のためにfsモジュールを追加

/// アプリケーション全体の状態を管理します。
pub struct App {
    pub editor: Editor,
    pub target_path: Option<PathBuf>, // 編集対象の元のファイルのパス
    pub temp_path: Option<PathBuf>,   // 編集中の内容を保存する一時ファイルのパス
    pub clipboard: Option<String>,    // アプリケーション内のクリップボードデータ
}

impl Default for App {
    fn default() -> Self {
        // デフォルトはプレーンテキストモードのエディタ
        Self {
            editor: Editor::new(String::new()),
            target_path: None,
            temp_path: None, // temp_pathも初期化
            clipboard: None, // クリップボードも初期化
        }
    }
}

/// Appがスコープを抜ける際に一時ファイルを削除するためのDrop実装
impl Drop for App {
    fn drop(&mut self) {
        if let Some(path) = &self.temp_path {
            // 一時ファイルが存在する場合に削除を試みる
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

            // 一時ファイルパスを構築します。例: .<元のファイル名>.inedit
            // 元のファイルのディレクトリに配置します。
            let temp_filename = format!(
                ".{}.inedit",
                original_path
                    .file_name() // ファイル名部分を取得
                    .and_then(|s| s.to_str()) // Stringに変換
                    .unwrap_or("untitled") // 変換できない場合は"untitled"をデフォルトに
            );
            let temp_path = PathBuf::from(".");
            let temp_path = original_path
                .parent() // 親ディレクトリを取得
                .unwrap_or_else(|| temp_path.as_path()) // 親ディレクトリがなければ現在のディレクトリ
                .join(temp_filename); // 一時ファイル名を結合
            app.temp_path = Some(temp_path.clone());

            eprintln!("元のファイルパス: {:?}", original_path);
            eprintln!("一時ファイルパス: {:?}", temp_path);

            // まず一時ファイルからの読み込みを試みます（クラッシュからの回復のため）
            if temp_path.exists() {
                match app.editor.load_from_file(&temp_path) {
                    Ok(_) => {
                        eprintln!("一時ファイル {:?} から正常に読み込みました。", temp_path);
                        // 一時ファイルから読み込めた場合は、これで初期化完了
                        return app;
                    }
                    Err(e) => {
                        eprintln!(
                            "一時ファイル {:?} の読み込み中にエラーが発生しました: {}。元のファイルに戻ります。",
                            temp_path, e
                        );
                        // エラーが発生した場合、元のファイルを試すために処理を続行
                    }
                }
            }

            // 一時ファイルが存在しないか読み込みに失敗した場合、元のファイルを試します
            if original_path.exists() {
                match app.editor.load_from_file(&original_path) {
                    Ok(_) => {
                        eprintln!("元のファイル {:?} を正常に読み込みました。", original_path);
                        // 元のファイルを読み込んだら、その内容をすぐに一時ファイルに書き込みます
                        if let Err(e) = app.editor.save_to_file(&temp_path) {
                            eprintln!(
                                "警告: 初期コンテンツを一時ファイル {:?} に書き込めませんでした: {}",
                                temp_path, e
                            );
                        } else {
                            eprintln!(
                                "初期コンテンツを一時ファイル {:?} に書き込みました。",
                                temp_path
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "元のファイル {:?} の読み込み中にエラーが発生しました: {}。空のバッファで開始します。",
                            original_path, e
                        );
                        // エラー時は空のバッファで続行し、空の一時ファイルを作成
                        if let Err(e) = fs::write(&temp_path, "") {
                            eprintln!(
                                "警告: 空の一時ファイル {:?} を作成できませんでした: {}",
                                temp_path, e
                            );
                        }
                    }
                }
            } else {
                eprintln!(
                    "元のファイルが存在しません: {:?}。新しいファイルバッファと一時ファイルを作成します。",
                    file_path_str
                );
                // 存在しないパスでも、そのパスを新しいファイルとして扱い、空の一時ファイルを作成
                if let Err(e) = fs::write(&temp_path, "") {
                    eprintln!(
                        "警告: 空の一時ファイル {:?} を作成できませんでした: {}",
                        temp_path, e
                    );
                }
            }
        } else {
            eprintln!(
                "ファイルパスが指定されていません。空のバッファ（プレーンテキストモード）で開始します。一時ファイルは作成されません。"
            );
            // 引数がない場合はデフォルトでPlainTextモード
        }
        app
    }

    /// 現在のファイルを保存します。元のファイルパスが設定されている必要があります。
    pub fn save_current_file(&self) -> io::Result<()> {
        if let Some(original_path) = &self.target_path {
            // エディタのバッファ内容を元のファイルに保存
            self.editor.save_to_file(original_path)?;
            eprintln!("ファイルは {:?} に保存されました。", original_path);

            // 正常に保存された後、一時ファイルを削除
            if let Some(temp_path) = &self.temp_path {
                if temp_path.exists() {
                    if let Err(e) = fs::remove_file(temp_path) {
                        eprintln!(
                            "警告: 一時ファイル {:?} を削除できませんでした: {}",
                            temp_path, e
                        );
                    } else {
                        eprintln!(
                            "一時ファイル {:?} は正常な保存後に削除されました。",
                            temp_path
                        );
                    }
                }
            }
            Ok(())
        } else {
            // ターゲットパスが設定されていない場合（新規ファイルなど）
            Err(io::Error::other(
                "ファイルを保存するターゲットパスが設定されていません。「名前を付けて保存」機能を使用してください。",
            ))
        }
    }
}
