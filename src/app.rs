use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};
use Frame;

pub struct App {
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self { should_quit: false }
    }
}

pub fn ui<B: Backend>(f: &mut Frame<>, _app: &App) {
    let size = f.area();
    let block = Block::default().title("Ratatui App").borders(Borders::ALL);
    f.render_widget(block, size);
}