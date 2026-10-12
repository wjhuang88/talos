#!/usr/bin/env python3
"""Bounded Windows CI experiment; restore every temporary Rust probe afterward.

No production patch, timeout change, retry-to-green, credentials, or command-content logging.
Run --self-test on any host to validate exact probe anchors without writing source files.
"""

from __future__ import annotations

import argparse
import difflib
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
BOUNDARY = "crates/talos-tools/src/process_boundary.rs"
AUTO = "crates/talos-desktop/src/runtime_host/auto_tests.rs"
HOST = "crates/talos-desktop/src/runtime_host.rs"
END_BY = float("inf")
CONTROLLED_TEST = "process_boundary::windows_tests::windows_auto_diagnostic_descendant_pipe_hold_is_observable_and_cleanup_bounded"
CONTROLLED_FIXTURE = '''    #[tokio::test(flavor = "current_thread")]
    async fn windows_auto_diagnostic_descendant_pipe_hold_is_observable_and_cleanup_bounded() {
        let script = r#"$child = 'Write-Output "WINDOWS_AUTO_PIPE_HOLDER_READY"; Start-Sleep -Seconds 60'; $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($child)); Start-Process -FilePath "$env:WINDIR\\System32\\WindowsPowerShell\\v1.0\\powershell.exe" -ArgumentList @('-NoLogo', '-NoProfile', '-NonInteractive', '-EncodedCommand', $encoded) -NoNewWindow | Out-Null; exit 0"#;
        let launched = Box::new(powershell(script)).launch().await.expect("owned fixture launch");
        let control = launched.control;
        let mut events = launched.events;
        let readiness = tokio::time::timeout(Duration::from_secs(15), async {
            let mut bytes = Vec::new();
            loop {
                match events.recv().await {
                    Some(BackgroundProcessEvent::Output(chunk)) => {
                        bytes.extend_from_slice(&chunk.bytes);
                        if String::from_utf8_lossy(&bytes).contains("WINDOWS_AUTO_PIPE_HOLDER_READY") {
                            return Ok(());
                        }
                        if bytes.len() > 32768 { return Err("fixture output exceeded bound".to_owned()); }
                    }
                    Some(BackgroundProcessEvent::SupervisionFailed(error)) => return Err(error),
                    Some(BackgroundProcessEvent::Exited(_)) | None => return Err("terminal before readiness".to_owned()),
                }
            }
        }).await;
        let held = if matches!(&readiness, Ok(Ok(()))) {
            tokio::time::timeout(Duration::from_millis(500), async {
                loop {
                    match events.recv().await {
                        Some(BackgroundProcessEvent::Output(_)) => {}
                        _ => break,
                    }
                }
            }).await.is_err()
        } else { false };
        eprintln!("WINDOWS_AUTO_DIAG controlled_pipe readiness={:?} pending_while_descendant_holds_pipe={}", readiness, held);
        // Always clean the owned Job Object before any assertion, including readiness failure.
        windows_auto_probe("controlled_cleanup_begin", 0);
        let cleanup = control.force_terminate().await;
        let terminal = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                match events.recv().await {
                    Some(BackgroundProcessEvent::Output(_)) => {}
                    Some(BackgroundProcessEvent::Exited(_)) => return true,
                    Some(BackgroundProcessEvent::SupervisionFailed(_)) | None => return false,
                }
            }
        }).await;
        eprintln!("WINDOWS_AUTO_DIAG controlled_pipe cleanup={:?} terminal_after_cleanup={:?}", cleanup, terminal);
        assert!(matches!(readiness, Ok(Ok(()))), "holder readiness: {readiness:?}");
        assert!(held, "launcher must not invent completion while descendant holds its pipes");
        assert!(cleanup.is_ok(), "Job Object termination: {cleanup:?}");
        assert!(matches!(terminal, Ok(true)), "bounded terminal after cleanup: {terminal:?}");
    }

'''


def replace_once(text: str, old: str, new: str) -> str:
    if text.count(old) != 1:
        raise ValueError(f"probe anchor must occur exactly once: {old[:100]!r}")
    return text.replace(old, new, 1)


def probes(originals: dict[str, str]) -> dict[str, str]:
    boundary = originals[BOUNDARY]
    helper = '''#[cfg(windows)]
fn windows_auto_probe(stage: &str, identity: usize) {
    eprintln!("WINDOWS_AUTO_DIAG time={:?} pid={} thread={:?} stage={} identity={}",
        std::time::SystemTime::now(), std::process::id(),
        std::thread::current().id(), stage, identity);
}

'''
    boundary = replace_once(boundary, "/// Windows Job Object launcher implementing ADR-068's assigned-before-resume boundary.", helper + "/// Windows Job Object launcher implementing ADR-068's assigned-before-resume boundary.")
    for old, new in [
        ("        let launched = create_windows_job(&self.program, &self.args, &self.cwd, &self.env)?;", "        windows_auto_probe(\"launch_enter\", 0);\n        let launched = create_windows_job(&self.program, &self.args, &self.cwd, &self.env)?;\n        windows_auto_probe(\"launch_return\", launched.process as usize);"),
        ("                BackgroundOutputStream::Stdout,\n                tx.clone(),", "                BackgroundOutputStream::Stdout,\n                tx.clone(),\n                process,"),
        ("                BackgroundOutputStream::Stderr,\n                tx.clone(),", "                BackgroundOutputStream::Stderr,\n                tx.clone(),\n                process,"),
        ('''                let result = windows_sys::Win32::System::Threading::WaitForSingleObject(
                    process,
                    windows_sys::Win32::System::Threading::INFINITE,
                );''', '''                windows_auto_probe("process_wait_enter", process as usize);
                let result = loop {
                    let observed = windows_sys::Win32::System::Threading::WaitForSingleObject(process, 1_000);
                    windows_auto_probe("native_wait_observed", process as usize);
                    let mut observed_code = 1u32;
                    let exit_read_ok = windows_sys::Win32::System::Threading::GetExitCodeProcess(process, &mut observed_code);
                    eprintln!("WINDOWS_AUTO_DIAG poll_result={} exit_read_ok={} exit_code={} identity={}", observed, exit_read_ok, observed_code, process as usize);
                    let zero_time = || windows_sys::Win32::Foundation::FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
                    let (mut created, mut exited, mut kernel, mut user) = (zero_time(), zero_time(), zero_time(), zero_time());
                    let times_read_ok = windows_sys::Win32::System::Threading::GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user);
                    let ticks = |value: windows_sys::Win32::Foundation::FILETIME| ((value.dwHighDateTime as u64) << 32) | value.dwLowDateTime as u64;
                    eprintln!("WINDOWS_AUTO_DIAG times_read_ok={} kernel_100ns={} user_100ns={} identity={}", times_read_ok, ticks(kernel), ticks(user), process as usize);
                    if observed != windows_sys::Win32::Foundation::WAIT_TIMEOUT { break observed; }
                };'''),
        ("                windows_sys::Win32::Foundation::CloseHandle(process);", "                windows_auto_probe(\"process_handle_close_begin\", process as usize);\n                windows_sys::Win32::Foundation::CloseHandle(process);\n                windows_auto_probe(\"process_handle_close_return\", process as usize);"),
        ("            unsafe { windows_sys::Win32::Foundation::CloseHandle(self.job) };", "            windows_auto_probe(\"job_close_begin\", self.job as usize);\n            unsafe { windows_sys::Win32::Foundation::CloseHandle(self.job) };\n            windows_auto_probe(\"job_close_return\", self.job as usize);"),
        ("                (result, code)\n            })", "                windows_auto_probe(\"process_wait_return\", process as usize);\n                eprintln!(\"WINDOWS_AUTO_DIAG wait_result={} exit_code={} identity={}\", result, code, process as usize);\n                (result, code)\n            })"),
        ("            let output = (stdout_task.await, stderr_task.await);", "            windows_auto_probe(\"process_wait_joined\", process);\n            let output = (stdout_task.await, stderr_task.await);\n            windows_auto_probe(\"both_pipes_joined\", process);"),
        ("        let count = reader\n            .read(&mut buffer)", "        windows_auto_probe(match stream { BackgroundOutputStream::Stdout => \"stdout_read_enter\", BackgroundOutputStream::Stderr => \"stderr_read_enter\" }, process);\n        let count = reader\n            .read(&mut buffer)"),
        ("        if count == 0 {\n            return Ok(());\n        }", "        windows_auto_probe(match stream { BackgroundOutputStream::Stdout => \"stdout_read_return\", BackgroundOutputStream::Stderr => \"stderr_read_return\" }, process);\n        eprintln!(\"WINDOWS_AUTO_DIAG bytes={} identity={} stream={:?}\", count, process, stream);\n        if count == 0 {\n            return Ok(());\n        }"),
        ("    sender: tokio::sync::mpsc::Sender<BackgroundProcessEvent>,\n) -> Result<(), String> {", "    sender: tokio::sync::mpsc::Sender<BackgroundProcessEvent>,\n    process: usize,\n) -> Result<(), String> {"),
        ("        if CreateProcessW(", "        windows_auto_probe(\"create_process_enter\", 0);\n        if CreateProcessW("),
        ("        process = info.hProcess;", "        windows_auto_probe(\"create_process_return\", info.hProcess as usize);\n        eprintln!(\"WINDOWS_AUTO_DIAG child_pid={} identity={}\", info.dwProcessId, info.hProcess as usize);\n        process = info.hProcess;"),
        ("        CloseHandle(thread);\n        thread =", "        windows_auto_probe(\"assigned_and_resumed\", process as usize);\n        CloseHandle(thread);\n        thread ="),
    ]:
        boundary = replace_once(boundary, old, new)
    boundary = replace_once(boundary, "    fn powershell(script: &str) -> WindowsBackgroundLauncher {", CONTROLLED_FIXTURE + "    fn powershell(script: &str) -> WindowsBackgroundLauncher {")
    auto = originals[AUTO]
    auto = replace_once(auto, "    let mut stages = Vec::new();", '''    let mut stages = Vec::new();
    let heartbeat = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            eprintln!("WINDOWS_AUTO_DIAG test_heartbeat enabled={} allow={} elapsed={:?}", enabled, allow, started.elapsed());
        }
    });''')
    auto = replace_once(auto, "    if outcome.is_err() {", '''    heartbeat.abort();
    eprintln!("WINDOWS_AUTO_DIAG test_outcome enabled={} allow={} ok={} stages={:?}", enabled, allow, outcome.is_ok(), stages);
    if outcome.is_err() {''')
    host = replace_once(originals[HOST], "                        let terminal = executor.block_on(async move {\n                            let (provider, context_limit, auto_enabled) = match provider() {", '''                        let terminal = executor.block_on(async move {
                            windows_auto_host_stage("host_task_enter");
                            // Temporary CI observation of this host executor, not the test executor.
                            tokio::spawn(async move {
                                loop {
                                    eprintln!("WINDOWS_AUTO_DIAG time={:?} pid={} thread={:?} stage=host_heartbeat",
                                        std::time::SystemTime::now(), std::process::id(), std::thread::current().id());
                                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                }
                            });
                            windows_auto_host_stage("provider_enter");
                            let (provider, context_limit, auto_enabled) = match provider() {''')
    host_helper = '''fn windows_auto_host_stage(stage: &str) {
    eprintln!("WINDOWS_AUTO_DIAG time={:?} pid={} thread={:?} stage={}",
        std::time::SystemTime::now(), std::process::id(), std::thread::current().id(), stage);
}

'''
    host = replace_once(host, "#[allow(clippy::too_many_arguments)]\nasync fn run_host(", host_helper + "#[allow(clippy::too_many_arguments)]\nasync fn run_host(")
    for old, new in [
        ("                            let approval_root = workspace_root.clone();", '                            windows_auto_host_stage("provider_return");\n                            windows_auto_host_stage("session_setup_enter");\n                            let approval_root = workspace_root.clone();'),
        ("                            let mut builder = RuntimeBuilder::new()", '                            windows_auto_host_stage("session_setup_return");\n                            windows_auto_host_stage("builder_setup_enter");\n                            let mut builder = RuntimeBuilder::new()'),
        ("                            match builder.build() {", '                            windows_auto_host_stage("builder_setup_return");\n                            windows_auto_host_stage("builder_build_enter");\n                            let build_result = builder.build();\n                            windows_auto_host_stage("builder_build_return");\n                            match build_result {'),
        ("    let initial_work = load_work_projection(work_source.clone()).await;", '    windows_auto_host_stage("initial_projection_enter");\n    let initial_work = load_work_projection(work_source.clone()).await;\n    windows_auto_host_stage("initial_projection_return");'),
        ("    loop {\n        tokio::select! {\n            result = async {", '    windows_auto_host_stage("command_loop_ready");\n    loop {\n        tokio::select! {\n            result = async {'),
        ("                    Some(RuntimeCommand::Submit(message)) => {", '                    Some(RuntimeCommand::Submit(message)) => {\n                        windows_auto_host_stage("submit_received");'),
        ("                        if let Err(error) = handle.submit(message).await\n", '                        windows_auto_host_stage("submit_enter");\n                        let submit_result = handle.submit(message).await;\n                        windows_auto_host_stage("submit_return");\n                        if let Err(error) = submit_result\n'),
    ]:
        host = replace_once(host, old, new)
    return {BOUNDARY: boundary, AUTO: auto, HOST: host}


def execute(command: list[str], log: Path, timeout: int) -> tuple[int, str]:
    started = time.monotonic()
    remaining = END_BY - started
    if remaining < 120:
        raise RuntimeError("total experiment budget exhausted; preserve restoration/upload margin")
    timeout = min(timeout, int(remaining - 60))
    with log.open("w", encoding="utf-8") as output:
        output.write(json.dumps({"command": command, "started": time.time()}) + "\n")
        output.flush()
        child = subprocess.Popen(command, cwd=ROOT, stdout=output, stderr=subprocess.STDOUT)
        try:
            code = child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            # Exact owned PID only; do not kill unrelated runner processes.
            code = 124
            try:
                cleanup = subprocess.run(["taskkill", "/PID", str(child.pid), "/T", "/F"], check=False, timeout=15, stdout=output, stderr=subprocess.STDOUT)
                output.write(json.dumps({"cleanup_exit_code": cleanup.returncode}) + "\n")
                child.wait(timeout=15)
            except (OSError, subprocess.TimeoutExpired) as error:
                output.write(json.dumps({"cleanup_error": str(error), "child_pid": child.pid}) + "\n")
        output.write(json.dumps({"exit_code": code, "elapsed": time.monotonic() - started}) + "\n")
    text = log.read_text(encoding="utf-8", errors="replace")
    for line in text.splitlines():
        if "WINDOWS_AUTO_DIAG" in line or "test result:" in line or "watchdog" in line or "panicked at" in line:
            print(line, flush=True)
    return code, text


def build(output: Path, phase: str, tools: bool = False) -> Path:
    command = ["cargo", "test", "-p", "talos-tools", "--features", "shell", "--lib"] if tools else ["cargo", "test", "-p", "talos-desktop", "--features", "desktop-ui"]
    command += ["--locked", "--no-run", "--message-format=json"]
    code, text = execute(command, output / f"{phase}-build.log", 1200)
    if code:
        raise RuntimeError(f"{phase} build failed: {code}; see artifact build log")
    binaries = []
    for line in text.splitlines():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        target = "talos_tools" if tools else "talos-desktop-mock"
        if record.get("reason") == "compiler-artifact" and record.get("profile", {}).get("test") and record.get("target", {}).get("name") == target and record.get("executable"):
            binaries.append(Path(record["executable"]))
    if len(set(binaries)) != 1:
        raise RuntimeError(f"expected exactly one Desktop test executable, found {binaries}")
    return binaries[0]


def matrix(binary: Path, output: Path, phase: str, results: list[dict], stress: bool = False, fresh_first: bool = False) -> None:
    modes = [
        ("suite-first", ["--test-threads=32"], 1),
        ("single", ["runtime_host::auto_tests::enabled_auto_allow_once_reports_without_manual_approval", "--exact", "--test-threads=1"], 50),
        ("suite-parallel", ["--test-threads=32"], 9),
    ] if fresh_first else [
        ("single", ["runtime_host::auto_tests::enabled_auto_allow_once_reports_without_manual_approval", "--exact", "--test-threads=1"], 50),
        ("suite-parallel", ["--test-threads=32"], 10),
    ] if stress else [
        ("single", ["runtime_host::auto_tests::enabled_auto_allow_once_reports_without_manual_approval", "--exact", "--test-threads=1"], 5),
        ("suite-serial", ["--test-threads=1"], 3),
        ("suite-parallel", ["--test-threads=16"], 5),
    ]
    for name, args, repetitions in modes:
        for index in range(repetitions):
            label = f"{phase}-{name}-{index + 1}"
            print(f"WINDOWS_AUTO_DIAG case={label}", flush=True)
            code, text = execute([str(binary), *args, "--nocapture"], output / f"{label}.log", 90)
            # Filtering is experimental only; mandatory full CI is unchanged.
            if code == 0 and ("test result: ok." not in text or "test result: ok. 0 passed;" in text):
                code = 125
            if name == "single" and "1 passed; 0 failed" not in text:
                code = code or 125
            results.append({"case": label, "exit_code": code})
            (output / "results.json").write_text(json.dumps(results, indent=2), encoding="utf-8")
            # Never discard a failed attempt or rerun it to replace its result.
            if code == 124:
                raise RuntimeError("outer watchdog expired; stopped bounded experiment")


def main() -> int:
    global END_BY
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--controlled-only", action="store_true", help="run only the controlled mechanism after a retained full matrix")
    mode.add_argument("--stress", action="store_true", help="bounded original-test sampling with host-runtime heartbeat; no synthetic fixture")
    mode.add_argument("--fresh-phase", choices=("baseline", "instrumented"), help="one phase per fresh runner; full parallel suite before isolated Talos calls")
    args = parser.parse_args()
    original_bytes = {path: (ROOT / path).read_bytes() for path in (BOUNDARY, AUTO, HOST)}
    originals = {path: data.decode("utf-8").replace("\r\n", "\n") for path, data in original_bytes.items()}
    patched = probes(originals)
    if args.self_test:
        assert all(patched[path] != originals[path] for path in originals)
        assert "Duration::from_secs(15)" in patched[AUTO]
        assert "Duration::from_secs(10)" in patched[AUTO]
        assert patched[BOUNDARY].count("windows_auto_probe(") >= 10
        assert patched[HOST].count("stage=host_heartbeat") == 1
        assert "WaitForSingleObject(process, 1_000)" in patched[BOUNDARY]
        assert "observed != windows_sys::Win32::Foundation::WAIT_TIMEOUT" in patched[BOUNDARY]
        assert "GetProcessTimes(process," in patched[BOUNDARY]
        try:
            replace_once("duplicate duplicate", "duplicate", "replacement")
        except ValueError:
            pass
        else:
            raise AssertionError("ambiguous anchors must fail closed")
        print("Probe anchors, preserved watchdogs, and ambiguity rejection: PASS")
        return 0
    if sys.platform != "win32":
        parser.error("the experiment requires a Windows runner")
    END_BY = time.monotonic() + 35 * 60
    output = ROOT / "windows-auto-diagnostics"
    output.mkdir(exist_ok=True)
    manifest = {"head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), "mode": "fresh-" + args.fresh_phase if args.fresh_phase else "stress" if args.stress else "controlled-only" if args.controlled_only else "full", "python": sys.version, "cpu_count": os.cpu_count(), "runner": {key: os.environ.get(key) for key in ("RUNNER_OS", "RUNNER_ARCH", "ImageOS", "ImageVersion")}, "source_sha256": {path: hashlib.sha256(text.encode()).hexdigest() for path, text in originals.items()}}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    overlay = "".join("".join(difflib.unified_diff(originals[path].splitlines(keepends=True), patched[path].splitlines(keepends=True), fromfile=path, tofile=path)) for path in originals)
    (output / "probe-overlay.patch").write_text(overlay, encoding="utf-8")
    results: list[dict] = []
    try:
        if args.fresh_phase:
            if args.fresh_phase == "instrumented":
                for path, text in patched.items():
                    (ROOT / path).write_text(text, encoding="utf-8", newline="\n")
            matrix(build(output, args.fresh_phase), output, args.fresh_phase, results, fresh_first=True)
            return int(any(result["exit_code"] != 0 for result in results))
        if not args.controlled_only:
            matrix(build(output, "baseline"), output, "baseline", results, args.stress)
        for path, text in patched.items():
            (ROOT / path).write_text(text, encoding="utf-8", newline="\n")
        if not args.controlled_only:
            matrix(build(output, "instrumented"), output, "instrumented", results, args.stress)
        if args.stress:
            return int(any(result["exit_code"] != 0 for result in results))
        binary = build(output, "controlled-pipe", tools=True)
        code, inventory = execute([str(binary), CONTROLLED_TEST, "--exact", "--list"], output / "controlled-inventory.log", 90)
        present = f"{CONTROLLED_TEST}: test" in inventory and "1 test, 0 benchmarks" in inventory
        results.append({"case": "controlled-inventory", "exit_code": code or (0 if present else 125)})
        (output / "results.json").write_text(json.dumps(results, indent=2), encoding="utf-8")
        if code or not present:
            raise RuntimeError("controlled exact fixture missing from compiled test inventory; stopped experiment")
        for index in range(2):
            label = f"controlled-pipe-{index + 1}"
            print(f"WINDOWS_AUTO_DIAG case={label}", flush=True)
            code, text = execute([str(binary), CONTROLLED_TEST, "--exact", "--test-threads=1", "--nocapture"], output / f"{label}.log", 90)
            if code == 0 and "1 passed; 0 failed" not in text:
                code = 125
            before_cleanup = text.split("stage=controlled_cleanup_begin", 1)[0]
            if code == 0 and (
                "stage=controlled_cleanup_begin" not in text
                or "stage=process_wait_return" not in before_cleanup
                or "wait_result=0 exit_code=0" not in before_cleanup
                or "pending_while_descendant_holds_pipe=true" not in before_cleanup
            ):
                code = 125
                print("WINDOWS_AUTO_DIAG controlled leader-exit observation missing; evidence inconclusive", flush=True)
            results.append({"case": label, "exit_code": code})
            (output / "results.json").write_text(json.dumps(results, indent=2), encoding="utf-8")
            if code == 124:
                raise RuntimeError("controlled pipe watchdog expired; stopped experiment")
    finally:
        for path, data in original_bytes.items():
            (ROOT / path).write_bytes(data)
        print("WINDOWS_AUTO_DIAG temporary Rust probes restored", flush=True)
    return int(any(result["exit_code"] != 0 for result in results))


if __name__ == "__main__":
    raise SystemExit(main())
