#!/usr/bin/env python3
"""Real REPL acceptance with a loopback OpenAI fixture, never a paid model.

Exercises zh/en/ja history, visible human prompts, manual Deny and durable resume.
Optional captures contain synthetic Auto-review messages only, without credentials.
Build the pinned binary with cargo build --locked -p talos-cli first.
"""
import argparse
import http.server
import json
import os
from pathlib import Path
import queue
import re
import selectors
import subprocess
import tempfile
import threading
import time

POINTS = {"zh": "是否继续检查？", "en": "Continue this inspection?", "ja": "検査を続行しますか？"}
HISTORIES = {
    "zh": "请检查项目目录中的文件",
    "en": "Please read the project file and run the tests",
    "ja": "このプロジェクトのファイルを確認してください",
}


class Fixture(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        messages = body["messages"]
        if "permission risk assessor" in messages[0].get("content", ""):
            payload = json.loads(messages[-1]["content"].split("\n", 1)[1])
            self.server.reviews.put(body)
            point = POINTS[payload["locale"].split("-")[0]]
            text = json.dumps({
                "schema_version": 1, "request_digest": payload["request_digest"],
                "decision": "human_required", "effect": "unknown",
                "reason_code": "uncertain", "confidence": "low",
                "effect_summary": point, "decision_points": [point],
            }, ensure_ascii=False)
            delta, finish = {"content": text}, "stop"
        elif messages[-1].get("role") == "user" and messages[-1].get("content") == "LOCALE_PROBE":
            delta = {"tool_calls": [{
                "index": 0, "id": "fixture_pwd", "type": "function",
                "function": {"name": "bash", "arguments": json.dumps({"command": "pwd"})},
            }]}
            finish = "tool_calls"
        else:
            self.server.conversations.append(body)
            marker = f"LOCALEFIXTUREDONE{len(self.server.conversations)}X"
            delta, finish = {"content": marker}, "stop"
        events = [
            {"choices": [{"index": 0, "delta": delta, "finish_reason": None}]},
            {"choices": [{"index": 0, "delta": {}, "finish_reason": finish}]},
        ]
        data = ("".join("data: " + json.dumps(e, ensure_ascii=False) + "\n\n" for e in events)
                + "data: [DONE]\n\n").encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


class Repl:
    def __init__(self, binary, workspace, task_env, session_id=None):
        command = [str(binary), "--repl", "--no-init", "--no-context", "-w", str(workspace)]
        if session_id:
            command += ["--session", session_id]
        self.process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.STDOUT, env=task_env, cwd=workspace)
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        self.transcript = bytearray()
        self.offset = 0

    def wait_for(self, marker, timeout=20):
        until = time.monotonic() + timeout
        encoded = marker.encode()
        while time.monotonic() < until:
            if encoded in self.transcript[self.offset:]:
                return
            for key, _ in self.selector.select(0.2):
                chunk = os.read(key.fileobj.fileno(), 8192)
                if not chunk:
                    raise RuntimeError(f"REPL exited {self.process.poll()}: {self.transcript.decode(errors='replace')}")
                self.transcript.extend(chunk)
        raise RuntimeError(f"waiting for {marker}: {self.transcript.decode(errors='replace')}")

    def send(self, text):
        self.offset = len(self.transcript)
        self.process.stdin.write((text + "\n").encode())
        self.process.stdin.flush()

    def close(self):
        self.process.terminate()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait()
        self.selector.close()


def completion_marker(server, before, history=None):
    """Match this turn's HTTP continuation, avoiding old TUI history redraws."""
    until = time.monotonic() + 20
    while time.monotonic() < until:
        for index, body in enumerate(server.conversations[before:], start=before):
            last = body["messages"][-1]
            matches = (last.get("role") == "user" and last.get("content") == history) if history else (
                last.get("role") == "tool" and "denied" in str(last.get("content", "")).lower()
            )
            if matches:
                return f"LOCALEFIXTUREDONE{index + 1}X"
        time.sleep(0.01)
    raise RuntimeError("this turn's user/denied-tool continuation was not received")


def seed(client, server, history):
    before = len(server.conversations)
    client.send(history)
    client.wait_for(completion_marker(server, before, history) + "\n> ")


def review(client, server, language, history):
    before = len(server.conversations)
    assert server.reviews.empty(), "no detection-only model calls"
    client.send("LOCALE_PROBE")
    client.wait_for(POINTS[language])
    client.wait_for("[3] Deny")
    request = server.reviews.get(timeout=2)
    assert server.reviews.empty(), "one assessment per approval"
    content = request["messages"][-1]["content"]
    payload = json.loads(content.split("\n", 1)[1])
    assert payload["locale"] == language, payload
    serialized = json.dumps(request["messages"], ensure_ascii=False)
    assert all(value not in serialized for value in HISTORIES.values()), "history must not be replayed to assessor"
    assert len(request["messages"]) == 2, "isolated assessment context"
    client.send("3")
    client.wait_for(completion_marker(server, before) + "\n> ")
    assert any(
        message.get("role") == "tool" and "denied" in str(message.get("content", "")).lower()
        for body in server.conversations[before:] for message in body["messages"]
    ), "manual Deny must reach the real tool result"
    return request


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--capture-dir", type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve()
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Fixture)
    server.reviews = queue.Queue()
    server.conversations = []
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        for language, history in HISTORIES.items():
            with tempfile.TemporaryDirectory(prefix="talos-locale-accept-") as directory:
                root = Path(directory)
                user_home = root / "user-home"
                config_dir = user_home / ".talos"
                config_dir.mkdir(parents=True)
                workspace = root / "workspace"
                workspace.mkdir()
                (config_dir / "config.toml").write_text(f'''provider = "openai"
model = "gpt-4o"
[providers.openai]
base_url = "http://127.0.0.1:{server.server_port}/v1"
api_key = "local-fixture-placeholder"
[auto]
enabled = true
locale = "en-US"
''')
                task_env = dict(os.environ, HOME=str(user_home), NO_PROXY="127.0.0.1,localhost")
                session_id = None
                for resumed in [False, True]:
                    client = Repl(binary, workspace, task_env, session_id)
                    try:
                        client.wait_for("interactive mode")
                        if not resumed:
                            for _ in range(3):
                                seed(client, server, history)
                        request = review(client, server, language, history)
                        session_id = re.search(r"\(session: ([a-f0-9-]+)\)", client.transcript.decode()).group(1)
                        if args.capture_dir:
                            args.capture_dir.mkdir(parents=True, exist_ok=True)
                            suffix = "resumed" if resumed else "initial"
                            (args.capture_dir / f"{language}-{suffix}.json").write_text(json.dumps(request, ensure_ascii=False))
                        print(f"{language} {'resume' if resumed else 'initial'}: detected history, localized prompt, Deny, isolated request PASS", flush=True)
                        if resumed:
                            switched = "ja" if language != "ja" else "en"
                            for _ in range(3):
                                seed(client, server, HISTORIES[switched])
                            switched_request = review(client, server, switched, HISTORIES[switched])
                            if args.capture_dir:
                                (args.capture_dir / f"{language}-switch-{switched}.json").write_text(json.dumps(switched_request, ensure_ascii=False))
                            print(f"{language} -> {switched}: bounded-window switching PASS", flush=True)
                    finally:
                        client.close()
    finally:
        server.shutdown()
        server.server_close()


if __name__ == "__main__":
    main()
