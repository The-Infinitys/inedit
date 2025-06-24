use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text}, // Textをインポート
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// エディタ本体 (テキストとネイティブカーソル) を描画します。
pub fn render_editor_block(f: &mut Frame, area: Rect, app: &App) {
    let editor_content = &app.editor.buffer;
    let cursor_x = app.editor.cursor.x;
    let cursor_y = app.editor.cursor.y;
    let selection_range = app.editor.get_selection_range(); // バイトオフセットでの選択範囲

    let mut lines_for_paragraph: Vec<Line> = Vec::new();

    // 全ての行をイテレートし、選択状態を考慮したスタイルを適用します。
    // Paragraphウィジェットが内部でスクロールを処理するため、
    // ここで表示範囲による行のフィルタリングは行いません。
    for (line_idx, line_str) in editor_content.lines().enumerate() {
        let mut spans: Vec<Span> = Vec::new();
        let mut current_byte_offset_in_line = 0;

        for c in line_str.chars() {
            let char_len_bytes = c.len_utf8();

            let mut style = Style::default();

            // 選択範囲のハイライト
            if let Some((sel_start, sel_end)) = selection_range {
                // この行の先頭からのグローバルバイトオフセットを計算
                // buffer.lines().take(line_idx) は、現在の行より前の全ての行のイテレータを返します。
                // その後、mapで各行のバイト長+改行文字のバイト長を計算し、sumで合計します。
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
                // このチェックは、文字の開始オフセットが選択範囲内にあるか、
                // 文字の終了オフセットが選択範囲内にあるか、
                // または選択範囲が文字を完全に包含しているかを確認します。
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
    // Paragraphは空のVec<Line>が渡されると何も描画しないため、カーソルが表示されない可能性があります。
    // このチェックは必要です。
    if editor_content.is_empty() {
        lines_for_paragraph.push(Line::from(vec![Span::raw("")]));
    }

    let mut paragraph = Paragraph::new(Text::from(lines_for_paragraph)) // Text::fromでVec<Line>をラップ
        .block(Block::default().borders(Borders::NONE)) // 枠線なし
        // Paragraphウィジェットにスクロールオフセットを適用させることで、
        // 適切な行と列の範囲が自動的に描画されます。
        .scroll((app.editor.scroll_offset_y, app.editor.scroll_offset_x));

    // 折り返し表示モードの設定
    if app.word_wrap_enabled {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    f.render_widget(paragraph, area);

    // ネイティブカーソルを描画する
    // カーソルの論理的な位置 (cursor_x, cursor_y) から、
    // スクロールオフセットを考慮した画面上の物理的な位置を計算します。
    let actual_cursor_x_on_screen = area.x + cursor_x.saturating_sub(app.editor.scroll_offset_x);
    let actual_cursor_y_on_screen = area.y + cursor_y.saturating_sub(app.editor.scroll_offset_y);

    // 計算されたカーソル位置が描画領域内にある場合のみ設定します。
    // これにより、カーソルがビューポート外にある場合に表示されないことを保証します。
    if actual_cursor_x_on_screen < area.right() && actual_cursor_y_on_screen < area.bottom() {
        f.set_cursor_position((actual_cursor_x_on_screen, actual_cursor_y_on_screen));
    }
}
