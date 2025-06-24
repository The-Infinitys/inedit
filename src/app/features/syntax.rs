//! シンタックスハイライト機能と、言語の自動判別ロジックを提供します。
use std::{collections::HashMap, path::Path};
use syntect::{
    parsing::{Regex, SyntaxReference, SyntaxSet},
    highlighting::{ThemeSet, Style as SyntectStyle, Theme},
    util::LinesWith	Endings,
};
use ratatui::style::{Color, Modifier, Style}; // ratatuiのスタイルをインポート

/// シンタックスハイライト処理に必要なデータを保持します。
pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    themes: HashMap<String, Theme>,
    pub current_theme_name: String,
}

impl Default for Highlighter {
    fn default() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let current_theme_name = "InspiredGitHub".to_string(); // デフォルトテーマ

        let mut themes = HashMap::new();
        themes.insert(current_theme_name.clone(), theme_set.themes[&current_theme_name].clone());

        Highlighter {
            syntax_set,
            theme_set,
            themes,
            current_theme_name,
        }
    }
}

impl Highlighter {
    /// デフォルトのテーマを使用してハイライターを初期化します。
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

    /// 指定された行に対してシンタックスハイライトを適用し、`ratatui::style::Style`のVecを返します。
    pub fn highlight_line<'a>(
        &self,
        line: &'a str,
        syntax: &SyntaxReference,
    ) -> Vec<(SyntectStyle, &'a str)> {
        let mut highlighter = syntect::highlighting::CodeHighlighter::new(syntax, self.get_current_theme());
        highlighter.highlight_line(line, &self.syntax_set)
            .unwrap_or_else(|_| {
                // エラー時は単一のデフォルトスタイルを返す
                vec![(SyntectStyle::default(), line)]
            })
    }

    /// `syntect`のスタイルを`ratatui`のスタイルに変換します。
    pub fn convert_syntect_style_to_ratatui_style(s: SyntectStyle) -> Style {
        let mut style = Style::default();

        if let Some(fg) = s.foreground {
            style = style.fg(convert_syntect_color(fg));
        }
        if let Some(bg) = s.background {
            style = style.bg(convert_syntect_color(bg));
        }

        if s.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
            style = style.add_modifier(Modifier::BOLD);
        }
        if s.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
            style = style.add_modifier(Modifier::ITALIC);
        }
        if s.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
            style = style.add_modifier(Modifier::UNDERLINED);
        }
        style
    }
}

/// `syntect`の`Color`を`ratatui`の`Color`に変換します。
fn convert_syntect_color(c: syntect::highlighting::Color) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}

// App構造体からHighlighterを利用するために、App::init()でHighlighterを初期化し、
// AppのフィールドとしてHighlighterを持つ必要があります。
// 例: pub highlighter: Highlighter,
// また、App::init()内でファイルの読み込み後、最初の数行とファイルパスから
// self.editor.current_syntax を決定するようにします。
