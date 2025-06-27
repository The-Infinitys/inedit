use super::Editor;
use ratatui::layout::Rect;

impl Editor {
    /// 描画領域のサイズに基づいてスクロールオフセットを調整し、カーソルが見えるようにします。
    pub fn adjust_viewport_offset(&mut self, viewport_area: Rect, word_wrap_enabled: bool) {
        let viewport_height = viewport_area.height;
        let viewport_width = viewport_area.width;
        const PADDING_Y: u16 = 3; // 垂直方向のパディング

        if word_wrap_enabled {
            // --- Wrap Mode ON ---
            // 1. Get visual lines and cursor position to avoid re-calculation
            let visual_lines = self.get_visual_lines_with_width_word_wrap(viewport_width as usize);
            let (visual_cursor_y, _) = self.get_cursor_visual_position_from_lines(&visual_lines);
            let visual_cursor_y = visual_cursor_y as u16;

            // 2. Adjust vertical scroll based on visual position
            if visual_cursor_y < self.scroll_offset_y + PADDING_Y {
                self.scroll_offset_y = visual_cursor_y.saturating_sub(PADDING_Y);
            }
            if visual_cursor_y >= self.scroll_offset_y + viewport_height.saturating_sub(PADDING_Y) {
                self.scroll_offset_y = visual_cursor_y
                    .saturating_add(1)
                    .saturating_sub(viewport_height)
                    .saturating_add(PADDING_Y);
            }

            // 3. Clamp scroll offset with total visual lines
            let total_visual_lines = visual_lines.len() as u16;

            if total_visual_lines > viewport_height {
                self.scroll_offset_y = self
                    .scroll_offset_y
                    .min(total_visual_lines.saturating_sub(viewport_height));
            } else {
                self.scroll_offset_y = 0;
            }

            // In wrap mode, horizontal scroll is always 0.
            self.scroll_offset_x = 0;
        } else {
            // --- Wrap Mode OFF ---
            let cursor_y = self.cursor.y;
            let cursor_x_logical = self.cursor.x;
            const PADDING_X: u16 = 5; // 水平方向のパディング

            // Vertical scroll (Y-axis)
            if cursor_y < self.scroll_offset_y + PADDING_Y {
                self.scroll_offset_y = cursor_y.saturating_sub(PADDING_Y);
            }
            if cursor_y >= self.scroll_offset_y + viewport_height.saturating_sub(PADDING_Y) {
                self.scroll_offset_y = cursor_y
                    .saturating_add(1)
                    .saturating_sub(viewport_height)
                    .saturating_add(PADDING_Y);
            }

            // Horizontal scroll (X-axis)
            let lines: Vec<&str> = self.buffer.lines().collect();
            let current_line_content = if (cursor_y as usize) < lines.len() {
                lines[cursor_y as usize]
            } else {
                ""
            };

            // Calculate visual width considering tabs as 4 spaces
            let prefix_logical = current_line_content
                .chars()
                .take(cursor_x_logical as usize)
                .collect::<String>();
            let visual_cursor_x_on_line =
                prefix_logical.replace('\t', "    ").chars().count() as u16;
            let current_line_visual_width =
                current_line_content.replace('\t', "    ").chars().count() as u16;

            if visual_cursor_x_on_line < self.scroll_offset_x + PADDING_X {
                self.scroll_offset_x = visual_cursor_x_on_line.saturating_sub(PADDING_X);
            } else if visual_cursor_x_on_line
                >= self.scroll_offset_x + viewport_width.saturating_sub(PADDING_X)
            {
                self.scroll_offset_x = visual_cursor_x_on_line
                    .saturating_add(1)
                    .saturating_sub(viewport_width)
                    .saturating_add(PADDING_X);
            }

            // Clamp vertical scroll
            let total_lines = self.buffer.lines().count() as u16;
            if total_lines > viewport_height {
                self.scroll_offset_y = self
                    .scroll_offset_y
                    .min(total_lines.saturating_sub(viewport_height));
            } else {
                self.scroll_offset_y = 0;
            }

            // Clamp horizontal scroll
            if current_line_visual_width > viewport_width {
                self.scroll_offset_x = self
                    .scroll_offset_x
                    .min(current_line_visual_width.saturating_sub(viewport_width));
            } else {
                self.scroll_offset_x = 0;
            }
        }
    }

    /// 折返しモード用: ビューポート高さに合わせた画面上の行リストを返す
    /// 戻り値は (バッファ行番号, 折返しインデックス, 画面行文字列)
    // pub fn get_visual_lines(&self) -> Vec<(usize, usize, String)> {
    //     let mut result = Vec::new();
    //     let lines: Vec<&str> = self.buffer.lines().collect();
    //     // 仮: 1画面行=40文字で折返し（本来はエディタ幅を引数で受けるべき）
    //     let wrap_width = 40;
    //     for (buf_idx, line) in lines.iter().enumerate() {
    //         if line.is_empty() {
    //             result.push((buf_idx, 0, String::new()));
    //             continue;
    //         }
    //         let line_with_spaces = line.replace('\t', "    ");
    //         let mut start = 0;
    //         let mut wrap_idx = 0;
    //         let chars: Vec<char> = line_with_spaces.chars().collect();
    //         while start < chars.len() {
    //             let end = (start + wrap_width).min(chars.len());
    //             let visual = chars[start..end].iter().collect::<String>();
    //             result.push((buf_idx, wrap_idx, visual));
    //             start = end;
    //             wrap_idx += 1;
    //         }
    //     }
    //     result
    // }
    /// 指定幅でwrapしたvisual linesを返す（インデント保持）
    pub fn get_visual_lines_with_width(&self, wrap_width: usize) -> Vec<(usize, usize, String)> {
        let mut result = Vec::new();
        let lines: Vec<&str> = self.buffer.lines().collect();
        for (buf_idx, line) in lines.iter().enumerate() {
            if line.is_empty() {
                result.push((buf_idx, 0, String::new()));
                continue;
            }
            let line_with_spaces = line.replace('\t', "    ");
            // インデント部分を抽出
            let indent: String = line_with_spaces
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect();
            let mut start = 0;
            let mut wrap_idx = 0;
            let chars: Vec<char> = line_with_spaces.chars().collect();
            while start < chars.len() {
                let is_first = wrap_idx == 0;
                let available_width = if is_first || wrap_width == usize::MAX {
                    wrap_width
                } else {
                    wrap_width.saturating_sub(indent.chars().count())
                };
                let end = if available_width == 0 || available_width == usize::MAX {
                    chars.len()
                } else {
                    (start + available_width).min(chars.len())
                };
                let mut visual = chars[start..end].iter().collect::<String>();
                if !is_first && !indent.is_empty() {
                    visual = format!("{}{}", indent, visual);
                }
                result.push((buf_idx, wrap_idx, visual));
                if end == chars.len() {
                    break;
                }
                start = end;
                wrap_idx += 1;
            }
        }
        result
    }
    /// 指定幅でwrapしたvisual linesを返す（インデント保持、単語単位wrap）
    pub fn get_visual_lines_with_width_word_wrap(
        &self,
        wrap_width: usize,
    ) -> Vec<(usize, usize, String)> {
        let mut result = Vec::new();
        let lines: Vec<&str> = self.buffer.lines().collect();
        for (buf_idx, line) in lines.iter().enumerate() {
            if line.is_empty() {
                result.push((buf_idx, 0, String::new()));
                continue;
            }
            let line_with_spaces = line.replace('\t', "    ");
            // インデント部分を抽出
            let indent: String = line_with_spaces
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect();
            let mut wrap_idx = 0;
            let mut current = 0;
            let chars: Vec<char> = line_with_spaces.chars().collect();
            let mut first = true;
            while current < chars.len() {
                let available_width = if first || wrap_width == usize::MAX {
                    wrap_width
                } else {
                    wrap_width.saturating_sub(indent.chars().count())
                };
                if available_width == 0 || available_width == usize::MAX {
                    let visual = chars[current..].iter().collect::<String>();
                    result.push((
                        buf_idx,
                        wrap_idx,
                        if first {
                            visual.clone()
                        } else {
                            format!("{}{}", indent, visual)
                        },
                    ));
                    break;
                }
                // 単語単位でwrap
                let mut end = current + available_width;
                if end >= chars.len() {
                    end = chars.len();
                } else {
                    // 途中で単語が切れる場合、直前の空白まで戻す
                    let mut back = end;
                    while back > current && !chars[back - 1].is_whitespace() {
                        back -= 1;
                    }
                    if back > current {
                        end = back;
                    }
                }
                if end == current {
                    // 1単語がwrap幅を超える場合は強制分割
                    end = (current + wrap_width).min(chars.len());
                }
                let mut visual = chars[current..end].iter().collect::<String>();
                if !first && !indent.is_empty() {
                    visual = format!("{}{}", indent, visual);
                }
                result.push((buf_idx, wrap_idx, visual));
                if end == chars.len() {
                    break;
                }
                current = end;
                wrap_idx += 1;
                first = false;
            }
        }
        result
    }

    /// 指定したvisual lineのグローバルバイトオフセットを返す（word wrap対応）
    pub fn get_visual_line_global_offset(
        &self,
        buf_idx: usize,
        wrap_idx: usize,
        wrap_width: usize,
    ) -> usize {
        let lines: Vec<&str> = self.buffer.lines().collect();
        if buf_idx >= lines.len() {
            return 0;
        }
        let mut offset = 0;
        // buf_idxまでの全行のバイト数+改行
        for line in lines.iter().take(buf_idx) {
            offset += line.len();
            offset += 1; // 改行
        }
        // wrap_idx分だけこの行の先頭からバイト数を加算
        let line = lines[buf_idx];
        let chars: Vec<char> = line.chars().collect();
        let mut current = 0;
        let mut widx = 0;
        let indent_len = line.chars().take_while(|c| c.is_whitespace()).count();
        while widx < wrap_idx && current < chars.len() {
            let available_width = if widx == 0 || wrap_width == usize::MAX {
                wrap_width
            } else {
                wrap_width.saturating_sub(indent_len)
            };
            let mut end = current + available_width;
            if end >= chars.len() {
                end = chars.len();
            } else {
                let mut back = end;
                while back > current && !chars[back - 1].is_whitespace() {
                    back -= 1;
                }
                if back > current {
                    end = back;
                }
            }
            if end == current {
                end = (current + available_width).min(chars.len());
            }
            for c in &chars[current..end] {
                offset += c.len_utf8();
            }
            current = end;
            widx += 1;
        }
        offset
    }
}
