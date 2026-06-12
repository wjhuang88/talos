//! ANSI color constants for CLI terminal output.

/// Reset all formatting.
pub(crate) const RESET: &str = "\x1b[0m";
/// Bold text.
pub(crate) const BOLD: &str = "\x1b[1m";

// Polar Night
/// nord3 — comments, timestamps (RGB: 76, 86, 106).
pub(crate) const NORD3: &str = "\x1b[38;2;76;86;106m";

// Frost
/// nord8 — primary accent, session IDs (RGB: 136, 192, 208).
pub(crate) const NORD8: &str = "\x1b[38;2;136;192;208m";

// Aurora
/// nord13 — warning, snippet highlights (RGB: 235, 203, 139).
pub(crate) const NORD13: &str = "\x1b[38;2;235;203;139m";
/// nord14 — success, project names (RGB: 163, 190, 140).
pub(crate) const NORD14: &str = "\x1b[38;2;163;190;140m";
