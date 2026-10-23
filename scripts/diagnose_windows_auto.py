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
import re
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import time
import datetime
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
BOUNDARY = "crates/talos-tools/src/process_boundary.rs"
AUTO = "crates/talos-desktop/src/runtime_host/auto_tests.rs"
HOST = "crates/talos-desktop/src/runtime_host.rs"
PLAN = "crates/talos-agent/src/request_plan.rs"
SESSION = "crates/talos-agent/src/session.rs"
STORE = "crates/talos-session/src/pending_submission.rs"
END_BY = float("inf")
LATER_CASES = tuple(f"instrumented-single-{i}" for i in range(1, 51)) + tuple(
    f"instrumented-suite-parallel-{i}" for i in range(1, 10))
LATER_SCAN_BYTES = 16_777_216
CONTROLLED_TEST = "process_boundary::windows_tests::windows_auto_diagnostic_descendant_pipe_hold_is_observable_and_cleanup_bounded"


def filetime_text(ticks: int) -> str:
    seconds, fraction = divmod(ticks, 10_000_000)
    value = datetime.datetime(1601, 1, 1, tzinfo=datetime.timezone.utc) + datetime.timedelta(seconds=seconds)
    if not 2000 <= value.year <= 2100:
        raise ValueError("unsupported event time")
    return value.strftime("%Y-%m-%dT%H:%M:%S") + f".{fraction:07d}Z"


def first_child_window(text: str) -> tuple[int, int, int]:
    text = text.replace("\r\n", "\n")
    lifetime = re.findall(r"^WINDOWS_AUTO_DIAG lifetime_read_ok=1 created_100ns=(\d+) exited_100ns=(\d+) identity=(\d+) pid=(\d+) thread=ThreadId\(\d+\)$", text, re.M)
    if len(lifetime) != 1:
        raise ValueError("ambiguous lifetime")
    created, exited, handle, parent = map(int, lifetime[0])
    if not 0 < parent <= 0xFFFFFFFF or not 0 < handle <= 0xFFFFFFFFFFFFFFFF:
        raise ValueError("invalid owner identity")
    # Require one complete self-contained owned-identity receipt. Never repair old fragments.
    children = re.findall(r"^WINDOWS_AUTO_DIAG child_created time=SystemTime \{ intervals: (\d+) \} child_pid=(\d+) identity=(\d+) pid=(\d+) thread=ThreadId\(\d+\)$", text, re.M)
    if (len(children) != 1 or not 0 < int(children[0][1]) <= 0xFFFFFFFF
            or tuple(map(int, children[0][2:])) != (handle, parent)
            or not created <= int(children[0][0]) <= exited):
        raise ValueError("ambiguous child")
    if not 0 < exited - created <= 90 * 10_000_000:
        raise ValueError("unsupported lifetime")
    filetime_text(created)
    filetime_text(exited)
    return int(children[0][1]), created, exited


def lifecycle_query(child: int, created: int, exited: int, *, console: bool = False) -> str:
    provider = "Microsoft-Windows-PowerShell" if console else "PowerShell"
    ids = "EventID=40961 or EventID=40962" if console else "EventID=400 or EventID=403 or EventID=600"
    return (f"*[System[Provider[@Name='{provider}'] and ({ids}) "
            f"and Execution[@ProcessID={child}] and TimeCreated[@SystemTime>='{filetime_text(created)}' "
            f"and @SystemTime<='{filetime_text(exited)}']]]")


def lifecycle_metadata(raw: bytes, child: int, created: int, exited: int, *, console: bool = False) -> list[dict]:
    # Query output is transient only; never persist XML, EventData, rendered messages or errors.
    if len(raw) > 131_072:
        raise ValueError("event byte cap")
    text = raw.decode("utf-16") if raw.startswith((b"\xff\xfe", b"\xfe\xff")) else raw.decode("utf-16-le")
    if "<!DOCTYPE" in text.upper() or "<!ENTITY" in text.upper():
        raise ValueError("unsupported XML declaration")
    root = ET.fromstring(text)
    ns = {"e": "http://schemas.microsoft.com/win/2004/08/events/event"}
    events = list(root)
    if root.tag != "Events" or len(events) > 16:
        raise ValueError("event count or root")
    result = []
    for event in events:
        system = event.find("e:System", ns)
        if system is None or event.tag != "{" + ns["e"] + "}Event":
            raise ValueError("event schema")
        event_id = int(system.findtext("e:EventID", namespaces=ns))
        pid = int(system.find("e:Execution", ns).attrib["ProcessID"])
        stamp = system.find("e:TimeCreated", ns).attrib["SystemTime"]
        # Canonical UTC 100ns text only, so lexical bounds are exact.
        if not re.fullmatch(r"\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{7}Z", stamp):
            raise ValueError("event timestamp")
        if (event_id not in ((40961, 40962) if console else (400, 403, 600)) or pid != child
                or system.find("e:Provider", ns).attrib.get("Name") != ("Microsoft-Windows-PowerShell" if console else "PowerShell")
                or system.findtext("e:Channel", namespaces=ns) != ("Microsoft-Windows-PowerShell/Operational" if console else "Windows PowerShell")
                or not filetime_text(created) <= stamp <= filetime_text(exited)):
            raise ValueError("event identity or lifetime")
        result.append({"event_id": event_id, "child_pid": pid, "time_utc": stamp})
    return result


def collect_first_child_events(output: Path, *, console: bool = False,
                               later: tuple[str, tuple[int, int, int]] | None = None) -> None:
    receipt = {"status": "unavailable", "events": []}
    cleanup_failed = False
    try:
        # All original attempts AND source restoration precede this collector.
        # Do not start a host utility if the existing experiment margin is exhausted.
        if END_BY - time.monotonic() < 45:
            raise ValueError("collection budget")
        case = "instrumented-suite-first-1" if later is None else later[0]
        if later is not None and case not in LATER_CASES:
            raise ValueError("unsupported source case")
        with (output / f"{case}.log").open("rb") as source:
            data = source.read(8_388_609)
        if len(data) > 8_388_608:
            raise ValueError("source byte cap")
        child, created, exited = first_child_window(data.decode("utf-8"))
        if later is not None:
            if (child, created, exited) != later[1]:
                raise ValueError("selected window changed")
            receipt["source_case"] = case
        receipt.update(child_pid=child, created_100ns=created, exited_100ns=exited)
        command = [str(Path(os.environ["SystemRoot"]) / "System32" / "wevtutil.exe"),
                   "qe", "Microsoft-Windows-PowerShell/Operational" if console else "Windows PowerShell",
                   "/q:" + lifecycle_query(child, created, exited, console=console),
                   "/c:17", "/f:xml", "/e:Events", "/uni:true"]
        # No Popen context: its __exit__ would add an unbounded wait/pipe close.
        utility = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        captured = []
        def read_bounded() -> None:
            try:
                with utility.stdout as stream:
                    captured.append(stream.read(131_073))
            except OSError:
                pass
        reader = threading.Thread(target=read_bounded, daemon=True)
        try:
            reader.start()
        except RuntimeError:
            try:
                utility.kill()
                utility.wait(timeout=5)
            except (OSError, subprocess.TimeoutExpired):
                cleanup_failed = True
            utility.stdout.close()  # No reader started, so no competing read lock.
            raise ValueError("reader unavailable") from None
        try:
            code = utility.wait(timeout=30)
        except (OSError, subprocess.TimeoutExpired):
            code = -1
            try:
                utility.kill()  # Exact owned collector only; never taskkill or other PIDs.
                utility.wait(timeout=5)
            except (OSError, subprocess.TimeoutExpired):
                cleanup_failed = True
        # The reader owns its pipe. Never close it synchronously from this thread.
        reader.join(timeout=5)
        if code != 0 or reader.is_alive() or len(captured) != 1:
            raise ValueError("query unavailable")
        receipt["events"] = lifecycle_metadata(captured[0], child, created, exited, console=console)
        receipt["status"] = "records" if receipt["events"] else "no_matching_records"
    except (OSError, ValueError, OverflowError, KeyError, TypeError, AttributeError, ET.ParseError):
        # Missing logs, unsupported schema and identities are unknown, never zero-duration evidence.
        pass
    prefix = "first-child" if later is None else "later-child"
    channel = "console" if console else "engine"
    (output / f"{prefix}-{channel}-events.json").write_text(json.dumps(receipt, indent=2), encoding="utf-8")
    if cleanup_failed:
        raise RuntimeError("owned event collector cleanup unconfirmed")


def select_later_child(output: Path) -> tuple[str, tuple[int, int, int]] | None:
    # Fixed matrix paths only. Rank intact observed lifetimes, never infer missing exits.
    receipt = {"status": "unavailable", "expected_cases": len(LATER_CASES),
               "valid_cases": 0, "invalid_cases": 0, "reason": "no_valid_window"}
    selected = None
    remaining_bytes = LATER_SCAN_BYTES
    try:
        for case in LATER_CASES:
            if END_BY - time.monotonic() < 45:
                raise ValueError("collection budget")
            if remaining_bytes <= 0:
                raise ValueError("scan byte cap")
            try:
                limit = min(8_388_608, remaining_bytes)
                with (output / f"{case}.log").open("rb") as source:
                    data = source.read(limit + 1)
            except OSError:
                receipt["invalid_cases"] += 1
                continue
            if len(data) > limit:
                raise OverflowError("scan byte cap")
            remaining_bytes -= len(data)
            try:
                window = first_child_window(data.decode("utf-8"))
            except (ValueError, OverflowError):
                receipt["invalid_cases"] += 1
                continue
            receipt["valid_cases"] += 1
            if selected is None or window[2] - window[1] > selected[1][2] - selected[1][1]:
                selected = (case, window)  # Fixed matrix order breaks ties deterministically.
        if selected is not None:
            case, (child, created, exited) = selected
            receipt.update(status="selected", source_case=case, child_pid=child,
                           created_100ns=created, exited_100ns=exited,
                           coverage="complete" if receipt["valid_cases"] == len(LATER_CASES) else "partial")
            receipt.pop("reason")
    except (ValueError, OverflowError):
        selected = None  # Budget/byte-cap exhaustion cannot establish even an observed maximum.
        receipt["reason"] = "budget_or_byte_cap"
    (output / "later-child-selection.json").write_text(json.dumps(receipt, indent=2), encoding="utf-8")
    return selected


def collect_later_child_events(output: Path) -> None:
    later = select_later_child(output)
    if later is None:
        for channel in ("engine", "console"):
            (output / f"later-child-{channel}-events.json").write_text(
                json.dumps({"status": "unavailable", "events": []}), encoding="utf-8")
        return
    collect_first_child_events(output, later=later)
    collect_first_child_events(output, console=True, later=later)


def lifecycle_self_test() -> None:
    import io
    from unittest.mock import patch

    created, exited = 134366156919094391, 134366157056455513
    source = ("WINDOWS_AUTO_DIAG child_created time=SystemTime { intervals: 134366156919112821 } child_pid=3400 identity=608 pid=8892 thread=ThreadId(69)\n"
              f"WINDOWS_AUTO_DIAG lifetime_read_ok=1 created_100ns={created} exited_100ns={exited} identity=608 pid=8892 thread=ThreadId(135)\n")
    assert first_child_window(source) == (3400, created, exited)
    assert first_child_window(source.replace("\n", "\r\n")) == (3400, created, exited)
    assert first_child_window("test other ... ok" + "\n" + source + "\ntest other ... ok\n") == (3400, created, exited)
    for bad in (source + source, source.replace("child_pid=3400", "child_pid=0"),
                source.replace("child_pid=3400 identity=608", "child_pid=3400\ninterleaved\nidentity=608"),
                source.replace("pid=8892 thread=ThreadId(69)", "pid=8893 thread=ThreadId(69)"),
                source.replace("134366156919112821", "134366156919094390"),
                source.replace("134366156919112821", str(exited + 1)),
                source.replace("identity=608 pid=8892 thread=ThreadId(69)", "identity=609 pid=8892 thread=ThreadId(69)"),
                source + source.split("\n", 1)[0].replace("pid=8892", "pid=8893") + "\n"):
        for candidate in (bad, bad.replace("\n", "\r\n")):
            try:
                first_child_window(candidate)
            except ValueError:
                pass
            else:
                raise AssertionError("ambiguous child must be rejected")
    legacy = ("WINDOWS_AUTO_DIAG time=SystemTime { intervals: 134366156919112821 } pid=8892 thread=ThreadId(69) stage=create_process_return identity=608\n"
              "WINDOWS_AUTO_DIAG child_pid=3400 identity=608\n" + source.split("\n", 1)[1])
    for candidate in (legacy, legacy.replace(" } pid=8892", "test other ... ok } pid=\n8892")):
        try:
            first_child_window(candidate)
        except ValueError:
            pass
        else:
            raise AssertionError("legacy or fragmented identity must not be reconstructed")
    stamp = filetime_text(created + 1)
    assert stamp == "2026-10-16T09:14:51.9094392Z"
    xml = (f'<Events><Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event"><System>'
           f'<Provider Name="PowerShell"/><EventID>400</EventID><Execution ProcessID="3400"/>'
           f'<TimeCreated SystemTime="{stamp}"/><Channel>Windows PowerShell</Channel></System>'
           '<EventData><Data>DO_NOT_PERSIST_COMMAND_OR_SECRET</Data></EventData></Event></Events>')
    raw = xml.encode("utf-16")
    expected = [{"event_id": 400, "child_pid": 3400, "time_utc": stamp}]
    assert lifecycle_metadata(raw, 3400, created, exited) == expected
    assert lifecycle_metadata(xml.replace("<EventID>400", "<EventID>600").encode("utf-16"),
                              3400, created, exited) == [{**expected[0], "event_id": 600}]
    assert "(EventID=400 or EventID=403 or EventID=600)" in lifecycle_query(3400, created, exited)
    for bad in (raw + b"x" * 131073, xml.replace('ProcessID="3400"', 'ProcessID="3401"').encode("utf-16"),
                xml.replace("<EventID>400", "<EventID>800").encode("utf-16"),
                xml.replace(stamp, filetime_text(exited + 1)).encode("utf-16"),
                ("<!DOCTYPE Events>" + xml).encode("utf-16"),
                ("<Events>" + xml[8:-9] * 17 + "</Events>").encode("utf-16")):
        try:
            lifecycle_metadata(bad, 3400, created, exited)
        except ValueError:
            pass
        else:
            raise AssertionError("unqualified events must be rejected")

    # Separate fixed channel/provider/ID allowlists; cross-channel events are rejected.
    console_xml = xml.replace('Name="PowerShell"', 'Name="Microsoft-Windows-PowerShell"').replace(
        "<EventID>400", "<EventID>40961").replace(
        "<Channel>Windows PowerShell", "<Channel>Microsoft-Windows-PowerShell/Operational")
    console_raw = console_xml.encode("utf-16")
    for event_id in (40961, 40962):
        candidate = console_xml.replace("40961", str(event_id)).encode("utf-16")
        assert lifecycle_metadata(candidate, 3400, created, exited, console=True) == [
            {**expected[0], "event_id": event_id}]
    for candidate, console in ((console_raw, False), (raw, True),
                               (console_xml.replace('ProcessID="3400"', 'ProcessID="3401"').encode("utf-16"), True),
                               (console_xml.replace(stamp, filetime_text(exited + 1)).encode("utf-16"), True)):
        try:
            lifecycle_metadata(candidate, 3400, created, exited, console=console)
        except ValueError:
            pass
        else:
            raise AssertionError("cross-channel or unqualified console event")
    query = lifecycle_query(3400, created, exited, console=True)
    assert "Provider[@Name='Microsoft-Windows-PowerShell']" in query
    assert "(EventID=40961 or EventID=40962)" in query and "Execution[@ProcessID=3400]" in query

    class Utility:
        def __init__(self, timeout: bool, reap_failure: bool):
            self.stdout = io.BytesIO(raw)
            self.timeout, self.reap_failure, self.killed = timeout, reap_failure, False
        def wait(self, timeout: int) -> int:
            if (timeout == 30 and self.timeout) or (timeout == 5 and self.reap_failure):
                raise subprocess.TimeoutExpired("fixture", timeout)
            return 0
        def kill(self) -> None:
            self.killed = True

    with tempfile.TemporaryDirectory() as directory:
        output = Path(directory)
        (output / "instrumented-suite-first-1.log").write_bytes(source.replace("\n", "\r\n").encode("utf-8"))
        for timeout, reap_failure in ((False, False), (True, False), (True, True)):
            utility = Utility(timeout, reap_failure)
            with patch.dict(os.environ, {"SystemRoot": directory}), patch.object(subprocess, "Popen", return_value=utility) as launch:
                try:
                    collect_first_child_events(output)
                except RuntimeError:
                    assert reap_failure
                else:
                    assert not reap_failure
            receipt = json.loads((output / "first-child-engine-events.json").read_text())
            assert receipt["status"] == ("unavailable" if timeout else "records")
            assert "DO_NOT_PERSIST" not in json.dumps(receipt)
            assert utility.killed == timeout
            argv = launch.call_args.args[0]
            assert argv[1:3] == ["qe", "Windows PowerShell"] and "/c:17" in argv
            assert argv[3] == "/q:" + lifecycle_query(3400, created, exited)
        utility = Utility(False, False)
        utility.stdout = io.BytesIO(console_raw)
        with patch.dict(os.environ, {"SystemRoot": directory}), patch.object(subprocess, "Popen", return_value=utility) as launch:
            collect_first_child_events(output, console=True)
        receipt = json.loads((output / "first-child-console-events.json").read_text())
        assert receipt["status"] == "records" and receipt["events"][0]["event_id"] == 40961
        assert "DO_NOT_PERSIST" not in json.dumps(receipt)
        assert launch.call_args.args[0][1:4] == ["qe", "Microsoft-Windows-PowerShell/Operational", "/q:" + query]
        with patch.dict(globals(), {"END_BY": 0}):
            with patch.object(subprocess, "Popen") as launch:
                collect_first_child_events(output, console=True)
            launch.assert_not_called()
        assert json.loads((output / "first-child-console-events.json").read_text())["status"] == "unavailable"
        # A completed classic query may leave too little existing budget for console.
        utility = Utility(False, False)
        with patch.dict(globals(), {"END_BY": 100}), patch.object(time, "monotonic", side_effect=(0, 60)):
            with patch.dict(os.environ, {"SystemRoot": directory}), patch.object(subprocess, "Popen", return_value=utility) as launch:
                collect_first_child_events(output)
                collect_first_child_events(output, console=True)
            assert launch.call_count == 1
        assert json.loads((output / "first-child-engine-events.json").read_text())["status"] == "records"
        assert json.loads((output / "first-child-console-events.json").read_text())["status"] == "unavailable"
        utility = Utility(False, False)
        with patch.dict(os.environ, {"SystemRoot": directory}), patch.object(subprocess, "Popen", return_value=utility), patch.object(threading.Thread, "start", side_effect=RuntimeError("fixture")):
            collect_first_child_events(output)
        assert utility.killed and utility.stdout.closed
        assert json.loads((output / "first-child-engine-events.json").read_text())["status"] == "unavailable"

        # Missing/invalid later logs are counted, never repaired or called full coverage.
        assert select_later_child(output) is None
        selection = json.loads((output / "later-child-selection.json").read_text())
        assert selection["valid_cases"] == 0 and selection["invalid_cases"] == 59
        for case in LATER_CASES:
            (output / f"{case}.log").write_bytes(source.encode("utf-8"))
        assert select_later_child(output) == (LATER_CASES[0], (3400, created, exited))
        assert json.loads((output / "later-child-selection.json").read_text())["coverage"] == "complete"
        longer = source.replace(str(exited), str(exited + 1_000_000))
        (output / f"{LATER_CASES[22]}.log").write_bytes(longer.replace("\n", "\r\n").encode("utf-8"))
        (output / f"{LATER_CASES[1]}.log").write_text("fragmented identity", encoding="utf-8")
        later = select_later_child(output)
        assert later == (LATER_CASES[22], (3400, created, exited + 1_000_000))
        selection = json.loads((output / "later-child-selection.json").read_text())
        assert selection["valid_cases"] == 58 and selection["invalid_cases"] == 1 and selection["coverage"] == "partial"
        huge = 10 ** 30
        overflow = source.replace(str(created), str(huge)).replace(str(exited), str(huge + 1_000_000)).replace(
            "134366156919112821", str(huge + 1))
        (output / f"{LATER_CASES[1]}.log").write_text(overflow, encoding="utf-8")
        assert select_later_child(output) == later
        assert json.loads((output / "later-child-selection.json").read_text())["coverage"] == "partial"
        utility = Utility(False, False)
        with patch.dict(os.environ, {"SystemRoot": directory}), patch.object(subprocess, "Popen", return_value=utility):
            collect_first_child_events(output, later=later)
        receipt = json.loads((output / "later-child-engine-events.json").read_text())
        assert receipt["status"] == "records" and receipt["source_case"] == LATER_CASES[22]
        # Re-read identity must match the selected window; no arbitrary paths/PIDs.
        for invalid in (("../outside", later[1]), (later[0], (3401, created, exited))):
            with patch.object(subprocess, "Popen") as launch:
                collect_first_child_events(output, later=invalid)
            launch.assert_not_called()
        first_receipt = (output / "first-child-engine-events.json").read_bytes()
        console_utility = Utility(False, False)
        console_utility.stdout = io.BytesIO(console_raw)
        with patch.dict(os.environ, {"SystemRoot": directory}), patch.object(subprocess, "Popen", side_effect=(Utility(False, False), console_utility)) as launch:
            collect_later_child_events(output)
        assert launch.call_count == 2
        assert (output / "first-child-engine-events.json").read_bytes() == first_receipt
        assert json.loads((output / "later-child-console-events.json").read_text())["status"] == "records"
        # Scan + first later query may leave insufficient budget for the console query.
        with patch.dict(globals(), {"END_BY": 100}), patch.object(time, "monotonic", side_effect=([0] * 60 + [60])):
            with patch.dict(os.environ, {"SystemRoot": directory}), patch.object(subprocess, "Popen", return_value=Utility(False, False)) as launch:
                collect_later_child_events(output)
            assert launch.call_count == 1
        assert json.loads((output / "later-child-engine-events.json").read_text())["status"] == "records"
        assert json.loads((output / "later-child-console-events.json").read_text())["status"] == "unavailable"
        for changes in ({"END_BY": 0}, {"LATER_SCAN_BYTES": 1}):
            with patch.dict(globals(), changes), patch.object(subprocess, "Popen") as launch:
                collect_later_child_events(output)
            launch.assert_not_called()
            assert json.loads((output / "later-child-selection.json").read_text())["status"] == "unavailable"
            assert json.loads((output / "later-child-console-events.json").read_text())["status"] == "unavailable"

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
    helper += '''#[cfg(windows)]
fn windows_auto_identity_receipt(receipt: String) {
    use std::io::Write as _;
    // One preformatted buffer reduces cross-stream formatting fragments. Still reject any damaged receipt.
    let framed = format!("\\n{receipt}\\n");
    let _ = std::io::stderr().lock().write_all(framed.as_bytes());
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
                        windows_auto_identity_receipt(format!("WINDOWS_AUTO_DIAG lifetime_read_ok=1 created_100ns={} exited_100ns={} identity={} pid={} thread={:?}", ticks(created), ticks(exited), process as usize, std::process::id(), std::thread::current().id()));
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
        ("        process = info.hProcess;", "        windows_auto_probe(\"create_process_return\", info.hProcess as usize);\n        windows_auto_identity_receipt(format!(\"WINDOWS_AUTO_DIAG child_created time={:?} child_pid={} identity={} pid={} thread={:?}\", std::time::SystemTime::now(), info.dwProcessId, info.hProcess as usize, std::process::id(), std::thread::current().id()));\n        process = info.hProcess;"),
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
    # Attribute the new assessor-entry timeout independently of concurrent Auto children.
    evaluation_awaits = host.count(".await")
    evaluation_helper = '''fn windows_auto_evaluation_stage(stage: &str) {
    use std::io::Write as _;
    let record = format!("\\nWINDOWS_AUTO_DIAG time={:?} pid={} thread={:?} stage={}\\n",
        std::time::SystemTime::now(), std::process::id(), std::thread::current().id(), stage);
    let _ = std::io::stderr().lock().write_all(record.as_bytes());
}

'''
    evaluation_helper += '''fn windows_auto_evaluation_unavailable_stage(reason: &str) -> &'static str {
    match reason {
        "workspace evidence capture timed out" => "eval_prod_reason_workspace_timeout",
        "workspace evidence capture is busy" => "eval_prod_reason_workspace_busy",
        "workspace evidence capture failed" => "eval_prod_reason_workspace_worker_failed",
        "artifact capture deadline exceeded" => "eval_prod_reason_artifact_timeout",
        "artifact capture already running" => "eval_prod_reason_artifact_busy",
        _ => "eval_prod_reason_other",
    }
}

'''
    host = replace_once(host, "fn windows_auto_host_stage(stage: &str) {", evaluation_helper + "fn windows_auto_host_stage(stage: &str) {")
    begin = host.index("    struct ObservedArtifactAssessor {")
    end = host.index("    async fn next_production_output(", begin)
    assessor = host[begin:end]
    assessor = replace_once(assessor, "            use talos_core::evaluation::{", '            windows_auto_evaluation_stage("eval_prod_assessor_enter");\n            use talos_core::evaluation::{')
    assessor = replace_once(assessor, "            self.entered.notify_one();", '            self.entered.notify_one();\n            windows_auto_evaluation_stage("eval_prod_assessor_notified");')
    host = host[:begin] + assessor + host[end:]
    begin = host.index("    async fn production_evaluation_uses_authorized_artifacts_and_rejects_stale_or_superseded_results()")
    end = host.index("    #[tokio::test", begin)
    fixture = host[begin:end]
    fixture = replace_once(fixture, '        for scenario in ["current", "stale", "new-submit", "restart"] {', '''        for scenario in ["current", "stale", "new-submit", "restart"] {
            windows_auto_evaluation_stage(match scenario {
                "current" => "eval_prod_case_current",
                "stale" => "eval_prod_case_stale",
                "new-submit" => "eval_prod_case_new_submit",
                _ => "eval_prod_case_restart",
            });''')
    fixture = replace_once(fixture, '                .expect("evaluate");', '                .expect("evaluate");\n            windows_auto_evaluation_stage("eval_prod_request_sent");')
    fixture = replace_once(fixture, '                tokio::time::timeout(std::time::Duration::from_secs(10), entered.notified())', '                windows_auto_evaluation_stage("eval_prod_assessor_gate_begin");\n                tokio::time::timeout(std::time::Duration::from_secs(10), entered.notified())')
    fixture = replace_once(fixture, '                    .expect("actual observations reach assessor");', '                    .inspect_err(|_| windows_auto_evaluation_stage("eval_prod_assessor_gate_timeout"))\n                    .expect("actual observations reach assessor");\n                windows_auto_evaluation_stage("eval_prod_assessor_gate_satisfied");')
    host = host[:begin] + fixture + host[end:]
    begin = host.index("                    Some(RuntimeCommand::Evaluate(goal)) => {")
    end = host.index("            event = handle.next_event() => {", begin)
    evaluation = host[begin:end]
    evaluation = replace_once(evaluation, "                    Some(RuntimeCommand::Evaluate(goal)) => {", '                    Some(RuntimeCommand::Evaluate(goal)) => {\n                        windows_auto_evaluation_stage("eval_prod_request_received");')
    evaluation = replace_once(evaluation, "                        if evaluation.is_some() {", '                        if evaluation.is_some() {\n                            windows_auto_evaluation_stage("eval_prod_busy_skip");')
    evaluation = replace_once(evaluation, '                            let reason = if evidence_source.is_none() {', '                            windows_auto_evaluation_stage("eval_prod_missing_source_or_service");\n                            let reason = if evidence_source.is_none() {')
    evaluation = replace_once(evaluation, "                        evaluation = Some(Box::pin(async move {", '                        evaluation = Some(Box::pin(async move {\n                        windows_auto_evaluation_stage("eval_prod_future_polled");')
    context_end = evaluation.index("                        let context = evaluation_context;")
    context = evaluation[:context_end]
    context = replace_once(context, "                            let workspace = match source.workspace_revision_async().await {", '''                            windows_auto_evaluation_stage("eval_prod_revision_enter");
                            let workspace_result = source.workspace_revision_async().await;
                            windows_auto_evaluation_stage("eval_prod_revision_return");
                            let workspace = match workspace_result {''')
    context = replace_once(context, "                                    return RuntimeOutput::EvaluationUnavailable { reason: error };", '                                    windows_auto_evaluation_stage("eval_prod_revision_unavailable");\n                                    windows_auto_evaluation_stage(windows_auto_evaluation_unavailable_stage(&error));\n                                    return RuntimeOutput::EvaluationUnavailable { reason: error };')
    context = replace_once(context, "                                    return RuntimeOutput::EvaluationUnavailable { reason: error.to_string() };", '                                    windows_auto_evaluation_stage("eval_prod_context_unavailable");\n                                    return RuntimeOutput::EvaluationUnavailable { reason: error.to_string() };')
    evaluation = context + evaluation[context_end:]
    evaluation = replace_once(evaluation, "                                let snapshot = match if let Some(registry) = artifact_registry {", '                                windows_auto_evaluation_stage("eval_prod_capture_enter");\n                                let snapshot_result = if let Some(registry) = artifact_registry {')
    evaluation = replace_once(evaluation, "                                } {\n                                    Ok(snapshot) => snapshot,", '                                };\n                                windows_auto_evaluation_stage("eval_prod_capture_return");\n                                let snapshot = match snapshot_result {\n                                    Ok(snapshot) => snapshot,')
    evaluation = replace_once(evaluation, "                                    Err(error) => return RuntimeOutput::EvaluationUnavailable { reason: error },\n                                };\n                                let mut result = service", '                                    Err(error) => { windows_auto_evaluation_stage("eval_prod_capture_unavailable"); windows_auto_evaluation_stage(windows_auto_evaluation_unavailable_stage(&error)); return RuntimeOutput::EvaluationUnavailable { reason: error }; },\n                                };\n                                windows_auto_evaluation_stage("eval_prod_service_enter");\n                                let mut result = service')
    evaluation = replace_once(evaluation, "                                    .evaluate_snapshot(source.session_id(), context.claim(), &snapshot)\n                                    .await;", '''                                    .evaluate_snapshot(source.session_id(), context.claim(), &snapshot)
                                    .await;
                                windows_auto_evaluation_stage(match &result {
                                    talos_runtime::EvaluatorOutcome::Report { .. } => "eval_prod_service_report",
                                    talos_runtime::EvaluatorOutcome::Failure(_) => "eval_prod_service_failure",
                                });''')
    host = host[:begin] + evaluation + host[end:]
    if host.count(".await") != evaluation_awaits:
        raise ValueError("evaluation probes must preserve await count")
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
        lifecycle_self_test()
        assert all(patched[path] != originals[path] for path in originals)
        assert "Duration::from_secs(15)" in patched[AUTO]
        assert "Duration::from_secs(10)" in patched[AUTO]
        assert patched[BOUNDARY].count("windows_auto_probe(") >= 10
        assert patched[HOST].count("stage=host_heartbeat") == 1
        for stage in ("case_current", "case_stale", "case_new_submit", "case_restart",
                      "request_sent", "request_received", "future_polled", "revision_enter",
                      "revision_return", "revision_unavailable", "context_unavailable",
                      "capture_enter", "capture_return", "capture_unavailable", "service_enter",
                      "service_report", "service_failure", "assessor_enter", "assessor_notified",
                      "assessor_gate_begin", "assessor_gate_timeout", "assessor_gate_satisfied",
                      "reason_workspace_timeout", "reason_workspace_busy", "reason_workspace_worker_failed",
                      "reason_artifact_timeout", "reason_artifact_busy", "reason_other"):
            assert patched[HOST].count('"eval_prod_' + stage + '"') == 1
        assert patched[HOST].count("windows_auto_evaluation_unavailable_stage(&error)") == 2
        assert patched[HOST].count('expect("actual observations reach assessor")') == originals[HOST].count('expect("actual observations reach assessor")') == 1
        assert "Duration::from_secs(10), entered.notified()" in patched[HOST]
        assert "WaitForSingleObject(process, 1_000)" in patched[BOUNDARY]
        assert "observed != windows_sys::Win32::Foundation::WAIT_TIMEOUT" in patched[BOUNDARY]
        assert "GetProcessTimes(process," in patched[BOUNDARY]
        assert patched[BOUNDARY].count("GetProcessTimes(process,") == 1
        assert patched[BOUNDARY].count("if observed == windows_sys::Win32::Foundation::WAIT_OBJECT_0 && times_read_ok != 0 {") == 1
        assert patched[BOUNDARY].count("lifetime_read_ok=1 created_100ns={}") == 1
        assert patched[BOUNDARY].count("WINDOWS_AUTO_DIAG child_created time={:?}") == 1
        assert patched[BOUNDARY].count("std::io::stderr().lock().write_all(framed.as_bytes())") == 1
        assert patched[BOUNDARY].count("windows_auto_identity_receipt(format!(") == 2
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
    fresh_matrix_completed = False
    try:
        if args.fresh_phase:
            if args.fresh_phase == "instrumented":
                for path, text in patched.items():
                    (ROOT / path).write_text(text, encoding="utf-8", newline="\n")
            matrix(build(output, args.fresh_phase), output, args.fresh_phase, results, fresh_first=True)
            fresh_matrix_completed = True
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
        if args.fresh_phase == "instrumented" and fresh_matrix_completed:
            collect_first_child_events(output)
            collect_first_child_events(output, console=True)
            collect_later_child_events(output)
    return int(any(result["exit_code"] != 0 for result in results))


if __name__ == "__main__":
    raise SystemExit(main())
