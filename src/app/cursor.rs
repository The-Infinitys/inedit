// src/app/cursor.rs

/// カーソルの位置と選択範囲を管理します。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub x: u16,                              // 列位置 (0-indexed)
    pub y: u16,                              // 行位置 (0-indexed)
    pub selection_start: Option<(u16, u16)>, // 選択範囲の開始位置 (x, y)
}

impl Cursor {
    /// 新しいカーソルを作成します。
    pub fn new(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            selection_start: None,
        }
    }

    /// カーソル位置を設定します。
    /// このメソッドは、実際にバッファの境界チェックは行いません。
    /// 境界チェックはEditor側で行うべきです。
    pub fn set_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    /// カーソルを次の行に移動します。
    pub fn next_line(&mut self) {
        self.y = self.y.saturating_add(1);
        self.x = 0; // 行頭に移動
        self.clear_selection();
    }

    /// カーソルを前の行に移動します。
    pub fn previous_line(&mut self) {
        self.y = self.y.saturating_sub(1);
        self.x = 0; // 行頭に移動
        self.clear_selection();
    }

    /// カーソルを次の文字に移動します。
    pub fn next_char(&mut self) {
        self.x = self.x.saturating_add(1);
        // 行末での改行移動はEditor側で調整
        self.clear_selection();
    }

    /// カーソルを前の文字に移動します。
    pub fn previous_char(&mut self) {
        if self.x > 0 {
            self.x = self.x.saturating_sub(1);
        } else {
            // 行頭で前の文字に移動する場合、前の行の末尾へ
            if self.y > 0 {
                self.y = self.y.saturating_sub(1);
                // xはEditor側で実際の行の長さに調整する必要がある
            }
        }
        self.clear_selection();
    }

    /// 選択範囲を開始します。
    pub fn start_selection(&mut self) {
        self.selection_start = Some((self.x, self.y));
    }

    /// 選択範囲をクリアします。
    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    /// 現在選択中かどうかを返します。
    pub fn is_selecting(&self) -> bool {
        self.selection_start.is_some()
    }
}
