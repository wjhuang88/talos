//! Optional source-only rust provider using the existing Arborium grammar.

talos_language_guest::export_provider!("rust", crate::analyze);

/// Highlight source with the same grammar and captures as the built-in provider.
pub fn analyze(source: &str) -> Result<talos_language_guest::Analysis, String> {
    talos_language_guest::analyze("rust", source)
}
