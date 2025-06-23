// src/app.rs
pub mod cursor;
pub mod editor;
pub mod features;
use editor::Editor;
use std::env;
use std::io;
use std::path::{Path, PathBuf};

pub struct App {
    pub editor: Editor,
    pub target_path: Option<PathBuf>,
}

impl Default for App {
    fn default() -> Self {
        // デフォルトはプレーンテキストモードのエディタ
        Self {
            editor: Editor::new(String::new()),
            target_path: None,
        }
    }
}

impl App {
    pub fn init() -> Self {
        let mut app = Self::default();

        let args: Vec<String> = env::args().collect();
        if let Some(file_path_str) = args.get(1) {
            let path = PathBuf::from(file_path_str);
            if path.exists() {
                match app.editor.load_from_file(&path) {
                    Ok(_) => {
                        app.target_path = Some(path);
                        eprintln!("Successfully loaded file: {:?}", file_path_str);
                    }
                    Err(e) => {
                        eprintln!("Error loading file {:?}: {}", file_path_str, e);
                        // エラー時は空のバッファで続行
                        app.target_path = Some(path);
                    }
                }
            } else {
                eprintln!(
                    "File does not exist: {:?}. Creating new file buffer.",
                    file_path_str
                );
                // 存在しないパスでも、そのパスを新しいファイルとして扱う
                app.target_path = Some(path.clone());
            }
        } else {
            eprintln!("No file path provided. Starting with an empty buffer (plain text mode).");
            // 引数がない場合はデフォルトでPlainText
        }
        app
    }

    pub fn save_current_file(&self) -> io::Result<()> {
        if let Some(path) = &self.target_path {
            self.editor.save_to_file(path)?;
            eprintln!("File saved to: {:?}", path);
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "No target path set to save the file.",
            ))
        }
    }
}
