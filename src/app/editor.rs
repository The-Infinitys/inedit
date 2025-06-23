// src/app/editor.rs

use super::cursor::Cursor;
use std::fs;
use std::io;
use std::path::Path;

/// テキストバッファとカーソルを管理し、編集操作を提供します。
pub struct Editor {
    pub buffer: String,
    pub cursor: Cursor,
    pub search_query: String,
    pub search_matches: Vec<(u16, u16)>,   // 検索結果の(y, x)位置
    pub current_search_idx: Option<usize>, // 現在の検索結果のインデックス
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            cursor: Cursor::default(),
            search_query: String::new(),
            search_matches: Vec::new(),
            current_search_idx: None,
        }
    }
}

impl Editor {
    /// 新しいエディタを作成します。
    pub fn new(initial_text: String) -> Self {
        let mut editor = Self {
            buffer: initial_text,
            cursor: Cursor::new(0, 0),
            search_query: String::new(),
            search_matches: Vec::new(),
            current_search_idx: None,
        };
        editor
    }

    /// ファイルからテキストを読み込み、エディタバッファを設定します。
    /// ファイル拡張子に基づいて言語モードを自動設定します。
    pub fn load_from_file(&mut self, path: &Path) -> io::Result<()> {
        let content = fs::read_to_string(path)?;
        self.buffer = content;
        self.set_cursor_position(0, 0);
        Ok(())
    }

    /// エディタバッファの内容をファイルに書き込みます。
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        fs::write(path, &self.buffer)
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
        self.set_cursor_position(self.cursor.x, self.cursor.y);
    }

    /// カーソルを前の行に移動します。
    pub fn previous_line(&mut self) {
        self.cursor.previous_line();
        self.set_cursor_position(self.cursor.x, self.cursor.y);
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
                if (current_line_idx + 1) < lines.len() {
                    self.cursor.next_line();
                }
            }
        }
        self.set_cursor_position(self.cursor.x, self.cursor.y);
    }

    /// カーソルを前の文字に移動します。
    pub fn previous_char(&mut self) {
        let lines: Vec<&str> = self.buffer.lines().collect();
        let current_line_idx = self.cursor.y as usize;

        if self.cursor.x > 0 {
            self.cursor.previous_char();
        } else {
            if current_line_idx > 0 {
                self.cursor.previous_line();
                let prev_line_len = lines[self.cursor.y as usize].chars().count() as u16;
                self.cursor.set_position(prev_line_len, self.cursor.y);
            }
        }
        self.set_cursor_position(self.cursor.x, self.cursor.y);
    }

    /// 現在のカーソル位置から選択範囲を取得します。
    /// 戻り値は (開始オフセット, 終了オフセット) のタプルで、バイト単位のオフセットです。
    /// 選択範囲がない場合はNoneを返します。
    pub fn get_selection_range(&self) -> Option<(usize, usize)> {
        if let Some(start_pos) = self.cursor.selection_start {
            let pos_to_offset = |x: u16, y: u16| -> usize {
                let mut offset = 0;
                for (i, line) in self.buffer.lines().enumerate() {
                    if i == y as usize {
                        offset += line
                            .chars()
                            .take(x as usize)
                            .map(|c| c.len_utf8())
                            .sum::<usize>();
                        break;
                    }
                    offset += line.len();
                    offset += 1; // Assume LF newline
                }
                offset
            };

            let current_offset = pos_to_offset(self.cursor.x, self.cursor.y);
            let start_offset = pos_to_offset(start_pos.0, start_pos.1);

            if current_offset == start_offset {
                None
            } else if current_offset > start_offset {
                Some((start_offset, current_offset))
            } else {
                Some((current_offset, start_offset))
            }
        } else {
            None
        }
    }

    /// バッファの内容全体を選択します。
    pub fn select_all(&mut self) {
        self.cursor.set_position(0, 0);
        self.cursor.start_selection();
        let lines: Vec<&str> = self.buffer.lines().collect();
        let last_line_idx = (lines.len() as u16).saturating_sub(1);
        let last_col_idx = if lines.is_empty() {
            0
        } else {
            lines[last_line_idx as usize].chars().count() as u16
        };
        self.cursor.set_position(last_col_idx, last_line_idx);
    }

    /// 選択された範囲のテキストをコピーします。
    pub fn copy_selection(&self) -> Option<String> {
        if let Some((start, end)) = self.get_selection_range() {
            Some(self.buffer[start..end].to_string())
        } else {
            None
        }
    }

    /// 選択された範囲のテキストを切り取り、バッファから削除します。
    pub fn cut_selection(&mut self) -> Option<String> {
        if let Some((start, end)) = self.get_selection_range() {
            let cut_text = self.buffer[start..end].to_string();
            self.buffer.replace_range(start..end, "");
            if let Some(sel_start_pos) = self.cursor.selection_start {
                self.set_cursor_position(sel_start_pos.0, sel_start_pos.1);
            } else {
                self.set_cursor_position(0, 0);
            }
            self.cursor.clear_selection();
            Some(cut_text)
        } else {
            None
        }
    }

    /// 指定されたテキストをカーソル位置にペーストします。
    pub fn paste_text(&mut self, text: &str) {
        if self.cursor.is_selecting() {
            self.cut_selection();
        }

        let current_offset = self.get_cursor_byte_offset();
        self.buffer.insert_str(current_offset, text);

        let new_cursor_offset = current_offset + text.len();
        self.set_cursor_from_byte_offset(new_cursor_offset);
        self.cursor.clear_selection();
    }

    /// カーソル位置に文字を挿入します。
    pub fn insert_char(&mut self, c: char) {
        if self.cursor.is_selecting() {
            self.cut_selection();
        }

        let current_offset = self.get_cursor_byte_offset();
        self.buffer.insert(current_offset, c);

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
            let mut char_boundary_offset = current_offset;
            while char_boundary_offset > 0
                && !self.buffer.is_char_boundary(char_boundary_offset - 1)
            {
                char_boundary_offset -= 1;
            }
            if char_boundary_offset > 0 {
                self.buffer.remove(char_boundary_offset - 1);
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
            let mut char_boundary_offset = current_offset;
            while char_boundary_offset < self.buffer.len()
                && !self.buffer.is_char_boundary(char_boundary_offset)
            {
                char_boundary_offset += 1;
            }
            if char_boundary_offset < self.buffer.len() {
                let next_char_boundary = self.buffer[char_boundary_offset..]
                    .chars()
                    .next()
                    .map_or(char_boundary_offset, |c| {
                        char_boundary_offset + c.len_utf8()
                    });

                self.buffer
                    .replace_range(char_boundary_offset..next_char_boundary, "");
            }
        }
        self.cursor.clear_selection();
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
            let new_cursor_offset = start_byte_offset + new_text.len();
            self.set_cursor_from_byte_offset(new_cursor_offset);
            self.cursor.clear_selection();
        }
    }

    /// 現在のカーソル位置をバイトオフセットに変換します。
    fn get_cursor_byte_offset(&self) -> usize {
        let mut offset = 0;
        let mut current_y = 0;
        for line in self.buffer.lines() {
            if current_y == self.cursor.y {
                offset += line
                    .chars()
                    .take(self.cursor.x as usize)
                    .map(|c| c.len_utf8())
                    .sum::<usize>();
                break;
            }
            offset += line.len();
            offset += 1;
            current_y += 1;
        }
        offset
    }

    /// バイトオフセットからカーソル位置 (x, y) を設定します。
    fn set_cursor_from_byte_offset(&mut self, byte_offset: usize) {
        let mut current_offset = 0;
        let mut y = 0;
        let mut x_chars_count = 0;

        for line in self.buffer.lines() {
            let line_len_bytes = line.len();
            let line_with_newline_len = line_len_bytes + 1;

            if byte_offset >= current_offset && byte_offset <= current_offset + line_len_bytes {
                let relative_offset = byte_offset - current_offset;

                x_chars_count = 0;
                let mut current_byte_in_line = 0;
                for c in line.chars() {
                    let char_len_bytes = c.len_utf8();
                    if current_byte_in_line + char_len_bytes > relative_offset {
                        break;
                    }
                    current_byte_in_line += char_len_bytes;
                    x_chars_count += 1;
                }

                x_chars_count = x_chars_count.min(line.chars().count());
                break;
            }

            current_offset += line_with_newline_len;
            y += 1;
        }
        self.cursor.set_position(x_chars_count as u16, y as u16);
    }

    /// カーソル位置の括弧に対応する括弧の位置を検索します。
    /// 戻り値は (y, x) のタプルです。
    pub fn find_matching_paren(&self) -> Option<(u16, u16)> {
        let lines: Vec<&str> = self.buffer.lines().collect();
        let current_y = self.cursor.y as usize;
        let current_x = self.cursor.x as usize;

        if current_y >= lines.len() {
            return None;
        }

        let current_line_chars: Vec<char> = lines[current_y].chars().collect();
        if current_x >= current_line_chars.len() {
            return None;
        }

        let char_at_cursor = current_line_chars[current_x];

        let (open_paren, close_paren, direction) = match char_at_cursor {
            '(' => ('(', ')', 1), // Forward search
            '{' => ('{', '}', 1),
            '[' => ('[', ']', 1),
            ')' => ('(', ')', -1), // Backward search
            '}' => ('{', '}', -1),
            ']' => ('[', ']', -1),
            _ => return None, // Not a paren
        };

        let mut balance = 0;

        if direction == 1 {
            // Forward search (from cursor to end)
            // Current line
            for x_idx in current_x..current_line_chars.len() {
                if current_line_chars[x_idx] == open_paren {
                    balance += 1;
                } else if current_line_chars[x_idx] == close_paren {
                    balance -= 1;
                }
                if balance == 0 {
                    return Some((current_y as u16, x_idx as u16));
                }
            }
            // Subsequent lines
            for y_idx in (current_y + 1)..lines.len() {
                let line_chars: Vec<char> = lines[y_idx].chars().collect();
                for x_idx in 0..line_chars.len() {
                    if line_chars[x_idx] == open_paren {
                        balance += 1;
                    } else if line_chars[x_idx] == close_paren {
                        balance -= 1;
                    }
                    if balance == 0 {
                        return Some((y_idx as u16, x_idx as u16));
                    }
                }
            }
        } else {
            // Backward search (from cursor to beginning)
            // Current line
            for x_idx in (0..=current_x).rev() {
                // Include current_x
                if current_line_chars[x_idx] == close_paren {
                    balance += 1;
                } else if current_line_chars[x_idx] == open_paren {
                    balance -= 1;
                }
                if balance == 0 {
                    return Some((current_y as u16, x_idx as u16));
                }
            }
            // Previous lines
            for y_idx in (0..current_y).rev() {
                let line_chars: Vec<char> = lines[y_idx].chars().collect();
                for x_idx in (0..line_chars.len()).rev() {
                    if line_chars[x_idx] == close_paren {
                        balance += 1;
                    } else if line_chars[x_idx] == open_paren {
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
        let lines: Vec<&str> = self.buffer.lines().collect();
        let current_y = self.cursor.y as usize;

        if current_y >= lines.len() {
            return suggestions;
        }

        let current_line_chars: Vec<char> = lines[current_y].chars().collect();
        let current_x = self.cursor.x as usize;

        // カーソル前の単語を取得
        let mut prefix = String::new();
        for i in (0..current_x).rev() {
            let ch = current_line_chars[i];
            if ch.is_alphanumeric() || ch == '_' {
                prefix.insert(0, ch);
            } else {
                break;
            }
        }

        if prefix.is_empty() {
            return suggestions;
        }

        // 仮のキーワードリスト (Rust風)
        let keywords = vec![
            "fn", "let", "if", "else", "while", "for", "match", "loop", "struct", "enum", "use",
            "mod", "pub", "mut", "return", "break", "continue",
        ];

        // 仮の識別子リスト (バッファ内の単語から取得)
        let mut identifiers: std::collections::HashSet<String> = std::collections::HashSet::new();
        for line_str in self.buffer.lines() {
            for word in line_str.split(|c: char| !c.is_alphanumeric() && c != '_') {
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

        suggestions.sort_unstable();
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

        for (y, line) in self.buffer.lines().enumerate() {
            for (x, _) in line.match_indices(query) {
                self.search_matches.push((y as u16, x as u16));
            }
        }

        if !self.search_matches.is_empty() {
            self.current_search_idx = Some(0);
            let (y, x) = self.search_matches[0];
            self.set_cursor_position(x, y);
        }
    }

    /// 次の検索結果に移動します。
    pub fn next_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let mut next_idx = match self.current_search_idx {
            Some(idx) => (idx + 1) % self.search_matches.len(),
            None => 0, // 最初の検索結果へ
        };

        // 現在のカーソル位置より後のマッチを探す
        let current_pos = (self.cursor.y, self.cursor.x);
        let mut found_next = false;
        for (i, &(match_y, match_x)) in self.search_matches.iter().enumerate() {
            if (match_y > current_pos.0) || (match_y == current_pos.0 && match_x > current_pos.1) {
                next_idx = i;
                found_next = true;
                break;
            }
        }

        // 現在位置より後に見つからなかった場合は、先頭に戻る
        if !found_next && !self.search_matches.is_empty() {
            next_idx = 0;
        }

        self.current_search_idx = Some(next_idx);
        let (y, x) = self.search_matches[next_idx];
        self.set_cursor_position(x, y);
    }

    /// 前の検索結果に移動します。
    pub fn previous_search_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let mut prev_idx = match self.current_search_idx {
            Some(idx) => {
                if idx == 0 {
                    self.search_matches.len() - 1
                } else {
                    idx - 1
                }
            }
            None => self.search_matches.len().saturating_sub(1), // 最後の検索結果へ
        };

        // 現在のカーソル位置より前のマッチを探す
        let current_pos = (self.cursor.y, self.cursor.x);
        let mut found_prev = false;
        for (i, &(match_y, match_x)) in self.search_matches.iter().rev().enumerate() {
            // 後ろから検索
            let original_idx = self.search_matches.len() - 1 - i;
            if (match_y < current_pos.0) || (match_y == current_pos.0 && match_x < current_pos.1) {
                prev_idx = original_idx;
                found_prev = true;
                break;
            }
        }

        // 現在位置より前に見つからなかった場合は、末尾に戻る
        if !found_prev && !self.search_matches.is_empty() {
            prev_idx = self.search_matches.len().saturating_sub(1);
        }

        self.current_search_idx = Some(prev_idx);
        let (y, x) = self.search_matches[prev_idx];
        self.set_cursor_position(x, y);
    }
}
