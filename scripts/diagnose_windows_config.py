#!/usr/bin/env python3
"""Temporary, bounded observation of CI3275's original config fixture; no repair."""
from __future__ import annotations

import argparse
import difflib
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

import diagnose_windows_auto as harness

ROOT = Path(__file__).resolve().parents[1]
STORE = "crates/talos-config/src/store.rs"
FS = "crates/talos-config/src/store/fs.rs"
TESTS = "crates/talos-config/src/store_finalization_tests.rs"
CASE = "store_finalization_tests::semantic_update_and_provider_unset_share_one_mutation_lock"


def probes(originals: dict[str, str]) -> dict[str, str]:
    replace = harness.replace_once
    store = originals[STORE]
    helper = '''std::thread_local! {
    static WINDOWS_CONFIG_ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(crate) struct WindowsConfigGuard(bool);

impl Drop for WindowsConfigGuard {
    fn drop(&mut self) {
        WINDOWS_CONFIG_ENABLED.with(|enabled| enabled.set(self.0));
    }
}

pub(crate) fn windows_config_enable() -> WindowsConfigGuard {
    WindowsConfigGuard(WINDOWS_CONFIG_ENABLED.with(|enabled| enabled.replace(true)))
}

pub(crate) fn windows_config_stage(stage: &str) {
    if !WINDOWS_CONFIG_ENABLED.with(std::cell::Cell::get) { return; }
    use std::io::Write;
    let record = format!("\\nWINDOWS_AUTO_DIAG time={:?} pid={} thread={:?} stage={}\\n",
        std::time::SystemTime::now(), std::process::id(), std::thread::current().id(), stage);
    let _ = std::io::stderr().lock().write_all(record.as_bytes());
}

fn windows_config_checkpoint(operation: FsOperation) {
    if WINDOWS_CONFIG_ENABLED.with(std::cell::Cell::get) {
        windows_config_stage(&format!("config_checkpoint_{operation:?}"));
    }
}

'''
    store = replace(store, "struct MutationLock {", helper + "struct MutationLock {")
    begin = store.index("    pub(crate) fn run(&self,")
    end = store.index("    pub fn recover_pending()", begin)
    run = store[begin:end]
    for anchor, stage in [
        ("        let _lock = self.acquire_mutation_lock()?;", "config_unset_lock_acquired"),
        ("        let recovery = self.recover_with_purpose(fs, RecoveryPurpose::Mutation)?;", "config_unset_recovery_return"),
        ("        let active_dir = self.txn_dir();", "config_unset_prepare_enter"),
        ("        prepare(fs, &active_dir, &txn, &txn_id)?;", "config_unset_prepare_return"),
        ("        Ok(outcome)", "config_unset_success"),
    ]:
        # Mark success before the original return; every other marker follows its anchor.
        run = replace(run, anchor, ('        windows_config_stage("' + stage + '");\n' + anchor)
                      if stage == "config_unset_success" else anchor + '\n        windows_config_stage("' + stage + '");')
    run = replace(run, "        if let Err(apply_err) = apply(", '        windows_config_stage("config_unset_apply_enter");\n        if let Err(apply_err) = apply(')
    # The success-side finalization is distinct from the two error/rollback branches.
    run = replace(run, "\n        match finalize_active(fs, &active_dir, &txn_id) {", '\n        windows_config_stage("config_unset_apply_return");\n        windows_config_stage("config_unset_finalize_enter");\n        match finalize_active(fs, &active_dir, &txn_id) {')
    store = store[:begin] + run + store[end:]
    begin = store.index("    fn acquire_mutation_lock(")
    end = store.index("    pub(super) fn txn_dir_parent", begin)
    lock = store[begin:end]
    for anchor, stage in [
        ("        let parent = self.txn_dir_parent();", "config_lock_directory_enter"),
        ("        std::fs::create_dir_all(&parent).map_err(ConfigError::IoError)?;", "config_lock_directory_return"),
        ("        let file = open_lock_file(&parent.join(MUTATION_LOCK_NAME))?;", "config_lock_open_return"),
        ("        file.lock().map_err(ConfigError::IoError)?;", "config_lock_wait_return"),
    ]:
        lock = replace(lock, anchor, anchor + '\n        windows_config_stage("' + stage + '");')
    lock = replace(lock, "        file.lock().map_err", '        windows_config_stage("config_lock_wait_enter");\n        file.lock().map_err')
    store = store[:begin] + lock + store[end:]

    fs = replace(originals[FS], "impl Fs for StdFs {", '''impl Fs for StdFs {
    fn checkpoint(&self, operation: FsOperation) -> Result<(), ConfigError> {
        super::windows_config_checkpoint(operation);
        Ok(())
    }
''')
    begin = originals[TESTS].index("fn semantic_update_and_provider_unset_share_one_mutation_lock() {")
    end = originals[TESTS].index("\n#[test]", begin)
    fixture = originals[TESTS][begin:end]
    for anchor, extra in [
        ("    use std::sync::{Arc, mpsc};", '    let _windows_config_guard = crate::store::windows_config_enable();\n    crate::store::windows_config_stage("config_fixture_begin");'),
        ("    let update = std::thread::spawn(move || {", '        let _windows_config_guard = crate::store::windows_config_enable();\n        crate::store::windows_config_stage("config_update_worker_enter");'),
        ("            .update_config(|current| {", '                crate::store::windows_config_stage("config_update_callback_enter");'),
        ('                entered_tx.send(()).expect("operation should succeed");', '                crate::store::windows_config_stage("config_update_entered_sent");'),
        ('                release_rx.recv().expect("operation should succeed");', '                crate::store::windows_config_stage("config_update_released");'),
        ('    entered_rx.recv().expect("operation should succeed");', '    crate::store::windows_config_stage("config_fixture_update_entered");'),
        ("    let unset = std::thread::spawn(move || {", '        let _windows_config_guard = crate::store::windows_config_enable();\n        crate::store::windows_config_stage("config_unset_worker_enter");'),
        ('        let result = unset_store.unset_provider("providers.custom-a");', '        crate::store::windows_config_stage(if result.is_ok() { "config_unset_return_ok" } else { "config_unset_return_error" });'),
        ('    release_tx.send(()).expect("operation should succeed");', '    crate::store::windows_config_stage("config_fixture_release_sent");'),
        ('    update.join().expect("operation should succeed");', '    crate::store::windows_config_stage("config_fixture_update_joined");\n    crate::store::windows_config_stage("config_fixture_completion_wait_enter");'),
        ('    unset.join().expect("operation should succeed");', '    crate::store::windows_config_stage("config_fixture_unset_joined");'),
    ]:
        fixture = replace(fixture, anchor, anchor + "\n" + extra)
    fixture = replace(fixture, '    release_tx.send(())', '    crate::store::windows_config_stage("config_fixture_release_enter");\n    release_tx.send(())')
    fixture = replace(fixture, '    update.join()', '    crate::store::windows_config_stage("config_fixture_update_join_enter");\n    update.join()')
    fixture = replace(fixture, '        unset_done_tx\n            .send(result)', '        crate::store::windows_config_stage("config_unset_result_send_enter");\n        unset_done_tx\n            .send(result)')
    fixture = replace(fixture, '            .send(result)\n            .expect("operation should succeed");', '            .send(result)\n            .expect("operation should succeed");\n        crate::store::windows_config_stage("config_unset_result_sent");')
    fixture = replace(fixture, '        .recv_timeout(Duration::from_secs(5))', '        .recv_timeout(Duration::from_secs(5))\n        .inspect_err(|_| crate::store::windows_config_stage("config_fixture_completion_wait_error"))\n        .inspect(|_| crate::store::windows_config_stage("config_fixture_completion_wait_return"))')
    tests = originals[TESTS][:begin] + fixture + originals[TESTS][end:]
    return {STORE: store, FS: fs, TESTS: tests}


def classify(code: int, text: str, expected: int) -> int:
    summaries = re.findall(r"^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", text, re.M)
    if len(summaries) != 1:
        return code or 125
    passed, failed, ignored = map(int, summaries[0])
    if passed + failed != expected or ignored or (code == 0 and failed):
        return code or 125
    return code


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--phase", choices=("baseline", "instrumented"))
    args = parser.parse_args()
    raw = {path: (ROOT / path).read_bytes() for path in (STORE, FS, TESTS)}
    originals = {path: data.decode("utf-8").replace("\r\n", "\n") for path, data in raw.items()}
    patched = probes(originals)
    for path in raw:
        for pattern in (r"\.await", r"\bassert(?:_eq|_ne)?!", r"\.expect\([^\n]*", r"Duration::from_(?:secs|millis)\([^)]*\)", r"\bunsafe\b"):
            if re.findall(pattern, originals[path]) != re.findall(pattern, patched[path]):
                raise ValueError("original assertions/awaits/timeouts/unsafe changed: " + path)
    if args.self_test:
        harness.restoration_self_test()
        assert patched[TESTS].count("crate::store::windows_config_enable();") == 3
        assert 'recv_timeout(Duration::from_secs(5))' in patched[TESTS]
        assert 'recv_timeout(Duration::from_millis(100))' in patched[TESTS]
        assert classify(0, "test result: ok. 225 passed; 0 failed; 0 ignored;", 225) == 0
        assert classify(101, "test result: FAILED. 224 passed; 1 failed; 0 ignored;", 225) == 101
        assert classify(0, "test result: ok. 0 passed; 0 failed; 0 ignored;", 1) == 125
        assert classify(0, "", 1) == 125
        print("Config probe anchors, invariants, result rejection and byte restoration: PASS")
        return 0
    if sys.platform != "win32" or not args.phase:
        parser.error("requires Windows and --phase")
    harness.END_BY = time.monotonic() + 20 * 60
    output = ROOT / "windows-config-diagnostics"
    output.mkdir(exist_ok=True)
    (output / "manifest.json").write_text(json.dumps({
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "phase": args.phase, "expected_test_processes": 6, "expected_tests": 230,
        "cpu_count": os.cpu_count(),
        "runner": {key: os.environ.get(key) for key in ("RUNNER_OS", "RUNNER_ARCH", "ImageOS", "ImageVersion")},
        "source_sha256": {path: hashlib.sha256(text.encode()).hexdigest() for path, text in originals.items()},
    }, indent=2), encoding="utf-8")
    (output / "probe-overlay.patch").write_text("".join("".join(difflib.unified_diff(originals[p].splitlines(True), patched[p].splitlines(True), fromfile=p, tofile=p)) for p in raw), encoding="utf-8")
    results = []
    try:
        if args.phase == "instrumented":
            for path, text in patched.items():
                (ROOT / path).write_text(text, encoding="utf-8", newline="\n")
        code, text = harness.execute(["cargo", "test", "-p", "talos-config", "--lib", "--locked", "--no-run", "--message-format=json"], output / "build.log", 900)
        if code:
            raise RuntimeError("config build failed; see retained build.log")
        binaries = set()
        for line in text.splitlines():
            try:
                record = json.loads(line)
            except json.JSONDecodeError:
                continue
            if record.get("reason") == "compiler-artifact" and record.get("profile", {}).get("test") and record.get("target", {}).get("name") == "talos_config" and record.get("executable"):
                binaries.add(record["executable"])
        if len(binaries) != 1:
            raise RuntimeError("config test executable identity ambiguous")
        binary = binaries.pop()
        # First original-test execution is the full default-concurrency config suite.
        for index in range(6):
            label = "suite-first" if index == 0 else f"single-{index}"
            command = [binary, "--nocapture"] if index == 0 else [binary, CASE, "--exact", "--nocapture"]
            code, text = harness.execute(command, output / (label + ".log"), 90)
            code = classify(code, text, 225 if index == 0 else 1)
            results.append({"case": label, "exit_code": code})
            (output / "results.json").write_text(json.dumps(results, indent=2), encoding="utf-8")
            if code == 124:
                raise RuntimeError("owned config test process watchdog expired; stop sampling")
    finally:
        harness.restore_sources(ROOT, raw, output)
    return int(any(result["exit_code"] != 0 for result in results))


if __name__ == "__main__":
    raise SystemExit(main())
