use crate::app::App;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io::{self, Write};

pub fn handle_events(app: &mut App, view_height: usize) -> io::Result<()> {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            let shift = key.modifiers.contains(KeyModifiers::SHIFT);
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

            // Undo/Redo用に編集前の状態を保存
            let mut push_undo = false;

            match key.code {
                KeyCode::Char('z') if ctrl && !shift => { // Ctrl+z
                    if let Some(prev) = app.undo_stack.pop() {
                        app.redo_stack.push(app.buffer.clone());
                        app.buffer = prev;
                    }
                }
                KeyCode::Char('z') if ctrl && shift => { // Ctrl+Shift+z
                    if let Some(next) = app.redo_stack.pop() {
                        app.undo_stack.push(app.buffer.clone());
                        app.buffer = next;
                    }
                }
                KeyCode::Char('x') if ctrl => { // Ctrl+x
                    if let Some((start, end)) = app.selection.take() {
                        let (sy, sx, ey, ex) = normalize_sel(start, end);
                        let (new_buffer, selected) = cut_selection(&app.buffer, sy, sx, ey, ex);
                        app.clipboard = Some(selected);
                        app.undo_stack.push(app.buffer.clone());
                        app.buffer = new_buffer;
                        app.cursor = (sy, sx);
                    }
                }
                KeyCode::Char('c') if ctrl => { // Ctrl+c
                    if let Some((start, end)) = app.selection {
                        let (sy, sx, ey, ex) = normalize_sel(start, end);
                        let selected = get_selection(&app.buffer, sy, sx, ey, ex);
                        app.clipboard = Some(selected);
                    }
                }
                KeyCode::Char('v') if ctrl => { // Ctrl+v
                    if let Some(ref clip) = app.clipboard {
                        push_undo = true;
                        if let Some((start, end)) = app.selection.take() {
                            let (sy, sx, ey, ex) = normalize_sel(start, end);
                            app.buffer = replace_selection(&app.buffer, sy, sx, ey, ex, clip);
                            app.cursor = (sy, sx + clip.chars().count());
                        } else {
                            app.buffer = insert_at(&app.buffer, app.cursor.0, app.cursor.1, clip);
                            app.cursor.1 += clip.chars().count();
                        }
                    }
                }
                KeyCode::Char('s') if ctrl => { // Ctrl+s
                    let save_path = if let Some(ref tmp_path) = app.tmp_file_path {
                        tmp_path.clone()
                    } else if let Some(ref file_path) = app.file_path {
                        file_path.clone()
                    } else {
                        print!("保存先のパスを入力してください: ");
                        io::stdout().flush().ok();
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).ok();
                        input.trim().to_string()
                    };
                    std::fs::write(&save_path, &app.buffer).ok();
                    if app.file_path.is_none() {
                        app.file_path = Some(save_path.clone());
                    }
                    if app.tmp_file_path.is_none() {
                        app.tmp_file_path = Some(save_path);
                    }
                }
                KeyCode::Char(c) if !ctrl => { // 通常の文字入力
                    push_undo = true;
                    if let Some((start, end)) = app.selection.take() {
                        let (sy, sx, ey, ex) = normalize_sel(start, end);
                        if is_bracket(c) {
                            let selected = get_selection(&app.buffer, sy, sx, ey, ex);
                            let wrapped = format!("{}{}{}", c, selected, matching_bracket(c));
                            app.buffer = replace_selection(&app.buffer, sy, sx, ey, ex, &wrapped);
                            app.cursor = (sy, sx + 1 + selected.chars().count());
                        } else {
                            app.buffer = replace_selection(&app.buffer, sy, sx, ey, ex, &c.to_string());
                            app.cursor = (sy, sx + 1);
                        }
                    } else {
                        app.buffer = insert_at(&app.buffer, app.cursor.0, app.cursor.1, &c.to_string());
                        app.cursor.1 += 1;
                    }
                }
                // ...他のキー処理...
                _ => {}
            }

            if push_undo {
                app.undo_stack.push(app.buffer.clone());
                app.redo_stack.clear();
            }
        }
    }
    Ok(())
}

// 選択範囲の正規化
fn normalize_sel(a: (usize, usize), b: (usize, usize)) -> (usize, usize, usize, usize) {
    if a < b { (a.0, a.1, b.0, b.1) } else { (b.0, b.1, a.0, a.1) }
}

// 選択範囲のテキスト取得
fn get_selection(buffer: &str, sy: usize, sx: usize, ey: usize, ex: usize) -> String {
    let lines: Vec<&str> = buffer.lines().collect();
    if sy == ey {
        lines.get(sy).map(|l| l.chars().skip(sx).take(ex - sx).collect()).unwrap_or_default()
    } else {
        let mut result = String::new();
        if let Some(line) = lines.get(sy) {
            result.push_str(&line.chars().skip(sx).collect::<String>());
            result.push('\n');
        }
        for i in sy+1..ey {
            if let Some(line) = lines.get(i) {
                result.push_str(line);
                result.push('\n');
            }
        }
        if let Some(line) = lines.get(ey) {
            result.push_str(&line.chars().take(ex).collect::<String>());
        }
        result
    }
}

// 選択範囲のテキストを削除し、削除部分を返す
fn cut_selection(buffer: &str, sy: usize, sx: usize, ey: usize, ex: usize) -> (String, String) {
    let selected = get_selection(buffer, sy, sx, ey, ex);
    let new_buffer = replace_selection(buffer, sy, sx, ey, ex, "");
    (new_buffer, selected)
}

// 選択範囲のテキストを置換
fn replace_selection(buffer: &str, sy: usize, sx: usize, ey: usize, ex: usize, text: &str) -> String {
    let mut lines: Vec<String> = buffer.lines().map(|l| l.to_string()).collect();
    if sy == ey {
        if let Some(line) = lines.get_mut(sy) {
            let mut chars: Vec<char> = line.chars().collect();
            chars.splice(sx..ex, text.chars());
            *line = chars.into_iter().collect();
        }
    } else {
        let mut new_lines = Vec::new();
        if let Some(line) = lines.get_mut(sy) {
            let mut chars: Vec<char> = line.chars().collect();
            chars.truncate(sx);
            new_lines.push(chars.into_iter().collect::<String>());
        }
        new_lines.extend(text.lines().map(|l| l.to_string()));
        if let Some(line) = lines.get_mut(ey) {
            let mut chars: Vec<char> = line.chars().collect();
            let tail: String = chars.into_iter().skip(ex).collect();
            if let Some(last) = new_lines.last_mut() {
                last.push_str(&tail);
            }
        }
        lines.splice(sy..=ey, new_lines);
    }
    lines.join("\n")
}

// 挿入
fn insert_at(buffer: &str, y: usize, x: usize, text: &str) -> String {
    replace_selection(buffer, y, x, y, x, text)
}

// 括弧判定
fn is_bracket(c: char) -> bool {
    matches!(c, '(' | '[' | '{' | '（' | '「' | '『' | '【' | '＜' | '〈' | '《' | '〔' | '｛' | '“' | '‘')
}
fn matching_bracket(c: char) -> char {
    match c {
        '(' => ')', '[' => ']', '{' => '}',
        '（' => '）', '「' => '」', '『' => '』', '【' => '】', '＜' => '＞', '〈' => '〉', '《' => '》', '〔' => '〕', '｛' => '｝',
        '“' => '”', '‘' => '’',
        _ => c,
    }
}
