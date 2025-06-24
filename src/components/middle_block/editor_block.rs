use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use syntect::parsing::SyntaxReference; // SyntaxReferenceをインポート

/// エディタ本体 (テキストとネイティブカーソル) を描画します。
pub fn render_editor_block(f: &mut Frame, area: Rect, app: &App) {
    let editor_content = &app.editor.buffer;
    let cursor_x = app.editor.cursor.x;
    let cursor_y = app.editor.cursor.y;
    let selection_range = app.editor.get_selection_range();

    let mut lines_for_paragraph: Vec<Line> = Vec::new();

    // 現在のシンタックスを取得 (Appで管理される)
    let syntax: &SyntaxReference = app
        .highlighter
        .syntax_set
        .find_syntax_by_name(&app.current_syntax_name)
        .unwrap_or_else(|| app.highlighter.syntax_set.find_syntax_plain_text());

    // 全ての行をイテレートし、シンタックスハイライトと選択状態を考慮したスタイルを適用します。
    for (line_idx, line_str) in editor_content.lines().enumerate() {
        let mut spans: Vec<Span> = Vec::new();
        let mut current_byte_offset_in_line = 0;

        // syntectによるハイライト結果を取得
        let highlighted_segments = app.highlighter.highlight_line(line_str, syntax);

        // 選択範囲のバイトオフセットを計算
        let (sel_start_byte, sel_end_byte) = if let Some((sel_start, sel_end)) = selection_range {
            (sel_start, sel_end)
        } else {
            (usize::MAX, usize::MAX) // 選択なし
        };

        // この行のグローバルバイト範囲
        let global_line_start_byte_offset = app
            .editor
            .buffer
            .lines()
            .take(line_idx)
            .map(|l| l.len() + 1) // +1は改行コードのバイトオフセット
            .sum::<usize>();
        let global_line_end_byte_offset = global_line_start_byte_offset + line_str.len();

        // この行が選択範囲と重なっているか
        let line_selected = sel_start_byte < global_line_end_byte_offset && sel_end_byte > global_line_start_byte_offset;

        for (syntect_style, text) in highlighted_segments {
            let base_style = super::super::super::app::features::syntax::Highlighter::convert_syntect_style_to_ratatui_style(syntect_style);

            let segment_global_start_offset = global_line_start_byte_offset + current_byte_offset_in_line;
            let segment_global_end_offset = segment_global_start_offset + text.len();

            // このセグメントが選択範囲と重なる場合のみ色を変える
            if line_selected {
                // セグメント内で選択範囲と重なる部分だけ色を変える
                let seg_sel_start = sel_start_byte.max(segment_global_start_offset);
                let seg_sel_end = sel_end_byte.min(segment_global_end_offset);

                if seg_sel_start < seg_sel_end {
                    // セグメント内で選択範囲が部分的に重なる場合
                    let rel_sel_start = seg_sel_start - segment_global_start_offset;
                    let rel_sel_end = seg_sel_end - segment_global_start_offset;

                    // 3分割: [非選択][選択][非選択]
                    // let text_bytes = text.as_bytes();
                    let left = &text[..rel_sel_start];
                    let mid = &text[rel_sel_start..rel_sel_end];
                    let right = &text[rel_sel_end..];

                    if !left.is_empty() {
                        spans.push(Span::styled(left.to_string(), base_style));
                    }
                    if !mid.is_empty() {
                        spans.push(Span::styled(mid.to_string(), base_style.bg(Color::Rgb(50, 50, 100))));
                    }
                    if !right.is_empty() {
                        spans.push(Span::styled(right.to_string(), base_style));
                    }
                } else {
                    // セグメント全体が非選択
                    spans.push(Span::styled(text.to_string(), base_style));
                }
            } else {
                // 選択範囲外
                spans.push(Span::styled(text.to_string(), base_style));
            }
            current_byte_offset_in_line += text.len();
        }
        lines_for_paragraph.push(Line::from(spans));
    }

    // バッファが空の場合、少なくとも1行を表示してカーソルが描画されるようにする
    if editor_content.is_empty() {
        lines_for_paragraph.push(Line::from(vec![Span::raw("")]));
    }

    // テーマの背景色と前景色を取得
    let theme_bg = app.highlighter.get_background_color();
    let theme_fg = app.highlighter.get_foreground_color();

    let mut paragraph = Paragraph::new(Text::from(lines_for_paragraph))
        .block(
            Block::default()
                .borders(Borders::NONE)
                .bg(theme_bg)
                .fg(theme_fg),
        )
        .scroll((app.editor.scroll_offset_y, app.editor.scroll_offset_x));

    // 折り返し表示モードの設定
    if app.word_wrap_enabled {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    f.render_widget(paragraph, area);

    // ネイティブカーソルを描画する
    let mut visual_cursor_x_on_line = 0u16;
    if let Some(current_line) = editor_content.lines().nth(cursor_y as usize) {
        let substring_before_cursor: String =
            current_line.chars().take(cursor_x as usize).collect();
        visual_cursor_x_on_line = Line::from(substring_before_cursor).width() as u16;
    }

    let actual_cursor_x_on_screen = area
        .x
        .saturating_add(visual_cursor_x_on_line)
        .saturating_sub(app.editor.scroll_offset_x);
    let actual_cursor_y_on_screen = area
        .y
        .saturating_add(cursor_y)
        .saturating_sub(app.editor.scroll_offset_y);

    if actual_cursor_x_on_screen < area.right() && actual_cursor_y_on_screen < area.bottom() {
        f.set_cursor_position((actual_cursor_x_on_screen, actual_cursor_y_on_screen));
    }
}
