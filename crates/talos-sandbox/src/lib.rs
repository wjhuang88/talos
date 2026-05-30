//! Platform-specific sandboxing for bash command execution.
//!
//! This crate provides a unified [`SandboxProvider`] trait with platform-specific
//! implementations:
//!
//! - **Linux**: [`BubblewrapSandbox`] using `bwrap` (Bubblewrap)
//! - **macOS**: [`SeatbeltSandbox`] using `sandbox-exec` with Seatbelt profiles
//!
//! Both implementations restrict filesystem write access to the workspace root
//! and block network access by default.
//!
//! # Graceful Fallback
//!
//! The sandbox is optional. If the required tool is not installed,
//! [`SandboxProvider::is_available`] returns `false` and [`SandboxProvider::execute`]
//! returns [`SandboxError::NotAvailable`]. Callers should decide whether to fall
//! back to unsandboxed execution or reject the command.
//!
//! # Example
//!
//! ```no_run
//! use talos_sandbox::{create_sandbox, SandboxConfig};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let sandbox = create_sandbox();
//! let config = SandboxConfig {
//!     workspace_root: PathBuf::from("/tmp/workspace"),
//!     allow_network: false,
//!     extra_read_paths: vec![],
//! };
//!
//! if sandbox.is_available() {
//!     let result = sandbox.execute("echo hello", &config).await?;
//!     println!("stdout: {}", result.stdout);
//! }
//! # Ok(())
//! # }
//! ```

use std::path::PathBuf;

use async_trait::async_trait;
use thiserror::Error;

pub mod hardening;

/// Errors that can occur during sandboxed command execution.
#[derive(Debug, Error)]
pub enum SandboxError {
    /// The sandbox tool (bwrap on Linux, sandbox-exec on macOS) is not installed.
    #[error("sandbox tool not available on this system")]
    NotAvailable,

    /// The sandboxed command execution failed.
    #[error("sandbox execution failed: {0}")]
    ExecutionFailed(String),

    /// The command was denied by the sandbox policy.
    #[error("permission denied by sandbox: {0}")]
    PermissionDenied(String),
}

/// Configuration for sandboxed command execution.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// The workspace root directory. Write access is restricted to this path.
    pub workspace_root: PathBuf,

    /// Whether to allow network access. Defaults to `false`.
    pub allow_network: bool,

    /// Additional paths to mount as read-only inside the sandbox.
    pub extra_read_paths: Vec<PathBuf>,
}

/// Result of a sandboxed command execution.
#[derive(Debug, Clone)]
pub struct SandboxResult {
    /// Standard output from the command.
    pub stdout: String,

    /// Standard error from the command.
    pub stderr: String,

    /// Exit code of the command.
    pub exit_code: i32,
}

/// A platform-specific sandbox provider.
///
/// Implementations execute commands within a restricted environment that limits
/// filesystem write access and network connectivity.
#[async_trait]
pub trait SandboxProvider: Send + Sync {
    /// Execute a command within the sandbox.
    ///
    /// # Arguments
    ///
    /// * `command` - The shell command to execute.
    /// * `config` - Sandbox configuration including workspace root and permissions.
    ///
    /// # Returns
    ///
    /// * `Ok(SandboxResult)` — Command executed successfully (exit code may be non-zero).
    /// * `Err(SandboxError::NotAvailable)` — Sandbox tool not installed.
    /// * `Err(SandboxError::ExecutionFailed)` — Sandbox execution encountered an error.
    /// * `Err(SandboxError::PermissionDenied)` — Command violated sandbox policy.
    async fn execute(
        &self,
        command: &str,
        config: &SandboxConfig,
    ) -> Result<SandboxResult, SandboxError>;

    /// Returns `true` if the sandbox tool is available on this system.
    fn is_available(&self) -> bool;
}

// ---------------------------------------------------------------------------
// Linux: Bubblewrap implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use tokio::process::Command;

    /// Sandbox implementation using Bubblewrap (`bwrap`) on Linux.
    ///
    /// Bubblewrap provides unprivileged container-like isolation using Linux
    /// namespaces. It restricts filesystem access and network connectivity.
    pub struct BubblewrapSandbox;

    impl Default for BubblewrapSandbox {
        fn default() -> Self {
            Self::new()
        }
    }

    impl BubblewrapSandbox {
        /// Creates a new `BubblewrapSandbox` instance.
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl SandboxProvider for BubblewrapSandbox {
        async fn execute(
            &self,
            command: &str,
            config: &SandboxConfig,
        ) -> Result<SandboxResult, SandboxError> {
            if !self.is_available() {
                return Err(SandboxError::NotAvailable);
            }

            let workspace = config
                .workspace_root
                .to_str()
                .ok_or_else(|| SandboxError::ExecutionFailed("workspace path is not valid UTF-8".into()))?;

            let mut cmd = Command::new("bwrap");

            cmd.args(["--ro-bind", "/", "/"]);
            cmd.args(["--bind", workspace, workspace]);
            cmd.args(["--dev", "/dev"]);
            cmd.args(["--proc", "/proc"]);
            cmd.args(["--tmpfs", "/tmp"]);

            for path in &config.extra_read_paths {
                if let Some(path_str) = path.to_str() {
                    cmd.args(["--ro-bind", path_str, path_str]);
                }
            }

            if !config.allow_network {
                cmd.arg("--unshare-net");
            }

            cmd.arg("--die-with-parent");
            cmd.args(["--", "sh", "-c", command]);

            let output = cmd
                .output()
                .await
                .map_err(|e| SandboxError::ExecutionFailed(format!("failed to spawn bwrap: {e}")))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(1);

            if stderr.contains("Permission denied") || stderr.contains("Operation not permitted") {
                return Err(SandboxError::PermissionDenied(stderr));
            }

            Ok(SandboxResult {
                stdout,
                stderr,
                exit_code,
            })
        }

        fn is_available(&self) -> bool {
            std::process::Command::new("which")
                .arg("bwrap")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::BubblewrapSandbox;

// ---------------------------------------------------------------------------
// macOS: Seatbelt (sandbox-exec) implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::io::Write;
    use tokio::process::Command;

    /// Sandbox implementation using `sandbox-exec` with Seatbelt profiles on macOS.
    ///
    /// Seatbelt is the macOS mandatory access control system. This implementation
    /// generates a dynamic profile that restricts filesystem writes to the workspace
    /// and blocks network access.
    pub struct SeatbeltSandbox;

    impl Default for SeatbeltSandbox {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SeatbeltSandbox {
        /// Creates a new `SeatbeltSandbox` instance.
        pub fn new() -> Self {
            Self
        }

        /// Generates a Seatbelt profile string based on the configuration.
        pub(crate) fn generate_profile(config: &SandboxConfig) -> Result<String, SandboxError> {
            let workspace = config
                .workspace_root
                .canonicalize()
                .map_err(|e| SandboxError::ExecutionFailed(format!("cannot canonicalize workspace: {e}")))?;
            let workspace = workspace
                .to_str()
                .ok_or_else(|| SandboxError::ExecutionFailed("workspace path is not valid UTF-8".into()))?;

            let mut profile = String::new();

            profile.push_str("(version 1)\n");
            profile.push_str("(deny default)\n");
            profile.push_str("(allow file-read*)\n");
            profile.push_str(&format!("(allow file-write* (subpath \"{workspace}\"))\n"));
            profile.push_str("(allow file-write* (subpath \"/private/tmp\"))\n");
            profile.push_str("(allow process*)\n");
            profile.push_str("(allow file-read-metadata (subpath \"/usr\"))\n");
            profile.push_str("(allow file-read-metadata (subpath \"/System\"))\n");
            profile.push_str("(allow file-read-metadata (subpath \"/Library\"))\n");

            for path in &config.extra_read_paths {
                if let Ok(canonical) = path.canonicalize() {
                    if let Some(path_str) = canonical.to_str() {
                        profile.push_str(&format!("(allow file-read* (subpath \"{path_str}\"))\n"));
                    }
                }
            }

            if config.allow_network {
                profile.push_str("(allow network*)\n");
            } else {
                profile.push_str("(deny network*)\n");
            }

            profile.push_str("(deny mach-lookup)\n");

            Ok(profile)
        }
    }

    #[async_trait]
    impl SandboxProvider for SeatbeltSandbox {
        async fn execute(
            &self,
            command: &str,
            config: &SandboxConfig,
        ) -> Result<SandboxResult, SandboxError> {
            if !self.is_available() {
                return Err(SandboxError::NotAvailable);
            }

            let profile = Self::generate_profile(config)?;

            let mut profile_file = tempfile::NamedTempFile::new()
                .map_err(|e| SandboxError::ExecutionFailed(format!("failed to create temp profile: {e}")))?;

            profile_file
                .write_all(profile.as_bytes())
                .map_err(|e| SandboxError::ExecutionFailed(format!("failed to write profile: {e}")))?;

            let profile_path = profile_file
                .path()
                .to_str()
                .ok_or_else(|| SandboxError::ExecutionFailed("profile path is not valid UTF-8".into()))?;

            let output = Command::new("sandbox-exec")
                .arg("-f")
                .arg(profile_path)
                .args(["sh", "-c", command])
                .output()
                .await
                .map_err(|e| SandboxError::ExecutionFailed(format!("failed to spawn sandbox-exec: {e}")))?;

            drop(profile_file);

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(1);

            if stderr.contains("Operation not permitted") || stderr.contains("denied") {
                return Err(SandboxError::PermissionDenied(stderr));
            }

            Ok(SandboxResult {
                stdout,
                stderr,
                exit_code,
            })
        }

        fn is_available(&self) -> bool {
            std::process::Command::new("which")
                .arg("sandbox-exec")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }
}

#[cfg(target_os = "macos")]
pub use macos::SeatbeltSandbox;

// ---------------------------------------------------------------------------
// Factory function
// ---------------------------------------------------------------------------

/// Creates a platform-appropriate sandbox provider.
///
/// Returns a boxed trait object that can be used polymorphically.
/// The caller should check [`SandboxProvider::is_available`] before use.
///
/// # Platform Support
///
/// - **Linux**: Returns [`BubblewrapSandbox`]
/// - **macOS**: Returns [`SeatbeltSandbox`]
/// - **Other**: Returns a stub that always reports `NotAvailable`
#[must_use]
pub fn create_sandbox() -> Box<dyn SandboxProvider> {
    #[cfg(target_os = "linux")]
    {
        Box::new(BubblewrapSandbox::new())
    }
    #[cfg(target_os = "macos")]
    {
        Box::new(SeatbeltSandbox::new())
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Box::new(UnsupportedSandbox)
    }
}

// ---------------------------------------------------------------------------
// Unsupported platform stub
// ---------------------------------------------------------------------------

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod unsupported {
    use super::*;

    /// Stub sandbox for unsupported platforms.
    ///
    /// Always reports as unavailable and returns `NotAvailable` on execute.
    pub struct UnsupportedSandbox;

    #[async_trait]
    impl SandboxProvider for UnsupportedSandbox {
        async fn execute(
            &self,
            _command: &str,
            _config: &SandboxConfig,
        ) -> Result<SandboxResult, SandboxError> {
            Err(SandboxError::NotAvailable)
        }

        fn is_available(&self) -> bool {
            false
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub use unsupported::UnsupportedSandbox;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sandbox_returns_boxed_trait() {
        let sandbox = create_sandbox();
        // The sandbox should be a valid trait object
        #[cfg(target_os = "linux")]
        assert!(!sandbox.is_available() || sandbox.is_available()); // bwrap may or may not be installed

        #[cfg(target_os = "macos")]
        assert!(sandbox.is_available()); // sandbox-exec is always available on macOS

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        assert!(!sandbox.is_available());
    }

    #[test]
    fn test_sandbox_config_defaults() {
        let config = SandboxConfig {
            workspace_root: PathBuf::from("/tmp/test"),
            allow_network: false,
            extra_read_paths: vec![],
        };

        assert_eq!(config.workspace_root, PathBuf::from("/tmp/test"));
        assert!(!config.allow_network);
        assert!(config.extra_read_paths.is_empty());
    }

    #[test]
    fn test_sandbox_result_struct() {
        let result = SandboxResult {
            stdout: "hello".into(),
            stderr: "".into(),
            exit_code: 0,
        };

        assert_eq!(result.stdout, "hello");
        assert_eq!(result.stderr, "");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_sandbox_error_display() {
        let err = SandboxError::NotAvailable;
        assert!(err.to_string().contains("not available"));

        let err = SandboxError::ExecutionFailed("test error".into());
        assert!(err.to_string().contains("test error"));

        let err = SandboxError::PermissionDenied("denied".into());
        assert!(err.to_string().contains("denied"));
    }

    // Linux-specific tests
    #[cfg(target_os = "linux")]
    mod linux_tests {
        use super::*;

        #[tokio::test]
        async fn test_bubblewrap_not_available_fallback() {
            // If bwrap is not installed, execute should return NotAvailable
            let sandbox = BubblewrapSandbox::new();
            let config = SandboxConfig {
                workspace_root: PathBuf::from("/tmp"),
                allow_network: false,
                extra_read_paths: vec![],
            };

            if !sandbox.is_available() {
                let result = sandbox.execute("echo hello", &config).await;
                assert!(matches!(result, Err(SandboxError::NotAvailable)));
            }
        }

        #[cfg(feature = "bwrap_installed")]
        #[tokio::test]
        async fn test_bubblewrap_basic_execution() {
            let sandbox = BubblewrapSandbox::new();
            let config = SandboxConfig {
                workspace_root: PathBuf::from("/tmp"),
                allow_network: false,
                extra_read_paths: vec![],
            };

            let result = sandbox.execute("echo hello", &config).await.unwrap();
            assert_eq!(result.exit_code, 0);
            assert!(result.stdout.contains("hello"));
        }

        #[cfg(feature = "bwrap_installed")]
        #[tokio::test]
        async fn test_bubblewrap_filesystem_restriction() {
            let sandbox = BubblewrapSandbox::new();
            let workspace = tempfile::tempdir().unwrap();
            let config = SandboxConfig {
                workspace_root: workspace.path().to_path_buf(),
                allow_network: false,
                extra_read_paths: vec![],
            };

            // Try to write outside workspace — should fail
            let result = sandbox
                .execute("echo test > /etc/test_sandbox_write", &config)
                .await;

            // Either the command fails (exit_code != 0) or sandbox blocks it
            if let Ok(r) = result {
                assert_ne!(r.exit_code, 0, "should not be able to write to /etc");
            }
        }
    }

    // macOS-specific tests
    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::*;

        #[test]
        fn test_seatbelt_profile_generation() {
            use crate::macos::SeatbeltSandbox;

            let workspace = tempfile::tempdir().unwrap();
            let config = SandboxConfig {
                workspace_root: workspace.path().to_path_buf(),
                allow_network: false,
                extra_read_paths: vec![],
            };

            let profile = SeatbeltSandbox::generate_profile(&config).unwrap();
            let canonical = workspace.path().canonicalize().unwrap();

            assert!(profile.contains("(version 1)"));
            assert!(profile.contains("(deny default)"));
            assert!(profile.contains("(allow file-read*)"));
            assert!(profile.contains(&format!(
                "(allow file-write* (subpath \"{}\"))",
                canonical.display()
            )));
            assert!(profile.contains("(allow file-write* (subpath \"/private/tmp\"))"));
            assert!(profile.contains("(deny network*)"));
            assert!(profile.contains("(deny mach-lookup)"));
            assert!(profile.contains("(allow process*)"));
        }

        #[test]
        fn test_seatbelt_profile_allows_network() {
            use crate::macos::SeatbeltSandbox;

            let workspace = tempfile::tempdir().unwrap();
            let config = SandboxConfig {
                workspace_root: workspace.path().to_path_buf(),
                allow_network: true,
                extra_read_paths: vec![],
            };

            let profile = SeatbeltSandbox::generate_profile(&config).unwrap();
            assert!(profile.contains("(allow network*)"));
            assert!(!profile.contains("(deny network*)"));
        }

        #[tokio::test]
        async fn test_seatbelt_is_available() {
            let sandbox = SeatbeltSandbox::new();
            assert!(sandbox.is_available());
        }

        #[tokio::test]
        async fn test_seatbelt_basic_execution() {
            let sandbox = SeatbeltSandbox::new();
            let workspace = tempfile::tempdir().unwrap();
            let config = SandboxConfig {
                workspace_root: workspace.path().to_path_buf(),
                allow_network: false,
                extra_read_paths: vec![],
            };

            let result = sandbox.execute("echo hello", &config).await.unwrap();
            assert_eq!(result.exit_code, 0);
            assert!(result.stdout.contains("hello"));
        }

        #[tokio::test]
        async fn test_seatbelt_filesystem_restriction() {
            let sandbox = SeatbeltSandbox::new();
            let workspace = tempfile::tempdir().unwrap();
            let config = SandboxConfig {
                workspace_root: workspace.path().to_path_buf(),
                allow_network: false,
                extra_read_paths: vec![],
            };

            // Try to write outside workspace — should be denied
            let result = sandbox
                .execute("echo test > /etc/test_sandbox_write", &config)
                .await;

            // Should either return an error or a non-zero exit code
            match result {
                Err(SandboxError::PermissionDenied(_)) => {} // Expected
                Ok(r) => assert_ne!(r.exit_code, 0, "should not be able to write to /etc"),
                Err(e) => panic!("unexpected error: {e}"),
            }
        }

        #[tokio::test]
        async fn test_seatbelt_workspace_write_allowed() {
            let sandbox = SeatbeltSandbox::new();
            let workspace = tempfile::tempdir().unwrap();
            let test_file = workspace.path().join("test.txt");
            let config = SandboxConfig {
                workspace_root: workspace.path().to_path_buf(),
                allow_network: false,
                extra_read_paths: vec![],
            };

            let result = sandbox
                .execute(&format!("echo hello > {}", test_file.display()), &config)
                .await
                .unwrap();

            assert_eq!(result.exit_code, 0);

            // Verify file was written
            let content = std::fs::read_to_string(&test_file).unwrap();
            assert_eq!(content.trim(), "hello");
        }
    }
}
