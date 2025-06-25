use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};
use syntect::parsing::SyntaxReference;
use unicode_width::UnicodeWidthStr;

/// エディタ本体 (テキストとネイティブカーソル) を描画します。
pub fn render_editor_block(f: &mut Frame, area: Rect, app: &mut App) {
    let theme_bg = app.highlighter.get_background_color(); // Unused
    let theme_fg = app.highlighter.get_foreground_color();

    let editor_block = Block::default()
        .borders(Borders::NONE)
        .bg(theme_bg)
        .fg(theme_fg); // Borders are handled by middle_block.rs
    let inner_area = editor_block.inner(area); // This will be the actual content area
    f.render_widget(editor_block, area);

    // The `EditorWidget` will handle the actual text rendering and cursor calculation
    let editor_widget = EditorWidget::new(app);
    f.render_widget(editor_widget, inner_area);

    // Calculate and set the cursor position
    let (cursor_visual_x, cursor_visual_y) = app.editor.logical_to_visual_pos(
        app.editor.cursor.get_current_pos(),
        inner_area.width, // Use the inner_area width for wrapping calculation
        app.word_wrap_enabled,
    );

    let final_cursor_x = inner_area.x + cursor_visual_x.saturating_sub(app.editor.scroll_offset_x); // Apply horizontal scroll
    let final_cursor_y =
        inner_area.y + cursor_visual_y.saturating_sub(app.editor.scroll_offset_visual_y);

    if final_cursor_x < inner_area.right() && final_cursor_y < inner_area.bottom() {
        f.set_cursor_position((final_cursor_x, final_cursor_y));
    } else {
        // If cursor is out of bounds, it's implicitly hidden by not setting its position.
    }
}

struct EditorWidget<'a> {
    app: &'a App,
}

impl<'a> EditorWidget<'a> {
    fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for EditorWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let editor = &self.app.editor;

        if area.width == 0 || area.height == 0 {
            return;
        }

        // `content_width` is the width available for text, excluding line numbers and scrollbar.
        // This is the width that `get_visual_lines_for_logical_line` should use for wrapping.
        let content_width = area.width;

        // let visual_line_y: u16 = 0; // Tracks the current visual line being processed (global)
        // let drawn_lines: u16 = 0; // Tracks how many lines have been drawn on screen (relative to `area.y`)

        let selection_range = self.app.editor.get_selection_range();
        let (sel_start_byte, sel_end_byte) = selection_range
            .map_or((usize::MAX, usize::MAX), |(sel_start, sel_end)| {
                (sel_start, sel_end)
            });

        // Get syntax highlighter for the current file
        let syntax: &SyntaxReference = self
            .app // Use self.app to access App fields
            .highlighter
            .syntax_set
            .find_syntax_by_name(&self.app.current_syntax_name) // Use self.app
            .unwrap_or_else(|| self.app.highlighter.syntax_set.find_syntax_plain_text()); // Use self.app

        let mut current_logical_line_byte_offset = 0; // Tracks byte offset of the start of the current logical line in the whole buffer for selection

        let mut visual_line_y: u16 = 0; // Tracks the current visual line being processed (global)
        let mut drawn_lines: u16 = 0; // Tracks how many lines have been drawn on screen (relative to `area.y`)

        'outer: for logical_line_idx in 0..editor.lines.len() {
            let line_content = &editor.lines[logical_line_idx];
            let highlighted_segments_for_logical_line =
                self.app.highlighter.highlight_line(line_content, syntax);

            let wrapped_segments =
                editor.get_visual_lines_for_logical_line(logical_line_idx, content_width);

            let mut current_char_offset_in_logical_line: usize = 0; // Tracks char index within the original logical line

            for line_segment in wrapped_segments.iter() {
                if visual_line_y >= editor.scroll_offset_visual_y {
                    let screen_y = area.y + drawn_lines;

                    if screen_y >= area.bottom() {
                        break 'outer;
                    }

                    let mut spans: Vec<Span> = Vec::new();
                    let mut current_segment_visual_width: u16 = 0; // Tracks visual width within the current line_segment

                    // Iterate through the highlighted segments of the *logical line*
                    // and extract parts that fall into the current `line_segment` (wrapped part)
                    let mut temp_logical_char_offset = 0; // Helper to track char offset in logical line for highlighted_segments

                    for (syntect_style, text_from_highlighter) in
                        &highlighted_segments_for_logical_line
                    {
                        let base_style = super::super::super::app::features::syntax::Highlighter::convert_syntect_style_to_ratatui_style(*syntect_style);

                        let segment_chars: Vec<char> = text_from_highlighter.chars().collect();
                        let segment_char_len = segment_chars.len();

                        // Determine the intersection of `text_from_highlighter` and `line_segment`
                        // `line_segment` covers chars from `current_char_offset_in_logical_line`
                        // to `current_char_offset_in_logical_line + line_segment.chars().count()`
                        let line_segment_start_char_in_logical =
                            current_char_offset_in_logical_line;
                        let line_segment_end_char_in_logical =
                            current_char_offset_in_logical_line + line_segment.chars().count();

                        let intersection_start =
                            temp_logical_char_offset.max(line_segment_start_char_in_logical);
                        let intersection_end = (temp_logical_char_offset + segment_char_len)
                            .min(line_segment_end_char_in_logical);

                        if intersection_start < intersection_end {
                            // There's an overlap, extract the relevant part
                            let relative_start_in_highlighted_segment =
                                intersection_start - temp_logical_char_offset;
                            let relative_end_in_highlighted_segment =
                                intersection_end - temp_logical_char_offset;

                            let part_to_render_chars: String = segment_chars
                                [relative_start_in_highlighted_segment
                                    ..relative_end_in_highlighted_segment]
                                .iter()
                                .collect();
                            // let part_to_render_visual_width = part_to_render_chars.width() as u16;

                            // Apply horizontal scrolling
                            let scrolled_part_to_render_chars: String = part_to_render_chars
                                .chars()
                                .skip(editor.scroll_offset_x as usize)
                                .collect();
                            let scrolled_part_to_render_visual_width =
                                scrolled_part_to_render_chars.width() as u16;

                            if current_segment_visual_width + scrolled_part_to_render_visual_width
                                > 0
                            {
                                // Only add if it's visible after scrolling
                                let mut final_style = base_style;

                                // Apply selection highlighting
                                let part_to_render_byte_offset_start =
                                    line_content[..intersection_start].len();
                                let part_to_render_byte_offset_end =
                                    line_content[..intersection_end].len();

                                let global_part_start_byte_offset = current_logical_line_byte_offset
                                    + part_to_render_byte_offset_start;
                                let global_part_end_byte_offset = current_logical_line_byte_offset
                                    + part_to_render_byte_offset_end;

                                if global_part_start_byte_offset < sel_end_byte
                                    && global_part_end_byte_offset > sel_start_byte
                                {
                                    final_style = final_style.bg(Color::Rgb(50, 50, 100));
                                }

                                spans
                                    .push(Span::styled(scrolled_part_to_render_chars, final_style));
                                current_segment_visual_width +=
                                    scrolled_part_to_render_visual_width;
                            }
                        }
                        temp_logical_char_offset += segment_char_len;
                    }

                    buf.set_line(area.x, screen_y, &Line::from(spans), content_width);
                    drawn_lines += 1;
                }
                current_char_offset_in_logical_line += line_segment.chars().count();
                visual_line_y += 1;
            }
            current_logical_line_byte_offset += line_content.len() + 1; // +1 for newline
        }
    }
}
