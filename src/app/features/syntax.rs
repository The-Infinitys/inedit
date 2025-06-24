//! シンタックスハイライト機能と、言語の自動判別ロジックを提供します。
use std::{collections::HashMap, path::Path};
use syntect::{
    parsing::{SyntaxReference, SyntaxSet},
    highlighting::{ThemeSet, Style as SyntectStyle, Theme, FontStyle, Color as SyntectColor},
    easy::HighlightLines,
};
use ratatui::style::{Color, Modifier, Style};

/// シンタックスハイライト処理に必要なデータを保持します。
pub struct Highlighter {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    themes: HashMap<String, Theme>,
    pub current_theme_name: String,
}

impl Default for Highlighter {
    fn default() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let current_theme_name = "InspiredGitHub".to_string(); // デフォルトテーマ

        let mut themes = HashMap::new();
        // ThemeSetから全てのテーマをHashMapにコピーして保持
        for (name, theme) in theme_set.themes.iter() {
            themes.insert(name.clone(), theme.clone());
        }

        Highlighter {
            syntax_set,
            theme_set,
            themes,
            current_theme_name,
        }
    }
}

impl Highlighter {
    /// 新しいHighlighterインスタンスを作成します。
    pub fn new() -> Self {
        Self::default()
    }

    /// ファイルパスから最適なシンタックスを推測します。
    pub fn get_syntax_for_file(&self, path: Option<&Path>, first_lines: &str) -> &SyntaxReference {
        if let Some(path) = path {
            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                if let Some(syntax) = self.syntax_set.find_syntax_by_extension(extension) {
                    return syntax;
                }
            }
        }
        // Shebang (例: #!/bin/bash) から言語を推測
        if let Some(syntax) = self.syntax_set.find_syntax_by_first_line(first_lines) {
            return syntax;
        }
        // デフォルトのプレーンテキストを返す
        self.syntax_set.find_syntax_plain_text()
    }

    /// 現在のテーマを取得します。
    pub fn get_current_theme(&self) -> &Theme {
        self.themes.get(&self.current_theme_name)
            .unwrap_or_else(|| &self.theme_set.themes["InspiredGitHub"]) // フォールバック
    }

    /// 指定された行に対してシンタックスハイライトを適用し、`syntect`のスタイルとテキストのペアのVecを返します。
    pub fn highlight_line<'a>(
        &self,
        line: &'a str,
        syntax: &SyntaxReference,
    ) -> Vec<(SyntectStyle, &'a str)> {
        let mut highlighter = HighlightLines::new(syntax, self.get_current_theme());
        highlighter.highlight_line(line, &self.syntax_set)
            .unwrap_or_else(|_| {
                vec![(SyntectStyle::default(), line)]
            })
    }

    /// `syntect`のスタイルを`ratatui`のスタイルに変換します。
    pub fn convert_syntect_style_to_ratatui_style(s: SyntectStyle) -> Style {
        let mut style = Style::default();

        style = style.fg(convert_syntect_color(s.foreground));
        style = style.bg(convert_syntect_color(s.background));

        if s.font_style.contains(FontStyle::BOLD) {
            style = style.add_modifier(Modifier::BOLD);
        }
        if s.font_style.contains(FontStyle::ITALIC) {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if s.font_style.contains(FontStyle::UNDERLINE) {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        style
    }

    /// テーマを切り替えます。成功した場合はtrueを返します。
    pub fn set_theme(&mut self, theme_name: &str) -> bool {
        if self.themes.contains_key(theme_name) {
            self.current_theme_name = theme_name.to_string();
            true
        } else {
            // ThemeSetからロードを試みる
            if let Some(theme) = self.theme_set.themes.get(theme_name) {
                self.themes.insert(theme_name.to_string(), theme.clone());
                self.current_theme_name = theme_name.to_string();
                true
            } else {
                false
            }
        }
    }

    /// 利用可能なテーマ名のリストを返します。
    pub fn list_themes(&self) -> Vec<String> {
        let mut themes: Vec<String> = self.theme_set.themes.keys().cloned().collect();
        themes.sort_unstable();
        themes
    }

    /// 現在のテーマの背景色を`ratatui::style::Color`で取得します。
    pub fn get_background_color(&self) -> Color {
        let theme = self.get_current_theme();
        theme.settings.background
            .map_or(Color::Black, convert_syntect_color)
    }

    /// 現在のテーマの前景色（基本テキスト色）を`ratatui::style::Color`で取得します。
    pub fn get_foreground_color(&self) -> Color {
        let theme = self.get_current_theme();
        theme.settings.foreground
            .map_or(Color::White, convert_syntect_color)
    }
}

/// `syntect`の`Color`を`ratatui`の`Color`に変換します。
fn convert_syntect_color(c: SyntectColor) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}
