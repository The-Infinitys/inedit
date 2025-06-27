// src/app/cursor.rs

/// カーソルの論理的な位置と選択範囲を管理します。
/// この構造体は、テキストバッファの内容やその境界に関する知識を持ちません。
/// 実際のカーソル位置の調整（バッファ境界内へのクランプ）はEditor側で行うべきです。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub x: u16,                              // 列位置 (0-indexed, 文字単位)
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

    /// カーソル位置を更新します。
    /// このメソッドは、`Editor`が境界チェックを行った後に、最終的なカーソル位置を
    /// 設定するために呼び出すことを想定しています。
    ///
    /// `extend_selection`が`true`の場合、現在のカーソル位置を基準に選択範囲を拡張します。
    /// まだ選択が開始されていない場合は、この呼び出しの前のカーソル位置が選択開始点となります。
    /// `false`の場合、既存の選択範囲はクリアされます。
    pub fn update_position(&mut self, new_x: u16, new_y: u16, extend_selection: bool) {
        if !extend_selection {
            // 選択を拡張しない場合、現在の選択をクリア
            self.clear_selection();
        } else if self.selection_start.is_none() {
            // 選択モードでまだ選択が開始されていない場合、現在の（移動前の）カーソル位置を選択開始点とする
            // 修正箇所: ここで new_x, new_y ではなく self.x, self.y を使用します。
            self.selection_start = Some((self.x, self.y));
        }
        self.x = new_x;
        self.y = new_y;
    }

    /// 現在のカーソル位置を取得します。
    pub fn get_current_pos(&self) -> (u16, u16) {
        (self.x, self.y)
    }

    /// 選択範囲をクリアします。
    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    /// 現在選択中かどうかを返します。
    pub fn is_selecting(&self) -> bool {
        self.selection_start.is_some()
    }

    /// 選択範囲が設定されている場合、正規化された開始点と終了点（(y, x)形式）を返します。
    /// 開始点と終了点は、常に論理的なドキュメント順でソートされます。
    /// 例: `((start_y, start_x), (end_y, end_x))`
    ///
    /// 注意: ここで返される座標は論理的なものであり、バイトオフセットではありません。
    /// バイトオフセットへの変換はEditor側で行う必要があります。
    pub fn get_normalized_selection_coords(&self) -> Option<((u16, u16), (u16, u16))> {
        if let Some(start_pos) = self.selection_start {
            // selection_startは(x, y)形式なので、(y, x)に変換
            let selection_start_y_x = (start_pos.1, start_pos.0);
            let current_pos_y_x = (self.y, self.x);

            // 選択開始点と現在のカーソル位置が同じ場合は選択範囲がない
            if selection_start_y_x == current_pos_y_x {
                None
            } else {
                let mut points = [selection_start_y_x, current_pos_y_x];
                // y座標でソートし、yが同じ場合はx座標でソート
                points.sort_unstable();
                Some((points[0], points[1]))
            }
        } else {
            None
        }
    }
}

// これらのメソッドは、カーソルの「希望する」移動先を計算するためのヘルパーです。
// 実際の最終的なカーソル位置（バッファ境界に合わせたもの）は、Editor側で決定されます。
impl Cursor {
    /// カーソルを1行下に移動したと仮定したY座標を返します。
    pub fn get_potential_next_line_y(&self) -> u16 {
        self.y.saturating_add(1)
    }

    /// カーソルを1行上に移動したと仮定したY座標を返します。
    pub fn get_potential_previous_line_y(&self) -> u16 {
        self.y.saturating_sub(1)
    }

    /// カーソルを1文字右に移動したと仮定したX座標を返します。
    pub fn get_potential_next_char_x(&self) -> u16 {
        self.x.saturating_add(1)
    }

    /// カーソルを1文字左に移動したと仮定したX座標を返します。
    pub fn get_potential_previous_char_x(&self) -> u16 {
        self.x.saturating_sub(1)
    }

    /// カーソルを現在の行の先頭に移動したと仮定したX座標を返します。
    pub fn get_potential_start_of_line_x(&self) -> u16 {
        0
    }

    /// カーソルを現在の行の末尾に移動したと仮定したX座標を返します。
    /// これはEditorが実際の行の長さを考慮して調整する「マーカー」として使用されるべきです。
    pub fn get_potential_end_of_line_x(&self) -> u16 {
        u16::MAX // Editorがこの値を「行末」として解釈することを期待
    }

    /// カーソルをドキュメントの先頭に移動したと仮定したY座標とX座標を返します。
    pub fn get_potential_document_start_pos(&self) -> (u16, u16) {
        (0, 0) // (y, x) 形式
    }

    /// カーソルをドキュメントの末尾に移動したと仮定したY座標とX座標を返します。
    /// これもEditorが実際のドキュメントの行数と最後の行の長さを考慮して調整する「マーカー」として使用されるべきです。
    pub fn get_potential_document_end_pos(&self) -> (u16, u16) {
        (u16::MAX, u16::MAX) // (y, x) 形式。Editorがこの値を「ドキュメント末尾」として解釈することを期待
    }
}
