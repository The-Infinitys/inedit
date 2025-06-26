use super::Editor;

impl Editor {
    /// 現在のカーソル位置からコード補完の候補を取得します。（非常に簡易版）
    /// 実際の補完は、言語サーバープロトコル (LSP) などで行われるのが一般的です。
    pub fn get_completion_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        let lines: Vec<&str> = self.buffer.lines().collect();
        let current_y = self.cursor.y as usize;

        if current_y >= lines.len() {
            return suggestions;
        }

        let current_line_chars: Vec<char> = lines[current_y].chars().collect();
        let current_x = self.cursor.x as usize;

        // カーソル前の単語を取得
        let mut prefix = String::new();
        // カーソルの1つ前から逆順に走査
        for i in (0..current_x).rev() {
            let ch = current_line_chars[i];
            // 単語を構成する文字（英数字とアンダースコア）を定義
            if ch.is_alphanumeric() || ch == '_' {
                prefix.insert(0, ch); // 正しい順序で単語を構築するため先頭に挿入
            } else {
                break; // 単語以外の文字に遭遇したら停止
            }
        }

        if prefix.is_empty() {
            return suggestions;
        }

        // 仮のキーワードリスト (Rust風)
        let keywords = vec![
            "fn", "let", "if", "else", "while", "for", "match", "loop", "struct", "enum", "use",
            "mod", "pub", "mut", "return", "break", "continue", "impl", "trait", "where", "async",
            "await", "unsafe",
        ];

        // 仮の識別子リスト (バッファ内の単語から取得)
        let mut identifiers: std::collections::HashSet<String> = std::collections::HashSet::new();
        for line_str in self.buffer.lines() {
            // 英数字とアンダースコア以外の文字で単語を分割
            for word in line_str.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
                if !word.is_empty() && !keywords.contains(&word) {
                    identifiers.insert(word.to_string());
                }
            }
        }

        // プレフィックスにマッチするキーワードを追加
        for keyword in keywords {
            if keyword.starts_with(&prefix) {
                suggestions.push(keyword.to_string());
            }
        }

        // プレフィックスにマッチする識別子を追加
        for id in identifiers.iter() {
            if id.starts_with(&prefix) {
                suggestions.push(id.clone());
            }
        }

        suggestions.sort_unstable(); // 候補をソート
        suggestions.dedup(); // 重複を削除
        suggestions
    }
}