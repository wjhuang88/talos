//! UI-neutral text and language contracts.

use serde::{Deserialize, Serialize};

#[cfg(feature = "code-intelligence")]
/// Built-in source-only symbol operations; no filesystem access is performed.
pub mod symbol;

/// Source-only built-in definition, reference, and import operations.
#[cfg(feature = "code-intelligence")]
pub mod symbol_queries;

#[cfg(feature = "code-intelligence")]
fn parse_builtin(language: &str, source: &str) -> Result<arborium::tree_sitter::Tree, String> {
    use arborium::tree_sitter::{ParseOptions, ParseState, Parser};
    use std::ops::ControlFlow;
    let start = std::time::Instant::now();
    let language = LanguageId::parse(language).ok_or("language not loaded")?;
    let mut parser = Parser::new();
    let grammar = arborium::get_language(language.as_str()).ok_or("language not loaded")?;
    parser.set_language(&grammar).map_err(|e| e.to_string())?;
    let mut progress = |_: &ParseState| {
        if start.elapsed().as_millis() > 500 {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
    let tree = parser
        .parse_with_options(
            &mut |offset, _| &source.as_bytes()[offset..],
            None,
            Some(ParseOptions::new().progress_callback(&mut progress)),
        )
        .ok_or_else(|| "parse failed".to_owned())?;
    // Validate before recursive compatibility visitors run. TreeCursor traversal itself
    // uses no Rust recursion, so an adversarial tree cannot exhaust the visitor stack.
    let mut cursor = tree.walk();
    let mut depth = 0usize;
    let mut nodes = 0usize;
    loop {
        nodes += 1;
        if depth > 128 || nodes > 1_000_000 || start.elapsed().as_millis() > 500 {
            return Err("parse budget exceeded".to_owned());
        }
        if cursor.goto_first_child() {
            depth += 1;
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                drop(cursor);
                return Ok(tree);
            }
            depth -= 1;
        }
    }
}

#[cfg(feature = "code-intelligence")]
fn guarded<T>(operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
        .unwrap_or_else(|_| Err("parse failed".to_owned()))
}

/// Built-in, renderer-independent highlighting adapter.
#[cfg(feature = "code-intelligence")]
pub struct BuiltinHighlighter(Option<arborium::Highlighter>);

#[cfg(feature = "code-intelligence")]
impl Default for BuiltinHighlighter {
    fn default() -> Self {
        Self(std::panic::catch_unwind(arborium::Highlighter::new).ok())
    }
}

#[cfg(feature = "code-intelligence")]
impl BuiltinHighlighter {
    /// Highlight a source using the existing built-in grammars, falling back on failure.
    pub fn highlight(&mut self, language: &LanguageId, source: &str) -> HighlightResult {
        let Some(highlighter) = self.0.as_mut() else {
            return HighlightResult::PlainText;
        };
        let start = std::time::Instant::now();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            highlighter.highlight_spans(language.as_str(), source)
        }));
        match result {
            Ok(Ok(spans)) if start.elapsed().as_millis() <= 500 => HighlightResult::Spans(
                spans
                    .into_iter()
                    .map(|s| HighlightSpan {
                        start: s.start as usize,
                        end: s.end as usize,
                        capture: s.capture,
                    })
                    .collect(),
            ),
            _ => HighlightResult::PlainText,
        }
    }

    /// Whether the built-in grammar bundle contains this language.
    pub fn supports(&self, language: &LanguageId) -> bool {
        std::panic::catch_unwind(|| arborium::get_language(language.as_str()).is_some())
            .unwrap_or(false)
    }
}

/// Resolve the existing symbol-tool extension policy, including case sensitivity.
/// Unknown extensions remain unsupported even when a similarly named grammar exists.
pub fn language_for_extension(extension: &str) -> Option<&'static str> {
    match Some(extension) {
        Some("rs") => Some("rust"),
        Some("py") => Some("python"),
        Some("ts") | Some("tsx") => Some("typescript"),
        Some("js") | Some("jsx") | Some("mjs") => Some("javascript"),
        Some("go") => Some("go"),
        Some("java") => Some("java"),
        Some("c") | Some("h") => Some("c"),
        Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") => Some("cpp"),
        Some("cs") => Some("c-sharp"),
        Some("sh") | Some("bash") | Some("zsh") => Some("bash"),
        Some("sql") => Some("sql"),
        Some("ps1") => Some("powershell"),
        Some("lua") => Some("lua"),
        Some("dart") => Some("dart"),
        Some("html") => Some("html"),
        Some("css") => Some("css"),
        Some("json") => Some("json"),
        Some("yaml") | Some("yml") => Some("yaml"),
        Some("toml") => Some("toml"),
        Some("md") => Some("markdown"),
        Some("rb") => Some("ruby"),
        Some("php") => Some("php"),
        Some("kt") | Some("kts") => Some("kotlin"),
        Some("swift") => Some("swift"),
        _ => None,
    }
}

/// Canonical language identifier used by text consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageId(String);

impl LanguageId {
    /// Normalize a user or file-extension alias into a canonical identifier.
    pub fn parse(value: &str) -> Option<Self> {
        let normalized = value.trim().trim_start_matches('.').to_ascii_lowercase();
        let canonical = match normalized.as_str() {
            "rs" | "rust" => "rust",
            "py" | "python" => "python",
            "js" | "jsx" | "javascript" => "javascript",
            "ts" | "tsx" | "typescript" => "typescript",
            "sh" | "bash" | "zsh" => "bash",
            "yml" | "yaml" => "yaml",
            "md" | "markdown" => "markdown",
            "c++" | "cpp" | "cc" | "cxx" => "cpp",
            "c#" | "cs" | "c-sharp" => "c-sharp",
            other if !other.is_empty() => other,
            _ => return None,
        };
        Some(Self(canonical.to_owned()))
    }

    /// Return the canonical identifier string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A source range with a semantic highlight capture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighlightSpan {
    /// Byte offset of the first source byte.
    pub start: usize,
    /// Byte offset immediately after the final source byte.
    pub end: usize,
    /// Stable capture name, independent of any renderer.
    pub capture: String,
}

/// Renderer-independent highlight result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighlightResult {
    /// Parsed spans are available.
    Spans(Vec<HighlightSpan>),
    /// No provider is available; consumers should render plain text.
    PlainText,
}

/// Provider-independent source request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRequest {
    /// Language selected by the caller.
    pub language: LanguageId,
    /// UTF-8 source text.
    pub source: String,
}

/// Renderer-neutral source location used by outline and symbol consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path supplied by the symbol consumer (relative or absolute).
    pub file: String,
    /// One-based line number.
    pub line: usize,
    /// One-based column number, or zero when only the line is known.
    pub column: usize,
}

/// A renderer-neutral symbol description.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolInfo {
    /// Symbol name.
    pub name: String,
    /// Stable semantic kind.
    pub kind: String,
    /// Workspace-relative file path.
    pub file: String,
    /// One-based line number.
    pub line: usize,
}

#[cfg(test)]
mod tests {
    use super::{HighlightResult, HighlightSpan, LanguageId};

    #[test]
    fn aliases_normalize_to_one_identifier() {
        assert_eq!(LanguageId::parse("TSX").unwrap().as_str(), "typescript");
        assert_eq!(LanguageId::parse(".rs").unwrap().as_str(), "rust");
    }

    #[test]
    fn highlight_result_is_renderer_neutral_and_serializable() {
        let result = HighlightResult::Spans(vec![HighlightSpan {
            start: 0,
            end: 3,
            capture: "keyword".into(),
        }]);
        let encoded = serde_json::to_string(&result).unwrap();
        assert!(encoded.contains("keyword"));
    }

    #[test]
    fn language_ids_trim_extensions_and_reject_empty_values() {
        assert_eq!(LanguageId::parse(" .PY ").unwrap().as_str(), "python");
        assert!(LanguageId::parse("   ").is_none());
    }
}
