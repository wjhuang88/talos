//! Search engine abstraction with legacy and ripgrep-backed implementations.
//!
//! This module defines the [`SearchEngine`] trait and provides two implementations:
//! - [`LegacySearchEngine`]: The original regex + walkdir implementation.
//! - [`RipgrepSearchEngine`]: A ripgrep library-backed implementation using
//!   `grep-searcher`, `grep-regex`, and `ignore` crates.

use std::fmt;
use std::path::Path;
use std::sync::Arc;

use grep_regex::RegexMatcher;
use grep_searcher::{BinaryDetection, SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;

/// Errors that can occur during search operations.
#[derive(Debug)]
pub enum SearchError {
    /// Invalid regex pattern.
    InvalidRegex(String),
    /// IO error during file reading or walking.
    Io(std::io::Error),
    /// A panic occurred in the search engine (should not happen in pure Rust).
    SearchPanic(String),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchError::InvalidRegex(e) => write!(f, "invalid regex: {e}"),
            SearchError::Io(e) => write!(f, "io error: {e}"),
            SearchError::SearchPanic(e) => write!(f, "search engine panic: {e}"),
        }
    }
}

impl std::error::Error for SearchError {}

impl From<std::io::Error> for SearchError {
    fn from(e: std::io::Error) -> Self {
        SearchError::Io(e)
    }
}

/// Matches found in a single file.
#[derive(Debug)]
pub struct FileMatches {
    /// Relative path to the file.
    pub path: String,
    /// Matching lines as (1-based line number, content) pairs.
    pub lines: Vec<(usize, String)>,
}

/// Output from a search operation.
#[derive(Debug)]
pub struct SearchOutput {
    /// All matches grouped by file.
    pub matches: Vec<FileMatches>,
    /// Whether results were truncated due to max_results limit.
    pub truncated: bool,
}

/// Trait for search engine implementations.
pub trait SearchEngine: Send + Sync {
    /// Search for `pattern` in files under `search_path`.
    ///
    /// - `include`: Optional glob pattern to filter files by basename.
    /// - `max_results`: Maximum total matches to return.
    fn search(
        &self,
        pattern: &str,
        search_path: &Path,
        include: Option<&glob::Pattern>,
        max_results: usize,
    ) -> Result<SearchOutput, SearchError>;
}

/// Legacy search engine using regex + walkdir (preserves exact current behavior).
pub struct LegacySearchEngine;

impl SearchEngine for LegacySearchEngine {
    fn search(
        &self,
        pattern: &str,
        search_path: &Path,
        include: Option<&glob::Pattern>,
        max_results: usize,
    ) -> Result<SearchOutput, SearchError> {
        use crate::file_tools::{is_binary_file, is_skip_dir};

        let re =
            regex::Regex::new(pattern).map_err(|e| SearchError::InvalidRegex(e.to_string()))?;

        let files: Vec<std::path::PathBuf> = if search_path.is_file() {
            vec![search_path.to_path_buf()]
        } else {
            walkdir::WalkDir::new(search_path)
                .into_iter()
                .filter_entry(|e| {
                    if e.depth() == 0 {
                        return true;
                    }
                    !(e.file_type().is_dir() && is_skip_dir(&e.file_name().to_string_lossy()))
                })
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect()
        };

        let root = search_path
            .canonicalize()
            .unwrap_or_else(|_| search_path.to_path_buf());

        let mut matches: Vec<(String, usize, String)> = Vec::new();

        for file_path in &files {
            if matches.len() >= max_results {
                break;
            }

            if let Some(pat) = include {
                let file_name = file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !pat.matches(&file_name) {
                    continue;
                }
            }

            if is_binary_file(file_path).map_err(|e| SearchError::Io(std::io::Error::other(e)))? {
                continue;
            }

            let content = match std::fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let display_path = file_path
                .strip_prefix(&root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            for (i, line) in content.lines().enumerate() {
                if matches.len() >= max_results {
                    break;
                }
                if re.is_match(line) {
                    matches.push((display_path.clone(), i + 1, line.trim_end().to_string()));
                }
            }
        }

        let mut grouped: Vec<FileMatches> = Vec::new();
        let mut current_file = String::new();
        for (file, line_num, line) in matches {
            if file != current_file {
                grouped.push(FileMatches {
                    path: file.clone(),
                    lines: Vec::new(),
                });
                current_file = file;
            }
            if let Some(fm) = grouped.last_mut() {
                fm.lines.push((line_num, line));
            }
        }

        Ok(SearchOutput {
            matches: grouped,
            truncated: false,
        })
    }
}

/// Ripgrep-backed search engine using grep-searcher, grep-regex, and ignore crates.
///
/// Respects `.gitignore` files (new capability) while preserving skip rules for
/// hidden directories, `target/`, and `node_modules/`.
pub struct RipgrepSearchEngine;

impl SearchEngine for RipgrepSearchEngine {
    fn search(
        &self,
        pattern: &str,
        search_path: &Path,
        include: Option<&glob::Pattern>,
        max_results: usize,
    ) -> Result<SearchOutput, SearchError> {
        let matcher =
            RegexMatcher::new(pattern).map_err(|e| SearchError::InvalidRegex(e.to_string()))?;

        let files: Vec<std::path::PathBuf> = if search_path.is_file() {
            vec![search_path.to_path_buf()]
        } else {
            let mut walker_builder = WalkBuilder::new(search_path);
            walker_builder.hidden(true);
            walker_builder.git_ignore(true);
            walker_builder.git_global(false);
            walker_builder.require_git(false);

            let mut overrides = OverrideBuilder::new(search_path);
            overrides
                .add("!target/")
                .map_err(|e| SearchError::InvalidRegex(e.to_string()))?;
            overrides
                .add("!node_modules/")
                .map_err(|e| SearchError::InvalidRegex(e.to_string()))?;
            walker_builder.overrides(
                overrides
                    .build()
                    .map_err(|e| SearchError::InvalidRegex(e.to_string()))?,
            );

            walker_builder
                .build()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .map(|e| e.path().to_path_buf())
                .collect()
        };

        let root = if search_path.is_file() {
            search_path
                .parent()
                .and_then(|p| p.canonicalize().ok())
                .unwrap_or_else(|| search_path.to_path_buf())
        } else {
            search_path
                .canonicalize()
                .unwrap_or_else(|_| search_path.to_path_buf())
        };

        let max_results = Arc::new(std::sync::atomic::AtomicUsize::new(max_results));
        let matches: Arc<std::sync::Mutex<Vec<(String, usize, String)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        for file_path in &files {
            if max_results.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                break;
            }

            if let Some(pat) = include {
                let file_name = file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !pat.matches(&file_name) {
                    continue;
                }
            }

            let display_path = file_path
                .strip_prefix(&root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            let matches_ref = Arc::clone(&matches);
            let max_results_ref = Arc::clone(&max_results);
            let display_path_ref = display_path.clone();
            let file_path = file_path.clone();

            let search_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut searcher = SearcherBuilder::new()
                    .binary_detection(BinaryDetection::quit(b'\x00'))
                    .build();

                struct LineSink<'a> {
                    matches: &'a std::sync::Mutex<Vec<(String, usize, String)>>,
                    max_results: &'a std::sync::atomic::AtomicUsize,
                    path: String,
                }

                impl Sink for LineSink<'_> {
                    type Error = std::io::Error;

                    fn matched(
                        &mut self,
                        _searcher: &grep_searcher::Searcher,
                        mat: &SinkMatch<'_>,
                    ) -> Result<bool, Self::Error> {
                        let current = self.max_results.load(std::sync::atomic::Ordering::SeqCst);
                        if current == 0 {
                            return Ok(false);
                        }
                        self.max_results
                            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);

                        let line = String::from_utf8_lossy(mat.bytes()).trim_end().to_string();
                        self.matches
                            .lock()
                            .map_err(|e| std::io::Error::other(format!("mutex poisoned: {e}")))?
                            .push((
                                self.path.clone(),
                                mat.line_number().unwrap_or(0) as usize,
                                line,
                            ));
                        Ok(true)
                    }
                }

                let sink = LineSink {
                    matches: &matches_ref,
                    max_results: &max_results_ref,
                    path: display_path_ref,
                };

                searcher.search_path(&matcher, &file_path, sink)
            }));

            match search_result {
                Ok(Ok(_)) => {}
                Ok(Err(_)) => {}
                Err(panic) => {
                    let msg = if let Some(s) = panic.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = panic.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "unknown panic".to_string()
                    };
                    return Err(SearchError::SearchPanic(msg));
                }
            }

            if max_results.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                break;
            }
        }

        let collected = matches
            .lock()
            .map_err(|e| SearchError::SearchPanic(format!("mutex poisoned: {e}")))?;
        let mut grouped: Vec<FileMatches> = Vec::new();
        let mut current_file = String::new();
        for (file, line_num, line) in collected.iter() {
            if file != &current_file {
                grouped.push(FileMatches {
                    path: file.clone(),
                    lines: Vec::new(),
                });
                current_file = file.clone();
            }
            if let Some(fm) = grouped.last_mut() {
                fm.lines.push((*line_num, line.clone()));
            }
        }

        Ok(SearchOutput {
            matches: grouped,
            truncated: false,
        })
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    use std::fs;

    fn engine() -> RipgrepSearchEngine {
        RipgrepSearchEngine
    }

    #[test]
    fn test_gitignore_respected() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("visible.rs"), "fn target() {}\n").unwrap();
        fs::write(dir.path().join("ignored.rs"), "fn target() {}\n").unwrap();
        fs::write(dir.path().join(".gitignore"), "ignored.rs\n").unwrap();

        let output = engine().search("target", dir.path(), None, 50).unwrap();

        let files: Vec<&str> = output.matches.iter().map(|m| m.path.as_str()).collect();
        assert!(files.iter().any(|f| f.contains("visible.rs")));
        assert!(!files.iter().any(|f| f.contains("ignored.rs")));
    }

    #[test]
    fn test_binary_file_skipped() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("text.rs"), "fn target() {}\n").unwrap();
        let mut binary = vec![b'h', b'e', b'l', b'l', b'o'];
        binary.push(0u8);
        binary.extend_from_slice(b"fn target() {}\n");
        fs::write(dir.path().join("binary.rs"), &binary).unwrap();

        let output = engine().search("target", dir.path(), None, 50).unwrap();

        let files: Vec<&str> = output.matches.iter().map(|m| m.path.as_str()).collect();
        assert!(files.iter().any(|f| f.contains("text.rs")));
        assert!(!files.iter().any(|f| f.contains("binary.rs")));
    }

    #[test]
    fn test_max_results_truncation() {
        let dir = tempfile::tempdir().unwrap();
        let mut content = String::new();
        for i in 0..20 {
            content.push_str(&format!("fn match_{}() {{}}\n", i));
        }
        fs::write(dir.path().join("many.rs"), &content).unwrap();

        let output = engine().search("match_", dir.path(), None, 5).unwrap();

        let total: usize = output.matches.iter().map(|m| m.lines.len()).sum();
        assert_eq!(total, 5);
    }

    #[test]
    fn test_unicode_content() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("unicode.rs"),
            "fn 你好() {}\nfn hello() {}\nfn 世界() {}\n",
        )
        .unwrap();

        let output = engine().search("你好", dir.path(), None, 50).unwrap();

        assert_eq!(output.matches.len(), 1);
        assert_eq!(output.matches[0].lines.len(), 1);
        assert!(output.matches[0].lines[0].1.contains("你好"));
    }

    #[test]
    fn test_include_with_path_scope() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn target() {}\n").unwrap();
        fs::write(dir.path().join("main.txt"), "fn target() {}\n").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "fn target() {}\n").unwrap();
        fs::write(dir.path().join("src/lib.txt"), "fn target() {}\n").unwrap();

        let pat = glob::Pattern::new("*.rs").unwrap();
        let output = engine()
            .search("target", dir.path(), Some(&pat), 50)
            .unwrap();

        let files: Vec<&str> = output.matches.iter().map(|m| m.path.as_str()).collect();
        assert!(files.iter().any(|f| f.contains("main.rs")));
        assert!(files.iter().any(|f| f.contains("lib.rs")));
        assert!(!files.iter().any(|f| f.ends_with(".txt")));
    }

    #[test]
    fn test_hidden_dir_at_depth_0() {
        let hidden_dir = tempfile::tempdir().unwrap();
        let hidden_path = hidden_dir.path().join(".hidden_project");
        fs::create_dir_all(&hidden_path).unwrap();
        fs::write(hidden_path.join("code.rs"), "fn target() {}\n").unwrap();

        let output = engine().search("target", &hidden_path, None, 50).unwrap();

        assert_eq!(output.matches.len(), 1);
        assert!(output.matches[0].path.contains("code.rs"));
    }

    #[test]
    fn test_target_dir_skipped() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("src.rs"), "fn target() {}\n").unwrap();
        fs::create_dir_all(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("target/build.rs"), "fn target() {}\n").unwrap();

        let output = engine().search("target", dir.path(), None, 50).unwrap();

        let files: Vec<&str> = output.matches.iter().map(|m| m.path.as_str()).collect();
        assert!(files.iter().any(|f| f.contains("src.rs")));
        assert!(!files.iter().any(|f| f.contains("target/")));
    }

    #[test]
    fn test_node_modules_dir_skipped() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("app.js"), "const target = 1;\n").unwrap();
        fs::create_dir_all(dir.path().join("node_modules/pkg")).unwrap();
        fs::write(
            dir.path().join("node_modules/pkg/index.js"),
            "const target = 2;\n",
        )
        .unwrap();

        let output = engine().search("target", dir.path(), None, 50).unwrap();

        let files: Vec<&str> = output.matches.iter().map(|m| m.path.as_str()).collect();
        assert!(files.iter().any(|f| f.contains("app.js")));
        assert!(!files.iter().any(|f| f.contains("node_modules/")));
    }

    #[test]
    fn test_single_file_search() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn hello() {}\nfn world() {}\n").unwrap();

        let output = engine()
            .search("hello", &dir.path().join("test.rs"), None, 50)
            .unwrap();

        assert_eq!(output.matches.len(), 1);
        assert!(output.matches[0].path.contains("test.rs"));
        assert_eq!(output.matches[0].lines.len(), 1);
        assert_eq!(output.matches[0].lines[0].0, 1);
    }

    #[test]
    fn test_no_matches_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("empty.rs"), "fn other() {}\n").unwrap();

        let output = engine()
            .search("nonexistent_xyz", dir.path(), None, 50)
            .unwrap();

        assert!(output.matches.is_empty());
    }

    #[test]
    fn test_invalid_regex_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "fn test() {}\n").unwrap();

        let result = engine().search("[invalid", dir.path(), None, 50);
        assert!(result.is_err());
        match result.unwrap_err() {
            SearchError::InvalidRegex(_) => {}
            other => panic!("expected InvalidRegex, got {:?}", other),
        }
    }

    #[test]
    fn test_legacy_parity_basic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn hello() {}\nfn world() {}\n").unwrap();
        fs::write(dir.path().join("b.txt"), "hello world\nfoo bar\n").unwrap();
        fs::create_dir_all(dir.path().join("sub")).unwrap();
        fs::write(
            dir.path().join("sub/c.rs"),
            "hello from sub\nanother line\n",
        )
        .unwrap();

        let ripgrep_out = RipgrepSearchEngine
            .search("hello", dir.path(), None, 50)
            .unwrap();
        let legacy_out = LegacySearchEngine
            .search("hello", dir.path(), None, 50)
            .unwrap();

        let ripgrep_total: usize = ripgrep_out.matches.iter().map(|m| m.lines.len()).sum();
        let legacy_total: usize = legacy_out.matches.iter().map(|m| m.lines.len()).sum();
        assert_eq!(ripgrep_total, legacy_total);
    }
}
