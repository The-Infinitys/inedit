use super::Editor;

impl Editor {
    /// 指定されたクエリでバッファを検索し、マッチ位置を保存します。
    pub fn search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.search_matches.clear();
        self.current_search_idx = None;

        if query.is_empty() {
            return;
        }

        // 全ての行を走査し、クエリにマッチする位置を収集
        for (y, line) in self.buffer.lines().enumerate() {
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
            if (match_y > current_cursor_pos.1)
                || (match_y == current_cursor_pos.1 && match_x >= current_cursor_pos.0)
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
            if (match_y < current_cursor_pos.1)
                || (match_y == current_cursor_pos.1 && match_x <= current_cursor_pos.0)
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
}