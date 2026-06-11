//! TurnObserver — captures signals during agent execution.

use crate::{Observation, SignalType};

/// Captures observations during agent turns.
pub struct TurnObserver {
    /// Current session ID
    session_id: Option<String>,
    /// Current turn number
    turn_number: u32,
    /// Accumulated observations for current turn
    observations: Vec<Observation>,
}

impl TurnObserver {
    /// Create a new TurnObserver.
    pub fn new(session_id: Option<String>) -> Self {
        Self {
            session_id,
            turn_number: 0,
            observations: Vec::new(),
        }
    }

    /// Find the byte offset of the first matching marker phrase in `text`.
    ///
    /// Searches case-insensitively. Returns `Some(byte_offset)` of the match
    /// start, or `None` if no marker is found.
    pub fn find_marker(text: &str, markers: &[&str]) -> Option<usize> {
        let lower = text.to_lowercase();
        for marker in markers {
            if let Some(pos) = lower.find(&marker.to_lowercase()) {
                return Some(pos);
            }
        }
        None
    }

    /// Extract a context window centered on `marker_pos`.
    ///
    /// Returns approximately `window_bytes / 2` bytes before and after the
    /// marker, with the marker phrase centered. Respects UTF-8 char boundaries.
    pub fn capture_window(text: &str, marker_pos: usize, window_bytes: usize) -> String {
        let half = window_bytes / 2;
        let text_len = text.len();

        let start = marker_pos.saturating_sub(half);
        let end = (marker_pos + half).min(text_len);

        let start = text
            .char_indices()
            .rev()
            .find(|(i, _)| *i <= start)
            .map(|(i, _)| i)
            .unwrap_or(0);

        let end = text
            .char_indices()
            .find(|(i, _)| *i >= end)
            .map(|(i, c)| (i + c.len_utf8()).min(text_len))
            .unwrap_or(text_len);

        let window = &text[start..end];

        let mut result = String::with_capacity(window.len() + 8);
        if start > 0 {
            result.push_str("...");
        }
        result.push_str(window);
        if end < text_len {
            result.push_str("...");
        }
        result
    }

    /// Truncate context to fit `max_bytes`, appending a marker if truncated.
    #[deprecated(
        since = "0.2.0",
        note = "Use find_marker + capture_window instead. This function keeps the head of the string, losing the actual signal."
    )]
    #[allow(deprecated)]
    pub fn truncate_context(context: String, max_bytes: usize) -> String {
        if max_bytes == 0 {
            return format!("... [truncated, original was {} bytes]", context.len());
        }
        let byte_len = context.len();
        if byte_len <= max_bytes {
            return context;
        }
        let marker = format!("... [truncated, original was {byte_len} bytes]");
        let marker_len = marker.len();
        if marker_len >= max_bytes {
            return marker;
        }
        let truncate_at = max_bytes - marker_len;
        let mut result = String::with_capacity(max_bytes);
        result.push_str(&context[..truncate_at]);
        result.push_str(&marker);
        result
    }

    /// Start a new turn.
    pub fn start_turn(&mut self) {
        self.turn_number += 1;
        self.observations.clear();
    }

    /// Record a correction signal.
    pub fn record_correction(&mut self, context: String, intensity: f64) {
        let obs = Observation::new(
            SignalType::Correction,
            intensity.clamp(0.0, 1.0),
            context,
            self.session_id.clone(),
            Some(self.turn_number),
        );
        self.observations.push(obs);
    }

    /// Record an error signal.
    pub fn record_error(&mut self, context: String, intensity: f64) {
        let obs = Observation::new(
            SignalType::Error,
            intensity.clamp(0.0, 1.0),
            context,
            self.session_id.clone(),
            Some(self.turn_number),
        );
        self.observations.push(obs);
    }

    /// Record a satisfaction signal.
    pub fn record_satisfaction(&mut self, context: String, intensity: f64) {
        let obs = Observation::new(
            SignalType::Satisfaction,
            intensity.clamp(0.0, 1.0),
            context,
            self.session_id.clone(),
            Some(self.turn_number),
        );
        self.observations.push(obs);
    }

    /// Record an inefficiency signal.
    pub fn record_inefficiency(&mut self, context: String, intensity: f64) {
        let obs = Observation::new(
            SignalType::Inefficiency,
            intensity.clamp(0.0, 1.0),
            context,
            self.session_id.clone(),
            Some(self.turn_number),
        );
        self.observations.push(obs);
    }

    /// Get all observations for the current turn.
    pub fn current_observations(&self) -> &[Observation] {
        &self.observations
    }

    /// Get the current turn number.
    pub fn turn_number(&self) -> u32 {
        self.turn_number
    }

    /// Get the session ID.
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Drain observations from the current turn.
    pub fn drain_observations(&mut self) -> Vec<Observation> {
        std::mem::take(&mut self.observations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_observer_new() {
        let observer = TurnObserver::new(Some("session-1".to_string()));
        assert_eq!(observer.turn_number(), 0);
        assert_eq!(observer.session_id(), Some("session-1"));
    }

    #[test]
    fn test_record_signals() {
        let mut observer = TurnObserver::new(None);
        observer.start_turn();

        observer.record_correction("User said to use functional style".to_string(), 0.8);
        observer.record_error("File not found".to_string(), 0.5);
        observer.record_satisfaction("Good response".to_string(), 0.9);
        observer.record_inefficiency("Took too many steps".to_string(), 0.3);

        let observations = observer.current_observations();
        assert_eq!(observations.len(), 4);
        assert_eq!(observations[0].signal_type, SignalType::Correction);
        assert_eq!(observations[1].signal_type, SignalType::Error);
        assert_eq!(observations[2].signal_type, SignalType::Satisfaction);
        assert_eq!(observations[3].signal_type, SignalType::Inefficiency);
    }

    #[test]
    fn test_drain_observations() {
        let mut observer = TurnObserver::new(None);
        observer.start_turn();
        observer.record_correction("test".to_string(), 0.5);

        let drained = observer.drain_observations();
        assert_eq!(drained.len(), 1);
        assert!(observer.current_observations().is_empty());
    }

    #[test]
    fn test_turn_increment() {
        let mut observer = TurnObserver::new(None);
        observer.start_turn();
        assert_eq!(observer.turn_number(), 1);

        observer.start_turn();
        assert_eq!(observer.turn_number(), 2);
    }

    #[test]
    #[allow(deprecated)]
    fn test_truncate_context_under_limit_unchanged() {
        let input = "short text".to_string();
        let result = TurnObserver::truncate_context(input.clone(), 4096);
        assert_eq!(result, input);
    }

    #[test]
    #[allow(deprecated)]
    fn test_truncate_context_over_limit_truncated_with_marker() {
        let input = "a".repeat(5000);
        let result = TurnObserver::truncate_context(input.clone(), 4096);
        assert!(result.len() <= 4096);
        assert!(result.contains("[truncated, original was 5000 bytes]"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_truncate_context_exact_limit_unchanged() {
        let input = "a".repeat(100);
        let result = TurnObserver::truncate_context(input.clone(), 100);
        assert_eq!(result, input);
    }

    #[test]
    #[allow(deprecated)]
    fn test_truncate_context_empty_max_bytes_returns_marker_only() {
        let input = "some context".to_string();
        let result = TurnObserver::truncate_context(input, 0);
        assert!(result.contains("[truncated, original was 12 bytes]"));
    }

    // ─── I021-S2: find_marker + capture_window tests ────────────────────────

    #[test]
    fn test_find_marker_returns_byte_offset() {
        let text = "Hello world, don't do that please";
        let pos = TurnObserver::find_marker(text, &["don't", "do not"]);
        assert_eq!(pos, Some(13));
    }

    #[test]
    fn test_find_marker_case_insensitive() {
        let text = "Hello world, DON'T do that please";
        let pos = TurnObserver::find_marker(text, &["don't"]);
        assert_eq!(pos, Some(13));
    }

    #[test]
    fn test_find_marker_chinese() {
        let text = "前面很多内容 不要用 sed 后面更多内容";
        let pos = TurnObserver::find_marker(text, &["不要用 sed"]);
        assert!(pos.is_some());
        assert!(text[pos.unwrap()..].starts_with("不要用 sed"));
    }

    #[test]
    fn test_find_marker_not_found() {
        let text = "Hello world, please continue";
        let pos = TurnObserver::find_marker(text, &["don't", "do not"]);
        assert_eq!(pos, None);
    }

    #[test]
    fn test_capture_window_marker_in_center() {
        let text = "AAAAAAAAAA marker BBBBBBBBBB";
        let pos = text.find("marker").unwrap();
        let window = TurnObserver::capture_window(text, pos, 20);

        assert!(
            window.contains("marker"),
            "window={window:?} len={}",
            window.len()
        );
    }

    #[test]
    fn test_capture_window_marker_at_start() {
        let text = "marker BBBBBBBBBBBBBBBBBBBB";
        let window = TurnObserver::capture_window(text, 0, 20);

        assert!(window.starts_with("marker"));
        assert!(!window.starts_with("..."));
    }

    #[test]
    fn test_capture_window_marker_at_end() {
        let text = "AAAAAAAAAAAAAAAAAAAA marker";
        let pos = text.find("marker").unwrap();
        let window = TurnObserver::capture_window(text, pos, 20);

        assert!(window.contains("marker"));
        assert!(window.ends_with("marker"));
    }

    #[test]
    fn test_capture_window_5mb_input_small_output() {
        let prefix = "x".repeat(5 * 1024 * 1024);
        let text = format!("{}{}", prefix, "不要用 sed");
        let pos = text.find("不要用 sed").unwrap();
        let window = TurnObserver::capture_window(&text, pos, 400);

        assert!(
            window.len() < 500,
            "window {} bytes exceeds 500 for 5MB input",
            window.len()
        );
        assert!(
            window.contains("不要用 sed"),
            "window must contain marker, got: {window:?}"
        );
    }

    #[test]
    fn test_capture_window_respects_utf8_boundaries() {
        let text = "AAAA 你好世界 marker 你好世界 BBBB";
        let pos = text.find("marker").unwrap();
        let window = TurnObserver::capture_window(text, pos, 30);

        assert!(window.contains("marker"));
        assert!(window.is_char_boundary(0));
    }

    #[test]
    fn test_capture_window_no_marker_uses_full_text() {
        let text = "short text";
        let window = TurnObserver::capture_window(text, 0, 400);
        assert_eq!(window, "short text");
    }

    #[test]
    fn test_capture_window_5mb_with_chinese_tail_contains_marker() {
        let prefix = "system_prompt content ".repeat(200_000);
        let text = format!("{}{}", prefix, "不要用 sed");
        let pos = text.find("不要用 sed").unwrap();
        let window = TurnObserver::capture_window(&text, pos, 400);

        assert!(
            window.len() < 500,
            "window {} bytes exceeds 500",
            window.len()
        );
        assert!(
            window.contains("不要用 sed"),
            "window must contain '不要用 sed', got: {window:?}"
        );
    }
}
