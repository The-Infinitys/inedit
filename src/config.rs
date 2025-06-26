//! アプリケーションの設定（カラーテーマ、キーバインドなど）を管理します。
//! 設定ファイルが存在しない場合はデフォルト設定で作成し、存在する場合は読み込みます。

use crossterm::event::{KeyCode, KeyModifiers};
use directories::ProjectDirs; // クロスプラットフォームな設定ファイルパス解決のため
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf}; // KeyCodeとKeyModifiersをインポート

const CONFIG_FILE_NAME: &str = "config.yaml";

/// アプリケーション全体のコンフィグレーション
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub color_theme: String,
    pub key_bindings: KeyBindings,
}

/// キーバインドの設定
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyBindings {
    // 終了
    pub exit_1: KeyEventConfig, // Ctrl+Q
    pub exit_2: KeyEventConfig, // Ctrl+W
    pub exit_3: KeyEventConfig, // Esc

    // ファイル操作
    pub save_file: KeyEventConfig, // Ctrl+S
    pub open_file: KeyEventConfig, // Ctrl+O (未実装だが予約)

    // 編集操作
    pub copy: KeyEventConfig,                 // Ctrl+C
    pub cut: KeyEventConfig,                  // Ctrl+X
    pub paste: KeyEventConfig,                // Ctrl+V
    pub select_all: KeyEventConfig,           // Ctrl+A
    pub insert_newline: KeyEventConfig,       // Enter
    pub insert_tab: KeyEventConfig,           // Tab
    pub delete_previous_char: KeyEventConfig, // Backspace
    pub delete_current_char: KeyEventConfig,  // Delete

    // Undo/Redo
    pub undo: KeyEventConfig, // Ctrl+Z
    pub redo: KeyEventConfig, // Ctrl+Y

    // カーソル移動
    pub move_left: KeyEventConfig,
    pub move_right: KeyEventConfig,
    pub move_up: KeyEventConfig,
    pub move_down: KeyEventConfig,
    pub move_line_start: KeyEventConfig,     // Home
    pub move_line_end: KeyEventConfig,       // End
    pub move_document_start: KeyEventConfig, // Ctrl+Home (または類似)
    pub move_document_end: KeyEventConfig,   // Ctrl+End (または類似)

    // 機能トグル
    pub toggle_word_wrap: KeyEventConfig, // Alt+Z
}

/// キーイベントをシリアライズ/デシリアライズ可能な形式で表現
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyEventConfig {
    pub code: String,           // KeyCodeを文字列で保存 (例: "Char(q)", "Esc", "Left")
    pub modifiers: Vec<String>, // KeyModifiersを文字列のVecで保存 (例: ["Control", "Shift"])
}

impl KeyEventConfig {
    /// `crossterm::event::KeyCode`と`crossterm::event::KeyModifiers`から`KeyEventConfig`を生成
    pub fn from_key_event(code: KeyCode, modifiers: KeyModifiers) -> Self {
        let code_str = format!("{:?}", code);
        let mut modifiers_vec = Vec::new();
        if modifiers.contains(KeyModifiers::CONTROL) {
            modifiers_vec.push("Control".to_string());
        }
        if modifiers.contains(KeyModifiers::ALT) {
            modifiers_vec.push("Alt".to_string());
        }
        if modifiers.contains(KeyModifiers::SHIFT) {
            modifiers_vec.push("Shift".to_string());
        }
        KeyEventConfig {
            code: code_str,
            modifiers: modifiers_vec,
        }
    }

    /// `KeyEventConfig`が指定された`crossterm::event::KeyEvent`と一致するかチェック
    pub fn matches(&self, key_event: &crossterm::event::KeyEvent) -> bool {
        // KeyCodeの比較
        let self_code_str = &self.code;
        let event_code_str = format!("{:?}", key_event.code);
        if self_code_str != &event_code_str {
            return false;
        }

        // --- 修正: self.modifiersが空なら、修飾キー問わずマッチする ---
        if self.modifiers.is_empty() {
            return true;
        }

        // KeyModifiersの比較（self.modifiersが全てevent_modifiers_vecに含まれていればOK）
        let mut event_modifiers_vec = Vec::new();
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            event_modifiers_vec.push("Control");
        }
        if key_event.modifiers.contains(KeyModifiers::ALT) {
            event_modifiers_vec.push("Alt");
        }
        if key_event.modifiers.contains(KeyModifiers::SHIFT) {
            event_modifiers_vec.push("Shift");
        }

        self.modifiers
            .iter()
            .all(|m| event_modifiers_vec.contains(&m.as_str()))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            color_theme: "Solarized (dark)".to_string(), // デフォルトのテーマ名
            key_bindings: KeyBindings::default(),
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        KeyBindings {
            exit_1: KeyEventConfig::from_key_event(KeyCode::Char('q'), KeyModifiers::CONTROL),
            exit_2: KeyEventConfig::from_key_event(KeyCode::Char('w'), KeyModifiers::CONTROL),
            exit_3: KeyEventConfig::from_key_event(KeyCode::Esc, KeyModifiers::NONE),

            save_file: KeyEventConfig::from_key_event(KeyCode::Char('s'), KeyModifiers::CONTROL),
            open_file: KeyEventConfig::from_key_event(KeyCode::Char('o'), KeyModifiers::CONTROL),

            copy: KeyEventConfig::from_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL),
            cut: KeyEventConfig::from_key_event(KeyCode::Char('x'), KeyModifiers::CONTROL),
            paste: KeyEventConfig::from_key_event(KeyCode::Char('v'), KeyModifiers::CONTROL),
            select_all: KeyEventConfig::from_key_event(KeyCode::Char('a'), KeyModifiers::CONTROL),
            insert_newline: KeyEventConfig::from_key_event(KeyCode::Enter, KeyModifiers::NONE),
            insert_tab: KeyEventConfig::from_key_event(KeyCode::Tab, KeyModifiers::NONE),
            delete_previous_char: KeyEventConfig::from_key_event(
                KeyCode::Backspace,
                KeyModifiers::NONE,
            ),
            delete_current_char: KeyEventConfig::from_key_event(
                KeyCode::Delete,
                KeyModifiers::NONE,
            ),

            // Undo/Redo
            undo: KeyEventConfig::from_key_event(KeyCode::Char('z'), KeyModifiers::CONTROL),
            redo: KeyEventConfig::from_key_event(KeyCode::Char('y'), KeyModifiers::CONTROL),

            move_left: KeyEventConfig::from_key_event(KeyCode::Left, KeyModifiers::NONE),
            move_right: KeyEventConfig::from_key_event(KeyCode::Right, KeyModifiers::NONE),
            move_up: KeyEventConfig::from_key_event(KeyCode::Up, KeyModifiers::NONE),
            move_down: KeyEventConfig::from_key_event(KeyCode::Down, KeyModifiers::NONE),
            move_line_start: KeyEventConfig::from_key_event(KeyCode::Home, KeyModifiers::NONE),
            move_line_end: KeyEventConfig::from_key_event(KeyCode::End, KeyModifiers::NONE),
            move_document_start: KeyEventConfig::from_key_event(
                KeyCode::Home,
                KeyModifiers::CONTROL,
            ), // Ctrl+Home
            move_document_end: KeyEventConfig::from_key_event(KeyCode::End, KeyModifiers::CONTROL), // Ctrl+End

            toggle_word_wrap: KeyEventConfig::from_key_event(KeyCode::Char('z'), KeyModifiers::ALT),
        }
    }
}

/// 設定ファイルのパスを取得します。
/// クロスプラットフォームに対応するため、`directories`クレートを使用します。
fn get_config_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "inedit", "Inedit") {
        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            // ディレクトリが存在しない場合は作成を試みる
            if let Err(_e) = fs::create_dir_all(config_dir) {
                // eprintln!("Error creating config directory {:?}: {}", config_dir, e);
                return None;
            }
        }
        Some(config_dir.join(CONFIG_FILE_NAME))
    } else {
        // eprintln!("Could not determine config directory.");
        None
    }
}

/// 設定ファイルを読み込みます。ファイルが存在しない場合はデフォルト設定で作成します。
pub fn load_or_create_config() -> Config {
    if let Some(config_path) = get_config_path() {
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match serde_yaml::from_str(&content) {
                    Ok(config) => {
                        // eprintln!("Config loaded from {:?}", config_path);
                        return config;
                    }
                    Err(_e) => {
                        // eprintln!("Error parsing config file {:?}: {}", config_path, e);
                        // パースエラーの場合はデフォルト設定を返し、上書き保存
                        let default_config = Config::default();
                        if let Err(_e) = save_config(&default_config) {
                            // eprintln!("Error saving default config: {}", e);
                        }
                        return default_config;
                    }
                },
                Err(_e) => {
                    // eprintln!("Error reading config file {:?}: {}", config_path, e);
                    // 読み込みエラーの場合もデフォルト設定を返し、上書き保存
                    let default_config = Config::default();
                    if let Err(_e) = save_config(&default_config) {
                        // eprintln!("Error saving default config: {}", e);
                    }
                    return default_config;
                }
            }
        } else {
            // ファイルが存在しない場合はデフォルト設定で作成
            let default_config = Config::default();
            if let Err(_e) = save_config(&default_config) {
                // eprintln!("Error saving default config: {}", e);
            }
            // eprintln!("Created default config at {:?}", config_path);
            return default_config;
        }
    }
    // コンフィグパスが取得できない場合もデフォルト設定を返す
    // eprintln!("Could not get config path, using default config.");
    Config::default()
}

/// 設定をファイルに保存します。
pub fn save_config(config: &Config) -> io::Result<()> {
    if let Some(config_path) = get_config_path() {
        let yaml_content = serde_yaml::to_string(config)
            .map_err(|e| io::Error::other(format!("Failed to serialize config: {}", e)))?;
        fs::write(&config_path, yaml_content)?;
        // eprintln!("Config saved to {:?}", config_path);
        Ok(())
    } else {
        Err(io::Error::other(
            "Could not determine config path for saving.",
        ))
    }
}
