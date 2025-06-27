use super::Editor;

impl Editor {
    /// カーソルを新しい論理位置に移動させます。
    /// バッファの境界を考慮して位置を調整し、その後Cursorの状態を更新します。
    /// `extend_selection`が`true`の場合、選択範囲を維持または開始します。
    pub fn set_cursor_position(&mut self, x: u16, y: u16, extend_selection: bool) {
        let lines: Vec<&str> = self.buffer.lines().collect();
        let num_lines = lines.len();

        let mut final_y = y;
        // Y座標をバッファの行数内にクランプ
        if num_lines == 0 {
            final_y = 0;
        } else {
            final_y = final_y.min((num_lines - 1) as u16);
        }

        let mut final_x = x;
        // X座標を現在の行の文字数内にクランプ
        if num_lines > 0 {
            let current_line_len = lines[final_y as usize].chars().count() as u16;
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

    /// カーソルを次の行に移動します。
    pub fn next_line(&mut self, extend_selection: bool) {
        let num_lines = self.buffer.lines().count() as u16;
        let current_y = self.cursor.y;

        if current_y < num_lines.saturating_sub(1) {
            // 次の行が存在する場合
            let potential_y = self.cursor.get_potential_next_line_y();
            let current_x = self.cursor.x; // 現在のX座標を維持しようとする
            self.set_cursor_position(current_x, potential_y, extend_selection);
        } else {
            // 次の行がない（最終行にいる）かバッファが空の場合、ドキュメントの末尾に移動
            self.move_cursor_to_document_end(extend_selection);
        }
    }

    /// カーソルを前の行に移動します。
    pub fn previous_line(&mut self, extend_selection: bool) {
        let current_y = self.cursor.y;
        if current_y > 0 {
            // 前の行が存在する場合
            let potential_y = self.cursor.get_potential_previous_line_y();
            let current_x = self.cursor.x; // 現在のX座標を維持しようとする
            self.set_cursor_position(current_x, potential_y, extend_selection);
        } else {
            // 前の行がない（先頭行にいる）場合、ドキュメントの先頭に移動
            self.move_cursor_to_document_start(extend_selection);
        }
    }

    /// カーソルを次の文字に移動します。
    pub fn next_char(&mut self, extend_selection: bool) {
        let lines: Vec<&str> = self.buffer.lines().collect();
        let current_y = self.cursor.y;
        let current_x = self.cursor.x;

        if (current_y as usize) < lines.len() {
            let current_line_len = lines[current_y as usize].chars().count() as u16;
            if current_x < current_line_len {
                // 現在の行内で次の文字へ
                self.set_cursor_position(current_x.saturating_add(1), current_y, extend_selection);
            } else if (current_y as usize + 1) < lines.len() {
                // 次の行が存在する場合
                // 行末にいる場合は次の行の先頭へ
                self.set_cursor_position(0, current_y.saturating_add(1), extend_selection);
            } else {
                // バッファの最後の行の末尾にいる場合は何もしない
                self.set_cursor_position(current_x, current_y, extend_selection); // 現在の位置を再設定（実質何もしない）
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
            self.set_cursor_position(current_x, current_y, extend_selection); // 現在の位置を再設定（実質何もしない）
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

    /// 現在のカーソル位置をバイトオフセットに変換します。
    pub(super) fn get_cursor_byte_offset(&self) -> usize {
        let mut offset = 0;
        for (current_y, line) in self.buffer.lines().enumerate() {
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
    pub(super) fn set_cursor_from_byte_offset(&mut self, byte_offset: usize, extend_selection: bool) {
        let mut current_offset = 0;
        let mut y = 0;
        let mut x_chars_count = 0;

        for line in self.buffer.lines() {
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
        let lines: Vec<&str> = self.buffer.lines().collect();
        let current_y = self.cursor.y as usize;
        let current_x = self.cursor.x as usize;

        if current_y >= lines.len() {
            return None;
        }

        let current_line_chars: Vec<char> = lines[current_y].chars().collect();
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
            for (y_idx, _item) in lines.iter().enumerate().skip(current_y + 1) {
                let line_chars: Vec<char> = lines[y_idx].chars().collect();
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
                let line_chars: Vec<char> = lines[y_idx].chars().collect();
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

    /// 折返しモード対応: wrap行単位でカーソルを上下移動
    pub fn next_visual_line(&mut self, area_width: usize, extend_selection: bool) {
        let visual_lines = self.get_visual_lines_with_width(area_width);
        let mut found = None;
        for (i, (buf_idx, wrap_idx, _)) in visual_lines.iter().enumerate() {
            if *buf_idx == self.cursor.y as usize && *wrap_idx == self.cursor_wrap_idx {
                found = Some(i);
                break;
            }
        }
        if let Some(i) = found {
            if i + 1 < visual_lines.len() {
                let (next_buf_idx, next_wrap_idx, next_line) = &visual_lines[i + 1];
                let x = self.cursor.x.min(next_line.chars().count() as u16);
                self.cursor.x = x;
                self.cursor.y = *next_buf_idx as u16;
                self.cursor_wrap_idx = *next_wrap_idx;
                self.cursor
                    .update_position(x, *next_buf_idx as u16, extend_selection);
            }
        }
    }
    pub fn previous_visual_line(&mut self, area_width: usize, extend_selection: bool) {
        let visual_lines = self.get_visual_lines_with_width(area_width);
        let mut found = None;
        for (i, (buf_idx, wrap_idx, _)) in visual_lines.iter().enumerate() {
            if *buf_idx == self.cursor.y as usize && *wrap_idx == self.cursor_wrap_idx {
                found = Some(i);
                break;
            }
        }
        if let Some(i) = found {
            if i > 0 {
                let (prev_buf_idx, prev_wrap_idx, prev_line) = &visual_lines[i - 1];
                let x = self.cursor.x.min(prev_line.chars().count() as u16);
                self.cursor.x = x;
                self.cursor.y = *prev_buf_idx as u16;
                self.cursor_wrap_idx = *prev_wrap_idx;
                self.cursor
                    .update_position(x, *prev_buf_idx as u16, extend_selection);
            }
        }
    }

    /// 事前に計算されたvisual_linesを使って、カーソルのビジュアル位置を返します。
    /// これにより、visual_linesの再計算を防ぎ、パフォーマンスを向上させます。
    pub fn get_cursor_visual_position_from_lines(
        &self, visual_lines: &[(usize, usize, String)]
    ) -> (usize, usize) {
        let logical_line_str = self
            .buffer
            .lines()
            .nth(self.cursor.y as usize)
            .unwrap_or("");

        // 3. カーソル位置までの論理的な部分文字列を取得し、そのビジュアル幅を計算
        let prefix_logical = logical_line_str
            .chars()
            .take(self.cursor.x as usize)
            .collect::<String>();
        let prefix_visual_width = prefix_logical.replace('\t', "    ").chars().count();

        // 4. 論理行全体のインデント幅を計算（タブ置換後）
        let indent_visual_width = logical_line_str
            .replace('\t', "    ")
            .chars()
            .take_while(|c| c.is_whitespace())
            .count();

        // 5. カーソルのある論理行に対応するビジュアル行を走査
        let mut cumulative_content_width = 0;
        for (visual_line_idx, (buf_idx, wrap_idx, v_line_str)) in visual_lines.iter().enumerate() {
            if *buf_idx != self.cursor.y as usize {
                continue;
            }

            // このビジュアル行のコンテンツ部分の幅を計算
            let content_width = if *wrap_idx == 0 {
                v_line_str.chars().count()
            } else {
                v_line_str.chars().count().saturating_sub(indent_visual_width)
            };

            if prefix_visual_width <= cumulative_content_width + content_width {
                // このビジュアル行にカーソルがある
                let relative_x = prefix_visual_width - cumulative_content_width;
                let visual_x = if *wrap_idx == 0 { relative_x } else { relative_x + indent_visual_width };
                return (visual_line_idx, visual_x);
            }
            cumulative_content_width += content_width;
        }

        // フォールバック: カーソルが論理行の末尾にある場合、最後のビジュアル行の末尾に配置
        if let Some((last_v_line_idx, (_, _, last_v_line_str))) = visual_lines
            .iter()
            .enumerate()
            .filter(|(_, (buf_idx, _, _))| *buf_idx == self.cursor.y as usize)
            .next_back()
        {
            if self.cursor.x as usize >= logical_line_str.chars().count() {
                return (last_v_line_idx, last_v_line_str.chars().count());
            }
        }

        (0, 0) // 最終フォールバック
    }

    /// 現在のカーソル位置がvisual_linesの何番目か、その中で何文字目かを返す（word wrap対応）
    /// 内部でvisual_linesを計算します。
    pub fn get_cursor_visual_position(&self, wrap_width: usize) -> (usize, usize) {
        // 1. ビジュアル行のリストを取得
        let visual_lines = self.get_visual_lines_with_width_word_wrap(wrap_width);
        // 2. 事前に計算したリストを使って位置を計算
        self.get_cursor_visual_position_from_lines(&visual_lines)
    }
}