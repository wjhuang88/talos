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
END_BY = float("inf")


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
        ("                let result = windows_sys::Win32::System::Threading::WaitForSingleObject(", "                windows_auto_probe(\"process_wait_enter\", process as usize);\n                let result = windows_sys::Win32::System::Threading::WaitForSingleObject("),
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
    return {BOUNDARY: boundary, AUTO: auto}


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


def build(output: Path, phase: str) -> Path:
    code, text = execute(["cargo", "test", "-p", "talos-desktop", "--features", "desktop-ui", "--locked", "--no-run", "--message-format=json"], output / f"{phase}-build.log", 1200)
    if code:
        raise RuntimeError(f"{phase} build failed: {code}; see artifact build log")
    binaries = []
    for line in text.splitlines():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if record.get("reason") == "compiler-artifact" and record.get("profile", {}).get("test") and record.get("target", {}).get("name") == "talos-desktop-mock" and record.get("executable"):
            binaries.append(Path(record["executable"]))
    if len(set(binaries)) != 1:
        raise RuntimeError(f"expected exactly one Desktop test executable, found {binaries}")
    return binaries[0]


def matrix(binary: Path, output: Path, phase: str, results: list[dict]) -> None:
    modes = [
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
            if code == 0 and "test result: ok." not in text:
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
    args = parser.parse_args()
    original_bytes = {path: (ROOT / path).read_bytes() for path in (BOUNDARY, AUTO)}
    originals = {path: data.decode("utf-8").replace("\r\n", "\n") for path, data in original_bytes.items()}
    patched = probes(originals)
    if args.self_test:
        assert all(patched[path] != originals[path] for path in originals)
        assert "Duration::from_secs(15)" in patched[AUTO]
        assert "Duration::from_secs(10)" in patched[AUTO]
        assert patched[BOUNDARY].count("windows_auto_probe(") >= 10
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
    manifest = {"head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), "python": sys.version, "runner": {key: os.environ.get(key) for key in ("RUNNER_OS", "RUNNER_ARCH", "ImageOS", "ImageVersion")}, "source_sha256": {path: hashlib.sha256(text.encode()).hexdigest() for path, text in originals.items()}}
    (output / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    overlay = "".join("".join(difflib.unified_diff(originals[path].splitlines(keepends=True), patched[path].splitlines(keepends=True), fromfile=path, tofile=path)) for path in originals)
    (output / "probe-overlay.patch").write_text(overlay, encoding="utf-8")
    results: list[dict] = []
    try:
        matrix(build(output, "baseline"), output, "baseline", results)
        for path, text in patched.items():
            (ROOT / path).write_text(text, encoding="utf-8", newline="\n")
        matrix(build(output, "instrumented"), output, "instrumented", results)
    finally:
        for path, data in original_bytes.items():
            (ROOT / path).write_bytes(data)
        print("WINDOWS_AUTO_DIAG temporary Rust probes restored", flush=True)
    return int(any(result["exit_code"] != 0 for result in results))


if __name__ == "__main__":
    raise SystemExit(main())
