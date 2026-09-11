#!/usr/bin/env python3
"""Unix-only real TUI smoke acceptance; no external service or Python dependency.

Run after building: python3 scripts/verify_i256_text_terminal.py [--binary PATH].
Checks terminal-emitted text and persisted transcript, not pixel/color correctness.
All provider traffic, HOME, workspace, and process ownership are fixture-local.
"""

import argparse
import codecs
import contextlib
import errno
import fcntl
import json
import os
from pathlib import Path
import pty
import re
import select
import struct
import subprocess
import tempfile
import termios
import threading
import time
import unicodedata
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


CHUNKS = [
    "I256_UNICODE café 中文\r", "\n\n```rust\n",
    'fn main() { println!("I256_CODE"); }\n', "```\n\n",
    "| column | value |\n", "| --- | --- |\n", "| I256_TABLE | 42 |\n\n",
    "```unknown_i256\nI256_PLAIN_FALLBACK\n```\n\n",
    "- I256_LIST\n\n> I256_QUOTE\n\n",
    "```rust\nI256_UNTERMINATED\n",
    "I256FINALTAIL",
]
PREVIEWS = ("receiving code block...", "rendering table...",
            "formatting list...", "formatting quote...")
MARKERS = ("I256_UNICODE", "café", "中文", "I256_CODE", "I256_TABLE",
           "I256_PLAIN_FALLBACK", "I256_LIST", "I256_QUOTE",
           "I256_UNTERMINATED", "I256FINALTAIL")
ANSI = re.compile(rb"\x1b\[[0-?]*[ -/]*[@-~]|\x1b\][^\x07]*(?:\x07|\x1b\\)")


class Screen:
    """Small VT screen for crossterm cursor/erase output, not an ANSI stripping oracle."""

    def __init__(self):
        self.rows = [[" "] * 140 for _ in range(48)]
        self.row = self.col = 0
        self.pending = ""
        self.decoder = codecs.getincrementaldecoder("utf-8")("replace")
        self.seen = set()
        self.unsupported = set()
        self.saved_screen = None
        self.autowrap = True

    def inspect(self):
        text = self.text()
        self.seen.update(marker for marker in MARKERS if marker in text)

    def text(self):
        return "\n".join("".join(row) for row in self.rows)

    def feed(self, data):
        self.pending += self.decoder.decode(data)
        while self.pending:
            text = self.pending
            if text.startswith("\x1b"):
                match = re.match(r"\x1b\[([0-?]*)([ -/]*)([@-~])", text)
                if match:
                    self.inspect()
                    raw, _, command = match.groups()
                    self.pending = text[match.end():]
                    if raw and raw[0] in "?<=>":
                        if raw == "?1049" and command in "hl":
                            if command == "h":
                                if self.saved_screen is not None:
                                    self.unsupported.add(match.group(0))
                                self.saved_screen = (self.rows, self.row, self.col)
                                self.rows = [[" "] * 140 for _ in range(48)]
                                self.row = self.col = 0
                            elif self.saved_screen is not None:
                                self.rows, self.row, self.col = self.saved_screen
                                self.saved_screen = None
                            else:
                                self.unsupported.add(match.group(0))
                            continue
                        if raw == "?7" and command in "hl":
                            self.autowrap = command == "h"
                            continue
                        # Cursor visibility, mouse/focus/paste and keyboard reporting
                        # modes do not change cells. Screen/scroll modes are NOT allowed.
                        allowed_modes = {"?25", "?1000", "?1002", "?1003", "?1004",
                                         "?1005", "?1006", "?1015", "?2004"}
                        if not ((command in "hl" and raw in allowed_modes)
                                or (command == "u" and raw in {">0", ">1", ">15", "<", "<1", "?"})):
                            self.unsupported.add(match.group(0))
                        continue
                    args = [int(v) if v else 0 for v in raw.split(";")]
                    n = args[0] or 1
                    if command in "Hf":
                        self.row = min(47, n - 1)
                        self.col = min(139, (args[1] or 1) - 1 if len(args) > 1 else 0)
                    elif command == "A": self.row = max(0, self.row - n)
                    elif command == "B": self.row = min(47, self.row + n)
                    elif command == "C": self.col = min(139, self.col + n)
                    elif command == "D": self.col = max(0, self.col - n)
                    elif command == "G": self.col = min(139, n - 1)
                    elif command == "d": self.row = min(47, n - 1)
                    elif command == "K":
                        if args[0] not in (0, 1, 2):
                            self.unsupported.add(match.group(0))
                            continue
                        start, end = (0, 140) if args[0] == 2 else ((0, self.col + 1) if args[0] == 1 else (self.col, 140))
                        self.rows[self.row][start:end] = [" "] * (end - start)
                    elif command == "J":
                        if args[0] == 2: self.rows = [[" "] * 140 for _ in range(48)]
                        elif args[0] == 3: pass  # Saved scrollback only, not visible cells.
                        elif args[0] == 0:
                            self.rows[self.row][self.col:] = [" "] * (140 - self.col)
                            for r in range(self.row + 1, 48): self.rows[r] = [" "] * 140
                        else: self.unsupported.add(match.group(0))
                    elif command == "S":
                        for _ in range(min(n, 48)): self.rows.pop(0); self.rows.append([" "] * 140)
                    elif command == "T":
                        for _ in range(min(n, 48)): self.rows.pop(); self.rows.insert(0, [" "] * 140)
                    elif command not in {"m", "n", "q", "c"}:
                        self.unsupported.add(match.group(0))
                    continue
                if text.startswith("\x1b]"):
                    match = re.match(r"\x1b\].*?(?:\x07|\x1b\\)", text, re.S)
                    if not match: return
                    # Window title and hyperlink metadata do not change cells.
                    if not text.startswith(("\x1b]0;", "\x1b]1;", "\x1b]2;", "\x1b]8;")):
                        self.unsupported.add(match.group(0))
                    self.pending = text[match.end():]
                    continue
                if len(text) < 2 or text.startswith("\x1b["): return
                self.unsupported.add(text[:2])
                self.pending = text[2:]
                continue
            char, self.pending = text[0], text[1:]
            if char == "\r": self.col = 0
            elif char == "\n":
                self.inspect()
                self.row += 1
                if self.row == 48:
                    self.rows.pop(0); self.rows.append([" "] * 140); self.row = 47
            elif char == "\b": self.col = max(0, self.col - 1)
            elif char == "\t": self.col = min(139, (self.col // 8 + 1) * 8)
            elif char >= " ":
                width = 2 if unicodedata.east_asian_width(char) in ("W", "F") else 1
                if unicodedata.combining(char):
                    if self.col: self.rows[self.row][self.col - 1] += char
                    continue
                if self.col + width > 140:
                    if self.autowrap:
                        self.col = 0
                        self.row += 1
                        if self.row == 48:
                            self.inspect()
                            self.rows.pop(0); self.rows.append([" "] * 140); self.row = 47
                    else:
                        self.col = 140 - width
                self.rows[self.row][self.col] = char
                if width == 2: self.rows[self.row][self.col + 1] = ""
                self.col += width
            elif char not in ("\x00", "\x07"):
                self.unsupported.add(char)
        self.inspect()


class Provider(BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def do_POST(self):
        self.connection.settimeout(5)
        size = int(self.headers.get("Content-Length", "0"))
        if size > 4 * 1024 * 1024:
            self.send_error(413)
            return
        request = json.loads(self.rfile.read(size))
        if not self.path.endswith("/chat/completions") or not request.get("stream"):
            self.send_error(400)
            return
        self.server.requests += 1
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Connection", "close")
        self.end_headers()
        try:
            for text in CHUNKS + [None]:
                payload = {"choices": [{"index": 0,
                    "delta": {"content": text} if text is not None else {},
                    "finish_reason": None if text is not None else "stop"}]}
                self.wfile.write(("data: " + json.dumps(payload) + "\n\n").encode())
                self.wfile.flush()
                time.sleep(0.12)
            self.wfile.write(b"data: [DONE]\n\n")
            self.wfile.flush()
            self.server.finished.set()
        except (BrokenPipeError, ConnectionResetError):
            pass


def run(binary, timeout):
    server = ThreadingHTTPServer(("127.0.0.1", 0), Provider)
    server.daemon_threads = True
    server.requests = 0
    server.finished = threading.Event()
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    child = None
    master = slave = None
    temporary_directory = tempfile.TemporaryDirectory(prefix="talos-i256-terminal-")
    try:
        with contextlib.nullcontext(temporary_directory.name) as temporary:
            root = Path(temporary)
            home, workspace = root / "home", root / "workspace"
            (home / ".talos").mkdir(parents=True)
            workspace.mkdir()
            (home / ".talos/config.toml").write_text(f'''provider = "fixture"
model = "i256-text"
[providers.fixture]
protocol = "openai-chat"
base_url = "http://127.0.0.1:{server.server_port}/v1"
api_key = "fixture-only"
[providers.fixture.timeout]
dispatch_timeout_secs = 5
first_packet_timeout_secs = 5
stream_idle_timeout_secs = 5
max_attempts = 1
''', encoding="utf-8")
            # Do not inherit user/provider/proxy credentials or workspace configuration.
            env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "HOME": str(home),
                   "TERM": "xterm-256color", "LANG": "en_US.UTF-8",
                   "NO_PROXY": "127.0.0.1,localhost", "TMPDIR": str(root)}
            master, slave = pty.openpty()
            fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 48, 140, 0, 0))
            child = subprocess.Popen([str(binary), "--no-init", "--no-context",
                "--provider", "fixture", "--model", "i256-text"],
                cwd=workspace, env=env, stdin=slave, stdout=slave, stderr=slave,
                start_new_session=True)
            os.close(slave)
            slave = None
            captured = bytearray()
            screen = Screen()
            query_buffer = b""
            start = time.monotonic()
            submitted = False
            completed_at = None
            stable_screen = None
            quit_at = None
            while time.monotonic() - start < timeout:
                if select.select([master], [], [], 0.05)[0]:
                    try:
                        data = os.read(master, 65536)
                    except OSError as error:
                        if error.errno != errno.EIO:
                            raise
                        break
                    if not data:
                        break
                    captured.extend(data)
                    screen.feed(data)
                    if len(captured) > 16 * 1024 * 1024:
                        raise AssertionError("unexpected terminal output flood")
                    query_buffer += data
                    # Answer crossterm cursor-position probes, including split CSI.
                    while b"\x1b[6n" in query_buffer:
                        _, query_buffer = query_buffer.split(b"\x1b[6n", 1)
                        os.write(master, b"\x1b[1;1R")
                    query_buffer = query_buffer[-8:]
                elapsed = time.monotonic() - start
                visible = ANSI.sub(b"", bytes(captured)).decode("utf-8", errors="replace")
                if not submitted and "Enter to send" in visible and elapsed > 2:
                    os.write(master, b"I256 local text acceptance")
                    time.sleep(0.15)
                    os.write(master, b"\r")
                    submitted = True
                final_screen = screen.text()
                if (server.finished.is_set() and all(marker in screen.seen for marker in MARKERS)
                        and "I256FINALTAIL" in final_screen
                        and not any(phrase in final_screen for phrase in
                                    ("receiving code block", "rendering table", "formatting list", "formatting quote"))):
                    if completed_at is None or final_screen != stable_screen:
                        completed_at = time.monotonic()
                        stable_screen = final_screen
                    if quit_at is None and time.monotonic() - completed_at > 1:
                        os.write(master, b"\x03")
                        quit_at = time.monotonic()
                elif quit_at is None:
                    completed_at = None
                    stable_screen = None
                if quit_at is not None and time.monotonic() - quit_at > 0.5:
                    os.write(master, b"\x03")
                    quit_at = time.monotonic()
                if child.poll() is not None:
                    break
            else:
                raise AssertionError(f"TUI deadline {timeout}s exceeded; missing={set(MARKERS) - screen.seen}; tail={visible[-1800:]!r}")
            child.wait(timeout=5)
            assert child.returncode == 0, f"TUI exit {child.returncode}: {visible[-1800:]}"
            assert completed_at is not None, f"missing terminal markers: {visible[-1800:]}"
            assert not screen.unsupported, f"screen evidence invalid: unsupported escapes {screen.unsupported!r}"
            assert not screen.pending, "screen evidence ended with an incomplete escape"
            assert server.requests == 1, f"unexpected provider calls: {server.requests}"
            transcripts = list(home.rglob("*.tlog")) + list(home.rglob("*.jsonl"))
            assert transcripts, "no persisted transcript; persistence acceptance unproven"
            stored = "\n".join(path.read_text(encoding="utf-8") for path in transcripts)
            assert "I256FINALTAIL" in stored, "final partial line not persisted"
            assert all(marker not in stored for marker in PREVIEWS), "hold preview persisted"
            print("PASS: real TUI text visible for Unicode/code/table/list/quote/unknown language and unterminated Markdown recovery")
            print("PASS: stable terminal screen retains final partial line after hold preview disappears")
            print("PASS: one local SSE request; final line persisted; no hold preview in transcripts; clean exit")
            print(f"Binary: {binary}")
    finally:
        if child is not None and child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=3)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait(timeout=3)
        for descriptor in (master, slave):
            if descriptor is not None:
                os.close(descriptor)
        server.shutdown()
        server.server_close()
        thread.join(timeout=2)
        temporary_directory.cleanup()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path,
                        default=Path(__file__).resolve().parents[1] / "target/debug/talos")
    parser.add_argument("--timeout", type=float, default=45)
    arguments = parser.parse_args()
    if arguments.timeout <= 0:
        parser.error("--timeout must be positive")
    run(arguments.binary.resolve(strict=True), arguments.timeout)
