#!/usr/bin/env python3
"""Real inline CLI checkpoint discovery and legacy recovery refusal; localhost only."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


def main():
    binary = Path(sys.argv[1] if len(sys.argv) > 1 else "target/debug/talos").resolve()
    requests, errors = [], []
    with tempfile.TemporaryDirectory(prefix="talos-history-cli-") as directory:
        root = Path(directory)
        environment = {k: v for k, v in os.environ.items() if not k.startswith("TALOS_")}
        environment.update(HOME=str(root), USERPROFILE=str(root))
        base = [str(binary), "--inline", "--no-init", "--no-context"]
        subprocess.run(base + ["--mock"], input="\n".join(f"prior fact {i}" for i in range(30)) + "\n",
                       text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                       env=environment, cwd=root, timeout=60, check=True)
        transcript, = root.rglob("*.tlog")
        original = transcript.read_bytes()

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def do_POST(self):
                try:
                    payload = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
                    step = len(requests)
                    requests.append(payload)
                    names = {tool["function"]["name"] for tool in payload["tools"]}
                    assert {"compact_context", "history_search"} <= names, names
                    if step == 0:
                        arguments = {"mode": "discover"}
                    elif step == 1:
                        result = next(m for m in payload["messages"] if m.get("tool_call_id") == "history-0")
                        discovery = json.loads(result["content"])
                        assert discovery["snapshot"] and discovery["sources"], discovery
                        arguments = {"mode": "search", "snapshot": discovery["snapshot"], "query": "prior"}
                    else:
                        assert step == 2, step
                        result = next(m for m in payload["messages"] if m.get("tool_call_id") == "history-1")
                        assert "history/checkpoint execution unavailable" in result["content"], result
                    if step < 2:
                        delta = {"tool_calls": [{"index": 0, "id": f"history-{step}", "type": "function",
                                 "function": {"name": "history_search", "arguments": json.dumps(arguments)}}]}
                        reason = "tool_calls"
                    else:
                        delta, reason = {"content": "LEGACY_RECOVERY_REFUSAL_CONFIRMED"}, "stop"
                    events = [{"choices": [{"delta": delta, "finish_reason": None}]},
                              {"choices": [{"delta": {}, "finish_reason": reason}]}]
                    body = ("".join("data: " + json.dumps(e) + "\n\n" for e in events) + "data: [DONE]\n\n").encode()
                    self.send_response(200)
                    self.send_header("Content-Type", "text/event-stream")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                except Exception as error:
                    errors.append(repr(error))
                    self.send_error(500)

        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        try:
            (root / ".talos" / "config.toml").write_text(
                'provider = "local-fixture"\nmodel = "history-fixture"\n'
                '[providers.local-fixture]\nprotocol = "openai-chat"\n'
                f'base_url = "http://127.0.0.1:{server.server_port}"\napi_key = "local-test-only"\n'
                '[providers.local-fixture.models.history-fixture]\ncontext_limit = 100000\noutput_limit = 2048\n')
            result = subprocess.run(base + ["--session", transcript.stem], input="recover prior fact\n",
                                    text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                    env=environment, cwd=root, timeout=60, check=True)
            assert not errors, errors
            assert len(requests) == 3, (len(requests), result.stdout)
            assert "LEGACY_RECOVERY_REFUSAL_CONFIRMED" in result.stdout, result.stdout
            assert transcript.read_bytes().startswith(original), "raw prefix changed"
            print("PASS: real CLI discovery, legacy recovery refusal, provider continuation, raw prefix preserved")
        finally:
            server.shutdown()
            server.server_close()


if __name__ == "__main__":
    main()
