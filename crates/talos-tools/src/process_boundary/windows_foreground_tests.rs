use super::*;

#[tokio::test]
async fn panicked_io_worker_never_confirms_cleanup() {
    let mut task = tokio::spawn(async {
        panic!("injected IO panic");
        #[allow(unreachable_code)]
        Ok::<(), String>(())
    });
    let mut joined = None;
    assert!(join(&mut task, &mut joined).await.is_err());
    assert!(!finish_io(&mut task, joined, tokio::time::Instant::now()).await);
}

#[tokio::test]
async fn ordinary_io_error_has_completed_resource_ownership() {
    let mut task = tokio::spawn(async { Err::<(), String>("injected IO error".into()) });
    let mut joined = None;
    assert!(join(&mut task, &mut joined).await.is_err());
    assert!(finish_io(&mut task, joined, tokio::time::Instant::now()).await);
}

#[tokio::test]
async fn dropped_reservation_is_sticky_but_other_operations_are_isolated() {
    let owner = OwnedCommandExecutor::default();
    let failed = owner.operation();
    let clean = owner.operation();
    drop(failed.scope.register().expect("admitted"));
    assert!(matches!(
        failed.scope.wait(Duration::from_secs(1)).await,
        Err(ToolCleanupError::Unconfirmed)
    ));
    assert!(clean.scope.wait(Duration::from_secs(1)).await.is_ok());
    assert!(matches!(
        owner.wait(Duration::from_secs(1)).await,
        Err(ToolCleanupError::Unconfirmed)
    ));
    owner.seal();
    assert!(clean.scope.register().is_err());
}

fn powershell(script: &str) -> Command {
    let root = std::env::var_os("SystemRoot").expect("Windows system root");
    let executable =
        std::path::PathBuf::from(root).join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
    let mut command = Command::new(executable);
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .current_dir(std::env::current_dir().expect("current directory"))
        .env_clear()
        .envs(std::env::vars_os());
    command
}

#[tokio::test]
async fn foreground_finite_binary_input_reaches_eof_and_returns_exact_output() {
    let owner = OwnedCommandExecutor::default();
    let operation = owner.operation();
    let input = vec![0, 1, 127, 128, 255, 10, 13];
    let result = operation
        .execute_with_input(
            powershell("[Console]::OpenStandardInput().CopyTo([Console]::OpenStandardOutput())"),
            Duration::from_secs(20),
            4096,
            input.clone(),
        )
        .await;
    owner.seal();
    owner
        .wait(Duration::from_secs(10))
        .await
        .expect("confirmed cleanup");
    let output = result.expect("finite stdin execution");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, input);
}

#[tokio::test]
async fn foreground_timeout_confirms_job_and_io_cleanup() {
    let owner = OwnedCommandExecutor::default();
    let result = owner
        .operation()
        .execute(
            powershell("Start-Sleep -Seconds 60"),
            Duration::from_millis(100),
            4096,
        )
        .await;
    owner.seal();
    owner
        .wait(Duration::from_secs(10))
        .await
        .expect("confirmed timeout cleanup");
    assert!(result.expect_err("deadline must fail").contains("deadline"));
}

#[tokio::test]
async fn dropped_foreground_wait_keeps_cleanup_owned() {
    let owner = OwnedCommandExecutor::default();
    let operation = owner.operation();
    let directory = tempfile::tempdir().expect("ready directory");
    let ready = directory.path().join("ready");
    let script = format!(
        "[System.IO.File]::WriteAllText('{}', 'ready'); Start-Sleep -Seconds 60",
        ready.to_string_lossy().replace('\'', "''"),
    );
    let task = tokio::spawn(async move {
        operation
            .execute(powershell(&script), Duration::from_secs(60), 4096)
            .await
    });
    let started = tokio::time::timeout(Duration::from_secs(20), async {
        while !tokio::fs::try_exists(&ready).await.expect("ready metadata") {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;
    // Cancel only after the child has acknowledged running; always clean up
    // before asserting readiness, including on a slow or failing Windows host.
    task.abort();
    let _ = task.await;
    owner.seal();
    let cleanup = owner.wait(Duration::from_secs(10)).await;
    assert!(started.is_ok(), "child never acknowledged startup");
    cleanup.expect("confirmed cancellation cleanup");
}
