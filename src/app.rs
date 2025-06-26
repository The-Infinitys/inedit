// src/app.rs
pub mod cursor;
pub mod editor;
pub mod features;
pub mod msg;
pub use crate::app::features::syntax::Highlighter;
pub use crate::components::popup::{ExitPopupOption, ExitPopupResult, ExitPopupState};
pub use crate::config::{Config, load_or_create_config, save_config};
use crate::{emsg, msg};
use editor::Editor;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant}; // Config関連をインポート

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
    Modified,
    Added,
}

/// アプリケーションのイベント処理結果を定義します。
#[derive(Debug, PartialEq, Eq)]
pub enum AppControlFlow {
    Continue,              // アプリケーションを通常通り続行
    Exit,                  // アプリケーションを終了
    TriggerSaveAndExit,    // 保存操作を行い、その後終了
    TriggerDiscardAndExit, // 変更を破棄し、その後終了
    ShowExitPopup,         // 終了ポップアップを表示し、ユーザーの入力を待つ
}

pub enum InputOverlay {
    None,
    Search {
        query: String,
        cursor: usize,
    },
    Replace {
        query: String,
        replace: String,
        cursor: usize,
        focus_replace: bool,
    },
    Suggest {
        prefix: String,
        suggestions: Vec<String>,
        selected: usize,
    },
}

/// アプリケーション全体の状態を管理します。
pub struct App {
    pub editor: Editor,
    pub target_path: Option<PathBuf>,
    pub temp_path: Option<PathBuf>,
    pub clipboard: Option<String>,
    pub messages: Vec<(MessageType, String, Instant)>,
    pub original_buffer: String,
    pub word_wrap_enabled: bool,
    pub line_statuses: Vec<LineStatus>,
    pub exit_popup_state: Option<ExitPopupState>,
    pub highlighter: Highlighter,
    pub current_syntax_name: String,
    pub config: Config, // 追加: アプリケーションの設定
    pub input_overlay: InputOverlay,
}

impl Default for App {
    fn default() -> Self {
        // configを先にロードしておく
        let config = load_or_create_config();
        let mut highlighter = Highlighter::new();
        // コンフィグからテーマを初期設定
        highlighter.set_theme(&config.color_theme);

        Self {
            editor: Editor::new(String::from("\n")),
            target_path: None,
            temp_path: None,
            clipboard: None,
            messages: Vec::new(),
            original_buffer: String::new(),
            word_wrap_enabled: false,
            line_statuses: Vec::new(),
            exit_popup_state: None,
            highlighter, // 初期化済みHighlighterを使用
            current_syntax_name: "Plain Text".to_string(),
            config, // 初期化済みconfigを使用
            input_overlay: InputOverlay::None,
        }
    }
}

/// Appがスコープを抜ける際に一時ファイルを削除するためのDrop実装
impl Drop for App {
    fn drop(&mut self) {
        if let Some(path) = &self.temp_path {
            if path.exists() {
                if let Err(e) = fs::remove_file(path) {
                    emsg!(
                        self,
                        "一時ファイル {:?} の削除中にエラーが発生しました: {}",
                        path,
                        e
                    );
                } else {
                    msg!(self, "一時ファイル {:?} を削除しました。", path);
                }
            }
        }
        // アプリケーション終了時に設定を保存
        if let Err(e) = save_config(&self.config) {
            emsg!(self, "Error saving config on exit: {}", e);
        }
    }
}

impl App {
    /// アプリケーションを初期化します。コマンドライン引数からファイルパスを読み込みます。
    pub fn init() -> Self {
        let mut app = Self::default(); // default()でconfigとhighlighterが初期化される

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
                .unwrap_or(temp_file.as_path())
                .join(temp_filename);
            app.temp_path = Some(temp_path.clone());

            msg!(app, "元のファイルパス: {:?}", original_path);
            msg!(app, "一時ファイルパス: {:?}", temp_path);
            msg!(
                app,
                "ファイルの行数: {}",
                app.editor.buffer.lines().collect::<Vec<&str>>().len()
            );

            // まず一時ファイルからの読み込みを試みる
            if temp_path.exists() {
                match app.editor.load_from_file(&temp_path) {
                    Ok(_) => {
                        msg!(
                            app,
                            "一時ファイル {:?} から正常に読み込みました。",
                            temp_path
                        );
                        app.original_buffer = app.editor.buffer.clone();
                    }
                    Err(e) => {
                        emsg!(
                            app,
                            "一時ファイル {:?} の読み込み中にエラーが発生しました: {}。元のファイルに戻ります。",
                            temp_path,
                            e
                        );
                        if original_path.exists() {
                            match app.editor.load_from_file(&original_path) {
                                Ok(_) => {
                                    msg!(
                                        app,
                                        "元のファイル {:?} を正常に読み込みました。",
                                        original_path
                                    );
                                    app.original_buffer = app.editor.buffer.clone();
                                    if let Err(e) = app.editor.save_to_file(&temp_path) {
                                        emsg!(
                                            app,
                                            "警告: 初期コンテンツを一時ファイル {:?} に書き込めませんでした: {}",
                                            temp_path,
                                            e
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
                    }
                }
            } else {
                // 一時ファイルが存在しない場合、元のファイルを試す
                if original_path.exists() {
                    match app.editor.load_from_file(&original_path) {
                        Ok(_) => {
                            msg!(
                                app,
                                "元のファイル {:?} を正常に読み込みました。",
                                original_path
                            );
                            app.original_buffer = app.editor.buffer.clone();
                            if let Err(e) = app.editor.save_to_file(&temp_path) {
                                emsg!(
                                    app,
                                    "警告: 初期コンテンツを一時ファイル {:?} に書き込めませんでした: {}",
                                    temp_path,
                                    e
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
            }
        } else {
            msg!(
                app,
                "ファイルパスが指定されていません。空のバッファ（プレーンテキストモード）で開始します。一時ファイルは作成されません。"
            );
        }

        // ファイル内容とパスに基づいてシンタックスを決定
        let first_lines: String = app
            .editor
            .buffer
            .lines()
            .take(5)
            .collect::<Vec<&str>>()
            .join("\n");
        let syntax = app
            .highlighter
            .get_syntax_for_file(app.target_path.as_deref(), &first_lines);
        app.current_syntax_name = syntax.name.clone();
        msg!(app, "言語: {}", app.current_syntax_name);

        app.calculate_diff_status();
        app
    }

    /// 現在のファイルを保存します。元のファイルパスが設定されている必要があります。
    pub fn save_current_file(&mut self) -> io::Result<()> {
        if let Some(original_path) = &self.target_path {
            self.editor.save_to_file(original_path)?;
            msg!(self, "ファイルは {:?} に保存されました。", original_path);
            self.original_buffer = self.editor.buffer.clone();
            self.calculate_diff_status();

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
                self.line_statuses.push(LineStatus::Added);
            }
        }
    }

    pub fn get_visible_message_count(&self) -> u16 {
        const MESSAGE_LIFETIME_SECS: u64 = 3;
        let now = Instant::now();
        self.messages
            .iter()
            .filter(|(_, _, timestamp)| {
                now.duration_since(*timestamp) < Duration::from_secs(MESSAGE_LIFETIME_SECS)
            })
            .count() as u16
    }

    /// 未保存の変更があるかどうかをチェックします。
    pub fn has_unsaved_changes(&self) -> bool {
        self.editor.buffer != self.original_buffer
    }

    /// 終了を試みます。未保存の変更がある場合はポップアップを表示する状態に設定します。
    pub fn trigger_exit_popup_if_needed(&mut self) {
        if self.has_unsaved_changes() {
            self.exit_popup_state = Some(ExitPopupState::default());
        }
    }

    /// 終了ポップアップのキーイベントを処理します。
    pub fn handle_exit_popup_key(
        &mut self,
        key_event: &crossterm::event::KeyEvent,
    ) -> ExitPopupResult {
        if let Some(state) = &mut self.exit_popup_state {
            match key_event.code {
                crossterm::event::KeyCode::Up => {
                    state.previous();
                    ExitPopupResult::None
                }
                crossterm::event::KeyCode::Down => {
                    state.next();
                    ExitPopupResult::None
                }
                crossterm::event::KeyCode::Enter => {
                    let result = match state.selected_option {
                        ExitPopupOption::SaveAndExit => ExitPopupResult::SaveAndExit,
                        ExitPopupOption::DiscardAndExit => ExitPopupResult::DiscardAndExit,
                        ExitPopupOption::Cancel => ExitPopupResult::Cancel,
                    };
                    self.exit_popup_state = None;
                    result
                }
                crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Char('S') => {
                    self.exit_popup_state = None;
                    ExitPopupResult::SaveAndExit
                }
                crossterm::event::KeyCode::Char('d') | crossterm::event::KeyCode::Char('D') => {
                    self.exit_popup_state = None;
                    ExitPopupResult::DiscardAndExit
                }
                crossterm::event::KeyCode::Char('c')
                | crossterm::event::KeyCode::Char('C')
                | crossterm::event::KeyCode::Esc => {
                    self.exit_popup_state = None;
                    ExitPopupResult::Cancel
                }
                _ => ExitPopupResult::None,
            }
        } else {
            ExitPopupResult::None
        }
    }

    /// Highlighterのテーマを変更します。
    pub fn set_highlighter_theme(&mut self, theme_name: &str) {
        if self.highlighter.set_theme(theme_name) {
            self.config.color_theme = theme_name.to_string(); // Configも更新
            msg!(self, "テーマを '{}' に変更しました。", theme_name);
        } else {
            emsg!(self, "テーマ '{}' は見つかりませんでした。", theme_name);
        }
    }
}
