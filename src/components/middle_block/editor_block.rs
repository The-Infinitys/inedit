use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
// use unicode_width::UnicodeWidthChar; // unicode-width は使用しません

/// エディタ本体 (テキストとネイティブカーソル) を描画します。
pub fn render_editor_block(f: &mut Frame, area: Rect, app: &App) {
    let editor_content = &app.editor.buffer;
    let cursor_x = app.editor.cursor.x; // 論理的な文字インデックス
    let cursor_y = app.editor.cursor.y;
    let selection_range = app.editor.get_selection_range(); // バイトオフセットでの選択範囲

    let mut lines_for_paragraph: Vec<Line> = Vec::new();

    // 全ての行をイテレートし、選択状態を考慮したスタイルを適用します。
    for (line_idx, line_str) in editor_content.lines().enumerate() {
        let mut spans: Vec<Span> = Vec::new();
        let mut current_byte_offset_in_line = 0;

        for c in line_str.chars() {
            let char_len_bytes = c.len_utf8();

            let mut style = Style::default();

            // 選択範囲のハイライト
            if let Some((sel_start, sel_end)) = selection_range {
                // この行の先頭からのグローバルバイトオフセットを計算
                let global_line_start_byte_offset = app
                    .editor
                    .buffer
                    .lines()
                    .take(line_idx)
                    .map(|l| l.len() + 1) // +1 for newline (LF is 1 byte)
                    .sum::<usize>();

                let char_global_start_offset =
                    global_line_start_byte_offset + current_byte_offset_in_line;
                let char_global_end_offset = char_global_start_offset + char_len_bytes;

                // 現在の文字が選択範囲内にあるかチェック
                if (char_global_start_offset >= sel_start && char_global_start_offset < sel_end)
                    || (char_global_end_offset > sel_start && char_global_end_offset <= sel_end)
                    || (sel_start >= char_global_start_offset && sel_start < char_global_end_offset)
                {
                    style = style.bg(Color::Rgb(50, 50, 100)); // 選択色
                }
            }

            spans.push(Span::styled(c.to_string(), style));
            current_byte_offset_in_line += char_len_bytes;
        }
        lines_for_paragraph.push(Line::from(spans));
    }

    // バッファが空の場合、少なくとも1行を表示してカーソルが描画されるようにする
    if editor_content.is_empty() {
        lines_for_paragraph.push(Line::from(vec![Span::raw("")]));
    }

    let mut paragraph = Paragraph::new(Text::from(lines_for_paragraph))
        .block(Block::default().borders(Borders::NONE))
        // Paragraphウィジェットにスクロールオフセットを適用させることで、
        // 適切な行と列の範囲が自動的に描画されます。
        .scroll((app.editor.scroll_offset_y, app.editor.scroll_offset_x));

    // 折り返し表示モードの設定
    if app.word_wrap_enabled {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    f.render_widget(paragraph, area);

    // ネイティブカーソルを描画する
    // カーソルの論理X座標 (cursor_x) を、現在の行における視覚的な幅（セル数）に変換
    let mut visual_cursor_x_on_line = 0u16;
    if let Some(current_line) = editor_content.lines().nth(cursor_y as usize) {
        // 現在の行の先頭からカーソル位置までの部分文字列を抽出し、その視覚的な幅を取得
        let substring_before_cursor: String = current_line.chars().take(cursor_x as usize).collect();
        visual_cursor_x_on_line = Line::from(substring_before_cursor).width() as u16;
    }

    // スクロールオフセットを考慮し、画面上の最終的なカーソルX座標を計算
    let actual_cursor_x_on_screen = area.x.saturating_add(visual_cursor_x_on_line).saturating_sub(app.editor.scroll_offset_x);
    // スクロールオフセットを考慮し、画面上の最終的なカーソルY座標を計算
    let actual_cursor_y_on_screen = area.y.saturating_add(cursor_y).saturating_sub(app.editor.scroll_offset_y);

    // カーソルが描画領域内にある場合のみ設定します。
    if actual_cursor_x_on_screen < area.right() && actual_cursor_y_on_screen < area.bottom() {
        f.set_cursor_position((actual_cursor_x_on_screen, actual_cursor_y_on_screen));
    }
}
