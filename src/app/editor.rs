// src/app/editor.rs
pub mod actions;
pub mod movement;
pub mod search;
pub mod suggest;
pub mod view;

use super::cursor::Cursor;
use std::fs;
use std::io;
use std::path::Path;

/// テキストバッファとカーソルを管理し、編集操作を提供します。
#[derive(Default)]
pub struct Editor {
    pub buffer: String,
    pub cursor: Cursor,
    pub search_query: String,
    pub search_matches: Vec<(u16, u16)>, // 検索結果の(y, x)位置 (文字単位)
    pub current_search_idx: Option<usize>, // 現在の検索結果のインデックス
    pub scroll_offset_y: u16,            // 垂直方向のスクロールオフセット (行単位)
    pub scroll_offset_x: u16,            // 水平方向のスクロールオフセット (文字単位)
    pub cursor_wrap_idx: usize,          // 折返しインデックスを追加
    pub(super) undo_stack: Vec<String>,
    pub(super) redo_stack: Vec<String>,
}

impl Editor {
    /// 新しいエディタを作成します。
    pub fn new(initial_text: String) -> Self {
        Self {
            buffer: initial_text,
            cursor: Cursor::new(0, 0),
            search_query: String::new(),
            search_matches: Vec::new(),
            current_search_idx: None,
            scroll_offset_y: 0, // 初期スクロールオフセット
            scroll_offset_x: 0, // 初期スクロールオフセット
            cursor_wrap_idx: 0, // 初期値
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// 指定されたパスからテキストを読み込み、エディタバッファを設定します。
    /// （App層によって、これが元ファイルか一時ファイルかが決定されます。）
    pub fn load_from_file(&mut self, path: &Path) -> io::Result<()> {
        let content = fs::read_to_string(path)?;
        self.buffer = content;
        // ファイルを読み込んだら、カーソルを先頭に設定し、選択をクリア
        self.set_cursor_position(0, 0, false);
        // 新しいファイルの内容なのでスクロールオフセットもリセット
        self.scroll_offset_y = 0;
        self.scroll_offset_x = 0;
        Ok(())
    }

    /// エディタバッファの内容を指定されたパスに書き込みます。
    /// （App層によって、これが元ファイルか一時ファイルかが決定されます。）
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        fs::write(path, &self.buffer)
    }
}
