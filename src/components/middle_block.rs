// src/components/middle_block.rs

use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
}; // App構造体を使用するためにインポート

/// エディタ本体 (テキストとカーソル) を描画します。
pub fn render_middle_block(f: &mut Frame, area: Rect, app: &App) {
    let editor_content = &app.editor.buffer;
    let cursor_x = app.editor.cursor.x;
    let cursor_y = app.editor.cursor.y;
    let selection_range = app.editor.get_selection_range(); // バイトオフセットでの選択範囲

    let mut lines_to_display: Vec<Line> = Vec::new();

    // 行ごとに処理
    for (line_idx, line_str) in editor_content.lines().enumerate() {
        let mut spans: Vec<Span> = Vec::new();
        let mut current_byte_offset_in_line = 0;

        // 文字ごとに処理して、選択状態やカーソル位置を考慮したスタイルを適用
        for (char_idx, c) in line_str.chars().enumerate() {
            let char_len_bytes = c.len_utf8();
            let char_end_byte_offset = current_byte_offset_in_line + char_len_bytes;

            let mut style = Style::default();

            // 選択範囲のハイライト
            if let Some((sel_start, sel_end)) = selection_range {
                // 現在の文字が選択範囲内にあるかチェック
                // この行の先頭からのバイトオフセット + 行頭までのバイトオフセットが、
                // グローバルな選択範囲のバイトオフセットと比較される必要がある。
                // 現時点ではmiddle_blockがグローバルなオフセットを知らないため、
                // get_selection_range()がEditorからバイトオフセットで返していることを利用。
                let global_line_start_byte_offset = app
                    .editor
                    .buffer
                    .lines()
                    .take(line_idx)
                    .map(|l| l.len() + 1) // +1 for newline
                    .sum::<usize>();

                let char_global_start_offset =
                    global_line_start_byte_offset + current_byte_offset_in_line;
                let char_global_end_offset = global_line_start_byte_offset + char_end_byte_offset;

                if (char_global_start_offset >= sel_start && char_global_start_offset < sel_end)
                    || (char_global_end_offset > sel_start && char_global_end_offset <= sel_end)
                    || (sel_start >= char_global_start_offset && sel_start < char_global_end_offset)
                {
                    style = style.bg(Color::Rgb(50, 50, 100)); // 選択色
                }
            }

            // カーソル位置の表示
            // カーソルが文字の直前にある場合
            if line_idx as u16 == cursor_y && char_idx as u16 == cursor_x {
                style = style.bg(Color::Cyan).fg(Color::Black); // カーソル色
            }
            // 空の行にカーソルがある場合や、行末にカーソルがある場合の特別処理
            if line_idx as u16 == cursor_y
                && cursor_x as usize == line_str.chars().count()
                && char_idx as u16 == cursor_x.saturating_sub(1)
            {
                // 行末にカーソルがある場合、最後の文字をハイライト
                style = style.bg(Color::Cyan).fg(Color::Black);
            }
            if line_idx as u16 == cursor_y && line_str.is_empty() && cursor_x == 0 {
                // 空の行にカーソルがある場合、特殊なスペース文字でカーソルを表示
                spans.push(Span::styled(
                    " ",
                    Style::default().bg(Color::Cyan).fg(Color::Black),
                ));
                // 他の文字は処理しない
                current_byte_offset_in_line += char_len_bytes; // これは実際には消費しないが、ループのため
                continue;
            }

            spans.push(Span::styled(c.to_string(), style));
            current_byte_offset_in_line += char_len_bytes;
        }

        // 行末にカーソルがあるが、その文字自体は選択されていない場合の処理
        if line_idx as u16 == cursor_y
            && cursor_x as usize == line_str.chars().count()
            && !line_str.is_empty()
        {
            // 行末にカーソルがあり、その位置に表示する文字がない場合は、1文字分のスペースをカーソルとして表示
            if let Some(last_span) = spans.last_mut() {
                // 既にカーソルが最終文字にハイライトされている場合は何もしない
                if !(last_span.style.bg == Some(Color::Cyan)
                    && last_span.style.fg == Some(Color::Black))
                {
                    spans.push(Span::styled(
                        " ",
                        Style::default().bg(Color::Cyan).fg(Color::Black),
                    ));
                }
            } else {
                // 空行でないがspansが空の場合（ありえないはずだが念のため）
                spans.push(Span::styled(
                    " ",
                    Style::default().bg(Color::Cyan).fg(Color::Black),
                ));
            }
        }
        if line_idx as u16 == cursor_y && line_str.is_empty() && cursor_x == 0 {
            // 空の行でカーソルが0列にある場合、すでに上で処理されているのでここでは何もしない
        } else if line_idx as u16 == cursor_y && cursor_x as usize > line_str.chars().count() {
            // カーソルが実際の行の文字数を超えている場合（パディングが必要）
            let num_spaces_to_add = cursor_x as usize - line_str.chars().count();
            for _ in 0..num_spaces_to_add {
                spans.push(Span::raw(" "));
            }
            // 最後にカーソルを表示
            spans.push(Span::styled(
                " ",
                Style::default().bg(Color::Cyan).fg(Color::Black),
            ));
        }

        lines_to_display.push(Line::from(spans));
    }

    // バッファが空で、カーソルが (0,0) の場合、空の行にカーソルを表示
    if editor_content.is_empty() && cursor_x == 0 && cursor_y == 0 {
        lines_to_display.push(Line::from(vec![Span::styled(
            " ",
            Style::default().bg(Color::Cyan).fg(Color::Black),
        )]));
    }
    // バッファの最後の行以降にカーソルがある場合、その行を適切に描画
    else if (cursor_y as usize) >= editor_content.lines().count() && !editor_content.is_empty() {
        let mut empty_line_spans = vec![];
        // カーソルのX位置までスペースを埋める
        for _ in 0..cursor_x {
            empty_line_spans.push(Span::raw(" "));
        }
        // カーソル自体を表示
        empty_line_spans.push(Span::styled(
            " ",
            Style::default().bg(Color::Cyan).fg(Color::Black),
        ));
        lines_to_display.push(Line::from(empty_line_spans));
    }

    let paragraph = Paragraph::new(lines_to_display)
        .block(Block::default().borders(Borders::NONE)) // 枠線なし
        .scroll((app.editor.cursor.y, 0)); // カーソル行に合わせてスクロール

    f.render_widget(paragraph, area);
}
