use std::path::Path;

/// 各行のdiff記号（' ', '+', '-'など）を返す
pub fn get_diff_marks<P: AsRef<Path>>(file_path: P, buffer: &str) -> Option<Vec<char>> {
    use git2::{DiffLineType, DiffOptions, Repository};
    let repo = Repository::discover(file_path.as_ref().parent()?).ok()?;
    let rel_path = file_path.as_ref().strip_prefix(repo.workdir()?).ok()?;
    let mut opts = DiffOptions::new();
    opts.pathspec(rel_path);

    let head = repo.head().ok()?.peel_to_tree().ok()?;
    let diff = repo
        .diff_tree_to_workdir_with_index(Some(&head), Some(&mut opts))
        .ok()?;

    // bufferの行数を数える（日本語対応のためlines()でOK）
    let mut marks = vec![' '; buffer.lines().count()];

    diff.foreach(
        &mut |delta, _| {
            // 対象ファイルのみ
            if let Some(path) = delta.new_file().path() {
                if path == rel_path {
                    return true;
                }
            }
            false
        },
        None,
        None,
        Some(&mut |delta, _hunk, line| {
            // 対象ファイルのみ
            if let Some(path) = delta.new_file().path() {
                if path != rel_path {
                    return true;
                }
            }
            match line.origin_value() {
                DiffLineType::Addition => {
                    if let Some(idx) = line.new_lineno() {
                        let idx = idx as usize - 1;
                        if idx < marks.len() {
                            marks[idx] = '+';
                        }
                    }
                }
                DiffLineType::Deletion => {
                    if let Some(idx) = line.old_lineno() {
                        let idx = idx as usize - 1;
                        if idx < marks.len() {
                            marks[idx] = '-';
                        }
                    }
                }
                _ => {}
            }
            true
        }),
    )
    .ok()?;

    Some(marks)
}
