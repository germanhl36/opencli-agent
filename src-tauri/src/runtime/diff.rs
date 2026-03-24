use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffLineKind {
    Added,
    Removed,
    Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedDiff {
    pub path: String,
    pub hunks: Vec<Hunk>,
    pub is_new_file: bool,
    pub is_deleted: bool,
}

pub fn generate_diff(path: &str, old_content: &str, new_content: &str) -> UnifiedDiff {
    let diff = TextDiff::from_lines(old_content, new_content);
    let mut hunks = Vec::new();

    for group in diff.grouped_ops(3).iter() {
        let mut lines = Vec::new();
        let first_op = group.first();
        let last_op = group.last();

        let old_start: usize = first_op.map(|op| op.old_range().start + 1).unwrap_or(1);
        let old_end: usize = last_op.map(|op| op.old_range().end).unwrap_or(0);
        let new_start: usize = first_op.map(|op| op.new_range().start + 1).unwrap_or(1);
        let new_end: usize = last_op.map(|op| op.new_range().end).unwrap_or(0);

        let header = format!(
            "@@ -{},{} +{},{} @@",
            old_start,
            old_end.saturating_sub(old_start.saturating_sub(1)),
            new_start,
            new_end.saturating_sub(new_start.saturating_sub(1))
        );

        let mut old_lineno = old_start;
        let mut new_lineno = new_start;

        for op in group {
            for change in diff.iter_changes(op) {
                let (kind, old_no, new_no): (DiffLineKind, Option<u32>, Option<u32>) =
                    match change.tag() {
                        ChangeTag::Delete => {
                            let n = old_lineno as u32;
                            old_lineno += 1;
                            (DiffLineKind::Removed, Some(n), None)
                        }
                        ChangeTag::Insert => {
                            let n = new_lineno as u32;
                            new_lineno += 1;
                            (DiffLineKind::Added, None, Some(n))
                        }
                        ChangeTag::Equal => {
                            let o = old_lineno as u32;
                            let n = new_lineno as u32;
                            old_lineno += 1;
                            new_lineno += 1;
                            (DiffLineKind::Context, Some(o), Some(n))
                        }
                    };
                lines.push(DiffLine {
                    kind,
                    content: change.value().to_string(),
                    old_lineno: old_no,
                    new_lineno: new_no,
                });
            }
        }

        hunks.push(Hunk { header, lines });
    }

    UnifiedDiff {
        path: path.to_string(),
        hunks,
        is_new_file: old_content.is_empty(),
        is_deleted: new_content.is_empty(),
    }
}
