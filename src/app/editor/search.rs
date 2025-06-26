/// テキストバッファ内でクエリに一致する位置をすべて返します。
/// 戻り値は (行番号, 文字インデックス) のベクタです。
pub fn search_all(buffer: &str, query: &str) -> Vec<(u16, u16)> {
    let mut matches = Vec::new();
    if query.is_empty() {
        return matches;
    }
    for (y, line) in buffer.lines().enumerate() {
        for (byte_x, _) in line.match_indices(query) {
            let char_x = line[..byte_x].chars().count() as u16;
            matches.push((y as u16, char_x));
        }
    }
    matches
}

/// 最初に一致した位置を返します。なければNone。
pub fn search_first(buffer: &str, query: &str) -> Option<(u16, u16)> {
    search_all(buffer, query).into_iter().next()
}
