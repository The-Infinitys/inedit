// src/app/editor.rs

use super::cursor::Cursor;
use ratatui::layout::Rect;
use std::fs;
use std::io;
use std::path::Path;
// use unicode_width::UnicodeWidthChar; // unicode-width は使用しません
use ratatui::text::Line as RatatuiLine; // ratatui::text::Line をエイリアスでインポート

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

    /// 描画領域のサイズに基づいてスクロールオフセットを調整し、カーソルが見えるようにします。
    ///
    /// **重要:** このメソッドは`scroll_offset_y`と`scroll_offset_x`を設定します。
    /// 実際の描画を行う際は、`scroll_offset_y`から始まり、`scroll_offset_y + viewport_area.height`までの行を描画するのではなく、
    /// 必ず `self.buffer.lines().count()`（バッファの実際の行数）を超えないようにしてください。
    /// 例えば、`for i in self.scroll_offset_y .. min(self.scroll_offset_y + viewport_area.height, self.buffer.lines().count() as u16)`
    /// のようにループの終端を制限することで、存在しない行が表示されるのを防ぐことができます。
    pub fn adjust_viewport_offset(&mut self, viewport_area: Rect) {
        let cursor_y = self.cursor.y;
        let cursor_x_logical = self.cursor.x; // 論理的な文字インデックス
        let viewport_height = viewport_area.height;
        let viewport_width = viewport_area.width;

        const PADDING_Y: u16 = 3; // 垂直方向のパディング
        const PADDING_X: u16 = 5; // 水平方向のパディング

        // 垂直スクロール (Y軸)
        // カーソルが上端に近づいた場合
        if cursor_y < self.scroll_offset_y + PADDING_Y {
            self.scroll_offset_y = cursor_y.saturating_sub(PADDING_Y);
        }
        // カーソルが下端に近づいた場合
        if cursor_y >= self.scroll_offset_y + viewport_height.saturating_sub(PADDING_Y) {
            self.scroll_offset_y = cursor_y
                .saturating_add(1)
                .saturating_sub(viewport_height)
                .saturating_add(PADDING_Y);
        }

        // 水平スクロール (X軸) - 行の長さとUnicode幅も考慮
        let lines: Vec<&str> = self.buffer.lines().collect();
        let current_line_content = if (cursor_y as usize) < lines.len() {
            lines[cursor_y as usize]
        } else {
            ""
        };

        // カーソルの論理X位置 (cursor_x_logical) までの部分文字列の視覚的な幅（セル数）を計算
        let visual_cursor_x_on_line = RatatuiLine::from(
            current_line_content
                .chars()
                .take(cursor_x_logical as usize)
                .collect::<String>(),
        )
        .width() as u16;

        // 現在の行の全幅も計算（スクロール範囲の調整用）
        let current_line_visual_width = RatatuiLine::from(current_line_content).width() as u16;


        if visual_cursor_x_on_line < self.scroll_offset_x + PADDING_X {
            // カーソルがビューポートの左端より左に移動した場合
            self.scroll_offset_x = visual_cursor_x_on_line.saturating_sub(PADDING_X);
        } else if visual_cursor_x_on_line >= self.scroll_offset_x + viewport_width.saturating_sub(PADDING_X) {
            // カーソルがビューポートの右端より右に移動した場合
            // カーソル自体を含めるため、少なくとも1セル分動かすことを考慮（正確な幅はParagraphが計算する）
            self.scroll_offset_x = visual_cursor_x_on_line
                .saturating_add(1)
                .saturating_sub(viewport_width)
                .saturating_add(PADDING_X);
        }

        // スクロールオフセットがマイナスにならないように、またバッファの範囲を超えないように調整
        let total_lines = self.buffer.lines().count() as u16;
        if total_lines > viewport_height {
            self.scroll_offset_y = self
                .scroll_offset_y
                .min(total_lines.saturating_sub(viewport_height));
        } else {
            self.scroll_offset_y = 0; // コンテンツがビューポートより短い場合、垂直スクロールは不要
        }

        if current_line_visual_width > viewport_width {
            self.scroll_offset_x = self
                .scroll_offset_x
                .min(current_line_visual_width.saturating_sub(viewport_width));
        } else {
            self.scroll_offset_x = 0; // 現在の行がビューポートより短い場合、水平スクロールは不要
        }

        // スクロールオフセットは常に0以上であることを保証
        self.scroll_offset_x = self.scroll_offset_x;
        self.scroll_offset_y = self.scroll_offset_y;
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
        // ドキュメントの先頭にカーソルを移動し、選択を開始
        self.move_cursor_to_document_start(true);
        // ドキュメントの末尾にカーソルを移動して選択を完了
        // set_cursor_position が u16::MAX を適切に処理することを期待
        self.move_cursor_to_document_end(true);
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
        if let Some((start_byte_offset, end_byte_offset)) = self.get_selection_range() {
            let cut_text = self.buffer[start_byte_offset..end_byte_offset].to_string();

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

    /// カーソル位置に文字を挿入します。
    pub fn insert_char(&mut self, c: char) {
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

    /// 現在のカーソル位置をバイトオフセットに変換します。
    fn get_cursor_byte_offset(&self) -> usize {
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
    fn set_cursor_from_byte_offset(&mut self, byte_offset: usize, extend_selection: bool) {
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
        for line_str in self.buffer.lines() {
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
}
