use std::sync::Arc;

use talos_config::{LogConfig, LogFormat};
use tracing_subscriber::EnvFilter;

const DEFAULT_LOG_LEVEL: &str = "info";

/// Initialize Talos CLI logging.
///
/// Terminal UI modes route logs to the existing log file sink to avoid corrupting
/// the alternate-screen display. Other modes write to stderr.
pub(crate) fn init_logger(config: Option<&LogConfig>, terminal_ui: bool) {
    let filter = env_filter(config);
    let format = config
        .map(|config| &config.format)
        .unwrap_or(&LogFormat::Pretty);

    if terminal_ui && let Some(writer) = open_log_writer() {
        init_with_file(filter, format, writer);
        return;
    }

    init_with_stderr(filter, format);
}

fn env_filter(config: Option<&LogConfig>) -> EnvFilter {
    if let Ok(filter) = EnvFilter::try_from_default_env() {
        return filter;
    }

    if let Some(filter) = config.and_then(|config| config.filter.as_deref())
        && !filter.trim().is_empty()
        && let Ok(filter) = EnvFilter::try_new(filter)
    {
        return filter;
    }

    let level = config
        .and_then(|config| config.level.as_deref())
        .filter(|level| !level.trim().is_empty())
        .unwrap_or(DEFAULT_LOG_LEVEL);

    EnvFilter::try_new(default_filter(level))
        .unwrap_or_else(|_| EnvFilter::new(default_filter(DEFAULT_LOG_LEVEL)))
}

fn default_filter(level: &str) -> String {
    format!(
        "{level},talos_agent=info,talos_provider=debug,talos_mcp=warn,talos_evolution=info,talos_tui=warn,talos_rpc=warn"
    )
}

fn init_with_file(filter: EnvFilter, format: &LogFormat, writer: Arc<std::fs::File>) {
    match format {
        LogFormat::Pretty => {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .pretty()
                .with_ansi(false)
                .with_writer(writer)
                .try_init();
        }
        LogFormat::Compact => {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .compact()
                .with_ansi(false)
                .with_writer(writer)
                .try_init();
        }
    }
}

fn init_with_stderr(filter: EnvFilter, format: &LogFormat) {
    match format {
        LogFormat::Pretty => {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .pretty()
                .with_writer(std::io::stderr)
                .try_init();
        }
        LogFormat::Compact => {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .compact()
                .with_writer(std::io::stderr)
                .try_init();
        }
    }
}

fn open_log_writer() -> Option<Arc<std::fs::File>> {
    let log_dir = dirs::home_dir()?.join(".talos").join("logs");
    std::fs::create_dir_all(&log_dir).ok()?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("talos.log"))
        .ok()?;
    Some(Arc::new(file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filter_contains_per_concern_defaults() {
        let filter = default_filter("info");
        assert!(filter.contains("talos_agent=info"));
        assert!(filter.contains("talos_provider=debug"));
        assert!(filter.contains("talos_mcp=warn"));
    }

    #[test]
    fn config_filter_takes_precedence_over_level() {
        let config = LogConfig {
            level: Some("error".to_string()),
            format: LogFormat::Compact,
            filter: Some("talos_provider=trace".to_string()),
        };
        let filter = env_filter(Some(&config)).to_string();
        assert!(filter.contains("talos_provider=trace"));
        assert!(!filter.contains("talos_provider=debug"));
    }

    #[test]
    fn invalid_level_falls_back_to_default() {
        let config = LogConfig {
            level: Some("not-a-level".to_string()),
            format: LogFormat::Pretty,
            filter: None,
        };
        let filter = env_filter(Some(&config)).to_string();
        assert!(filter.contains("talos_agent=info"));
    }
}
