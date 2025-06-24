use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExitPopupOption {
    SaveAndExit,
    DiscardAndExit,
    Cancel,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExitPopupResult {
    SaveAndExit,
    DiscardAndExit,
    Cancel,
    None,
}

pub struct ExitPopupState {
    pub selected_option: ExitPopupOption,
    pub input_text: String,
    pub input_mode: bool,
}

impl Default for ExitPopupState {
    fn default() -> Self {
        Self {
            selected_option: ExitPopupOption::SaveAndExit,
            input_text: String::new(),
            input_mode: false,
        }
    }
}

impl ExitPopupState {
    pub fn previous(&mut self) {
        self.selected_option = match self.selected_option {
            ExitPopupOption::SaveAndExit => ExitPopupOption::Cancel,
            ExitPopupOption::DiscardAndExit => ExitPopupOption::SaveAndExit,
            ExitPopupOption::Cancel => ExitPopupOption::DiscardAndExit,
        };
    }
    pub fn next(&mut self) {
        self.selected_option = match self.selected_option {
            ExitPopupOption::SaveAndExit => ExitPopupOption::DiscardAndExit,
            ExitPopupOption::DiscardAndExit => ExitPopupOption::Cancel,
            ExitPopupOption::Cancel => ExitPopupOption::SaveAndExit,
        };
    }
}

/// ポップアップの種類
pub enum PopupKind<'a> {
    Exit,
    Input { message: &'a str, input: &'a str },
}

/// ポップアップ描画の共通関数
pub fn render_popup(f: &mut Frame, area: Rect, kind: PopupKind, state: &ExitPopupState) {
    let (title, lines, popup_height) = match kind {
        PopupKind::Exit => {
            let text = vec![
                Line::from(Span::raw("")),
                Line::from(Span::raw(
                    "You have unsaved changes. Do you want to save them?",
                )),
                Line::from(Span::raw("")),
                render_option(
                    state.selected_option,
                    ExitPopupOption::SaveAndExit,
                    "  [S]ave and Exit  ",
                ),
                render_option(
                    state.selected_option,
                    ExitPopupOption::DiscardAndExit,
                    "  [D]iscard and Exit  ",
                ),
                render_option(
                    state.selected_option,
                    ExitPopupOption::Cancel,
                    "  [C]ancel  ",
                ),
                Line::from(Span::raw("")),
            ];
            ("Unsaved Changes", text, 9)
        }
        PopupKind::Input { message, input } => {
            let text = vec![
                Line::from(Span::raw("")),
                Line::from(Span::styled(message, Style::default().fg(Color::Yellow))),
                Line::from(Span::raw("")),
                Line::from(Span::styled(
                    format!("> {}", input),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::raw("")),
                Line::from(Span::styled(
                    "Backspaceでキャンセル",
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            ("Input", text, 8)
        }
    };

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let popup_width = 60;
    let popup_area = Rect::new(
        area.width.saturating_sub(popup_width) / 2,
        area.height.saturating_sub(popup_height) / 2,
        popup_width,
        popup_height,
    );

    f.render_widget(Clear, popup_area);
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center),
        popup_area,
    );
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
