use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// 終了ポップアップの選択肢
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExitPopupOption {
    SaveAndExit,
    DiscardAndExit,
    Cancel,
}

/// 終了ポップアップの結果
#[derive(Debug, PartialEq, Eq)]
pub enum ExitPopupResult {
    SaveAndExit,
    DiscardAndExit,
    Cancel,
    /// ユーザーがまだ選択していない状態 (ポップアップが表示されているが、Enterが押されていない)
    None,
}

/// 終了ポップアップの状態
pub struct ExitPopupState {
    pub selected_option: ExitPopupOption,
}

impl Default for ExitPopupState {
    fn default() -> Self {
        Self {
            selected_option: ExitPopupOption::SaveAndExit, // デフォルトは「保存して終了」
        }
    }
}

impl ExitPopupState {
    /// 選択肢を上に移動
    pub fn previous(&mut self) {
        self.selected_option = match self.selected_option {
            ExitPopupOption::SaveAndExit => ExitPopupOption::Cancel,
            ExitPopupOption::DiscardAndExit => ExitPopupOption::SaveAndExit,
            ExitPopupOption::Cancel => ExitPopupOption::DiscardAndExit,
        };
    }

    /// 選択肢を下に移動
    pub fn next(&mut self) {
        self.selected_option = match self.selected_option {
            ExitPopupOption::SaveAndExit => ExitPopupOption::DiscardAndExit,
            ExitPopupOption::DiscardAndExit => ExitPopupOption::Cancel,
            ExitPopupOption::Cancel => ExitPopupOption::SaveAndExit,
        };
    }
}

/// 終了ポップアップを描画します。
pub fn render_exit_popup(f: &mut Frame, area: Rect, state: &ExitPopupState) {
    let block = Block::default()
        .title(Span::styled("Unsaved Changes", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let text = vec![
        Line::from(Span::raw("")), // Top margin
        Line::from(Span::raw("You have unsaved changes. Do you want to save them?")),
        Line::from(Span::raw("")), // Margin between message and options
        render_option(state.selected_option, ExitPopupOption::SaveAndExit, "  [S]ave and Exit  "),
        render_option(state.selected_option, ExitPopupOption::DiscardAndExit, "  [D]iscard and Exit  "),
        render_option(state.selected_option, ExitPopupOption::Cancel, "  [C]ancel  "),
        Line::from(Span::raw("")), // Bottom margin
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    // Calculate the area for the popup to be centered
    // Ensure the popup width and height are reasonable given the content
    let popup_width = 60; // Fixed width for the popup
    let popup_height = 9; // Number of lines in the text + block borders (7 lines content + 2 for borders)

    let popup_area = Rect::new(
        area.width.saturating_sub(popup_width) / 2,
        area.height.saturating_sub(popup_height) / 2,
        popup_width,
        popup_height,
    );

    // Clear the background behind the popup
    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

fn render_option<'a>(
    current_selection: ExitPopupOption,
    option_type: ExitPopupOption,
    text: &'a str,
) -> Line<'a> {
    let mut style = Style::default().fg(Color::White);
    if current_selection == option_type {
        style = style.add_modifier(Modifier::BOLD).bg(Color::Blue);
    }
    Line::from(Span::styled(text, style))
}
