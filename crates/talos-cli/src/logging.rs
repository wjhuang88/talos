use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use talos_config::{LogConfig, LogFileConfig, LogFormat};
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

    let file_config = resolve_file_config(config, terminal_ui);
    if let Some(writer) = file_config.and_then(|fc| open_log_writer(&fc)) {
        init_with_file(filter, format, writer);
        return;
    }

    init_with_stderr(filter, format);
}

fn resolve_file_config(config: Option<&LogConfig>, terminal_ui: bool) -> Option<LogFileConfig> {
    if let Some(log_config) = config {
        if let Some(ref fc) = log_config.file {
            if fc.enabled {
                return Some(fc.clone());
            }
            return None;
        }
        if terminal_ui {
            return Some(LogFileConfig::default());
        }
        return None;
    }
    if terminal_ui {
        return Some(LogFileConfig::default());
    }
    None
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

fn init_with_file(filter: EnvFilter, format: &LogFormat, writer: Arc<RotatingWriter>) {
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

/// A file writer that performs size-based rotation when the current file
/// exceeds the configured maximum size.
pub(crate) struct RotatingWriter {
    file: Mutex<File>,
    path: PathBuf,
    max_size_bytes: u64,
    max_files: usize,
    current_size: AtomicU64,
}

impl RotatingWriter {
    pub(crate) fn new(path: PathBuf, max_size_mb: u64, max_files: usize) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let current_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(Self {
            file: Mutex::new(file),
            path,
            max_size_bytes: max_size_mb * 1_000_000,
            max_files,
            current_size: AtomicU64::new(current_size),
        })
    }

    fn rotate(&self) -> io::Result<()> {
        let mut file = self.file.lock().expect("mutex poisoned");
        file.sync_all()?;

        for i in (1..self.max_files).rev() {
            let old = self.path_with_index(i);
            let new = self.path_with_index(i + 1);
            if old.exists() {
                fs::rename(&old, &new)?;
            }
        }

        let target = self.path_with_index(1);
        fs::rename(&self.path, &target)?;

        let excess = self.max_files + 1;
        let excess_path = self.path_with_index(excess);
        if excess_path.exists() {
            let _ = fs::remove_file(&excess_path);
        }

        *file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        self.current_size.store(0, Ordering::Relaxed);
        Ok(())
    }

    fn path_with_index(&self, index: usize) -> PathBuf {
        let mut path = self.path.clone();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        path.set_file_name(format!("{file_name}.{index}"));
        path
    }
}

impl Write for &RotatingWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = self.current_size.load(Ordering::Relaxed);
        if size + buf.len() as u64 >= self.max_size_bytes {
            self.rotate()?;
        }
        let n = self.file.lock().expect("mutex poisoned").write(buf)?;
        self.current_size.fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.lock().expect("mutex poisoned").flush()
    }
}

fn open_log_writer(config: &LogFileConfig) -> Option<Arc<RotatingWriter>> {
    let path = match &config.path {
        Some(p) => expand_path(p),
        None => {
            let home = dirs::home_dir()?;
            home.join(".talos").join("logs").join("talos.log")
        }
    };

    let writer = RotatingWriter::new(path, config.max_size_mb, config.max_files).ok()?;
    Some(Arc::new(writer))
}

fn expand_path(path: &Path) -> PathBuf {
    if let Ok(rest) = path.strip_prefix("~")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    path.to_path_buf()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use talos_config::LogRotation;
    use tempfile::tempdir;

    #[test]
    fn default_file_config_values() {
        let config = LogFileConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_size_mb, 16);
        assert_eq!(config.max_files, 5);
        assert_eq!(config.rotation, LogRotation::Size);
        assert!(config.path.is_none());
    }

    #[test]
    fn file_rotates_when_exceeding_max_size() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.log");

        let writer = RotatingWriter::new(path.clone(), 1, 3).unwrap();

        let chunk = vec![b'x'; 500_000];
        (&writer).write_all(&chunk).unwrap();
        assert!(path.exists());
        assert!(!dir.path().join("test.log.1").exists());

        (&writer).write_all(&chunk).unwrap();
        assert!(path.exists());
        assert!(dir.path().join("test.log.1").exists());
    }

    #[test]
    fn old_files_beyond_max_files_are_deleted() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.log");

        let writer = RotatingWriter::new(path.clone(), 1, 2).unwrap();

        let chunk = vec![b'x'; 500_000];
        for _ in 0..5 {
            (&writer).write_all(&chunk).unwrap();
        }

        assert!(path.exists());
        assert!(dir.path().join("test.log.1").exists());
        assert!(dir.path().join("test.log.2").exists());
        assert!(!dir.path().join("test.log.3").exists());
    }

    #[test]
    fn rotation_shifts_existing_files() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.log");

        let writer = RotatingWriter::new(path.clone(), 1, 5).unwrap();

        let chunk = vec![b'x'; 500_000];
        (&writer).write_all(&chunk).unwrap();
        (&writer).write_all(&chunk).unwrap();

        let content_1 = fs::read_to_string(dir.path().join("test.log.1")).unwrap();
        assert_eq!(content_1.len(), 500_000);
    }

    #[test]
    fn path_expansion_handles_tilde() {
        let path = expand_path(Path::new("~/logs/talos.log"));
        assert!(!path.starts_with("~"));
        assert!(path.ends_with("logs/talos.log"));
    }

    #[test]
    fn resolve_file_config_tui_enables_by_default() {
        let config = resolve_file_config(None, true);
        assert!(config.is_some());
        let fc = config.unwrap();
        assert!(fc.enabled);
    }

    #[test]
    fn resolve_file_config_non_tui_defaults_to_none() {
        let config = resolve_file_config(None, false);
        assert!(config.is_none());
    }

    #[test]
    fn resolve_file_config_respects_explicit_disabled() {
        let log_config = LogConfig {
            file: Some(LogFileConfig {
                enabled: false,
                ..Default::default()
            }),
            ..Default::default()
        };
        let config = resolve_file_config(Some(&log_config), true);
        assert!(config.is_none());
    }

    #[test]
    fn resolve_file_config_respects_explicit_enabled() {
        let log_config = LogConfig {
            file: Some(LogFileConfig {
                enabled: true,
                max_size_mb: 32,
                ..Default::default()
            }),
            ..Default::default()
        };
        let config = resolve_file_config(Some(&log_config), false);
        assert!(config.is_some());
        assert_eq!(config.unwrap().max_size_mb, 32);
    }
}
