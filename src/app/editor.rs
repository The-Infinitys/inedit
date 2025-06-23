// src/app/editor.rs

use super::cursor::Cursor; // appモジュール内のcursorモジュールからCursorをインポート

/// テキストバッファとカーソルを管理し、編集操作を提供します。
#[derive(Default)]
pub struct Editor {
    pub buffer: String,
    pub cursor: Cursor,
}

impl Editor {
    /// 新しいエディタを作成します。
    pub fn new(initial_text: String) -> Self {
        Self {
            buffer: initial_text,
            cursor: Cursor::new(0, 0),
        }
    }

    /// カーソルを新しい位置に移動させます。
    /// バッファの境界を考慮して位置を調整します。
    pub fn set_cursor_position(&mut self, x: u16, y: u16) {
        let lines: Vec<&str> = self.buffer.lines().collect();
        let num_lines = lines.len();

        let mut new_y = y;
        if num_lines == 0 {
            new_y = 0;
        } else {
            new_y = new_y.min((num_lines - 1) as u16);
        }

        let mut new_x = x;
        if num_lines > 0 {
            let current_line_len = lines[new_y as usize].chars().count() as u16;
            new_x = new_x.min(current_line_len);
        } else {
            new_x = 0;
        }

        self.cursor.set_position(new_x, new_y);
        self.cursor.clear_selection(); // カーソル移動時は選択を解除
    }

    /// カーソルを次の行に移動します。
    pub fn next_line(&mut self) {
        self.cursor.next_line();
        self.set_cursor_position(self.cursor.x, self.cursor.y); // 境界調整
    }

    /// カーソルを前の行に移動します。
    pub fn previous_line(&mut self) {
        self.cursor.previous_line();
        self.set_cursor_position(self.cursor.x, self.cursor.y); // 境界調整
    }

    /// カーソルを次の文字に移動します。
    pub fn next_char(&mut self) {
        let lines: Vec<&str> = self.buffer.lines().collect();
        let current_line_idx = self.cursor.y as usize;

        if current_line_idx < lines.len() {
            let current_line = lines[current_line_idx];
            if (self.cursor.x as usize) < current_line.chars().count() {
                self.cursor.next_char();
            } else {
                // 現在の行の末尾にいる場合、次の行の先頭へ
                if (current_line_idx + 1) < lines.len() {
                    self.cursor.next_line();
                }
            }
        }
        self.set_cursor_position(self.cursor.x, self.cursor.y); // 境界調整
    }

    /// カーソルを前の文字に移動します。
    pub fn previous_char(&mut self) {
        let lines: Vec<&str> = self.buffer.lines().collect();
        let current_line_idx = self.cursor.y as usize;

        if self.cursor.x > 0 {
            self.cursor.previous_char();
        } else {
            // 行頭にいる場合、前の行の末尾へ
            if current_line_idx > 0 {
                self.cursor.previous_line();
                // 前の行の長さにxを調整する必要がある
                let prev_line_len = lines[self.cursor.y as usize].chars().count() as u16;
                self.cursor.set_position(prev_line_len, self.cursor.y);
            }
        }
        self.set_cursor_position(self.cursor.x, self.cursor.y); // 境界調整
    }

    /// 現在のカーソル位置から選択範囲を取得します。
    /// 戻り値は (開始オフセット, 終了オフセット) のタプルで、バイト単位のオフセットです。
    /// 選択範囲がない場合はNoneを返します。
    pub fn get_selection_range(&self) -> Option<(usize, usize)> {
        if let Some(start_pos) = self.cursor.selection_start {
            // (x, y)座標をバイトオフセットに変換するヘルパー関数
            let pos_to_offset = |x: u16, y: u16| -> usize {
                let mut offset = 0;
                for (i, line) in self.buffer.lines().enumerate() {
                    if i == y as usize {
                        // 行の先頭からのバイトオフセットを計算
                        // UTF-8の場合、文字数とバイト数が一致しない場合があるため、chars().take()を使う
                        offset += line
                            .chars()
                            .take(x as usize)
                            .map(|c| c.len_utf8())
                            .sum::<usize>();
                        break;
                    }
                    offset += line.len(); // 行のバイト長
                    offset += 1; // 改行文字のバイト長 (LFの場合)
                }
                offset
            };

            let current_offset = pos_to_offset(self.cursor.x, self.cursor.y);
            let start_offset = pos_to_offset(start_pos.0, start_pos.1);

            if current_offset == start_offset {
                None // 選択範囲がない場合
            } else if current_offset > start_offset {
                Some((start_offset, current_offset))
            } else {
                Some((current_offset, start_offset)) // 逆方向選択の場合
            }
        } else {
            None
        }
    }

    /// バッファの内容全体を選択します。
    pub fn select_all(&mut self) {
        self.cursor.set_position(0, 0); // カーソルを先頭に移動
        self.cursor.start_selection(); // 選択開始
        let lines: Vec<&str> = self.buffer.lines().collect();
        let last_line_idx = (lines.len() as u16).saturating_sub(1);
        let last_col_idx = if lines.is_empty() {
            0
        } else {
            lines[last_line_idx as usize].chars().count() as u16
        };
        self.cursor.set_position(last_col_idx, last_line_idx); // カーソルを末尾に移動
    }

    /// 選択された範囲のテキストをコピーします。
    /// クリップボード機能はOS依存なので、ここではStringを返します。
    pub fn copy_selection(&self) -> Option<String> {
        if let Some((start, end)) = self.get_selection_range() {
            Some(self.buffer[start..end].to_string())
        } else {
            None
        }
    }

    /// 選択された範囲のテキストを切り取り、バッファから削除します。
    /// クリップボード機能はOS依存なので、ここではStringを返します。
    pub fn cut_selection(&mut self) -> Option<String> {
        if let Some((start, end)) = self.get_selection_range() {
            let cut_text = self.buffer[start..end].to_string();
            self.buffer.replace_range(start..end, "");
            // カーソル位置を更新する必要がある (切り取られた部分の開始位置に移動)
            // これは複雑なロジックになりがちなので、ここでは選択開始位置に設定する
            if let Some(sel_start_pos) = self.cursor.selection_start {
                self.set_cursor_position(sel_start_pos.0, sel_start_pos.1);
            } else {
                self.set_cursor_position(0, 0); // 念のため先頭
            }
            self.cursor.clear_selection();
            Some(cut_text)
        } else {
            None
        }
    }

    /// 指定されたテキストをカーソル位置にペーストします。
    /// （クリップボードから取得したテキストを引数で渡す想定）
    pub fn paste_text(&mut self, text: &str) {
        // 選択範囲があれば、まずそれを削除
        if self.cursor.is_selecting() {
            self.cut_selection(); // cut_selectionはカーソルを調整し、選択をクリアする
        }

        let current_offset = self.get_cursor_byte_offset();
        self.buffer.insert_str(current_offset, text);

        // ペースト後のカーソル位置を調整
        let new_cursor_offset = current_offset + text.len();
        self.set_cursor_from_byte_offset(new_cursor_offset);
        self.cursor.clear_selection();
    }

    /// カーソル位置に文字を挿入します。
    pub fn insert_char(&mut self, c: char) {
        // 選択範囲があれば、まずそれを削除
        if self.cursor.is_selecting() {
            self.cut_selection(); // cut_selectionはカーソルを調整し、選択をクリアする
        }

        let current_offset = self.get_cursor_byte_offset();
        self.buffer.insert(current_offset, c);

        // カーソルを挿入した文字の直後に移動
        if c == '\n' {
            self.set_cursor_position(0, self.cursor.y.saturating_add(1));
        } else {
            self.set_cursor_position(self.cursor.x.saturating_add(1), self.cursor.y);
        }
        self.cursor.clear_selection();
    }

    /// カーソル位置の文字、または選択範囲を削除します。（Backspace相当）
    pub fn delete_previous_char(&mut self) {
        if self.cursor.is_selecting() {
            self.cut_selection();
            return;
        }

        let current_offset = self.get_cursor_byte_offset();
        if current_offset > 0 {
            // UTF-8文字の境界を探す
            let mut char_boundary_offset = current_offset;
            while char_boundary_offset > 0
                && !self.buffer.is_char_boundary(char_boundary_offset - 1)
            {
                char_boundary_offset -= 1;
            }
            if char_boundary_offset > 0 {
                self.buffer.remove(char_boundary_offset - 1);
                // カーソル位置を更新
                self.set_cursor_from_byte_offset(char_boundary_offset - 1);
            }
        }
        self.cursor.clear_selection();
    }

    /// カーソル位置の文字、または選択範囲を削除します。（Deleteキー相当）
    pub fn delete_current_char(&mut self) {
        if self.cursor.is_selecting() {
            self.cut_selection();
            return;
        }

        let current_offset = self.get_cursor_byte_offset();
        if current_offset < self.buffer.len() {
            // UTF-8文字の境界を探す
            let mut char_boundary_offset = current_offset;
            // Find the start of the next character
            while char_boundary_offset < self.buffer.len()
                && !self.buffer.is_char_boundary(char_boundary_offset)
            {
                char_boundary_offset += 1;
            }
            // Now char_boundary_offset is at the start of the character to delete
            if char_boundary_offset < self.buffer.len() {
                // Find the end of the character to delete
                let next_char_boundary = self.buffer[char_boundary_offset..]
                    .chars()
                    .next()
                    .map_or(char_boundary_offset, |c| {
                        char_boundary_offset + c.len_utf8()
                    });

                self.buffer
                    .replace_range(char_boundary_offset..next_char_boundary, "");
                // カーソル位置は変更しない
            }
        }
        self.cursor.clear_selection();
    }

    /// 指定された範囲のテキストを新しいテキストで置き換えます。
    /// （選択範囲のテキストを置き換える用途を想定）
    pub fn replace_buffer_range(
        &mut self,
        start_byte_offset: usize,
        end_byte_offset: usize,
        new_text: &str,
    ) {
        if start_byte_offset <= end_byte_offset && end_byte_offset <= self.buffer.len() {
            self.buffer
                .replace_range(start_byte_offset..end_byte_offset, new_text);
            // 置き換え後のカーソル位置を調整
            let new_cursor_offset = start_byte_offset + new_text.len();
            self.set_cursor_from_byte_offset(new_cursor_offset);
            self.cursor.clear_selection();
        }
    }

    /// 現在のカーソル位置をバイトオフセットに変換します。
    fn get_cursor_byte_offset(&self) -> usize {
        let mut offset = 0;
        for (current_y, line) in self.buffer.lines().enumerate() {
            if current_y == self.cursor.y as usize {
                // 現在の行の先頭からのバイトオフセットを計算
                offset += line
                    .chars()
                    .take(self.cursor.x as usize)
                    .map(|c| c.len_utf8())
                    .sum::<usize>();
                break;
            }
            offset += line.len(); // 行のバイト長
            offset += 1; // 改行文字のバイト長 (LFを仮定)
        }
        offset
    }

    /// バイトオフセットからカーソル位置 (x, y) を設定します。
    fn set_cursor_from_byte_offset(&mut self, byte_offset: usize) {
        let mut current_offset = 0;
        let mut y = 0;
        let mut x_chars_count = 0; // Track characters count for x

        for line in self.buffer.lines() {
            let line_len_bytes = line.len(); // 行のバイト長
            let line_with_newline_len = line_len_bytes + 1; // 改行含む

            if byte_offset >= current_offset && byte_offset <= current_offset + line_len_bytes {
                // カーソルがこの行内にある
                let relative_offset = byte_offset - current_offset;

                // Characters are iterated to find the x position
                x_chars_count = 0;
                let mut current_byte_in_line = 0;
                for c in line.chars() {
                    let char_len_bytes = c.len_utf8();
                    if current_byte_in_line + char_len_bytes > relative_offset {
                        // This character is past the target byte offset, so stop here
                        break;
                    }
                    current_byte_in_line += char_len_bytes;
                    x_chars_count += 1;
                }

                // Ensure x is not beyond the actual character count of the line
                x_chars_count = x_chars_count.min(line.chars().count());
                break;
            }

            current_offset += line_with_newline_len;
            y += 1;
        }
        self.cursor.set_position(x_chars_count as u16, y as u16);
    }
}
