use ratatui::{
    style::{Color, Style},
    widgets::Paragraph,
};


// BottomBarコンポーネント
pub fn bottom_bar<'a>() -> Paragraph<'a> {
    let fg = Color::Black;
    let bg = Color::White;
    Paragraph::new(" Ctrl+Q: 終了 ").style(Style::default().fg(bg).bg(fg))
}