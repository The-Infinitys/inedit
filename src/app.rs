pub struct App {
    pub buffer: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            buffer: String::from("Hello ratatui!"),
        }
    }
}

