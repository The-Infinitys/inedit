
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use crate::app::App; // App構造体を使用するためにインポート

/// エディタ本体 (テキストとネイティブカーソル) を描画します。
pub fn render_editor_block(f: &mut Frame, area: Rect, app: &App) {
    let editor_content = &app.editor.buffer;
    let cursor_x = app.editor.cursor.x;
    let cursor_y = app.editor.cursor.y;
    let selection_range = app.editor.get_selection_range(); // バイトオフセットでの選択範囲

    let mut lines_to_display: Vec<Line> = Vec::new();

    // 行ごとに処理
    // 表示すべき範囲の行のみをイテレート
    let start_line_for_display = app.editor.scroll_offset_y as usize;
    let end_line_for_display = (app.editor.scroll_offset_y + area.height) as usize;

    for (line_idx, line_str) in editor_content.lines().enumerate() {
        if line_idx < start_line_for_display || line_idx >= end_line_for_display {
            continue; // 表示範囲外の行はスキップ
        }

        let mut spans: Vec<Span> = Vec::new();
        let mut current_byte_offset_in_line = 0;

        // 文字ごとに処理して、選択状態を考慮したスタイルを適用
        for c in line_str.chars() {
            let char_len_bytes = c.len_utf8();

            let mut style = Style::default();

            // 選択範囲のハイライト
            if let Some((sel_start, sel_end)) = selection_range {
                // この行の先頭からのグローバルバイトオフセットを計算
                let global_line_start_byte_offset = app.editor.buffer.lines()
                    .take(line_idx)
                    .map(|l| l.len() + 1) // +1 for newline
                    .sum::<usize>();

                let char_global_start_offset = global_line_start_byte_offset + current_byte_offset_in_line;
                let char_global_end_offset = char_global_start_offset + char_len_bytes;

                // 現在の文字が選択範囲内にあるかチェック
                if (char_global_start_offset >= sel_start && char_global_start_offset < sel_end) ||
                   (char_global_end_offset > sel_start && char_global_end_offset <= sel_end) ||
                   (sel_start >= char_global_start_offset && sel_start < char_global_end_offset)
                {
                    style = style.bg(Color::Rgb(50, 50, 100)); // 選択色
                }
            }

            spans.push(Span::styled(c.to_string(), style));
            current_byte_offset_in_line += char_len_bytes;
        }

        lines_to_display.push(Line::from(spans));
    }

    // バッファが空の場合、少なくとも1行を表示してカーソルが描画されるようにする
    if editor_content.is_empty() {
        lines_to_display.push(Line::from(vec![Span::raw("")]));
    }


    let mut paragraph = Paragraph::new(lines_to_display)
        .block(Block::default().borders(Borders::NONE)) // 枠線なし
        // スクロールオフセットを適用
        .scroll((app.editor.scroll_offset_y, app.editor.scroll_offset_x));

    // 折り返し表示モードの設定
    if app.word_wrap_enabled {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    f.render_widget(paragraph, area);

    // ネイティブカーソルを描画する
    // カーソルの実際の画面上の位置を計算
    // 表示領域の相対Y座標: cursor_y - scroll_offset_y
    // 表示領域の相対X座標: cursor_x - scroll_offset_x
    let actual_cursor_x_on_screen = area.x + cursor_x.saturating_sub(app.editor.scroll_offset_x);
    let actual_cursor_y_on_screen = area.y + cursor_y.saturating_sub(app.editor.scroll_offset_y);

    // カーソルが描画領域内にある場合のみ設定
    if actual_cursor_x_on_screen < area.right() && actual_cursor_y_on_screen < area.bottom() {
        f.set_cursor_position((actual_cursor_x_on_screen, actual_cursor_y_on_screen));
    }
}
