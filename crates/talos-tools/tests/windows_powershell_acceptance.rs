//! Windows execution-level acceptance evidence for I170 / TOOL-023-C.
//!
//! These tests deliberately exercise the public production `BashTool::execute` path. On Windows
//! that path constructs and supervises `powershell.exe` through the same command builder used by
//! the product. Process-global environment mutation is confined to this integration-test process,
//! serialized by one lock, and restored by an unwind-safe guard.

#![cfg(windows)]

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use talos_core::tool::AgentTool;
use talos_tools::BashTool;

static ENVIRONMENT_LOCK: Mutex<()> = Mutex::new(());

const LD_PRELOAD_SENTINEL: &str = "talos-i170-parent-ld-preload";
const DYLD_INSERT_LIBRARIES_SENTINEL: &str = "talos-i170-parent-dyld-insert-libraries";

const NORMAL_COMMAND: &str = r#"Write-Output "stdout-ok"; [Console]::Error.WriteLine("stderr-ok"); (Get-Location).Path; exit 7"#;
const TIMEOUT_COMMAND: &str = r#"Write-Output "before-timeout"; Write-Output "direct-child-pid=$PID"; Start-Sleep -Seconds 10; Write-Output "after-timeout""#;

struct EnvironmentRestore {
    original: Vec<(&'static str, Option<OsString>)>,
}

impl EnvironmentRestore {
    fn set(values: &[(&'static str, &'static str)]) -> Self {
        let original = values
            .iter()
            .map(|(name, _)| (*name, env::var_os(name)))
            .collect();

        for (name, value) in values {
            // SAFETY: this integration-test process serializes every environment mutation in this
            // file through ENVIRONMENT_LOCK. The guard restores all original states before the lock
            // is released, including variables that were originally absent.
            unsafe {
                env::set_var(name, value);
            }
        }

        Self { original }
    }
}

impl Drop for EnvironmentRestore {
    fn drop(&mut self) {
        for (name, value) in &self.original {
            // SAFETY: EnvironmentRestore is dropped while ENVIRONMENT_LOCK is still held. This
            // unwind-safe cleanup restores the exact pre-test state before another test can mutate
            // the process environment.
            unsafe {
                match value {
                    Some(value) => env::set_var(name, value),
                    None => env::remove_var(name),
                }
            }
        }
    }
}

fn test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn captured_output(content: &str) -> &str {
    content.split_once('\n').map_or(content, |(_, body)| body)
}

fn parse_direct_child_pid(content: &str) -> u32 {
    captured_output(content)
        .lines()
        .find_map(|line| line.strip_prefix("direct-child-pid="))
        .and_then(|value| value.trim().parse().ok())
        .expect("timeout walkthrough must record the direct PowerShell child PID")
}

fn direct_shell_child_is_gone(pid: u32) -> bool {
    let probe = format!(
        "if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ exit 1 }} else {{ exit 0 }}"
    );
    Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &probe,
        ])
        .status()
        .is_ok_and(|status| status.success())
}

fn runner_value(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| "<unset>".to_owned())
}

#[tokio::test]
async fn windows_spawned_powershell_child_cannot_observe_dangerous_parent_environment() {
    let original = [
        ("LD_PRELOAD", env::var_os("LD_PRELOAD")),
        (
            "DYLD_INSERT_LIBRARIES",
            env::var_os("DYLD_INSERT_LIBRARIES"),
        ),
    ];

    {
        let _environment_lock = ENVIRONMENT_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _restore = EnvironmentRestore::set(&[
            ("LD_PRELOAD", LD_PRELOAD_SENTINEL),
            (
                "DYLD_INSERT_LIBRARIES",
                DYLD_INSERT_LIBRARIES_SENTINEL,
            ),
        ]);

        let tool = BashTool::new(test_dir());
        let command = r#"$names = @('LD_PRELOAD','DYLD_INSERT_LIBRARIES'); foreach ($name in $names) { $value = [Environment]::GetEnvironmentVariable($name); if ($null -eq $value) { Write-Output "$name=<missing>" } else { Write-Output "$name=$value" } }"#;
        let result = tool
            .execute(serde_json::json!({ "command": command }))
            .await;

        assert!(
            !result.is_error,
            "production PowerShell child failed: {}",
            result.content
        );
        assert!(result.content.contains("LD_PRELOAD=<missing>"));
        assert!(
            result
                .content
                .contains("DYLD_INSERT_LIBRARIES=<missing>")
        );
        assert!(!result.content.contains(LD_PRELOAD_SENTINEL));
        assert!(!result.content.contains(DYLD_INSERT_LIBRARIES_SENTINEL));

        assert_eq!(env::var("LD_PRELOAD").as_deref(), Ok(LD_PRELOAD_SENTINEL));
        assert_eq!(
            env::var("DYLD_INSERT_LIBRARIES").as_deref(),
            Ok(DYLD_INSERT_LIBRARIES_SENTINEL)
        );

        println!("I170_ENV_ISOLATION production_path=BashTool::execute");
        println!("I170_ENV_ISOLATION child_LD_PRELOAD=<missing>");
        println!("I170_ENV_ISOLATION child_DYLD_INSERT_LIBRARIES=<missing>");
        println!("I170_ENV_ISOLATION parent_LD_PRELOAD={LD_PRELOAD_SENTINEL}");
        println!(
            "I170_ENV_ISOLATION parent_DYLD_INSERT_LIBRARIES={DYLD_INSERT_LIBRARIES_SENTINEL}"
        );
        println!("I170_ENV_ISOLATION locking=process-local-static-mutex");
        println!("I170_ENV_ISOLATION cleanup=drop-guard-restores-present-or-absent-state");
    }

    for (name, expected) in original {
        assert_eq!(
            env::var_os(name),
            expected,
            "{name} was not restored to its pre-test state"
        );
    }
}

#[tokio::test]
async fn windows_direct_powershell_walkthrough_records_exact_head_and_timeout_evidence() {
    let head = runner_value("GITHUB_SHA");
    if let Ok(expected_head) = env::var("I170_EXPECTED_HEAD") {
        assert_eq!(
            head, expected_head,
            "walkthrough must execute on the workflow's exact checked-out Head"
        );
    }

    println!("I170_WALKTHROUGH exact_head={head}");
    println!("I170_WALKTHROUGH runner_os={}", runner_value("RUNNER_OS"));
    println!("I170_WALKTHROUGH runner_name={}", runner_value("RUNNER_NAME"));
    println!("I170_WALKTHROUGH image_os={}", runner_value("ImageOS"));
    println!(
        "I170_WALKTHROUGH image_version={}",
        runner_value("ImageVersion")
    );

    let working_dir = test_dir();
    let tool = BashTool::new(working_dir.clone());
    let normal = tool
        .execute(serde_json::json!({ "command": NORMAL_COMMAND }))
        .await;

    assert!(normal.is_error, "exit 7 must be surfaced as a tool error");
    assert!(normal.content.contains("stdout-ok"));
    assert!(normal.content.contains("stderr-ok"));
    assert!(normal.content.ends_with("[exit 7]"));
    assert!(
        normal
            .content
            .to_ascii_lowercase()
            .contains(&working_dir.to_string_lossy().to_ascii_lowercase()),
        "PowerShell did not report the configured working directory: {}",
        normal.content
    );

    println!("I170_WALKTHROUGH normal_command={NORMAL_COMMAND}");
    println!("I170_WALKTHROUGH normal_stdout=stdout-ok");
    println!("I170_WALKTHROUGH normal_stderr=stderr-ok");
    println!(
        "I170_WALKTHROUGH normal_cwd={}",
        working_dir.to_string_lossy()
    );
    println!("I170_WALKTHROUGH normal_exit_code=7");
    println!("I170_WALKTHROUGH normal_result_begin");
    println!("{}", normal.content);
    println!("I170_WALKTHROUGH normal_result_end");

    let timeout_tool = BashTool::new(working_dir).with_timeout(Duration::from_secs(1));
    let started = Instant::now();
    let timeout = timeout_tool
        .execute(serde_json::json!({ "command": TIMEOUT_COMMAND }))
        .await;
    let elapsed = started.elapsed();
    let timeout_output = captured_output(&timeout.content);

    assert!(timeout.is_error);
    assert!(timeout_output.contains("before-timeout"));
    assert!(timeout_output.contains("[timeout]"));
    assert!(
        !timeout_output.lines().any(|line| line == "after-timeout"),
        "after-timeout appeared as completed command output: {}",
        timeout.content
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "timeout walkthrough exceeded the bounded deadline: {elapsed:?}"
    );

    let direct_child_pid = parse_direct_child_pid(&timeout.content);
    assert!(
        direct_shell_child_is_gone(direct_child_pid),
        "direct PowerShell child {direct_child_pid} remained alive after BashTool returned"
    );

    println!("I170_WALKTHROUGH timeout_command={TIMEOUT_COMMAND}");
    println!("I170_WALKTHROUGH timeout_partial_output=before-timeout");
    println!("I170_WALKTHROUGH timeout_marker=[timeout]");
    println!(
        "I170_WALKTHROUGH timeout_elapsed_ms={}",
        elapsed.as_millis()
    );
    println!("I170_WALKTHROUGH timeout_after_output_observed=false");
    println!("I170_WALKTHROUGH timeout_direct_child_pid={direct_child_pid}");
    println!("I170_WALKTHROUGH timeout_direct_child_cleaned=true");
    println!("I170_WALKTHROUGH timeout_result_begin");
    println!("{}", timeout.content);
    println!("I170_WALKTHROUGH timeout_result_end");
    println!(
        "I170_WALKTHROUGH residual=Talos kills and waits for the direct shell child on timeout, but does not currently guarantee termination of the full descendant process tree created by the shell."
    );
}
