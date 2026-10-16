#!/usr/bin/env python3
"""Bounded Windows CI experiment; restore every temporary Rust probe afterward.

No production patch, timeout change, retry-to-green, credentials, or command-content logging.
Run --self-test on any host to validate anchors and byte-restoration fixtures without
modifying checkout sources.
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
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
BOUNDARY = "crates/talos-tools/src/process_boundary.rs"
AUTO = "crates/talos-desktop/src/runtime_host/auto_tests.rs"
HOST = "crates/talos-desktop/src/runtime_host.rs"
PLAN = "crates/talos-agent/src/request_plan.rs"
SESSION = "crates/talos-agent/src/session.rs"
STORE = "crates/talos-session/src/pending_submission.rs"
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


def restore_sources(root: Path, original_bytes: dict[str, bytes], output: Path) -> None:
    for path, data in original_bytes.items():
        (root / path).write_bytes(data)
    restoration = {}
    for path, data in original_bytes.items():
        actual = (root / path).read_bytes()
        restoration[path] = {
            "original_raw_sha256": hashlib.sha256(data).hexdigest(),
            "restored_raw_sha256": hashlib.sha256(actual).hexdigest(),
            "matches": actual == data,
        }
    (output / "restoration.json").write_text(json.dumps(restoration, indent=2), encoding="utf-8")
    if not all(entry["matches"] for entry in restoration.values()):
        raise RuntimeError("temporary source restoration readback mismatch; see restoration.json")


def restoration_self_test() -> None:
    from unittest.mock import patch

    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        original = {"first.rs": b"fixture\r\n", "second.rs": b"\xff\x00\n"}
        restore_sources(root, original, root)
        assert all((root / path).read_bytes() == data for path, data in original.items())
        read_bytes = Path.read_bytes
        # Simulate a bad readback after writes: every file was restored, but the
        # evidence must reject the mismatch rather than claim successful cleanup.
        with patch.object(Path, "read_bytes", lambda path: b"mismatch" if path.name == "first.rs" else read_bytes(path)):
            try:
                restore_sources(root, original, root)
            except RuntimeError:
                pass
            else:
                raise AssertionError("restoration mismatch must fail closed")
        receipt = json.loads((root / "restoration.json").read_text(encoding="utf-8"))
        assert not receipt["first.rs"]["matches"] and receipt["second.rs"]["matches"]
        assert all((root / path).read_bytes() == data for path, data in original.items())


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
                    // Exit FILETIME is undefined until exit; only use a successful read after a signaled wait.
                    if observed == windows_sys::Win32::Foundation::WAIT_OBJECT_0 && times_read_ok != 0 {
                        eprintln!("WINDOWS_AUTO_DIAG lifetime_read_ok=1 created_100ns={} exited_100ns={} identity={} pid={} thread={:?}", ticks(created), ticks(exited), process as usize, std::process::id(), std::thread::current().id());
                    }
                    let mut io = windows_sys::Win32::System::Threading::IO_COUNTERS {
                        ReadOperationCount: 0, WriteOperationCount: 0, OtherOperationCount: 0,
                        ReadTransferCount: 0, WriteTransferCount: 0, OtherTransferCount: 0,
                    };
                    let io_read_ok = windows_sys::Win32::System::Threading::GetProcessIoCounters(process, &mut io);
                    let io_error = if io_read_ok == 0 { windows_sys::Win32::Foundation::GetLastError() } else { 0 };
                    eprintln!("WINDOWS_AUTO_DIAG time={:?} pid={} thread={:?} io_read_ok={} io_error={} read_ops={} write_ops={} other_ops={} read_bytes={} write_bytes={} other_bytes={} identity={}", std::time::SystemTime::now(), std::process::id(), std::thread::current().id(), io_read_ok, io_error, io.ReadOperationCount, io.WriteOperationCount, io.OtherOperationCount, io.ReadTransferCount, io.WriteTransferCount, io.OtherTransferCount, process as usize);
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
    auto = replace_once(auto, "        self.conversation.stream(messages).await", '''        windows_auto_host_stage("conversation_stream_enter");
        let stream_result = self.conversation.stream(messages).await;
        windows_auto_host_stage("conversation_stream_return");
        stream_result''')
    auto = replace_once(auto, "        self.assessments.fetch_add(1, Ordering::SeqCst);", '        windows_auto_host_stage("auto_review_stream_enter");\n        self.assessments.fetch_add(1, Ordering::SeqCst);')
    auto = replace_once(auto, '''        MockProvider::new()
            .with_response(response.to_string())
            .stream(&[])
            .await''', '''        let review_stream_result = MockProvider::new()
            .with_response(response.to_string())
            .stream(&[])
            .await;
        windows_auto_host_stage("auto_review_stream_return");
        review_stream_result''')
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
    host = replace_once(host, "    impl Drop for EvaluationDropReceipt {\n        fn drop(&mut self) {\n            self.0.notify_one();", '    impl Drop for EvaluationDropReceipt {\n        fn drop(&mut self) {\n            windows_auto_host_stage("eval_assessor_drop");\n            self.0.notify_one();')
    host = replace_once(host, "            let _receipt = EvaluationDropReceipt(self.dropped.clone());\n            self.started.notify_one();", '            let _receipt = EvaluationDropReceipt(self.dropped.clone());\n            windows_auto_host_stage("eval_assessor_enter");\n            self.started.notify_one();')
    begin = host.index("    async fn paused_evaluation_does_not_block_interrupt_or_shutdown()")
    end = host.index("    #[async_trait::async_trait]\n    impl talos_runtime::EvaluatorAssessor for EvaluationFixtureAssessor", begin)
    evaluation = host[begin:end]
    for old, new in [
        ("        for interrupt in [true, false] {", '        for interrupt in [true, false] {\n            windows_auto_host_stage(if interrupt { "eval_interrupt_case_begin" } else { "eval_shutdown_case_begin" });'),
        ('            .expect("host");', '            .expect("host");\n            windows_auto_host_stage("eval_host_ready");'),
        ('                    .expect("evaluate");', '                    .expect("evaluate");\n                windows_auto_host_stage("eval_evaluate_sent");'),
        ('                started.notified().await;', '                windows_auto_host_stage("eval_wait_started");\n                started.notified().await;\n                windows_auto_host_stage("eval_started_observed");'),
        ('                .expect("stop command");', '                .expect("stop command");\n                windows_auto_host_stage(if interrupt { "eval_interrupt_enqueued" } else { "eval_shutdown_enqueued" });'),
        ('                dropped.notified().await;', '                windows_auto_host_stage("eval_wait_drop");\n                dropped.notified().await;\n                windows_auto_host_stage("eval_drop_observed");'),
        ('                                assert_eq!(reason, "evaluation cancelled");', '                                assert_eq!(reason, "evaluation cancelled");\n                                windows_auto_host_stage("eval_cancel_receipt");'),
        ('                    host.try_send(RuntimeCommand::Shutdown).expect("shutdown");', '                    host.try_send(RuntimeCommand::Shutdown).expect("shutdown");\n                    windows_auto_host_stage("eval_shutdown_after_cancel_enqueued");'),
        ('                        RuntimeOutput::Stopped => break,', '                        RuntimeOutput::Stopped => { windows_auto_host_stage("eval_stopped_receipt"); break; },'),
        ('            .await\n            .expect("host must respond before the 30-second evaluator deadline");', '            .await\n            .inspect_err(|_| windows_auto_host_stage(if interrupt { "eval_interrupt_composite_timeout" } else { "eval_shutdown_composite_timeout" }))\n            .expect("host must respond before the 30-second evaluator deadline");'),
    ]:
        evaluation = replace_once(evaluation, old, new)
    host = host[:begin] + evaluation + host[end:]
    for old, new in [
        ("                            let approval_root = workspace_root.clone();", '                            windows_auto_host_stage("provider_return");\n                            windows_auto_host_stage("session_setup_enter");\n                            let approval_root = workspace_root.clone();'),
        ("                            let mut builder = RuntimeBuilder::new()", '                            windows_auto_host_stage("session_setup_return");\n                            windows_auto_host_stage("builder_setup_enter");\n                            let mut builder = RuntimeBuilder::new()'),
        ("                            match builder.build() {", '                            windows_auto_host_stage("builder_setup_return");\n                            windows_auto_host_stage("builder_build_enter");\n                            let build_result = builder.build();\n                            windows_auto_host_stage("builder_build_return");\n                            match build_result {'),
        ("    let initial_work = load_work_projection(work_source.clone()).await;", '    windows_auto_host_stage("initial_projection_enter");\n    let initial_work = load_work_projection(work_source.clone()).await;\n    windows_auto_host_stage("initial_projection_return");'),
        ("    loop {\n        tokio::select! {\n            result = async {", '    windows_auto_host_stage("command_loop_ready");\n    loop {\n        tokio::select! {\n            result = async {'),
        ("                    Some(RuntimeCommand::Submit(message)) => {", '                    Some(RuntimeCommand::Submit(message)) => {\n                        windows_auto_host_stage("submit_received");'),
        ("                        if let Err(error) = handle.submit(message).await\n", '                        windows_auto_host_stage("submit_enter");\n                        let submit_result = handle.submit(message).await;\n                        windows_auto_host_stage("submit_return");\n                        if let Err(error) = submit_result\n'),
        ("                                    report_output.publish(RuntimeOutput::AutoDecision {", '                                    windows_auto_host_stage("auto_report_publish_enter");\n                                    report_output.publish(RuntimeOutput::AutoDecision {'),
        ("                                        evaluator: report.evaluator,\n                                    });", '                                        evaluator: report.evaluator,\n                                    });\n                                    windows_auto_host_stage("auto_report_publish_return");'),
    ]:
        host = replace_once(host, old, new)
    plan_helper = '''fn windows_auto_turn_stage(stage: &str) {
    eprintln!("WINDOWS_AUTO_DIAG time={:?} pid={} thread={:?} stage={}",
        std::time::SystemTime::now(), std::process::id(), std::thread::current().id(), stage);
}

'''
    plan = replace_once(originals[PLAN], "impl Agent {", plan_helper + "impl Agent {")
    for old, new in [
        ("        let (memory_query, input_messages) = Self::structured_session_inputs(items);", '        windows_auto_turn_stage("prepare_session_enter");\n        let (memory_query, input_messages) = Self::structured_session_inputs(items);\n        windows_auto_turn_stage("structured_inputs_return");'),
        ("        self.prepare_turn_start(\n            memory_query,", "        let prepared_result = self.prepare_turn_start(\n            memory_query,"),
        ("            Some(request_context_limit),\n        )\n        .await", '            Some(request_context_limit),\n        )\n        .await;\n        windows_auto_turn_stage("prepare_session_return");\n        prepared_result'),
        ("        let turn_id = TurnId::new();", '        windows_auto_turn_stage("prepare_turn_enter");\n        let turn_id = TurnId::new();'),
        ("        let active_tool_presentation_policy = self.tool_presentation_policy.clone();", '        windows_auto_turn_stage("tool_description_enter");\n        let active_tool_presentation_policy = self.tool_presentation_policy.clone();'),
        ("        let selected_protocol = self\n", '        windows_auto_turn_stage("tool_description_return");\n        windows_auto_turn_stage("protocol_select_enter");\n        let selected_protocol = self\n'),
        ("        let (mut messages, persist_start) = match self\n", '        windows_auto_turn_stage("protocol_select_return");\n        windows_auto_turn_stage("prompt_build_enter");\n        let (mut messages, persist_start) = match self\n'),
        ("        messages.pop();", '        windows_auto_turn_stage("prompt_build_return");\n        messages.pop();'),
        ("        if let Err(error) = self\n            .run_hook(&hook_ctx, HookEvent::TurnStart { turn_id })", '        windows_auto_turn_stage("turn_hook_enter");\n        if let Err(error) = self\n            .run_hook(&hook_ctx, HookEvent::TurnStart { turn_id })'),
        ("        let mut continuation_parts = Vec::new();", '        windows_auto_turn_stage("turn_hook_return");\n        let mut continuation_parts = Vec::new();\n        windows_auto_turn_stage("initial_seal_enter");'),
        ("        Ok(PreparedSessionTurn {", '        windows_auto_turn_stage("initial_seal_return");\n        Ok(PreparedSessionTurn {'),
    ]:
        plan = replace_once(plan, old, new)
    actor_helper = '''fn windows_auto_actor_stage(stage: &str) {
    eprintln!("WINDOWS_AUTO_DIAG time={:?} pid={} thread={:?} stage={}",
        std::time::SystemTime::now(), std::process::id(), std::thread::current().id(), stage);
}

'''
    session = replace_once(originals[SESSION], "fn immediate_started_turn(", actor_helper + "fn immediate_started_turn(")
    for old, new in [
        ("    pub async fn run(&mut self) {\n        self.reconcile_running_submissions();", '    pub async fn run(&mut self) {\n        windows_auto_actor_stage("actor_run_enter");\n        let reconcile_started = std::time::Instant::now();\n        self.reconcile_running_submissions();\n        let reconcile_elapsed = reconcile_started.elapsed();\n        windows_auto_actor_stage("actor_reconcile_return");\n        eprintln!("WINDOWS_AUTO_DIAG actor_reconcile_elapsed_ns={} pid={} thread={:?}", reconcile_elapsed.as_nanos(), std::process::id(), std::thread::current().id());'),
        ("        if recovery_fenced {", '        windows_auto_actor_stage("actor_recover_return");\n        if recovery_fenced {'),
        ("        let mut paused = self.restore_pending_submissions(", '        windows_auto_actor_stage("actor_restore_enter");\n        let mut paused = self.restore_pending_submissions('),
        ("        let mut shutting_down = false;", '        windows_auto_actor_stage("actor_restore_return");\n        let mut shutting_down = false;'),
        ("                let (images, image_bytes) = submission.image_totals();", '                windows_auto_actor_stage("actor_pending_popped");\n                let (images, image_bytes) = submission.image_totals();'),
        ("                let start_committed = self\n", '                windows_auto_actor_stage("actor_commit_start_enter");\n                let start_committed = self\n'),
        ("                if !start_committed {", '                windows_auto_actor_stage("actor_commit_start_return");\n                if !start_committed {'),
        ("                        SessionOp::Submit { message } => {", '                        SessionOp::Submit { message } => {\n                            windows_auto_actor_stage("actor_submit_received");'),
        ('''                                SubmissionKind::UserTurn,
                                message,
                                Vec::new(),
                            );
                            if self.accept_submission(''', '''                                SubmissionKind::UserTurn,
                                message,
                                Vec::new(),
                            );
                            windows_auto_actor_stage("actor_submit_normalized");
                            if self.accept_submission('''),
        ('''                                paused = false;
                            }
                        }
                        SessionOp::SubmitMultimodal''', '''                                paused = false;
                            }
                            windows_auto_actor_stage("actor_submit_processed");
                        }
                        SessionOp::SubmitMultimodal'''),
        ("        let runtime_managed = self.runtime_admission.is_some();", '        windows_auto_actor_stage("actor_start_submission_enter");\n        let runtime_managed = self.runtime_admission.is_some();'),
        ("        if runtime_managed && self.compactor.should_compact(&self.history) {", '        windows_auto_actor_stage("actor_compaction_check_enter");\n        if runtime_managed && self.compactor.should_compact(&self.history) {'),
        ("        let prepared_turn = if !runtime_managed {", '        windows_auto_actor_stage("actor_pre_prepare");\n        let prepared_turn = if !runtime_managed {'),
    ]:
        session = replace_once(session, old, new)
    pause_sites = ("custody_failure", "missing_completion", "sq_disconnect", "interrupt", "shutdown")
    pause_parts = session.split("self.pending_store.pause_unstarted()")
    if len(pause_parts) != len(pause_sites) + 1:
        raise ValueError("expected exactly five original Actor pause call sites")
    session = pause_parts[0]
    for site, suffix in zip(pause_sites, pause_parts[1:]):
        session += ('{ windows_auto_actor_stage("pause_' + site + '_begin"); '
                    'let pause_result = self.pending_store.pause_unstarted(); '
                    'windows_auto_actor_stage("pause_' + site + '_return"); pause_result }' + suffix)
    session = replace_once(session, "                        SessionOp::Shutdown => {", '                        SessionOp::Shutdown => {\n                            windows_auto_actor_stage("actor_shutdown_received");')
    session = replace_once(session, "        if self.runtime_admission.is_none()\n            && let Err(error) = self.background_jobs.finalize().await", '        windows_auto_actor_stage("actor_loop_exit");\n        if self.runtime_admission.is_none()\n            && let Err(error) = self.background_jobs.finalize().await')
    session = replace_once(session, '                message: format!("background cleanup incomplete: {error}"),\n            });\n        }\n    }', '                message: format!("background cleanup incomplete: {error}"),\n            });\n        }\n        windows_auto_actor_stage("actor_finalize_return");\n    }')
    host = replace_once(host, "                    Some(RuntimeCommand::Shutdown) | None => {", '                    Some(RuntimeCommand::Shutdown) | None => {\n                        windows_auto_host_stage("host_shutdown_received");')
    host = replace_once(host, "    match handle.shutdown().await {", '    windows_auto_host_stage("runtime_shutdown_begin");\n    let shutdown_result = handle.shutdown().await;\n    windows_auto_host_stage("runtime_shutdown_return");\n    match shutdown_result {')
    host = replace_once(host, "                        finish_host_executor(executor, terminal, &capture_slots)", '                        windows_auto_host_stage("host_executor_finalize_begin");\n                        let final_terminal = finish_host_executor(executor, terminal, &capture_slots);\n                        windows_auto_host_stage("host_executor_finalize_return");\n                        final_terminal')
    host = replace_once(host, "                let _ = terminal_tx.send(terminal.clone());", '                windows_auto_host_stage("terminal_publish_begin");\n                let _ = terminal_tx.send(terminal.clone());\n                windows_auto_host_stage("terminal_publish_return");')
    store_helper = '''fn windows_auto_store_timing(stage: &str, elapsed: std::time::Duration) {
    eprintln!("WINDOWS_AUTO_DIAG time={:?} pid={} thread={:?} stage={} elapsed_ns={}",
        std::time::SystemTime::now(), std::process::id(), std::thread::current().id(), stage, elapsed.as_nanos());
}

'''
    store = originals[STORE]
    begin = store.index("    fn recover_states(")
    end = store.index("    /// Returns one durable submission record", begin)
    recover = store[begin:end]
    body_begin = recover.index("        let _guard = self.guard()?;")
    body_end = recover.rindex("\n    }")
    body = recover[body_begin:body_end]
    for stage, operation in [
        ("pending_guard", "        let _guard = self.guard()?;"),
        ("pending_connection", "        let connection = self.connection()?;"),
        ("pending_schema", "        ensure_schema(&connection)?;"),
        ("pending_statement", "        let mut statement = connection.prepare(&sql)?;"),
        ("pending_query_map", "        let rows = statement.query_map([], read_record_tuple)?;"),
    ]:
        body = replace_once(body, operation, '        let operation_started = std::time::Instant::now();\n' + operation + f'\n        windows_auto_store_timing("{stage}", operation_started.elapsed());')
    body = replace_once(body, "        rows.map(|row| tuple_to_record(row?)).collect()", '''        let operation_started = std::time::Instant::now();
        let records: Result<Vec<PendingSubmissionRecord>, PendingSubmissionError> = rows.map(|row| tuple_to_record(row?)).collect();
        windows_auto_store_timing("pending_rows_collect", operation_started.elapsed());
        records''')
    # Original locals drop inside the closure in their original reverse declaration order.
    # Total time includes those drops; individual timers capture before their logging.
    wrapped = recover[:body_begin] + '''        let recovery_started = std::time::Instant::now();
        let recovery_result: Result<Vec<PendingSubmissionRecord>, PendingSubmissionError> = (|| {
''' + "\n".join("    " + line for line in body.splitlines()) + '''
        })();
        windows_auto_store_timing("pending_recover_total", recovery_started.elapsed());
        recovery_result
    }

'''
    store = replace_once(store, recover, wrapped)
    begin = store.index("    pub fn pause_unstarted(")
    end = store.index("    /// Explicitly terminalizes", begin)
    pause = store[begin:end]
    body_begin = pause.index("        let _guard = self.guard()?;")
    body_end = pause.rindex("\n    }")
    body = pause[body_begin:body_end]
    body = replace_once(body, "        if !self.sidecar_exists()? {", '''        let inspection_started = std::time::Instant::now();
        let sidecar_exists = self.sidecar_exists()?;
        windows_auto_store_timing(if sidecar_exists { "pause_inspection_existing" } else { "pause_inspection_absent" }, inspection_started.elapsed());
        if !sidecar_exists {''')
    for stage, operation in [
        ("pause_guard", "        let _guard = self.guard()?;"),
        ("pause_connection", "        let mut connection = self.connection()?;"),
        ("pause_immediate", "        let transaction = immediate(&mut connection)?;"),
        ("pause_schema", "        ensure_schema(&transaction)?;"),
        ("pause_update", '''        let changed = transaction.execute(
            "UPDATE pending_submissions SET state = 'paused_pending'
             WHERE state = 'accepted_pending'",
            [],
        )?;'''),
        ("pause_commit", "        transaction.commit()?;"),
    ]:
        body = replace_once(body, operation, '        let operation_started = std::time::Instant::now();\n' + operation + f'\n        windows_auto_store_timing("{stage}", operation_started.elapsed());')
    wrapped_pause = pause[:body_begin] + '''        let pause_started = std::time::Instant::now();
        let pause_result: Result<usize, PendingSubmissionError> = (|| {
''' + body + '''
        })();
        windows_auto_store_timing("pending_pause_total", pause_started.elapsed());
        pause_result
    }

'''
    store = replace_once(store, pause, wrapped_pause)
    connection = '''    fn connection(&self) -> Result<Connection, PendingSubmissionError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(self.path.as_ref())?;
        connection.busy_timeout(SQLITE_BUSY_TIMEOUT)?;
        retry_sqlite_busy(SQLITE_BUSY_TIMEOUT, || {
            connection.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")
        })?;
        Ok(connection)
    }'''
    timed_connection = connection
    for stage, operation in [
        ("pending_directory", "        if let Some(parent) = self.path.parent() {\n            std::fs::create_dir_all(parent)?;\n        }"),
        ("pending_open", "        let connection = Connection::open(self.path.as_ref())?;"),
        ("pending_busy_config", "        connection.busy_timeout(SQLITE_BUSY_TIMEOUT)?;"),
        ("pending_pragma", '        retry_sqlite_busy(SQLITE_BUSY_TIMEOUT, || {\n            connection.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")\n        })?;'),
    ]:
        timed_connection = replace_once(timed_connection, operation, '        let operation_started = std::time::Instant::now();\n' + operation + f'\n        windows_auto_store_timing("{stage}", operation_started.elapsed());')
    store = replace_once(store, connection, timed_connection)
    store = replace_once(store, "use std::fs::OpenOptions;", store_helper + "use std::fs::OpenOptions;")
    return {BOUNDARY: boundary, AUTO: auto, HOST: host, PLAN: plan, SESSION: session, STORE: store}


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
    original_bytes = {path: (ROOT / path).read_bytes() for path in (BOUNDARY, AUTO, HOST, PLAN, SESSION, STORE)}
    originals = {path: data.decode("utf-8").replace("\r\n", "\n") for path, data in original_bytes.items()}
    patched = probes(originals)
    if args.self_test:
        restoration_self_test()
        assert all(patched[path] != originals[path] for path in originals)
        assert "Duration::from_secs(15)" in patched[AUTO]
        assert "Duration::from_secs(10)" in patched[AUTO]
        assert patched[BOUNDARY].count("windows_auto_probe(") >= 10
        assert patched[HOST].count("stage=host_heartbeat") == 1
        assert "WaitForSingleObject(process, 1_000)" in patched[BOUNDARY]
        assert "observed != windows_sys::Win32::Foundation::WAIT_TIMEOUT" in patched[BOUNDARY]
        assert "GetProcessTimes(process," in patched[BOUNDARY]
        assert patched[BOUNDARY].count("GetProcessTimes(process,") == 1
        assert patched[BOUNDARY].count("if observed == windows_sys::Win32::Foundation::WAIT_OBJECT_0 && times_read_ok != 0 {") == 1
        assert patched[BOUNDARY].count("lifetime_read_ok=1 created_100ns={}") == 1
        assert patched[BOUNDARY].count("GetProcessIoCounters(process, &mut io)") == 1
        assert "let io_error = if io_read_ok == 0 { windows_sys::Win32::Foundation::GetLastError() } else { 0 };" in patched[BOUNDARY]
        assert all(field + ": 0" in patched[BOUNDARY] for field in (
            "ReadOperationCount", "WriteOperationCount", "OtherOperationCount",
            "ReadTransferCount", "WriteTransferCount", "OtherTransferCount"))
        assert patched[PLAN].count('windows_auto_turn_stage("prepare_session_enter")') == 1
        assert patched[PLAN].count('windows_auto_turn_stage("initial_seal_return")') == 1
        assert patched[PLAN].count(".await") == originals[PLAN].count(".await")
        assert patched[SESSION].count('windows_auto_actor_stage("actor_submit_received")') == 1
        assert patched[SESSION].count('windows_auto_actor_stage("actor_pre_prepare")') == 1
        assert patched[SESSION].count(".await") == originals[SESSION].count(".await")
        assert patched[STORE].count(".await") == originals[STORE].count(".await")
        assert patched[STORE].count('windows_auto_store_timing("pending_recover_total"') == 1
        assert patched[STORE].count('windows_auto_store_timing("pending_pause_total"') == 1
        assert patched[STORE].count('"pause_inspection_absent"') == 1
        assert patched[STORE].count("self.sidecar_exists()?") == originals[STORE].count("self.sidecar_exists()?")
        assert patched[HOST].count("Duration::from_secs(5)") == originals[HOST].count("Duration::from_secs(5)")
        assert patched[HOST].count('"eval_assessor_enter"') == 1
        assert patched[HOST].count('"eval_assessor_drop"') == 1
        assert patched[HOST].count('"eval_interrupt_composite_timeout"') == 1
        assert patched[HOST].count('"eval_shutdown_composite_timeout"') == 1
        assert patched[SESSION].count("self.pending_store.pause_unstarted()") == 5
        assert all(patched[SESSION].count('windows_auto_actor_stage("pause_' + site + '_begin")') == 1 for site in ("custody_failure", "missing_completion", "sq_disconnect", "interrupt", "shutdown"))
        # The pre-existing host heartbeat adds one sleep; shutdown adds no await.
        assert patched[HOST].count(".await") == originals[HOST].count(".await") + 1
        assert all(patched[path].count("unsafe") == originals[path].count("unsafe") for path in originals)
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
        restore_sources(ROOT, original_bytes, output)
        print("WINDOWS_AUTO_DIAG temporary Rust probes restored; raw readback verified", flush=True)
    return int(any(result["exit_code"] != 0 for result in results))


if __name__ == "__main__":
    raise SystemExit(main())
