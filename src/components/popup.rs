use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

pub enum PopupResult {
    Option(usize), // 選択肢のインデックス
    Cancel,
}

/// ポップアップを表示する（任意のメッセージ・選択肢対応）
pub fn show_popup<B: ratatui::backend::Backend>(
    f: &mut Frame,
    area: Rect,
    message: &str,
    choices: &[&str],
) {
    let mut lines = vec![Line::from(message)];
    lines.push(Line::from("")); // 空行
    for (i, choice) in choices.iter().enumerate() {
        lines.push(Line::from(format!("{}: {}", i + 1, choice)));
    }
    let para = Paragraph::new(lines).block(Block::default().title("確認").borders(Borders::ALL));
    let popup_area = Rect {
        x: area.width / 4,
        y: area.height / 3,
        width: area.width / 2,
        height: (choices.len() + 4) as u16,
    };
    f.render_widget(para, popup_area);
}
