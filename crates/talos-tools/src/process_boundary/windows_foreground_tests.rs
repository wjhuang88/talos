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
async fn foreground_output_limit_terminates_writer_and_confirms_cleanup() {
    let owner = OwnedCommandExecutor::default();
    let result = owner
        .operation()
        .execute(
            powershell(
                "$bytes = New-Object byte[] 8192; $out = [Console]::OpenStandardOutput(); while ($true) { $out.Write($bytes, 0, $bytes.Length); $out.Flush() }",
            ),
            Duration::from_secs(20),
            1024,
        )
        .await;
    owner.seal();
    owner
        .wait(Duration::from_secs(10))
        .await
        .expect("confirmed output-limit cleanup");
    assert!(
        result
            .expect_err("output must exceed allowance")
            .contains("output limit"),
    );
}

#[tokio::test]
async fn foreground_leader_exit_reaps_descendant_with_redirected_stdio() {
    let directory = tempfile::tempdir().expect("descendant fixture");
    let quote = |path: &std::path::Path| path.to_string_lossy().replace('\'', "''");
    let pid = directory.path().join("child.pid");
    let ready = directory.path().join("child.ready");
    let child_script = directory.path().join("child.ps1");
    let stdout = directory.path().join("child.stdout");
    let stderr = directory.path().join("child.stderr");
    std::fs::write(
        &child_script,
        format!(
            "$ErrorActionPreference = 'Stop'; [System.IO.File]::WriteAllText('{}', [string]$PID); [System.IO.File]::WriteAllText('{}', 'ready'); while ($true) {{ Start-Sleep -Seconds 60 }}",
            quote(&pid),
            quote(&ready),
        ),
    )
    .expect("child script");
    // The descendant owns different stdio files, so EOF from the leader is
    // insufficient evidence of cleanup. Its acknowledged loop cannot exit naturally.
    let script = format!(
        "$ErrorActionPreference = 'Stop'; $child = Start-Process -FilePath (Join-Path $PSHOME 'powershell.exe') -ArgumentList '-NoLogo','-NoProfile','-NonInteractive','-ExecutionPolicy','Bypass','-File','\"{}\"' -RedirectStandardOutput '{}' -RedirectStandardError '{}' -PassThru; $until = [DateTime]::UtcNow.AddSeconds(10); while (!(Test-Path -LiteralPath '{}')) {{ if ($child.HasExited -or [DateTime]::UtcNow -gt $until) {{ throw 'child failed to acknowledge' }}; Start-Sleep -Milliseconds 10 }}; if ($child.HasExited) {{ throw 'child exited before leader' }}; [System.IO.File]::WriteAllText('{}', [string]$child.Id)",
        quote(&child_script),
        quote(&stdout),
        quote(&stderr),
        quote(&ready),
        quote(&pid),
    );
    let owner = OwnedCommandExecutor::default();
    let result = owner
        .operation()
        .execute(powershell(&script), Duration::from_secs(20), 4096)
        .await;
    owner.seal();
    let cleanup = owner.wait(Duration::from_secs(10)).await;
    // Observe and stop survivors before asserting execution or cleanup results.
    // The child publishes its own PID before entering its persistent loop.
    let acknowledgement = std::fs::read_to_string(ready);
    let pid: u32 = std::fs::read_to_string(pid)
        .expect("descendant was started")
        .parse()
        .expect("numeric descendant id");
    let observer = OwnedCommandExecutor::default();
    let observed = observer
        .operation()
        .execute(
            powershell(&format!(
                "if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ $process = Get-CimInstance Win32_Process -Filter 'ProcessId = {pid}'; if ($process.CommandLine -and $process.CommandLine.Contains('{}')) {{ Stop-Process -Id {pid} -Force -ErrorAction Stop }}; exit 1 }} else {{ exit 0 }}",
                quote(&child_script),
            )),
            Duration::from_secs(20),
            4096,
        )
        .await;
    observer.seal();
    observer
        .wait(Duration::from_secs(10))
        .await
        .expect("observer cleanup");
    assert!(observed.expect("observe descendant").status.success());
    cleanup.expect("confirmed descendant cleanup");
    assert!(result.expect("leader execution").status.success());
    assert_eq!(acknowledgement.expect("child acknowledgement"), "ready");
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
