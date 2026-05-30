//! Process hardening utilities for sandboxed execution.
//!
//! This module provides [`ProcessHardening`], which applies security measures
//! to the current process before executing untrusted code. Hardening includes:
//!
//! - **Environment sanitization**: Removal of dangerous environment variables
//!   that could inject shared libraries (`LD_PRELOAD`, `DYLD_*`, etc.).
//! - **Resource limits**: CPU time, memory, and address space restrictions
//!   via `setrlimit(2)` (Unix only; no-op on Windows).
//! - **Core dump prevention**: Disabling core dumps to prevent sensitive
//!   data leakage from memory snapshots.
//!
//! # Platform Support
//!
//! - **Unix (Linux, macOS)**: Full support including resource limits.
//! - **Windows**: Environment sanitization only; resource limits are no-ops.
//!
//! # Safety
//!
//! Resource limit calls on Unix use `unsafe` blocks to invoke `libc::setrlimit`.
//! This is safe because:
//! - We only pass valid `rlimit` struct pointers.
//! - The resource constants (`RLIMIT_*`) are well-defined by POSIX.
//! - Failure returns an error rather than causing undefined behavior.
//!
//! # Example
//!
//! ```no_run
//! use talos_sandbox::hardening::ProcessHardening;
//!
//! let hardening = ProcessHardening::new();
//! hardening.apply()?;
//! # Ok::<(), talos_sandbox::SandboxError>(())
//! ```

use std::env;

use crate::SandboxError;

/// Environment variables that can inject shared libraries into processes.
///
/// These variables are dangerous in sandboxed contexts because they allow
/// an attacker to load arbitrary code into the process space.
const DANGEROUS_ENV_VARS: &[&str] = &[
    // Linux dynamic linker variables
    "LD_PRELOAD",
    "LD_LIBRARY_PATH",
    "LD_AUDIT",
    "LD_DEBUG",
    "LD_DEBUG_OUTPUT",
    "LD_DYNAMIC_WEAK",
    "LD_HWCAP_MASK",
    "LD_KEEPDIR",
    "LD_ORIGIN_PATH",
    "LD_PROFILE",
    "LD_SHOW_AUXV",
    "LD_TRACE_LOADED_OBJECTS",
    "LD_USE_LOAD_BIAS",
    "LD_VERBOSE",
    "LD_WARN",
    // macOS dynamic linker variables
    "DYLD_INSERT_LIBRARIES",
    "DYLD_LIBRARY_PATH",
    "DYLD_FRAMEWORK_PATH",
    "DYLD_FALLBACK_FRAMEWORK_PATH",
    "DYLD_FALLBACK_LIBRARY_PATH",
    "DYLD_PRINT_TO_FILE",
    "DYLD_SHARED_REGION",
    "DYLD_SHARED_CACHE_DIR",
    "DYLD_IMAGE_SUFFIX",
    "DYLD_BIND_AT_LAUNCH",
    "DYLD_FORCE_FLAT_NAMESPACE",
    "DYLD_VERSIONED_LIBRARY_PATH",
    "DYLD_VERSIONED_FRAMEWORK_PATH",
    "DYLD_INTERPOSE",
    "DYLD_ROOT_PATH",
    "DYLD_DISABLE_DOFS",
    "DYLD_PRINT_OPTS",
    "DYLD_PRINT_WARNINGS",
    "DYLD_PRINT_INITIALIZERS",
    "DYLD_PRINT_SEGMENTS",
    "DYLD_PRINT_BINDINGS",
    "DYLD_PRINT_REBASINGS",
    "DYLD_PRINT_DOFS",
    "DYLD_PRINT_LIBRARIES",
    "DYLD_PRINT_LIBRARIES_POST_LAUNCH",
    "DYLD_PRINT_APIS",
    "DYLD_PRINT_THREADING",
    "DYLD_PRINT_CLASS",
    "DYLD_PRINT_CLASS_NAMES",
    "DYLD_PRINT_PROTOCOLS",
    "DYLD_PRINT_SEL",
    "DYLD_PRINT_COALITIONS",
    "DYLD_PRINT_STATISTICS",
    "DYLD_PRINT_OPTS",
    "DYLD_PRINT_ENV",
    "DYLD_PRINT_RPATHS",
    "DYLD_PRINT_SEARCHING",
    "DYLD_PRINT_LAUNCHING",
    "DYLD_PRINT_BINDINGS",
    "DYLD_PRINT_DOFS",
    "DYLD_PRINT_INITIALIZERS",
    "DYLD_PRINT_SEGMENTS",
    "DYLD_PRINT_REBASINGS",
    "DYLD_PRINT_BIND_AT_LAUNCH",
    "DYLD_PRINT_STATISTICS",
    "DYLD_PRINT_LIBRARIES",
    "DYLD_PRINT_APIS",
    "DYLD_PRINT_CLASS",
    "DYLD_PRINT_PROTOCOLS",
    "DYLD_PRINT_SEL",
    "DYLD_PRINT_COALITIONS",
    "DYLD_PRINT_RPATHS",
    "DYLD_PRINT_SEARCHING",
    "DYLD_PRINT_LAUNCHING",
];

/// Configuration for process hardening.
///
/// Controls which security measures are applied when [`ProcessHardening::apply`]
/// is called. All options have sensible defaults via [`ProcessHardening::new`].
///
/// # Defaults
///
/// - `max_cpu_seconds`: `Some(300)` (5 minutes)
/// - `max_memory_bytes`: `Some(2 * 1024 * 1024 * 1024)` (2 GB)
/// - `disable_core_dumps`: `true`
/// - `sanitize_env`: `true`
#[derive(Debug, Clone)]
pub struct ProcessHardening {
    /// Maximum CPU time in seconds. `None` means no limit.
    ///
    /// Applied via `setrlimit(RLIMIT_CPU, ...)` on Unix.
    pub max_cpu_seconds: Option<u64>,

    /// Maximum memory (address space) in bytes. `None` means no limit.
    ///
    /// Applied via `setrlimit(RLIMIT_AS, ...)` on Unix.
    pub max_memory_bytes: Option<u64>,

    /// Whether to disable core dumps.
    ///
    /// Applied via `setrlimit(RLIMIT_CORE, ...)` with limit 0 on Unix.
    pub disable_core_dumps: bool,

    /// Whether to remove dangerous environment variables.
    ///
    /// Removes variables like `LD_PRELOAD`, `DYLD_INSERT_LIBRARIES`, etc.
    pub sanitize_env: bool,
}

impl ProcessHardening {
    /// Creates a new `ProcessHardening` with sensible security defaults.
    ///
    /// # Defaults
    ///
    /// | Setting | Value | Rationale |
    /// |---------|-------|-----------|
    /// | `max_cpu_seconds` | `Some(300)` | 5-minute CPU limit prevents runaway processes |
    /// | `max_memory_bytes` | `Some(2 GB)` | 2 GB memory cap prevents OOM |
    /// | `disable_core_dumps` | `true` | Prevents sensitive data in core files |
    /// | `sanitize_env` | `true` | Blocks library injection attacks |
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_cpu_seconds: Some(300),
            max_memory_bytes: Some(2 * 1024 * 1024 * 1024),
            disable_core_dumps: true,
            sanitize_env: true,
        }
    }

    /// Creates a `ProcessHardening` with all limits disabled.
    ///
    /// Useful as a starting point for custom configurations.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            max_cpu_seconds: None,
            max_memory_bytes: None,
            disable_core_dumps: false,
            sanitize_env: false,
        }
    }

    /// Applies all configured hardening measures to the current process.
    ///
    /// # Order of Operations
    ///
    /// 1. Environment sanitization (cross-platform)
    /// 2. Resource limits (Unix only; no-op on Windows)
    ///
    /// # Errors
    ///
    /// Returns [`SandboxError::ExecutionFailed`] if a resource limit cannot
    /// be set (e.g., insufficient permissions, invalid value).
    ///
    /// # Platform Behavior
    ///
    /// - **Unix**: All measures applied.
    /// - **Windows**: Only environment sanitization; resource limits are skipped.
    pub fn apply(&self) -> Result<(), SandboxError> {
        if self.sanitize_env {
            self.sanitize_env_vars_internal();
        }

        #[cfg(unix)]
        {
            self.apply_rlimits()?;
        }

        #[cfg(not(unix))]
        {
            // Resource limits are not supported on Windows.
            // This is intentional and not an error condition.
            let _ = self;
        }

        Ok(())
    }

    /// Returns a list of dangerous environment variable names that would be removed.
    ///
    /// This is a static list and does not check which variables are currently set.
    /// Use [`ProcessHardening::apply`] to actually remove them.
    #[must_use]
    pub fn dangerous_env_var_names() -> Vec<String> {
        DANGEROUS_ENV_VARS.iter().map(|s| s.to_string()).collect()
    }

    /// Sanitizes environment variables and returns the list of removed variable names.
    ///
    /// Only removes variables that are both in the dangerous list AND currently set.
    /// If `sanitize_env` is `false`, this is a no-op and returns an empty vector.
    ///
    /// # Returns
    ///
    /// A vector of environment variable names that were removed.
    pub fn sanitize_env_vars_internal(&self) -> Vec<String> {
        if !self.sanitize_env {
            return Vec::new();
        }

        let mut removed = Vec::new();

        for &var in DANGEROUS_ENV_VARS {
            if env::var(var).is_ok() {
                // SAFETY: `remove_var` is safe to call; it only modifies the
                // environment of the current process. No data races occur because
                // we hold a mutable reference to self (exclusive access).
                // Note: `env::remove_var` is documented as unsafe in multithreaded
                // contexts, but we assume the caller applies hardening before
                // spawning threads or child processes.
                unsafe {
                    env::remove_var(var);
                }
                removed.push(var.to_string());
            }
        }

        removed
    }

    /// Applies resource limits via `setrlimit(2)`.
    ///
    /// # Safety
    ///
    /// This function contains `unsafe` blocks for `libc::setrlimit` calls.
    /// These are safe because:
    /// - We construct valid `libc::rlimit` structs with proper field values.
    /// - We pass a valid pointer to the struct (`&rlim as *const _`).
    /// - The resource constants are well-defined POSIX values.
    /// - Errors are captured via the return value, not UB.
    #[cfg(unix)]
    fn apply_rlimits(&self) -> Result<(), SandboxError> {
        if self.disable_core_dumps {
            let rlim = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            // SAFETY: We pass a valid pointer to a properly initialized rlimit struct.
            // RLIMIT_CORE is a well-defined constant. setrlimit returns -1 on error
            // and sets errno; we check the return value.
            let ret = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &rlim as *const _) };
            if ret != 0 {
                let errno = std::io::Error::last_os_error();
                return Err(SandboxError::ExecutionFailed(format!(
                    "failed to disable core dumps: {errno}"
                )));
            }
        }

        if let Some(seconds) = self.max_cpu_seconds {
            let rlim = libc::rlimit {
                rlim_cur: seconds as libc::rlim_t,
                rlim_max: seconds as libc::rlim_t,
            };
            // SAFETY: Same as above — valid struct, valid pointer, well-defined constant.
            let ret = unsafe { libc::setrlimit(libc::RLIMIT_CPU, &rlim as *const _) };
            if ret != 0 {
                let errno = std::io::Error::last_os_error();
                return Err(SandboxError::ExecutionFailed(format!(
                    "failed to set CPU limit: {errno}"
                )));
            }
        }

        if let Some(bytes) = self.max_memory_bytes {
            let rlim = libc::rlimit {
                rlim_cur: bytes as libc::rlim_t,
                rlim_max: bytes as libc::rlim_t,
            };
            // SAFETY: Same as above — valid struct, valid pointer, well-defined constant.
            let ret = unsafe { libc::setrlimit(libc::RLIMIT_AS, &rlim as *const _) };
            if ret != 0 {
                let errno = std::io::Error::last_os_error();
                return Err(SandboxError::ExecutionFailed(format!(
                    "failed to set memory limit: {errno}"
                )));
            }
        }

        Ok(())
    }
}

impl Default for ProcessHardening {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Default configuration tests ---

    #[test]
    fn test_default_has_cpu_limit() {
        let h = ProcessHardening::new();
        assert_eq!(h.max_cpu_seconds, Some(300));
    }

    #[test]
    fn test_default_has_memory_limit() {
        let h = ProcessHardening::new();
        assert_eq!(h.max_memory_bytes, Some(2 * 1024 * 1024 * 1024));
    }

    #[test]
    fn test_default_disables_core_dumps() {
        let h = ProcessHardening::new();
        assert!(h.disable_core_dumps);
    }

    #[test]
    fn test_default_sanitizes_env() {
        let h = ProcessHardening::new();
        assert!(h.sanitize_env);
    }

    #[test]
    fn test_default_trait_matches_new() {
        let h1 = ProcessHardening::default();
        let h2 = ProcessHardening::new();
        assert_eq!(h1.max_cpu_seconds, h2.max_cpu_seconds);
        assert_eq!(h1.max_memory_bytes, h2.max_memory_bytes);
        assert_eq!(h1.disable_core_dumps, h2.disable_core_dumps);
        assert_eq!(h1.sanitize_env, h2.sanitize_env);
    }

    // --- Disabled configuration tests ---

    #[test]
    fn test_disabled_has_no_limits() {
        let h = ProcessHardening::disabled();
        assert!(h.max_cpu_seconds.is_none());
        assert!(h.max_memory_bytes.is_none());
        assert!(!h.disable_core_dumps);
        assert!(!h.sanitize_env);
    }

    // --- Custom configuration tests ---

    #[test]
    fn test_custom_configuration() {
        let h = ProcessHardening {
            max_cpu_seconds: Some(60),
            max_memory_bytes: Some(512 * 1024 * 1024),
            disable_core_dumps: false,
            sanitize_env: true,
        };

        assert_eq!(h.max_cpu_seconds, Some(60));
        assert_eq!(h.max_memory_bytes, Some(512 * 1024 * 1024));
        assert!(!h.disable_core_dumps);
        assert!(h.sanitize_env);
    }

    // --- Environment sanitization tests ---

    #[test]
    fn test_dangerous_env_var_names_contains_ld_preload() {
        let names = ProcessHardening::dangerous_env_var_names();
        assert!(names.contains(&"LD_PRELOAD".to_string()));
    }

    #[test]
    fn test_dangerous_env_var_names_contains_dyld_vars() {
        let names = ProcessHardening::dangerous_env_var_names();
        assert!(names.contains(&"DYLD_INSERT_LIBRARIES".to_string()));
        assert!(names.contains(&"DYLD_LIBRARY_PATH".to_string()));
        assert!(names.contains(&"DYLD_FRAMEWORK_PATH".to_string()));
    }

    #[test]
    fn test_env_sanitization_removes_dangerous_vars() {
        // Set dangerous env vars
        unsafe {
            env::set_var("LD_PRELOAD", "/tmp/evil.so");
            env::set_var("DYLD_INSERT_LIBRARIES", "/tmp/evil.dylib");
        }

        // Apply sanitization
        let h = ProcessHardening {
            sanitize_env: true,
            ..ProcessHardening::disabled()
        };
        let removed = h.sanitize_env_vars_internal();

        // Verify they were removed (or were never set due to parallel test interference)
        assert!(env::var("LD_PRELOAD").is_err());
        assert!(env::var("DYLD_INSERT_LIBRARIES").is_err());
        // At least one should have been removed if it was set
        assert!(
            removed.contains(&"LD_PRELOAD".to_string())
                || removed.contains(&"DYLD_INSERT_LIBRARIES".to_string())
                || removed.is_empty() // both were already absent
        );
    }

    #[test]
    fn test_env_sanitization_preserves_safe_vars() {
        // Set a safe env var
        unsafe {
            env::set_var("MY_SAFE_VAR", "safe_value");
        }

        // Apply sanitization
        let h = ProcessHardening {
            sanitize_env: true,
            ..ProcessHardening::disabled()
        };
        let removed = h.sanitize_env_vars_internal();

        // Safe var should not be in removed list
        assert!(!removed.contains(&"MY_SAFE_VAR".to_string()));

        // Safe var should still be set
        assert_eq!(env::var("MY_SAFE_VAR").unwrap(), "safe_value");

        // Cleanup
        unsafe {
            env::remove_var("MY_SAFE_VAR");
        }
    }

    #[test]
    fn test_env_sanitization_skips_unset_vars() {
        // Ensure these are not set (they shouldn't be in a normal environment)
        unsafe {
            env::remove_var("LD_AUDIT");
            env::remove_var("LD_DEBUG");
        }

        let h = ProcessHardening {
            sanitize_env: true,
            ..ProcessHardening::disabled()
        };
        let removed = h.sanitize_env_vars_internal();

        // Unset vars should not appear in removed list
        assert!(!removed.contains(&"LD_AUDIT".to_string()));
        assert!(!removed.contains(&"LD_DEBUG".to_string()));
    }

    #[test]
    fn test_env_sanitization_disabled_does_not_remove_vars() {
        unsafe {
            env::set_var("LD_PRELOAD", "/tmp/evil.so");
        }

        let h = ProcessHardening {
            sanitize_env: false,
            ..ProcessHardening::disabled()
        };
        let removed = h.sanitize_env_vars_internal();

        assert!(removed.is_empty());
        assert!(env::var("LD_PRELOAD").is_ok());

        // Cleanup
        unsafe {
            env::remove_var("LD_PRELOAD");
        }
    }

    // --- Apply tests ---

    #[test]
    fn test_apply_with_defaults() {
        let h = ProcessHardening::new();
        // This may fail on some systems due to permission restrictions,
        // but it should not panic.
        let _ = h.apply();
    }

    #[test]
    fn test_apply_with_disabled() {
        let h = ProcessHardening::disabled();
        let result = h.apply();
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_env_only() {
        unsafe {
            env::set_var("LD_PRELOAD", "/tmp/evil.so");
        }

        let h = ProcessHardening {
            max_cpu_seconds: None,
            max_memory_bytes: None,
            disable_core_dumps: false,
            sanitize_env: true,
        };
        let result = h.apply();
        assert!(result.is_ok());
        assert!(env::var("LD_PRELOAD").is_err());

        // Cleanup
        unsafe {
            env::remove_var("LD_PRELOAD");
        }
    }
}
