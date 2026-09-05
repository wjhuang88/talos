//! UI-neutral text and language contracts.

use serde::{Deserialize, Serialize};

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
    /// Workspace-relative file path.
    pub file: String,
    /// One-based line number.
    pub line: usize,
    /// Zero-based column number.
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
