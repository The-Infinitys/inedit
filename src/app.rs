pub struct App {
    pub buffer: String,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            buffer: String::from("Hello ratatui!"),
        }
    }
}

