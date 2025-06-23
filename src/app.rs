// src/app.rs

use editor::Editor;
use std::path::PathBuf; // appモジュール内のeditorモジュールからEditorをインポート
pub mod cursor;
pub mod editor;
// App構造体はEditorのラッパーとして残します。
// 実際のロジックはEditorに集約させます。
pub struct App {
    pub editor: Editor,
    pub target_path: PathBuf,
}

impl Default for App {
    fn default() -> Self {
        Self {
            editor: Editor::default(),
            target_path: PathBuf::new(),
        }
    }
}

impl App {
    pub fn init() -> Self {
        let app = Self::default();
        // ここで第一引数からテキストを読み込むロジックを実装できます。
        // app.editor.set_buffer_from_file("path/to/file.txt"); // 例
        app
    }
}

// 以前Appにあったset_cursor関数はEditorに移動し、
// Appからはeditorフィールドを介して呼び出すことになります。
// 例: app.editor.set_cursor_position(x, y);
