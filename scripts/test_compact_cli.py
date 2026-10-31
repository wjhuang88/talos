#!/usr/bin/env python3
"""Exercise real inline /compact with an isolated mock session (no paid calls)."""
import os
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import threading


def main():
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/talos").resolve()
    with tempfile.TemporaryDirectory(prefix="talos-compact-cli-") as directory:
        root = Path(directory)
        # Only the child receives an isolated configuration/session home.
        environment = {key: value for key, value in os.environ.items()
                       if not key.startswith("TALOS_")}
        environment.update(HOME=str(root), USERPROFILE=str(root))
        arguments = [str(binary), "--inline", "--mock", "--no-init", "--no-context"]

        def run(extra, text):
            result = subprocess.run(arguments + extra, input=text, text=True,
                                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                    env=environment, cwd=root, timeout=45, check=True)
            return result.stdout

        run([], "COMPACT_RAW_SENTINEL\n")
        transcripts = list(root.rglob("*.tlog")) + list(root.rglob("*.jsonl"))
        assert len(transcripts) == 1, transcripts
        transcript = transcripts[0]
        original = transcript.read_bytes()
        assert b"COMPACT_RAW_SENTINEL" in original
        output = run(["--session", transcript.stem], "/compact extra\n/compact\n")
        assert "Usage: /compact" in output, output
        assert "Compaction " in output and "skipped: estimated tokens" in output, output
        assert transcript.read_bytes() == original, "manual command rewrote raw transcript"
        assert b"/compact" not in original
        output = run(["--fork", transcript.stem], "/compact\n")
        assert "Forked Session" in output, output
        assert "skipped: estimated tokens" in output, output
        forks = [path for path in root.rglob("*.tlog") if path != transcript]
        assert len(forks) == 1, forks
        assert forks[0].read_bytes() == original, "fork compact changed cloned raw history"
        assert transcript.read_bytes() == original, "fork compact changed parent"
        legacy = transcript.parent / f"{uuid.uuid4()}.jsonl"
        legacy.write_text(json.dumps({
            "id": str(uuid.uuid4()), "timestamp": "2026-10-01T00:00:00Z",
            "role": "user", "content": "LEGACY_COMPACT_SENTINEL",
        }) + "\n", encoding="utf-8")
        legacy_bytes = legacy.read_bytes()
        for option in ["--session", "--fork"]:
            output = run([option, legacy.stem], "/compact\n")
            assert "skipped: estimated tokens" in output, output
            assert legacy.read_bytes() == legacy_bytes, "legacy parent changed"
        legacy_forks = [path for path in root.rglob("*.jsonl") if path != legacy]
        assert len(legacy_forks) == 1, legacy_forks
        assert legacy_forks[0].read_bytes() == legacy_bytes, "legacy fork changed raw bytes"
        print("CLI compact smoke passed: TLOG/legacy resume/fork visible results; raw transcripts unchanged")
        requests = []
        response_mode = ["valid"]

        class SummaryHandler(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def do_POST(self):
                payload = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
                requests.append(payload)
                prompt = payload["messages"][-1]["content"]
                if isinstance(prompt, str) and prompt.startswith("Produce only a JSON checkpoint"):
                    sources = json.loads(prompt.split("\nSources: ")[-1])
                    item = {"text": "Retain the local report", "sources": [sources[0]["source"]]}
                    checkpoint = {"schema_version": 1, "summary": item, "goal": item, "next_step": item}
                    for key in ["constraints", "completed", "in_progress", "pending", "decisions",
                                "facts", "assumptions", "resources", "open_loops"]:
                        checkpoint[key] = []
                    summary = json.dumps(checkpoint) if response_mode[0] == "valid" else "invalid-private-summary"
                else:
                    summary = "LOCAL_CHECKPOINT_CONTINUATION_OK"
                events = [
                    {"choices": [{"delta": {"content": summary}, "finish_reason": None}]},
                    {"choices": [{"delta": {}, "finish_reason": "stop"}]},
                ]
                body = ("".join("data: " + json.dumps(event) + "\n\n" for event in events)
                        + "data: [DONE]\n\n").encode()
                self.send_response(200)
                self.send_header("Content-Type", "text/event-stream")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

        server = ThreadingHTTPServer(("127.0.0.1", 0), SummaryHandler)
        worker = threading.Thread(target=server.serve_forever, daemon=True)
        worker.start()
        try:
            config = root / ".talos" / "config.toml"
            config.write_text(
                'provider = "local-fixture"\nmodel = "compact-fixture"\n'
                '[providers.local-fixture]\nprotocol = "openai-chat"\n'
                f'base_url = "http://127.0.0.1:{server.server_port}"\n'
                'api_key = "local-test-only"\n'
                '[providers.local-fixture.models.compact-fixture]\n'
                'context_limit = 100000\noutput_limit = 2048\n', encoding="utf-8")
            large = transcript.parent / f"{uuid.uuid4()}.jsonl"
            entries = []
            for index in range(30):
                for role, text in [("user", "Preserve local report. " + "x" * 12000),
                                   ("assistant", "Recorded")]:
                    entries.append(json.dumps({"id": str(uuid.uuid4()),
                        "timestamp": "2026-10-01T00:00:00Z", "role": role, "content": text}))
            large.write_text("\n".join(entries) + "\n", encoding="utf-8")
            saved = large.read_bytes()
            arguments.remove("--mock")
            output = run(["--session", large.stem], "/compact\n")
            assert "applied: estimated tokens" in output, output
            assert requests, "no real provider request"
            manual_summary_calls = len(requests)
            assert large.read_bytes() == saved, "successful compaction rewrote raw history"
            print("CLI compact application passed: local HTTP summary; raw history unchanged")
            response_mode[0] = "invalid"
            previous_calls = len(requests)
            output = run(["--session", large.stem], "/compact\n")
            assert "failed (preparation or validation rejected): estimated tokens" in output, output
            assert "invalid-private-summary" not in output, output
            assert len(requests) > previous_calls, "failure path did not contact provider"
            assert large.read_bytes() == saved, "failed compaction rewrote raw history"
            print("CLI compact failure passed: visible measured refusal; private summary excluded")
            response_mode[0] = "valid"
            previous_calls = len(requests)
            output = run(["--session", large.stem], "/compact\nContinue local report\n")
            assert "applied: estimated tokens" in output, output
            assert "LOCAL_CHECKPOINT_CONTINUATION_OK" in output, output
            ordinary = [request for request in requests[previous_calls:]
                        if not request["messages"][-1]["content"].startswith("Produce only a JSON checkpoint")]
            assert ordinary, "no ordinary continuation request"
            summary_calls = len(requests[previous_calls:]) - len(ordinary)
            assert summary_calls == manual_summary_calls, "continuation regenerated the manual checkpoint"
            assert "Automatic compaction started" not in output, output
            assert any("[Derived checkpoint;" in json.dumps(request["messages"])
                       for request in ordinary), "ordinary request omitted installed checkpoint"
            assert large.read_bytes().startswith(saved), "continuation rewrote original history"
            assert b"[Derived checkpoint;" not in large.read_bytes(), "derived checkpoint persisted as history"
            print("CLI checkpoint reuse passed: next ordinary request uses derived projection")
        finally:
            server.shutdown()
            server.server_close()
            worker.join(timeout=5)


if __name__ == "__main__":
    main()
