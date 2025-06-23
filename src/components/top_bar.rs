use ratatui::{
    style::{Color, Style},
    widgets::Paragraph,
};
pub mod about;
// TopBarコンポーネント
pub fn top_bar<'a>() -> Paragraph<'a> {
    let fg = Color::Black;
    let bg = Color::White;
    Paragraph::new(" inedit - text editor ").style(Style::default().fg(bg).bg(fg))
}
