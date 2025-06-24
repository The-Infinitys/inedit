use std::collections::HashSet;

/// カーソル直前の単語を取得します。
pub fn get_prefix(buffer: &str, cursor_y: u16, cursor_x: u16) -> String {
    let lines: Vec<&str> = buffer.lines().collect();
    if (cursor_y as usize) >= lines.len() {
        return String::new();
    }
    let line = lines[cursor_y as usize];
    let chars: Vec<char> = line.chars().collect();
    let mut prefix = String::new();
    for i in (0..cursor_x as usize).rev() {
        let ch = chars.get(i).copied().unwrap_or('\0');
        if ch.is_alphanumeric() || ch == '_' {
            prefix.insert(0, ch);
        } else {
            break;
        }
    }
    prefix
}

/// 補完候補を返します（Rust風のキーワード＋バッファ内の識別子）。
pub fn get_suggestions(buffer: &str, prefix: &str) -> Vec<String> {
    let keywords = [
        "fn", "let", "if", "else", "while", "for", "match", "loop", "struct", "enum", "use", "mod",
        "pub", "mut", "return", "break", "continue", "impl", "trait", "where", "async", "await",
        "unsafe",
    ];
    let mut suggestions: Vec<String> = keywords
        .iter()
        .filter(|kw| kw.starts_with(prefix))
        .map(|s| s.to_string())
        .collect();

    let mut identifiers = HashSet::new();
    for line in buffer.lines() {
        for word in line.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
            if !word.is_empty() && word.starts_with(prefix) && !keywords.contains(&word) {
                identifiers.insert(word.to_string());
            }
        }
    }
    suggestions.extend(identifiers);
    suggestions.sort();
    suggestions.dedup();
    suggestions
}
