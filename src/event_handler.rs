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
                KeyCode::Char('w') if ctrl => {
                    app.should_quit = true;
                }
                KeyCode::Char('z') if ctrl && !shift => {
                    // Ctrl+z
                    if let Some(prev) = app.undo_stack.pop() {
                        app.redo_stack.push(app.buffer.clone());
                        app.buffer = prev;
                    }
                }
                KeyCode::Char('z') if ctrl && shift => {
                    // Ctrl+Shift+z
                    if let Some(next) = app.redo_stack.pop() {
                        app.undo_stack.push(app.buffer.clone());
                        app.buffer = next;
                    }
                }
                KeyCode::Char('x') if ctrl => {
                    // Ctrl+x
                    if let Some((start, end)) = app.selection.take() {
                        let (sy, sx, ey, ex) = normalize_sel(start, end);
                        let (new_buffer, selected) = cut_selection(&app.buffer, sy, sx, ey, ex);
                        app.clipboard = Some(selected);
                        app.undo_stack.push(app.buffer.clone());
                        app.buffer = new_buffer;
                        app.cursor = (sy, sx);
                    }
                }
                KeyCode::Char('c') if ctrl => {
                    // Ctrl+c
                    if let Some((start, end)) = app.selection {
                        let (sy, sx, ey, ex) = normalize_sel(start, end);
                        let selected = get_selection(&app.buffer, sy, sx, ey, ex);
                        app.clipboard = Some(selected);
                    }
                }
                KeyCode::Char('v') if ctrl => {
                    // Ctrl+v
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
                        app.is_saved = false;
                    }
                }
                KeyCode::Char('s') if ctrl => {
                    let save_path = if let Some(ref file_path) = app.file_path {
                        file_path.clone()
                    } else {
                        print!("保存先のパスを入力してください: ");
                        io::stdout().flush().ok();
                        let mut input = String::new();
                        io::stdin().read_line(&mut input).ok();
                        let path = input.trim().to_string();
                        app.file_path = Some(path.clone());
                        path
                    };
                    std::fs::write(&save_path, &app.buffer).ok();
                    app.is_saved = true;
                }
                KeyCode::Char(c) if !ctrl => {
                    // 通常の文字入力
                    push_undo = true;
                    if let Some((start, end)) = app.selection.take() {
                        let (sy, sx, ey, ex) = normalize_sel(start, end);
                        if is_bracket(c) {
                            let selected = get_selection(&app.buffer, sy, sx, ey, ex);
                            let wrapped = format!("{}{}{}", c, selected, matching_bracket(c));
                            app.buffer = replace_selection(&app.buffer, sy, sx, ey, ex, &wrapped);
                            app.cursor = (sy, sx + 1 + selected.chars().count());
                        } else {
                            app.buffer =
                                replace_selection(&app.buffer, sy, sx, ey, ex, &c.to_string());
                            app.cursor = (sy, sx + 1);
                        }
                    } else {
                        app.buffer =
                            insert_at(&app.buffer, app.cursor.0, app.cursor.1, &c.to_string());
                        app.cursor.1 += 1;
                    }
                    app.is_saved = false;
                }
                KeyCode::Backspace => {
                    push_undo = true;
                    if let Some((start, end)) = app.selection.take() {
                        let (sy, sx, ey, ex) = normalize_sel(start, end);
                        app.buffer = replace_selection(&app.buffer, sy, sx, ey, ex, "");
                        app.cursor = (sy, sx);
                    } else if app.cursor.1 > 0 {
                        let y = app.cursor.0;
                        let x = app.cursor.1;
                        app.buffer = replace_selection(&app.buffer, y, x - 1, y, x, "");
                        app.cursor.1 -= 1;
                    } else if app.cursor.0 > 0 {
                        // 行頭でBackspace: 前の行と結合
                        let y = app.cursor.0;
                        let prev_line_len = app
                            .buffer
                            .lines()
                            .nth(y - 1)
                            .map(|l| l.chars().count())
                            .unwrap_or(0);
                        app.buffer = replace_selection(&app.buffer, y - 1, prev_line_len, y, 0, "");
                        app.cursor.0 -= 1;
                        app.cursor.1 = prev_line_len;
                    }
                    app.is_saved = false;
                }
                KeyCode::Enter => {
                    push_undo = true;
                    if let Some((start, end)) = app.selection.take() {
                        let (sy, sx, ey, ex) = normalize_sel(start, end);
                        app.buffer = replace_selection(&app.buffer, sy, sx, ey, ex, "\n");
                        app.cursor = (sy, sx + 1);
                    } else {
                        app.buffer = insert_at(&app.buffer, app.cursor.0, app.cursor.1, "\n");
                        app.cursor.0 += 1;
                        app.cursor.1 = 0;
                    }
                }
                KeyCode::Left => {
                    if app.cursor.1 > 0 {
                        app.cursor.1 -= 1;
                    } else if app.cursor.0 > 0 {
                        app.cursor.0 -= 1;
                        app.cursor.1 = app
                            .buffer
                            .lines()
                            .nth(app.cursor.0)
                            .map(|l| l.chars().count())
                            .unwrap_or(0);
                    }
                    if shift {
                        let sel_start = app.selection.map(|(s, _)| s).unwrap_or(app.cursor);
                        app.selection = Some((sel_start, app.cursor));
                    } else {
                        app.selection = None;
                    }
                }
                KeyCode::Right => {
                    let line = app.buffer.lines().nth(app.cursor.0).unwrap_or("");
                    if app.cursor.1 < line.chars().count() {
                        app.cursor.1 += 1;
                    } else if app.cursor.0 + 1 < app.buffer.lines().count() {
                        app.cursor.0 += 1;
                        app.cursor.1 = 0;
                    }
                    if shift {
                        let sel_start = app.selection.map(|(s, _)| s).unwrap_or(app.cursor);
                        app.selection = Some((sel_start, app.cursor));
                    } else {
                        app.selection = None;
                    }
                }
                KeyCode::Up => {
                    if app.cursor.0 > 0 {
                        app.cursor.0 -= 1;
                        let line = app.buffer.lines().nth(app.cursor.0).unwrap_or("");
                        app.cursor.1 = app.cursor.1.min(line.chars().count());
                        if app.cursor.0 < app.scroll {
                            app.scroll = app.cursor.0;
                        }
                    }
                    if shift {
                        let sel_start = app.selection.map(|(s, _)| s).unwrap_or(app.cursor);
                        app.selection = Some((sel_start, app.cursor));
                    } else {
                        app.selection = None;
                    }
                }
                KeyCode::Down => {
                    let max = app.buffer.lines().count().saturating_sub(1);
                    if app.cursor.0 < max {
                        app.cursor.0 += 1;
                        let line = app.buffer.lines().nth(app.cursor.0).unwrap_or("");
                        app.cursor.1 = app.cursor.1.min(line.chars().count());
                        let view_bottom = app.scroll + view_height;
                        if app.cursor.0 >= view_bottom {
                            app.scroll = app.cursor.0 + 1 - view_height;
                        }
                    }
                    if shift {
                        let sel_start = app.selection.map(|(s, _)| s).unwrap_or(app.cursor);
                        app.selection = Some((sel_start, app.cursor));
                    } else {
                        app.selection = None;
                    }
                }
                KeyCode::Home => {
                    app.cursor.1 = 0;
                    if shift {
                        let sel_start = app.selection.map(|(s, _)| s).unwrap_or(app.cursor);
                        app.selection = Some((sel_start, app.cursor));
                    } else {
                        app.selection = None;
                    }
                }
                KeyCode::End => {
                    let line = app.buffer.lines().nth(app.cursor.0).unwrap_or("");
                    app.cursor.1 = line.chars().count();
                    if shift {
                        let sel_start = app.selection.map(|(s, _)| s).unwrap_or(app.cursor);
                        app.selection = Some((sel_start, app.cursor));
                    } else {
                        app.selection = None;
                    }
                }
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

// --- 以下は補助関数（省略せずにそのまま使ってください） ---

fn normalize_sel(a: (usize, usize), b: (usize, usize)) -> (usize, usize, usize, usize) {
    if a < b {
        (a.0, a.1, b.0, b.1)
    } else {
        (b.0, b.1, a.0, a.1)
    }
}

fn get_selection(buffer: &str, sy: usize, sx: usize, ey: usize, ex: usize) -> String {
    let lines: Vec<&str> = buffer.lines().collect();
    if sy == ey {
        lines
            .get(sy)
            .map(|l| l.chars().skip(sx).take(ex - sx).collect())
            .unwrap_or_default()
    } else {
        let mut result = String::new();
        if let Some(line) = lines.get(sy) {
            result.push_str(&line.chars().skip(sx).collect::<String>());
            result.push('\n');
        }
        for i in sy + 1..ey {
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

fn cut_selection(buffer: &str, sy: usize, sx: usize, ey: usize, ex: usize) -> (String, String) {
    let selected = get_selection(buffer, sy, sx, ey, ex);
    let new_buffer = replace_selection(buffer, sy, sx, ey, ex, "");
    (new_buffer, selected)
}

fn replace_selection(
    buffer: &str,
    sy: usize,
    sx: usize,
    ey: usize,
    ex: usize,
    text: &str,
) -> String {
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
            let chars: Vec<char> = line.chars().collect();
            let tail: String = chars.into_iter().skip(ex).collect();
            if let Some(last) = new_lines.last_mut() {
                last.push_str(&tail);
            }
        }
        lines.splice(sy..=ey, new_lines);
    }
    lines.join("\n")
}

fn insert_at(buffer: &str, y: usize, x: usize, text: &str) -> String {
    replace_selection(buffer, y, x, y, x, text)
}

fn is_bracket(c: char) -> bool {
    matches!(
        c,
        '(' | '[' | '{' | '（' | '「' | '『' | '【' | '＜' | '〈' | '《' | '〔' | '｛' | '“' | '‘'
    )
}
fn matching_bracket(c: char) -> char {
    match c {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '（' => '）',
        '「' => '」',
        '『' => '』',
        '【' => '】',
        '＜' => '＞',
        '〈' => '〉',
        '《' => '》',
        '〔' => '〕',
        '｛' => '｝',
        '“' => '”',
        '‘' => '’',
        _ => c,
    }
}
