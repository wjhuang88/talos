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
    pub fn as_str(&self) -> &str { &self.0 }
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

#[cfg(test)]
mod tests {
    use super::LanguageId;

    #[test]
    fn aliases_normalize_to_one_identifier() {
        assert_eq!(LanguageId::parse("TSX").unwrap().as_str(), "typescript");
        assert_eq!(LanguageId::parse(".rs").unwrap().as_str(), "rust");
    }
}
