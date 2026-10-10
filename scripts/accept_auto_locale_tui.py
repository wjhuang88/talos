#!/usr/bin/env python3
"""POSIX real-TUI locale/resume/rotation acceptance against loopback fixtures.

Uses a disposable controlling PTY, HOME and workspace. Never invokes a paid model.
Checks rendered prompt text and the provider's real tool-result denial, not layout.
"""
import argparse
import fcntl
import http.server
import json
import os
from pathlib import Path
import pty
import queue
import re
import selectors
import struct
import subprocess
import sys
import tempfile
import termios
import threading
import time

from accept_auto_locale import Fixture, HISTORIES, POINTS, completion_marker


class Tui:
    def __init__(self, binary, workspace, task_env, session_id=None):
        self.master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 50, 140, 0, 0))
        command = [str(binary), "--tui", "--no-init", "--no-context", "-w", str(workspace)]
        if session_id:
            command += ["--session", session_id]
        # The fixture HTTP server already has threads. Python preexec_fn can
        # deadlock between fork and exec there, before any wait_for deadline.
        # Run the controlling-terminal setup in a fresh, single-threaded child
        # interpreter, then replace it with the real CLI (same PID and PTY).
        bootstrap = (
            "import fcntl, os, sys, termios; "
            "fcntl.ioctl(0, termios.TIOCSCTTY, 0); "
            "os.execvpe(sys.argv[1], sys.argv[1:], os.environ)"
        )
        self.process = subprocess.Popen(
            [sys.executable, "-c", bootstrap, *command],
            stdin=slave, stdout=slave, stderr=slave, env=task_env, cwd=workspace,
            start_new_session=True,
        )
        os.close(slave)
        # PTYs are not ordinary sockets: readiness can be stale, and the
        # platform default (kqueue on Darwin) has different terminal semantics.
        # Never let a blocking read/write bypass the interaction deadline.
        os.set_blocking(self.master, False)
        self.selector = selectors.SelectSelector()
        self.selector.register(self.master, selectors.EVENT_READ)
        self.transcript = bytearray()
        self.offset = 0

    def wait_for(self, marker, timeout=25):
        print(f"TUI waiting for {marker!r}", flush=True)
        until = time.monotonic() + timeout
        while time.monotonic() < until:
            text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", self.transcript[self.offset:].decode(errors="replace"))
            # Cursor-diff output can omit already-present whitespace. Assert
            # text emission independently of screen layout and spacing.
            if "".join(marker.split()) in "".join(text.split()):
                return
            for _ in self.selector.select(0.2):
                try:
                    data = os.read(self.master, 65536)
                except BlockingIOError:
                    continue
                if not data:
                    raise RuntimeError(f"PTY closed while waiting for {marker}: {text}")
                self.transcript.extend(data)
                if b"\x1b[6n" in data:
                    self.write(b"\x1b[1;1R", until)
        raise RuntimeError(f"waiting for {marker}: {text}")

    def write(self, data, until):
        while data:
            if time.monotonic() >= until:
                raise RuntimeError("PTY input deadline exceeded")
            try:
                written = os.write(self.master, data)
                data = data[written:]
            except BlockingIOError:
                time.sleep(0.01)

    def send(self, value, enter=True):
        self.offset = len(self.transcript)
        until = time.monotonic() + 25
        self.write(value.encode(), until)
        if enter:
            if value.startswith("/"):
                # Existing slash picker suppresses Enter inside its 50ms IME
                # window. Let typed text settle like ordinary user key input.
                time.sleep(0.15)
            self.write(b"\r", until)

    def close(self):
        self.process.terminate()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait(timeout=5)
        self.selector.close()
        os.close(self.master)


def review(client, server, expected):
    before = len(server.conversations)
    assert server.reviews.empty()
    client.send("LOCALE_PROBE")
    client.wait_for(POINTS[expected.split("-")[0]])
    request = server.reviews.get(timeout=2)
    assert server.reviews.empty(), "one assessment per approval"
    assert len(request["messages"]) == 2
    payload = json.loads(request["messages"][-1]["content"].split("\n", 1)[1])
    assert payload["locale"] == expected, payload
    content = json.dumps(request["messages"], ensure_ascii=False)
    assert all(history not in content for history in HISTORIES.values())
    client.send("3", enter=False)
    client.wait_for(completion_marker(server, before))
    assert any(
        message.get("role") == "tool" and "denied" in str(message.get("content", "")).lower()
        for body in server.conversations[before:] for message in body["messages"]
    ), "manual Deny must reach the actual tool result"
    return request


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--capture-dir", type=Path)
    args = parser.parse_args()
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Fixture)
    server.reviews, server.conversations = queue.Queue(), []
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        for language, history in HISTORIES.items():
            with tempfile.TemporaryDirectory(prefix="talos-tui-locale-") as directory:
                root = Path(directory)
                user_home, workspace = root / "user-home", root / "workspace"
                config_dir = user_home / ".talos"
                config_dir.mkdir(parents=True)
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
                task_env = dict(os.environ, HOME=str(user_home), NO_PROXY="127.0.0.1,localhost", TERM="xterm-256color")
                session_id = None
                for resumed in [False, True]:
                    phase = "resumed" if resumed else "initial"
                    print(f"TUI {language} {phase}: starting child", flush=True)
                    client = Tui(args.binary.resolve(), workspace, task_env, session_id)
                    print(f"TUI {language} {phase}: child started", flush=True)
                    try:
                        client.wait_for("gpt-4o")
                        if not resumed:
                            for _ in range(3):
                                before = len(server.conversations)
                                client.send(history)
                                client.wait_for(completion_marker(server, before, history))
                        request = review(client, server, language)
                        if args.capture_dir:
                            args.capture_dir.mkdir(parents=True, exist_ok=True)
                            (args.capture_dir / f"tui-{language}-{phase}.json").write_text(json.dumps(request, ensure_ascii=False))
                        if not resumed:
                            files = list((config_dir / "sessions").rglob("*.tlog"))
                            assert len(files) == 1, "resume exact persisted session"
                            session_id = files[0].stem
                        else:
                            client.send("/new")
                            client.wait_for("New session started.")
                            request = review(client, server, "en-US")
                            if args.capture_dir:
                                (args.capture_dir / f"tui-{language}-rotated.json").write_text(json.dumps(request, ensure_ascii=False))
                            print(f"TUI {language} /new: configured en-US fallback and manual Deny PASS", flush=True)
                        print(f"TUI {language} {phase}: localized prompt, manual Deny, isolated request PASS", flush=True)
                    finally:
                        client.close()
    finally:
        server.shutdown()
        server.server_close()


if __name__ == "__main__":
    main()
