use super::Editor;
use arboard::Clipboard;

impl Editor {
    /// 現在のカーソル位置から選択範囲を取得します。
    /// 戻り値は (開始バイトオフセット, 終了バイトオフセット) のタプルです。
    /// 選択範囲がない場合はNoneを返します。
    pub fn get_selection_range(&self) -> Option<(usize, usize)> {
        if let Some((start_coords, end_coords)) = self.cursor.get_normalized_selection_coords() {
            // (y, x) 座標をバイトオフセットに変換するヘルパー関数
            let coords_to_byte_offset = |y_coord: u16, x_coord: u16| -> usize {
                let mut offset = 0;
                for (i, line) in self.buffer.lines().enumerate() {
                    if i == y_coord as usize {
                        // 指定された行と列の文字オフセットまでのバイト数を計算
                        offset += line
                            .chars()
                            .take(x_coord as usize)
                            .map(|c| c.len_utf8())
                            .sum::<usize>();
                        break;
                    }
                    offset += line.len();
                    offset += 1; // 改行文字（LF）のバイト長を仮定
                }
                offset
            };

            let start_byte_offset = coords_to_byte_offset(start_coords.0, start_coords.1);
            let end_byte_offset = coords_to_byte_offset(end_coords.0, end_coords.1);

            // 選択範囲が実際のテキストと重複しないように調整
            if start_byte_offset == end_byte_offset {
                None
            } else {
                Some((start_byte_offset, end_byte_offset))
            }
        } else {
            None
        }
    }

    /// バッファの内容全体を選択します。
    pub fn select_all(&mut self) {
        // 1. 先頭にカーソルを移動し、選択開始
        self.set_cursor_position(0, 0, false); // 選択解除して先頭へ
        // 2. 末尾にカーソルを移動し、選択を拡張
        self.move_cursor_to_document_end(true); // extend_selection = true で末尾まで選択
    }

    /// 選択された範囲のテキストをコピーします。
    /// クリップボードが利用可能ならOSクリップボードにも書き込む
    pub fn copy_selection(&mut self) -> Option<String> {
        if let Some((start, end)) = self.get_selection_range() {
            let text = self.buffer[start..end].to_string();
            // OSクリップボードに書き込む（失敗しても無視）
            if let Ok(mut clipboard) = Clipboard::new() {
                let _ = clipboard.set_text(text.clone());
            }
            Some(text)
        } else {
            None
        }
    }

    /// 選択された範囲のテキストを切り取り、バッファから削除します。
    /// クリップボードが利用可能ならOSクリップボードにも書き込む
    pub fn cut_selection(&mut self) -> Option<String> {
        if let Some((start_byte_offset, end_byte_offset)) = self.get_selection_range() {
            let cut_text = self.buffer[start_byte_offset..end_byte_offset].to_string();

            // OSクリップボードに書き込む（失敗しても無視）
            if let Ok(mut clipboard) = Clipboard::new() {
                let _ = clipboard.set_text(cut_text.clone());
            }

            // 切り取り後のカーソル位置を、選択範囲の開始位置に調整する
            // バイトオフセットから新しいカーソル座標を計算
            let lines_before_cut: Vec<&str> = self.buffer[..start_byte_offset].lines().collect();
            let new_y = (lines_before_cut.len() as u16).saturating_sub(1); // 切り取り開始行
            let new_x = if new_y == u16::MAX {
                // バッファの先頭で切り取りの場合
                0
            } else {
                lines_before_cut
                    .last()
                    .map_or(0, |last_line| last_line.chars().count() as u16)
            };

            self.buffer
                .replace_range(start_byte_offset..end_byte_offset, ""); // 選択範囲を削除

            // カーソル位置を切り取った後の位置に設定し、選択を解除
            self.set_cursor_position(new_x, new_y, false);
            Some(cut_text)
        } else {
            None
        }
    }

    /// 指定されたテキストをカーソル位置にペーストします。
    pub fn paste_text(&mut self, text: &str) {
        if self.cursor.is_selecting() {
            self.cut_selection(); // 選択範囲がある場合はまず切り取る
        }

        let current_offset = self.get_cursor_byte_offset(); // 現在のカーソル位置のバイトオフセット
        self.buffer.insert_str(current_offset, text); // テキストを挿入

        let new_cursor_offset = current_offset + text.len(); // 新しいカーソル位置のバイトオフセット
        self.set_cursor_from_byte_offset(new_cursor_offset, false); // カーソル位置を更新し、選択を解除
    }

    /// OSクリップボードから貼り付ける。失敗した場合はapp.clipboardを使う
    pub fn paste_from_clipboard(&mut self, clipboard: &Option<String>) {
        // OSクリップボードが利用可能ならそちらを優先
        if let Ok(mut sys_clip) = Clipboard::new() {
            if let Ok(text) = sys_clip.get_text() {
                self.paste_text(&text);
                return;
            }
        }
        // 失敗した場合はapp.clipboardを使う
        if let Some(text) = clipboard {
            self.paste_text(text);
        }
    }

    /// カーソル位置に文字を挿入します。
    pub fn insert_char(&mut self, c: char) {
        self.push_undo();
        if self.cursor.is_selecting() {
            self.cut_selection(); // 選択範囲がある場合はまず切り取る
        }

        let current_offset = self.get_cursor_byte_offset(); // 現在のカーソル位置のバイトオフセット
        self.buffer.insert(current_offset, c); // 文字を挿入

        // カーソル位置を更新（改行の場合は次の行の先頭、それ以外はX座標を1進める）
        if c == '\n' {
            self.set_cursor_position(0, self.cursor.y.saturating_add(1), false);
        } else {
            self.set_cursor_position(self.cursor.x.saturating_add(1), self.cursor.y, false);
        }
    }

    /// カーソル位置の文字、または選択範囲を削除します。（Backspace相当）
    pub fn delete_previous_char(&mut self) {
        if self.cursor.is_selecting() {
            self.cut_selection();
            return;
        }

        let current_offset = self.get_cursor_byte_offset(); // 現在のカーソル位置のバイトオフセット
        if current_offset > 0 {
            // 文字の境界を考慮して、前の文字のバイト開始位置を特定
            let mut char_start_offset = current_offset;
            // マルチバイト文字の途中にカーソルがある場合は、文字の先頭まで戻る
            while char_start_offset > 0 && !self.buffer.is_char_boundary(char_start_offset - 1) {
                char_start_offset -= 1;
            }
            if char_start_offset > 0 {
                // 前の文字（char_start_offset - 1 から始まる文字）を削除
                self.buffer.remove(char_start_offset - 1);
                // カーソル位置を新しい位置に調整（選択はクリア）
                self.set_cursor_from_byte_offset(char_start_offset - 1, false);
            }
        }
    }

    /// カーソル位置の文字、または選択範囲を削除します。（Deleteキー相当）
    pub fn delete_current_char(&mut self) {
        if self.cursor.is_selecting() {
            self.cut_selection();
            return;
        }

        let current_offset = self.get_cursor_byte_offset(); // 現在のカーソル位置のバイトオフセット
        if current_offset < self.buffer.len() {
            // 現在のカーソル位置から次の文字のバイト終了位置を特定
            let mut char_start_for_deletion = current_offset;
            // マルチバイト文字の途中にカーソルがある場合は、文字の先頭まで進む
            while char_start_for_deletion < self.buffer.len()
                && !self.buffer.is_char_boundary(char_start_for_deletion)
            {
                char_start_for_deletion += 1;
            }

            // `chars().next()` を使って現在のカーソル位置の文字（または次の文字）のバイト長を取得
            let char_len_to_delete = self.buffer[char_start_for_deletion..]
                .chars()
                .next()
                .map_or(0, |c| c.len_utf8());

            if char_len_to_delete > 0 {
                self.buffer.replace_range(
                    char_start_for_deletion..(char_start_for_deletion + char_len_to_delete),
                    "",
                );
                // Deleteキーの場合、カーソル位置は変更しない（選択はクリア）
                self.set_cursor_position(self.cursor.x, self.cursor.y, false);
            }
        }
    }

    /// 指定された範囲のテキストを新しいテキストで置き換えます。
    pub fn replace_buffer_range(
        &mut self,
        start_byte_offset: usize,
        end_byte_offset: usize,
        new_text: &str,
    ) {
        if start_byte_offset <= end_byte_offset && end_byte_offset <= self.buffer.len() {
            self.buffer
                .replace_range(start_byte_offset..end_byte_offset, new_text);
            let new_cursor_offset = start_byte_offset + new_text.len(); // 置換後の新しいカーソル位置
            self.set_cursor_from_byte_offset(new_cursor_offset, false); // カーソル位置を更新し、選択をクリア
        }
    }

    /// 編集操作の前に呼び出して履歴を積む
    fn push_undo(&mut self) {
        self.undo_stack.push(self.buffer.clone());
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.buffer.clone());
            self.buffer = prev;
            // カーソル位置も復元したい場合は別途保存が必要
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.buffer.clone());
            self.buffer = next;
        }
    }
}