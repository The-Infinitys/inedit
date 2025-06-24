use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Modifier}, // Modifierも利用
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
    let syntax: &SyntaxReference = app.highlighter.syntax_set.find_syntax_by_name(&app.current_syntax_name)
        .unwrap_or_else(|| app.highlighter.syntax_set.find_syntax_plain_text());

    // 全ての行をイテレートし、シンタックスハイライトと選択状態を考慮したスタイルを適用します。
    for (line_idx, line_str) in editor_content.lines().enumerate() {
        let mut spans: Vec<Span> = Vec::new();
        let mut current_byte_offset_in_line = 0;

        // syntectによるハイライト結果を取得
        let highlighted_segments = app.highlighter.highlight_line(line_str, syntax);

        for (syntect_style, text) in highlighted_segments {
            let mut base_style = super::super::super::app::features::syntax::Highlighter::convert_syntect_style_to_ratatui_style(syntect_style);

            // 選択範囲のハイライトを上書き
            if let Some((sel_start_byte, sel_end_byte)) = selection_range {
                let global_line_start_byte_offset = app
                    .editor
                    .buffer
                    .lines()
                    .take(line_idx)
                    .map(|l| l.len() + 1) // +1は改行コードのバイトオフセット
                    .sum::<usize>();

                let segment_global_start_offset = global_line_start_byte_offset + current_byte_offset_in_line;
                let segment_global_end_offset = segment_global_start_offset + text.len(); // text.len()はバイト長

                // セグメントが選択範囲と重なるかチェック
                if segment_global_start_offset < sel_end_byte && segment_global_end_offset > sel_start_byte {
                    base_style = base_style.bg(Color::Rgb(50, 50, 100)); // 選択色
                }
            }
            spans.push(Span::styled(text.to_string(), base_style));
            current_byte_offset_in_line += text.len();
        }
        lines_for_paragraph.push(Line::from(spans));
    }

    // バッファが空の場合、少なくとも1行を表示してカーソルが描画されるようにする
    if editor_content.is_empty() {
        lines_for_paragraph.push(Line::from(vec![Span::raw("")]));
    }

    let mut paragraph = Paragraph::new(Text::from(lines_for_paragraph))
        .block(Block::default().borders(Borders::NONE))
        .scroll((app.editor.scroll_offset_y, app.editor.scroll_offset_x));

    // 折り返し表示モードの設定
    if app.word_wrap_enabled {
        paragraph = paragraph.wrap(Wrap { trim: false });
    }

    f.render_widget(paragraph, area);

    // ネイティブカーソルを描画する
    let mut visual_cursor_x_on_line = 0u16;
    if let Some(current_line) = editor_content.lines().nth(cursor_y as usize) {
        let substring_before_cursor: String = current_line.chars().take(cursor_x as usize).collect();
        visual_cursor_x_on_line = Line::from(substring_before_cursor).width() as u16;
    }

    let actual_cursor_x_on_screen = area.x.saturating_add(visual_cursor_x_on_line).saturating_sub(app.editor.scroll_offset_x);
    let actual_cursor_y_on_screen = area.y.saturating_add(cursor_y).saturating_sub(app.editor.scroll_offset_y);

    if actual_cursor_x_on_screen < area.right() && actual_cursor_y_on_screen < area.bottom() {
        f.set_cursor_position((actual_cursor_x_on_screen, actual_cursor_y_on_screen));
    }
}
