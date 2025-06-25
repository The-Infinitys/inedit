// src/app/editor.rs
pub mod search;
pub mod suggest;

use super::cursor::Cursor;
use arboard::Clipboard;
use ratatui::layout::Rect; // Rectはadjust_viewport_offsetで使用するため残す
use std::fs;
use unicode_width::UnicodeWidthChar; // `ch.width()`のためにインポート
use std::io;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

/// テキストバッファとカーソルを管理し、編集操作を提供します。
pub struct Editor {
    pub lines: Vec<String>,
    pub cursor: Cursor,
    pub search_query: String,
    pub search_matches: Vec<(u16, u16)>, // 検索結果の(y, x)位置 (文字単位)
    pub current_search_idx: Option<usize>, // 現在の検索結果のインデックス
    pub scroll_offset_visual_y: u16, // 垂直方向のスクロールオフセット (視覚行単位)
    pub scroll_offset_x: u16,        // 水平方向のスクロールオフセット (文字単位)
    undo_stack: Vec<Vec<String>>,
    redo_stack: Vec<Vec<String>>,
}

impl Editor {
    /// 新しいエディタを作成します。
    pub fn new(initial_text: String) -> Self {
        Self {
            lines: initial_text.lines().map(String::from).collect(),
            cursor: Cursor::new(0, 0),
            search_query: String::new(),
            search_matches: Vec::new(),
            current_search_idx: None,
            scroll_offset_visual_y: 0,
            scroll_offset_x: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// 指定されたパスからテキストを読み込み、エディタバッファを設定します。
    /// （App層によって、これが元ファイルか一時ファイルかが決定されます。）
    pub fn load_from_file(&mut self, path: &Path) -> io::Result<()> {
        let content = fs::read_to_string(path).unwrap_or_default();
        self.lines = content.lines().map(String::from).collect();
        // ファイルを読み込んだら、カーソルを先頭に設定し、選択をクリア
        self.set_cursor_position(0, 0, false);
        // 新しいファイルの内容なのでスクロールオフセットもリセット
        self.scroll_offset_visual_y = 0;
        self.scroll_offset_x = 0;
        Ok(())
    }

    /// エディタバッファの内容を指定されたパスに書き込みます。
    /// （App層によって、これが元ファイルか一時ファイルかが決定されます。）
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let content = self.lines.join("\n");
        fs::write(path, content)
    }

    /// カーソルを新しい論理位置に移動させます。
    /// バッファの境界を考慮して位置を調整し、その後Cursorの状態を更新します。
    /// `extend_selection`が`true`の場合、選択範囲を維持または開始します。
    pub fn set_cursor_position(&mut self, x: u16, y: u16, extend_selection: bool) {
        let num_lines = self.lines.len();

        let mut final_y = y;
        // Y座標をバッファの行数内にクランプ
        if num_lines == 0 {
            final_y = 0;
        } else {
            final_y = final_y.min(num_lines.saturating_sub(1) as u16);
        }

        let mut final_x = x;
        // X座標を現在の行の文字数内にクランプ
        if num_lines > 0 {
            let current_line_len = self.lines[final_y as usize].chars().count() as u16;
            // `u16::MAX`が渡された場合は行末に設定
            if x == u16::MAX {
                final_x = current_line_len;
            } else {
                final_x = final_x.min(current_line_len);
            }
        } else {
            final_x = 0;
        }

        // Cursorのupdate_positionメソッドを呼び出し、実際のカーソル位置を更新
        self.cursor
            .update_position(final_x, final_y, extend_selection);

        // カーソル位置の更新後、ビューポートのスクロールオフセットを調整
        // ただし、このメソッドは描画領域のサイズを知らないため、調整は別途 `adjust_viewport_offset` で行う
    }

    /// 描画領域のサイズに基づいてスクロールオフセットを調整し、カーソルが見えるようにします。
    ///
    /// `word_wrap_enabled` の状態を考慮します。
    pub fn adjust_viewport_offset(&mut self, viewport_area: Rect, word_wrap_enabled: bool) {
        let viewport_height = viewport_area.height;
        let viewport_width = viewport_area.width;

        let (cursor_visual_x, cursor_visual_y) =
            self.logical_to_visual_pos(self.cursor.get_current_pos(), viewport_width, word_wrap_enabled);

        // 垂直スクロール (Y軸 - 視覚行)
        if cursor_visual_y < self.scroll_offset_visual_y {
            self.scroll_offset_visual_y = cursor_visual_y;
        }
        if cursor_visual_y >= self.scroll_offset_visual_y + viewport_height {
            self.scroll_offset_visual_y = cursor_visual_y - viewport_height + 1;
        }

        // 水平スクロール (X軸 - 視覚列)
        if word_wrap_enabled {
            self.scroll_offset_x = 0;
        } else {
            if cursor_visual_x < self.scroll_offset_x {
                self.scroll_offset_x = cursor_visual_x;
            }
            if cursor_visual_x >= self.scroll_offset_x + viewport_width {
                self.scroll_offset_x = cursor_visual_x - viewport_width + 1;
            }
        }
    }

    /// カーソルを次の視覚行に移動します。
    pub fn next_visual_line(&mut self, viewport_width: u16, word_wrap_enabled: bool, extend_selection: bool) {
        if !word_wrap_enabled {
            self.next_line(extend_selection);
            return;
        }

        let (current_visual_x, current_visual_y) =
            self.logical_to_visual_pos(self.cursor.get_current_pos(), viewport_width, true);

        let (new_logical_x, new_logical_y) =
            self.visual_to_logical_pos((current_visual_x, current_visual_y + 1), viewport_width);

        self.set_cursor_position(new_logical_x, new_logical_y, extend_selection);
    }

    /// カーソルを前の視覚行に移動します。
    pub fn previous_visual_line(&mut self, viewport_width: u16, word_wrap_enabled: bool, extend_selection: bool) {
        if !word_wrap_enabled {
            self.previous_line(extend_selection);
            return;
        }

        let (current_visual_x, current_visual_y) =
            self.logical_to_visual_pos(self.cursor.get_current_pos(), viewport_width, true);

        if current_visual_y == 0 { return; }

        let (new_logical_x, new_logical_y) =
            self.visual_to_logical_pos((current_visual_x, current_visual_y.saturating_sub(1)), viewport_width);

        self.set_cursor_position(new_logical_x, new_logical_y, extend_selection);
    }

    /// カーソルを次の行に移動します。
    pub fn next_line(&mut self, extend_selection: bool) {
        let potential_y = self.cursor.get_potential_next_line_y();
        let current_x = self.cursor.x; // 現在のX座標を維持しようとする
        self.set_cursor_position(current_x, potential_y, extend_selection);
    }

    /// カーソルを前の行に移動します。
    pub fn previous_line(&mut self, extend_selection: bool) {
        let potential_y = self.cursor.get_potential_previous_line_y();
        let current_x = self.cursor.x; // 現在のX座標を維持しようとする
        self.set_cursor_position(current_x, potential_y, extend_selection);
    }

    /// カーソルを次の文字に移動します。
    pub fn next_char(&mut self, extend_selection: bool) {
        let current_y = self.cursor.y;
        let current_x = self.cursor.x;

        if (current_y as usize) < self.lines.len() {
            let current_line_len = self.lines[current_y as usize].chars().count() as u16;
            if current_x < current_line_len {
                // 現在の行内で次の文字へ
                self.set_cursor_position(current_x.saturating_add(1), current_y, extend_selection);
            } else if (current_y as usize + 1) < self.lines.len() {
                // 次の行が存在する場合
                // 行末にいる場合は次の行の先頭へ
                self.set_cursor_position(0, current_y.saturating_add(1), extend_selection);
            } else {
                // バッファの最後の行の末尾にいる場合は何もしない
            }
        } else {
            // バッファが空または最後の行の末尾（カーソルがその行を超えている）にいる場合は何もしない
            self.set_cursor_position(current_x, current_y, extend_selection); // 現在の位置を再設定（実質何もしない）
        }
    }

    /// カーソルを前の文字に移動します。
    pub fn previous_char(&mut self, extend_selection: bool) {
        let current_y = self.cursor.y;
        let current_x = self.cursor.x;

        if current_x > 0 {
            // 現在の行内で前の文字へ
            self.set_cursor_position(current_x.saturating_sub(1), current_y, extend_selection);
        } else if current_y > 0 {
            // 行頭にいる場合は前の行の末尾へ
            let previous_line_y = current_y.saturating_sub(1);
            // 前の行の実際の長さを取得し、X座標を設定（`u16::MAX`でEditorに「行末」を伝える）
            self.set_cursor_position(u16::MAX, previous_line_y, extend_selection);
        } else {
            // バッファが空または最初の行の先頭にいる場合は何もしない
        }
    }

    /// カーソルを現在の行の先頭に移動します。
    pub fn move_cursor_to_line_start(&mut self, extend_selection: bool) {
        let current_y = self.cursor.y;
        self.set_cursor_position(
            self.cursor.get_potential_start_of_line_x(),
            current_y,
            extend_selection,
        );
    }

    /// カーソルを現在の行の末尾に移動します。
    pub fn move_cursor_to_line_end(&mut self, extend_selection: bool) {
        let current_y = self.cursor.y;
        // u16::MAX を渡して、set_cursor_position に行末を計算させる
        self.set_cursor_position(
            self.cursor.get_potential_end_of_line_x(),
            current_y,
            extend_selection,
        );
    }

    /// カーソルをドキュメントの先頭に移動します。
    pub fn move_cursor_to_document_start(&mut self, extend_selection: bool) {
        let (potential_y, potential_x) = self.cursor.get_potential_document_start_pos();
        self.set_cursor_position(potential_x, potential_y, extend_selection);
    }

    /// カーソルをドキュメントの末尾に移動します。
    pub fn move_cursor_to_document_end(&mut self, extend_selection: bool) {
        let (potential_y, potential_x) = self.cursor.get_potential_document_end_pos();
        // u16::MAX を渡して、set_cursor_position にドキュメントの末尾を計算させる
        self.set_cursor_position(potential_x, potential_y, extend_selection);
    }

    /// 現在のカーソル位置から選択範囲を取得します。
    /// 戻り値は (開始バイトオフセット, 終了バイトオフセット) のタプルです。
    /// 選択範囲がない場合はNoneを返します。
    pub fn get_selection_range(&self) -> Option<(usize, usize)> {
        if let Some((start_coords, end_coords)) = self.cursor.get_normalized_selection_coords() {
            // (y, x) 座標をバイトオフセットに変換するヘルパー関数
            let coords_to_byte_offset = |y_coord: u16, x_coord: u16| -> usize {
                let mut offset = 0;
                for (i, line) in self.lines.iter().enumerate() {
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
            let buffer_as_string = self.lines.join("\n");
            let text = buffer_as_string[start..end].to_string();
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
            let buffer_as_string = self.lines.join("\n");
            let cut_text = buffer_as_string[start_byte_offset..end_byte_offset].to_string();

            // OSクリップボードに書き込む（失敗しても無視）
            if let Ok(mut clipboard) = Clipboard::new() {
                let _ = clipboard.set_text(cut_text.clone());
            }

            // 切り取り後のカーソル位置を、選択範囲の開始位置に調整する
            // バイトオフセットから新しいカーソル座標を計算
            let (start_y, start_x) = self.cursor.get_normalized_selection_coords().unwrap().0;
            let (new_x, new_y) = (start_x, start_y);

            // 選択範囲を削除
            let mut temp_buffer = buffer_as_string;
            temp_buffer.replace_range(start_byte_offset..end_byte_offset, "");
            self.lines = temp_buffer.lines().map(String::from).collect();
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
        let mut buffer_as_string = self.lines.join("\n");
        buffer_as_string.insert_str(current_offset, text);
        self.lines = buffer_as_string.lines().map(String::from).collect();
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

        let (x, y) = (self.cursor.x as usize, self.cursor.y as usize);
        if c == '\n' {
            let current_line = &mut self.lines[y];
            let rest_of_line = current_line.split_off(x);
            self.lines.insert(y + 1, rest_of_line);
            self.set_cursor_position(0, self.cursor.y.saturating_add(1), false);
        } else {
            if y >= self.lines.len() {
                self.lines.push(String::new());
            }
            self.lines[y].insert(x, c);
            self.set_cursor_position(self.cursor.x.saturating_add(1), self.cursor.y, false);
        }
    }

    /// カーソル位置の文字、または選択範囲を削除します。（Backspace相当）
    pub fn delete_previous_char(&mut self) {
        if self.cursor.is_selecting() {
            self.cut_selection();
            return;
        }

        let (x, y) = (self.cursor.x as usize, self.cursor.y as usize);

        if x > 0 {
            self.lines[y].remove(x - 1);
            self.set_cursor_position(self.cursor.x - 1, self.cursor.y, false);
        } else if y > 0 {
            // 行頭でバックスペース、前の行と結合
            let current_line = self.lines.remove(y);
            let prev_line_len = self.lines[y - 1].len() as u16;
            self.lines[y - 1].push_str(&current_line);
            self.set_cursor_position(prev_line_len, self.cursor.y - 1, false);
        }
    }

    /// カーソル位置の文字、または選択範囲を削除します。（Deleteキー相当）
    pub fn delete_current_char(&mut self) {
        if self.cursor.is_selecting() {
            self.cut_selection();
            return;
        }

        let (x, y) = (self.cursor.x as usize, self.cursor.y as usize);

        if y < self.lines.len() && x < self.lines[y].len() {
            self.lines[y].remove(x);
        } else if y + 1 < self.lines.len() {
            // 行末でデリート、次の行と結合
            let next_line = self.lines.remove(y + 1);
            self.lines[y].push_str(&next_line);
        }
    }

    /// 指定された範囲のテキストを新しいテキストで置き換えます。
    pub fn replace_buffer_range(
        &mut self,
        start_byte_offset: usize,
        end_byte_offset: usize,
        new_text: &str,
    ) {
        let mut buffer_as_string = self.lines.join("\n");
        if start_byte_offset <= end_byte_offset && end_byte_offset <= buffer_as_string.len() {
            buffer_as_string.replace_range(start_byte_offset..end_byte_offset, new_text);
            self.lines = buffer_as_string.lines().map(String::from).collect();

            let new_cursor_offset = start_byte_offset + new_text.len(); // 置換後の新しいカーソル位置
            self.set_cursor_from_byte_offset(new_cursor_offset, false); // カーソル位置を更新し、選択をクリア
        }
    }

    /// 現在のカーソル位置をバイトオフセットに変換します。
    fn get_cursor_byte_offset(&self) -> usize {
        let mut offset = 0;
        for (current_y, line) in self.lines.iter().enumerate() {
            if current_y == self.cursor.y as usize {
                // 現在の行のカーソルX位置までのバイト数を計算
                offset += line
                    .chars()
                    .take(self.cursor.x as usize)
                    .map(|c| c.len_utf8())
                    .sum::<usize>();
                break;
            }
            offset += line.len(); // 行のバイト長
            offset += 1; // 改行文字 (LF) のバイト長を仮定
        }
        offset
    }

    /// バイトオフセットからカーソル位置 (x, y) を設定します。
    fn set_cursor_from_byte_offset(&mut self, byte_offset: usize, extend_selection: bool) {
        let mut current_offset = 0;
        let mut y = 0;
        let mut x_chars_count = 0;

        let buffer_as_string = self.lines.join("\n");
        for line in buffer_as_string.lines() {
            let line_len_bytes = line.len();
            // 改行文字を含む行の全長。Rustのlines()は改行を含まないので、ここで+1する
            let line_with_newline_len = line_len_bytes + 1; // LFを仮定

            // バイトオフセットが現在の行内にあるかチェック
            if byte_offset >= current_offset && byte_offset <= current_offset + line_len_bytes {
                let relative_offset = byte_offset - current_offset;

                x_chars_count = 0;
                let mut current_byte_in_line = 0;
                for c in line.chars() {
                    let char_len_bytes = c.len_utf8();
                    if current_byte_in_line + char_len_bytes > relative_offset {
                        // オフセットが現在の文字の途中にあれば、その文字の先頭にカーソルを置く
                        break;
                    }
                    current_byte_in_line += char_len_bytes;
                    x_chars_count += 1;
                }

                // 行の文字数を超えないように調整
                x_chars_count = x_chars_count.min(line.chars().count());
                break;
            }

            current_offset += line_with_newline_len;
            y += 1;
        }
        // set_cursor_positionを通じて、Cursorのupdate_positionを呼び出す
        self.set_cursor_position(x_chars_count as u16, y as u16, extend_selection);
    }

    /// カーソル位置の括弧に対応する括弧の位置を検索します。
    /// 戻り値は (y, x) のタプルです。
    pub fn find_matching_paren(&self) -> Option<(u16, u16)> {
        let current_y = self.cursor.y as usize;
        let current_x = self.cursor.x as usize;

        if current_y >= self.lines.len() {
            return None;
        }

        let current_line_chars: Vec<char> = self.lines[current_y].chars().collect();
        // カーソルが現在の行の文字数の境界にいる場合も考慮
        if current_x > current_line_chars.len() {
            return None;
        }

        // カーソルが文字の間にいる場合は、その前の文字をチェックする（カーソルがその文字の右側にあるとみなす）
        let char_at_cursor_or_before = if current_x < current_line_chars.len() {
            current_line_chars[current_x]
        } else if current_x > 0 {
            current_line_chars[current_x - 1] // 行末の場合は前の文字を見る
        } else {
            return None; // 空の行の先頭
        };

        let (open_paren, close_paren, direction) = match char_at_cursor_or_before {
            '(' => ('(', ')', 1), // 順方向検索
            '{' => ('{', '}', 1),
            '[' => ('[', ']', 1),
            ')' => ('(', ')', -1), // 逆方向検索
            '}' => ('{', '}', -1),
            ']' => ('[', ']', -1),
            _ => return None, // 括弧ではない
        };

        let mut balance = 0;

        if direction == 1 {
            // 順方向検索 (カーソル位置から終端まで)
            // 現在の行
            for (x_idx, ch) in current_line_chars.iter().enumerate().skip(current_x) {
                if *ch == open_paren {
                    balance += 1;
                } else if *ch == close_paren {
                    balance -= 1;
                }
                if balance == 0 {
                    return Some((current_y as u16, x_idx as u16));
                }
            }
            // 後続の行
            for (y_idx, _item) in self.lines.iter().enumerate().skip(current_y + 1) {
                let line_chars: Vec<char> = self.lines[y_idx].chars().collect();
                for (x_idx, ch) in line_chars.iter().enumerate() {
                    if *ch == open_paren {
                        balance += 1;
                    } else if *ch == close_paren {
                        balance -= 1;
                    }
                    if balance == 0 {
                        return Some((y_idx as u16, x_idx as u16));
                    }
                }
            }
        } else {
            // 逆方向検索 (カーソル位置から先頭まで)
            // 現在の行 (カーソル位置の1つ前から逆順に走査)
            let start_x_for_backward_search = if current_x > 0 {
                current_x.saturating_sub(1)
            } else {
                0
            };

            for x_idx in (0..=start_x_for_backward_search).rev() {
                let ch = current_line_chars[x_idx];
                if ch == close_paren {
                    balance += 1;
                } else if ch == open_paren {
                    balance -= 1;
                }
                if balance == 0 {
                    return Some((current_y as u16, x_idx as u16));
                }
            }
            // 前の行
            for y_idx in (0..current_y).rev() {
                let line_chars: Vec<char> = self.lines[y_idx].chars().collect();
                for x_idx in (0..line_chars.len()).rev() {
                    let ch = line_chars[x_idx];
                    if ch == close_paren {
                        balance += 1;
                    } else if ch == open_paren {
                        balance -= 1;
                    }
                    if balance == 0 {
                        return Some((y_idx as u16, x_idx as u16));
                    }
                }
            }
        }
        None
    }

    /// 現在のカーソル位置からコード補完の候補を取得します。（非常に簡易版）
    /// 実際の補完は、言語サーバープロトコル (LSP) などで行われるのが一般的です。
    pub fn get_completion_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        let current_y = self.cursor.y as usize;

        if current_y >= self.lines.len() {
            return suggestions;
        }

        let current_line_chars: Vec<char> = self.lines[current_y].chars().collect();
        let current_x = self.cursor.x as usize;

        // カーソル前の単語を取得
        let mut prefix = String::new();
        // カーソルの1つ前から逆順に走査
        for i in (0..current_x).rev() {
            let ch = current_line_chars[i];
            // 単語を構成する文字（英数字とアンダースコア）を定義
            if ch.is_alphanumeric() || ch == '_' {
                prefix.insert(0, ch); // 正しい順序で単語を構築するため先頭に挿入
            } else {
                break; // 単語以外の文字に遭遇したら停止
            }
        }

        if prefix.is_empty() {
            return suggestions;
        }

        // 仮のキーワードリスト (Rust風)
        let keywords = vec![
            "fn", "let", "if", "else", "while", "for", "match", "loop", "struct", "enum", "use",
            "mod", "pub", "mut", "return", "break", "continue", "impl", "trait", "where", "async",
            "await", "unsafe",
        ];

        // 仮の識別子リスト (バッファ内の単語から取得)
        let mut identifiers: std::collections::HashSet<String> = std::collections::HashSet::new();
        for line_str in self.lines.iter() {
            // 英数字とアンダースコア以外の文字で単語を分割
            for word in line_str.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
                if !word.is_empty() && !keywords.contains(&word) {
                    identifiers.insert(word.to_string());
                }
            }
        }

        // プレフィックスにマッチするキーワードを追加
        for keyword in keywords {
            if keyword.starts_with(&prefix) {
                suggestions.push(keyword.to_string());
            }
        }

        // プレフィックスにマッチする識別子を追加
        for id in identifiers.iter() {
            if id.starts_with(&prefix) {
                suggestions.push(id.clone());
            }
        }

        suggestions.sort_unstable(); // 候補をソート
        suggestions.dedup(); // 重複を削除
        suggestions
    }

    /// 指定されたクエリでバッファを検索し、マッチ位置を保存します。
    pub fn search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.search_matches.clear();
        self.current_search_idx = None;

        if query.is_empty() {
            return;
        }

        // 全ての行を走査し、クエリにマッチする位置を収集
        for (y, line) in self.lines.iter().enumerate() {
            // `match_indices` はバイトオフセットを返すため、文字オフセットに変換が必要
            for (byte_x, _) in line.match_indices(query) {
                // バイトオフセットから文字オフセットに変換
                let char_x = line[..byte_x].chars().count();
                self.search_matches.push((y as u16, char_x as u16));
            }
        }

        if !self.search_matches.is_empty() {
            self.current_search_idx = Some(0);
            let (y, x) = self.search_matches[0];
            self.set_cursor_position(x, y, false); // 最初のマッチ位置にカーソルを移動（選択はクリア）
        }
    }

    /// 次の検索結果に移動します。
    pub fn next_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let current_cursor_pos = self.cursor.get_current_pos(); // (x, y)
        let mut next_idx_found = false;
        let mut closest_next_idx = 0; // 現在位置より後のマッチがない場合のフォールバック（最初のマッチ）

        // 現在のカーソル位置より「後」にあるマッチを検索
        for (i, &(match_y, match_x)) in self.search_matches.iter().enumerate() {
            // Cursorの座標は (x, y) なので、比較を (y, x) に合わせて行う
            if (match_y > current_cursor_pos.1) || // 次の行
               (match_y == current_cursor_pos.1 && match_x >= current_cursor_pos.0)
            {
                // 同じ行でカーソル位置以降
                closest_next_idx = i;
                next_idx_found = true;
                break;
            }
        }

        // 現在位置より後に見つかった場合はその位置へ
        // 見つからなかった場合は、リストの先頭に戻る (ラップアラウンド)
        let final_idx = if next_idx_found { closest_next_idx } else { 0 };

        self.current_search_idx = Some(final_idx);
        let (y, x) = self.search_matches[final_idx];
        self.set_cursor_position(x, y, false); // 検索結果に移動（選択はクリア）
    }

    /// 前の検索結果に移動します。
    pub fn previous_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let current_cursor_pos = self.cursor.get_current_pos(); // (x, y)
        let mut prev_idx_found = false;
        let mut closest_prev_idx = self.search_matches.len().saturating_sub(1); // 現在位置より前のマッチがない場合のフォールバック（最後のマッチ）

        // 現在のカーソル位置より「前」にあるマッチを検索 (リストを逆順に走査)
        for (i, &(match_y, match_x)) in self.search_matches.iter().rev().enumerate() {
            let original_idx = self.search_matches.len() - 1 - i; // 元のインデックスに変換
            // Cursorの座標は (x, y) なので、比較を (y, x) に合わせて行う
            if (match_y < current_cursor_pos.1) || // 前の行
               (match_y == current_cursor_pos.1 && match_x <= current_cursor_pos.0)
            {
                // 同じ行でカーソル位置以前
                closest_prev_idx = original_idx;
                prev_idx_found = true;
                break;
            }
        }

        // 現在位置より前に見つからなかった場合は、リストの末尾に戻る (ラップアラウンド)
        let final_idx = if prev_idx_found {
            closest_prev_idx
        } else {
            self.search_matches.len().saturating_sub(1)
        };

        self.current_search_idx = Some(final_idx);
        let (y, x) = self.search_matches[final_idx];
        self.set_cursor_position(x, y, false); // 検索結果に移動（選択はクリア）
    }

    pub fn copy_selection_to_clipboard(&mut self) -> Option<String> {
        if let Some((start, end)) = self.get_selection_range() {
            let buffer_as_string = self.lines.join("\n");
            let text = buffer_as_string[start..end].to_string();
            let mut clipboard = Clipboard::new().ok();
            if let Some(ref mut cb) = clipboard {
                let _ = cb.set_text(text.clone());
            }
            Some(text)
        } else {
            None
        }
    }

    /// 編集操作の前に呼び出して履歴を積む
    fn push_undo(&mut self) {
        self.undo_stack.push(self.lines.clone());
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.lines.clone());
            self.lines = prev;
            // カーソル位置も復元したい場合は別途保存が必要
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.lines.clone());
            self.lines = next;
        }
    }

    /// 論理カーソル位置を視覚的な位置に変換します。
    pub fn logical_to_visual_pos(&self, logical_pos: (u16, u16), wrap_width: u16, word_wrap_enabled: bool) -> (u16, u16) {
        let (logical_x, logical_y) = logical_pos;

        if !word_wrap_enabled || wrap_width == 0 {
            let visual_x = self.lines.get(logical_y as usize).map_or(0, |line| {
                line.chars().take(logical_x as usize).collect::<String>().width() as u16
            });
            return (visual_x, logical_y);
        }

        let mut visual_y_offset = 0;
        for i in 0..logical_y as usize {
            visual_y_offset += self.get_visual_lines_for_logical_line(i, wrap_width).len() as u16;
        }

        let wrapped_lines = self.get_visual_lines_for_logical_line(logical_y as usize, wrap_width);
        let mut chars_seen = 0;
        for (wrap_idx, line_segment) in wrapped_lines.iter().enumerate() {
            let segment_len = line_segment.chars().count();
            if logical_x as usize <= chars_seen + segment_len {
                let x_in_segment = logical_x as usize - chars_seen;
                let visual_x = line_segment.chars().take(x_in_segment).collect::<String>().width() as u16;
                return (visual_x, visual_y_offset + wrap_idx as u16);
            }
            chars_seen += segment_len;
        }

        // カーソルが行末にある場合
        let last_segment_width = wrapped_lines.last().map_or(0, |s| s.width() as u16);
        (last_segment_width, visual_y_offset + wrapped_lines.len().saturating_sub(1) as u16)
    }

    /// 視覚的な位置を論理カーソル位置に変換します。
    pub fn visual_to_logical_pos(&self, visual_pos: (u16, u16), wrap_width: u16) -> (u16, u16) {
        let (visual_x, visual_y) = visual_pos;
        let mut current_visual_y: u16 = 0;

        for logical_y in 0..self.lines.len() {
            let wrapped_lines = self.get_visual_lines_for_logical_line(logical_y, wrap_width);
            if visual_y < current_visual_y + wrapped_lines.len() as u16 {
                let wrap_idx = (visual_y - current_visual_y) as usize;
                let line_segment = &wrapped_lines[wrap_idx];

                let mut logical_x_offset = 0;
                for i in 0..wrap_idx {
                    logical_x_offset += wrapped_lines[i].chars().count();
                }

                let mut current_visual_width = 0;
                for (char_idx, ch) in line_segment.chars().enumerate() {
                    let char_width = ch.width().unwrap_or(1) as u16;
                    if current_visual_width + char_width > visual_x {
                        return ((logical_x_offset + char_idx) as u16, logical_y as u16);
                    }
                    current_visual_width += char_width;
                }
                return ((logical_x_offset + line_segment.chars().count()) as u16, logical_y as u16);
            }
            current_visual_y += wrapped_lines.len() as u16;
        }
        (0, self.lines.len().saturating_sub(1) as u16)
    }

    /// 1つの論理行を折り返した結果の視覚行のリストを返します。
    pub fn get_visual_lines_for_logical_line(&self, line_idx: usize, wrap_width: u16) -> Vec<String> {
        let line = match self.lines.get(line_idx) {
            Some(l) => l,
            None => return vec![String::new()],
        };
        if wrap_width == 0 || line.width() <= wrap_width as usize {
            return vec![line.clone()];
        }
        textwrap::wrap(line, wrap_width as usize).into_iter().map(|s| s.into_owned()).collect()
    }
}
