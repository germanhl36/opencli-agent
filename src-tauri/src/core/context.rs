use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use ignore::WalkBuilder;
use crate::error::OpenCLIError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub path: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub modified_at: i64,
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSnapshot {
    pub files: Vec<FileEntry>,
    pub total_tokens: u32,
    pub truncated: bool,
}

const TOKEN_BUDGET: u32 = 32_000;
const CHARS_PER_TOKEN: usize = 4;
const EXCERPT_MAX_BYTES: usize = 8_000;

pub struct ContextBuilder {
    working_dir: PathBuf,
    extra_ignore_patterns: Vec<String>,
}

impl ContextBuilder {
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            working_dir,
            extra_ignore_patterns: Vec::new(),
        }
    }

    pub fn with_ignore_patterns(mut self, patterns: Vec<String>) -> Self {
        self.extra_ignore_patterns = patterns;
        self
    }

    pub fn build_snapshot(&self, token_budget: Option<u32>) -> Result<ContextSnapshot, OpenCLIError> {
        let budget = token_budget.unwrap_or(TOKEN_BUDGET);
        let budget_chars = (budget as usize) * CHARS_PER_TOKEN;

        let mut entries: Vec<(PathBuf, SystemTime, u64)> = Vec::new();

        let walker = WalkBuilder::new(&self.working_dir)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .ignore(true)
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    let path = entry.path().to_path_buf();
                    if !path.is_file() {
                        continue;
                    }

                    // Skip binary files by extension
                    if is_binary_path(&path) {
                        continue;
                    }

                    // Check extra ignore patterns
                    let path_str = path.to_string_lossy().to_string();
                    if self.extra_ignore_patterns.iter().any(|pat| {
                        glob::Pattern::new(pat)
                            .map(|p| p.matches(&path_str))
                            .unwrap_or(false)
                    }) {
                        continue;
                    }

                    let metadata = std::fs::metadata(&path).ok();
                    let modified = metadata
                        .as_ref()
                        .and_then(|m| m.modified().ok())
                        .unwrap_or(SystemTime::UNIX_EPOCH);
                    let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                    entries.push((path, modified, size));
                }
                Err(_) => continue,
            }
        }

        // Sort by modification time descending (most recent first)
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        let mut file_entries: Vec<FileEntry> = Vec::new();
        let mut total_chars = 0usize;
        let mut truncated = false;

        for (path, modified, size) in entries {
            if total_chars >= budget_chars {
                truncated = true;
                break;
            }

            let rel_path = path
                .strip_prefix(&self.working_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            let mime = guess_mime(&path);
            let modified_ts = modified
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let excerpt = if size <= EXCERPT_MAX_BYTES as u64 {
                std::fs::read_to_string(&path).ok().map(|s| {
                    let truncated_excerpt = if s.len() > EXCERPT_MAX_BYTES {
                        s[..EXCERPT_MAX_BYTES].to_string()
                    } else {
                        s
                    };
                    total_chars += truncated_excerpt.len();
                    truncated_excerpt
                })
            } else {
                // Read just a portion for large files
                let partial = read_partial_file(&path, EXCERPT_MAX_BYTES);
                total_chars += partial.as_ref().map(|s| s.len()).unwrap_or(0);
                partial
            };

            file_entries.push(FileEntry {
                path: rel_path,
                mime_type: mime,
                size_bytes: size,
                modified_at: modified_ts,
                excerpt,
            });
        }

        let total_tokens = (total_chars / CHARS_PER_TOKEN) as u32;

        Ok(ContextSnapshot {
            files: file_entries,
            total_tokens,
            truncated,
        })
    }
}

fn is_binary_path(path: &Path) -> bool {
    let binary_extensions = [
        "png", "jpg", "jpeg", "gif", "ico", "svg", "bmp", "webp",
        "pdf", "zip", "tar", "gz", "bz2", "xz", "7z", "rar",
        "exe", "dll", "so", "dylib", "a", "lib",
        "mp3", "mp4", "wav", "avi", "mov", "mkv",
        "woff", "woff2", "ttf", "otf", "eot",
        "lock",
    ];
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        return binary_extensions.contains(&ext_str.as_str());
    }
    false
}

fn guess_mime(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => "text/x-rust",
        Some("ts") | Some("tsx") => "application/typescript",
        Some("js") | Some("jsx") => "application/javascript",
        Some("json") => "application/json",
        Some("yaml") | Some("yml") => "application/yaml",
        Some("toml") => "application/toml",
        Some("md") => "text/markdown",
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("py") => "text/x-python",
        Some("go") => "text/x-go",
        Some("java") => "text/x-java",
        Some("c") | Some("h") => "text/x-c",
        Some("cpp") | Some("hpp") => "text/x-c++",
        Some("sh") => "application/x-sh",
        Some("txt") => "text/plain",
        _ => "text/plain",
    }
    .to_string()
}

fn read_partial_file(path: &Path, max_bytes: usize) -> Option<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = vec![0u8; max_bytes];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);
    String::from_utf8(buf.clone()).ok().or_else(|| {
        Some(String::from_utf8_lossy(&buf).into_owned())
    })
}
