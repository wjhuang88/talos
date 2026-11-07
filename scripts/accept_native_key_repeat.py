#!/usr/bin/env python3
"""Real CLI/PTY protocol acceptance; not physical keyboard/IME certification."""
import argparse
import http.server
import json
import os
from pathlib import Path
import sys
import tempfile
import threading
import time

sys.dont_write_bytecode = True
from accept_auto_locale_tui import Tui


class Provider(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        self.server.requests.append(body)
        reply = {'choices': [{'index': 0, 'delta': {'content': 'repeat fixture done'},
                              'finish_reason': 'stop'}]}
        payload = ('data: ' + json.dumps(reply) + '\n\ndata: [DONE]\n\n').encode()
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Content-Length', str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    args = parser.parse_args()
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Provider)
    server.requests = []
    threading.Thread(target=server.serve_forever, daemon=True).start()
    try:
        with tempfile.TemporaryDirectory(prefix='talos-repeat-') as directory:
            root = Path(directory)
            user_home, workspace = root / 'home', root / 'workspace'
            config = user_home / '.talos'
            config.mkdir(parents=True)
            workspace.mkdir()
            (config / 'config.toml').write_text(f'''provider = "openai"
model = "gpt-4o"
[providers.openai]
base_url = "http://127.0.0.1:{server.server_port}/v1"
api_key = "local-fixture-placeholder"
''')
            env = dict(os.environ, HOME=str(user_home), TERM='xterm-kitty',
                       NO_PROXY='127.0.0.1,localhost')
            client = Tui(args.binary.resolve(), workspace, env)
            try:
                # Pump startup queries; the inherited helper answers cursor position.
                def settle(seconds=0.3):
                    until = time.monotonic() + seconds
                    while time.monotonic() < until:
                        client.pump(until)

                client.wait_for("gpt-4o")
                def send(data):
                    client.write(data, time.monotonic() + 5)
                    settle()

                send('keep中e\u0301👩‍💻'.encode())
                # Kitty CSI-u: Backspace press/repeat/release, then orphan repeat.
                send(b'\x1b[127;1:1u\x1b[127;1:2u\x1b[127;1:3u\x1b[127;1:2u')
                # Remove CJK with forward Delete after Left, then insert replacement.
                send(b'\x1b[1;1:1D\x1b[3;1:1~')
                send(b'!')
                send(b'\r\x1b[13;1:2u')
                until = time.monotonic() + 20
                while not server.requests and time.monotonic() < until:
                    client.pump(until)
                assert server.requests, 'provider received no request; terminal tail: ' + client.transcript[-6000:].decode(errors='replace')
                users = [m['content'] for m in server.requests[0]['messages'] if m['role'] == 'user']
                assert users[-1] == 'keep!', users
                settle(0.5)
                assert len(server.requests) == 1, 'repeated Enter must not submit twice'
                print('PASS: real TUI CSI-u repeat/release, grapheme deletion, cursor/Delete, one-shot submit')
            finally:
                client.close()
    finally:
        server.shutdown()
        server.server_close()


if __name__ == '__main__':
    main()
