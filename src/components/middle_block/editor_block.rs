use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use syntect::parsing::SyntaxReference;

/// エディタ本体 (テキストとネイティブカーソル) を描画します。
pub fn render_editor_block(f: &mut Frame, area: Rect, app: &App) {
    // 折返しモード対応: wrap幅をword_wrap_enabledで切り替え
    let editor_area_width = super::left_block::get_editor_area_width(area);
    let wrap_width = if app.word_wrap_enabled {
        editor_area_width as usize
    } else {
        usize::MAX // wrapしない場合は非常に大きな値
    };
    let visual_lines = app.editor.get_visual_lines_with_width_word_wrap(wrap_width);
    let total_visual_lines = visual_lines.len();
    let start = app.editor.scroll_offset_y as usize;
    let end = (start + area.height as usize).min(total_visual_lines);
    let visible_lines = &visual_lines[start..end];

    let mut lines_for_paragraph: Vec<Line> = Vec::new();
    let syntax: &SyntaxReference = app
        .highlighter
        .syntax_set
        .find_syntax_by_name(&app.current_syntax_name)
        .unwrap_or_else(|| app.highlighter.syntax_set.find_syntax_plain_text());

    // 選択範囲のバイトオフセット
    let selection_range = app.editor.get_selection_range();
    let (sel_start_byte, sel_end_byte) = if let Some((sel_start, sel_end)) = selection_range {
        (sel_start, sel_end)
    } else {
        (usize::MAX, usize::MAX)
    };

    // 各visual lineごとに描画
    for (buf_idx, wrap_idx, line_str) in visible_lines.iter() {
        let mut spans: Vec<Span> = Vec::new();
        let highlighted_segments = app.highlighter.highlight_line(line_str, syntax);
        // visual lineのグローバルバイトオフセットを計算
        let global_line_start_byte_offset = app
            .editor
            .get_visual_line_global_offset(*buf_idx, *wrap_idx, wrap_width);
        let global_line_end_byte_offset = global_line_start_byte_offset + line_str.len();
        let line_selected = sel_start_byte < global_line_end_byte_offset
            && sel_end_byte > global_line_start_byte_offset;
        let mut current_byte_offset_in_line = 0;
        for (syntect_style, text) in highlighted_segments {
            let base_style = super::super::super::app::features::syntax::Highlighter::convert_syntect_style_to_ratatui_style(syntect_style);
            let segment_global_start_offset =
                global_line_start_byte_offset + current_byte_offset_in_line;
            let segment_global_end_offset = segment_global_start_offset + text.len();
            if line_selected {
                let seg_sel_start = sel_start_byte.max(segment_global_start_offset);
                let seg_sel_end = sel_end_byte.min(segment_global_end_offset);
                if seg_sel_start < seg_sel_end {
                    let rel_sel_start = seg_sel_start - segment_global_start_offset;
                    let rel_sel_end = seg_sel_end - segment_global_start_offset;
                    let left = &text[..rel_sel_start];
                    let mid = &text[rel_sel_start..rel_sel_end];
                    let right = &text[rel_sel_end..];
                    if !left.is_empty() {
                        spans.push(Span::styled(left.to_string(), base_style));
                    }
                    if !mid.is_empty() {
                        spans.push(Span::styled(
                            mid.to_string(),
                            base_style.bg(Color::Rgb(50, 50, 100)),
                        ));
                    }
                    if !right.is_empty() {
                        spans.push(Span::styled(right.to_string(), base_style));
                    }
                } else {
                    spans.push(Span::styled(text.to_string(), base_style));
                }
            } else {
                spans.push(Span::styled(text.to_string(), base_style));
            }
            current_byte_offset_in_line += text.len();
        }
        lines_for_paragraph.push(Line::from(spans));
    }
    // ビューポートの高さに満たない場合は空行で埋める
    while lines_for_paragraph.len() < area.height as usize {
        lines_for_paragraph.push(Line::from(vec![Span::raw("")]));
    }
    let theme_bg = app.highlighter.get_background_color();
    let theme_fg = app.highlighter.get_foreground_color();
    let mut paragraph = Paragraph::new(Text::from(lines_for_paragraph)).block(
        Block::default()
            .borders(Borders::NONE)
            .bg(theme_bg)
            .fg(theme_fg),
    );
    // 折返し表示モードの設定
    if app.word_wrap_enabled {
        paragraph = paragraph.wrap(Wrap { trim: false });
    } else {
        // 折返し無効時はx方向スクロールを有効化
        paragraph = paragraph.scroll((0, app.editor.scroll_offset_x as u16));
    }
    f.render_widget(paragraph, area);
    // カーソル描画
    // `get_cursor_visual_position`は、カーソルのビジュアル行インデックスと、
    // その行の先頭からのバイトオフセットを返すと仮定します。
    let (cursor_visual_idx, cursor_x_byte_offset) =
        app.editor.get_cursor_visual_position(wrap_width);

    // ビジュアル行のインデックスが、計算済みの`visual_lines`の範囲内にあるか確認します。
    if cursor_visual_idx < visual_lines.len() {
        // 該当するビジュアル行の文字列スライスを取得します。
        let (_, _, line_str) = &visual_lines[cursor_visual_idx];

        // `get_cursor_visual_position`が返す`cursor_x_byte_offset`は、カーソルの
        // 文字数ベースの位置（何文字目か）を示していると解釈します。
        // これをUTF-8文字列における正しいバイトオフセットに変換し、
        // マルチバイト文字を正しく扱えるようにします。
        let target_char_count = cursor_x_byte_offset;
        let byte_offset = line_str
            .char_indices()
            .nth(target_char_count)
            .map(|(idx, _)| idx)
            .unwrap_or(line_str.len());

        let cursor_x_in_visual = Span::from(&line_str[..byte_offset]).width();

        // カーソルの画面上のY座標を計算します（垂直スクロールを考慮）。
        let cursor_screen_y =
            area.y + (cursor_visual_idx as u16).saturating_sub(app.editor.scroll_offset_y);

        // カーソルが描画エリア内にあれば、位置を設定します。
        if cursor_screen_y < area.bottom() {
            if app.word_wrap_enabled {
                let cursor_screen_x = area.x + cursor_x_in_visual as u16;
                if cursor_screen_x < area.right() {
                    f.set_cursor_position((cursor_screen_x, cursor_screen_y));
                }
            } else {
                // 折り返し無効時は、水平スクロールを考慮します。
                if let Some(relative_x) =
                    (cursor_x_in_visual as u16).checked_sub(app.editor.scroll_offset_x)
                {
                    let cursor_screen_x = area.x + relative_x;
                    if cursor_screen_x < area.right() {
                        f.set_cursor_position((cursor_screen_x, cursor_screen_y));
                    }
                }
            }
        }
    }
}
