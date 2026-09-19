//! Optional source-only python provider using the existing Arborium grammar.

talos_language_guest::export_provider!("python", crate::analyze);

/// Highlight source with the same grammar and captures as the built-in provider.
pub fn analyze(source: &str) -> Result<talos_language_guest::Analysis, String> {
    talos_language_guest::analyze("python", source)
}
