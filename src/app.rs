pub struct App {
    pub should_quit: bool,
    pub buffer: String,
    pub file_path: Option<String>, // 追加
    pub cursor: (usize, usize),    // カーソル位置 (行, 列)
    pub scroll: usize, // ←追加
    pub fold_mode: bool, // ←追加
    pub selection: Option<((usize, usize), (usize, usize))>, // 選択範囲
}

impl Default for App {
    fn default() -> Self {
        Self::new(String::new(), None)
    }
}

impl App {
    pub fn new(buffer: String, file_path: Option<String>) -> Self {
        Self {
            should_quit: false,
            buffer,
            file_path,
            cursor: (0, 0),
            scroll: 0,
            fold_mode: false, // ←初期値
            selection: None,
        }
    }
}
