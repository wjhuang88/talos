//! Shared formatting helpers for the TUI layer.
//!
//! Provides human-readable number and duration formatting used by both
//! the status bar and the exit summary.

/// Format a token count into a human-readable string.
///
/// # Examples
///
/// ```
/// // 999 -> "999"
/// // 12345 -> "12.3k"
/// // 1234567 -> "1.2M"
/// ```
pub fn format_tokens(n: u64) -> String {
    if n < 1_000 {
        n.to_string()
    } else if n < 1_000_000 {
        let scaled = n as f64 / 1_000.0;
        if scaled.fract() == 0.0 {
            format!("{scaled:.0}k")
        } else {
            format!("{scaled:.1}k")
        }
    } else if n < 1_000_000_000 {
        let scaled = n as f64 / 1_000_000.0;
        if scaled.fract() == 0.0 {
            format!("{scaled:.0}M")
        } else {
            format!("{scaled:.1}M")
        }
    } else {
        let scaled = n as f64 / 1_000_000_000.0;
        if scaled.fract() == 0.0 {
            format!("{scaled:.0}B")
        } else {
            format!("{scaled:.1}B")
        }
    }
}

/// Format a duration in seconds into a human-readable string.
///
/// # Examples
///
/// ```
/// // 0 -> "0s"
/// // 75 -> "1m 15s"
/// // 3661 -> "1h 1m 1s"
/// ```
pub fn format_duration(secs: u64) -> String {
    if secs == 0 {
        return "0s".to_string();
    }

    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    let mut parts = Vec::new();
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 {
        parts.push(format!("{seconds}s"));
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── format_tokens ──────────────────────────────────────────────────

    #[test]
    fn test_format_tokens_zero() {
        assert_eq!(format_tokens(0), "0");
    }

    #[test]
    fn test_format_tokens_under_1k() {
        assert_eq!(format_tokens(1), "1");
        assert_eq!(format_tokens(999), "999");
    }

    #[test]
    fn test_format_tokens_exactly_1k() {
        assert_eq!(format_tokens(1_000), "1k");
    }

    #[test]
    fn test_format_tokens_thousands() {
        assert_eq!(format_tokens(1_234), "1.2k");
        assert_eq!(format_tokens(12_345), "12.3k");
        assert_eq!(format_tokens(999_999), "1000.0k");
    }

    #[test]
    fn test_format_tokens_exactly_1m() {
        assert_eq!(format_tokens(1_000_000), "1M");
    }

    #[test]
    fn test_format_tokens_millions() {
        assert_eq!(format_tokens(1_234_567), "1.2M");
        assert_eq!(format_tokens(12_345_678), "12.3M");
    }

    #[test]
    fn test_format_tokens_exactly_1b() {
        assert_eq!(format_tokens(1_000_000_000), "1B");
    }

    #[test]
    fn test_format_tokens_billions() {
        assert_eq!(format_tokens(1_234_567_890), "1.2B");
    }

    #[test]
    fn test_format_tokens_round_trips_clean() {
        assert_eq!(format_tokens(5_000), "5k");
        assert_eq!(format_tokens(50_000), "50k");
        assert_eq!(format_tokens(500_000), "500k");
    }

    // ── format_duration ────────────────────────────────────────────────

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(format_duration(0), "0s");
    }

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_duration(1), "1s");
        assert_eq!(format_duration(59), "59s");
    }

    #[test]
    fn test_format_duration_exactly_1m() {
        assert_eq!(format_duration(60), "1m");
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(75), "1m 15s");
        assert_eq!(format_duration(125), "2m 5s");
    }

    #[test]
    fn test_format_duration_exactly_1h() {
        assert_eq!(format_duration(3600), "1h");
    }

    #[test]
    fn test_format_duration_hours_minutes_seconds() {
        assert_eq!(format_duration(3661), "1h 1m 1s");
        assert_eq!(format_duration(7325), "2h 2m 5s");
    }

    #[test]
    fn test_format_duration_hours_and_minutes_no_seconds() {
        assert_eq!(format_duration(3660), "1h 1m");
        assert_eq!(format_duration(7200), "2h");
    }

    #[test]
    fn test_format_duration_large_values() {
        assert_eq!(format_duration(86400), "24h");
        assert_eq!(format_duration(90061), "25h 1m 1s");
    }
}
