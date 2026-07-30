#!/usr/bin/env python3
"""Deterministic local SSE provider used only by the post-merge I168 verification."""

from __future__ import annotations

import argparse
import json
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

_COUNTS: dict[tuple[str, str], int] = {}
_LOCK = threading.Lock()
_ARGS: argparse.Namespace


def sse(event: str | None, payload: dict[str, Any]) -> bytes:
    prefix = f"event: {event}\n" if event else ""
    return f"{prefix}data: {json.dumps(payload, separators=(',', ':'))}\n\n".encode()


def openai_text(text: str, reason: str | None) -> bytes:
    return sse(
        None,
        {
            "choices": [
                {
                    "index": 0,
                    "delta": {"content": text},
                    "finish_reason": reason,
                }
            ]
        },
    )


def openai_tool() -> bytes:
    return sse(
        None,
        {
            "choices": [
                {
                    "index": 0,
                    "delta": {
                        "tool_calls": [
                            {
                                "index": 0,
                                "id": "call_fixture",
                                "type": "function",
                                "function": {
                                    "name": "git_status",
                                    "arguments": "{}",
                                },
                            }
                        ]
                    },
                    "finish_reason": "tool_calls",
                }
            ]
        },
    )


def anthropic_text(text: str) -> bytes:
    return sse(
        "content_block_delta",
        {"index": 0, "delta": {"type": "text_delta", "text": text}},
    )


def anthropic_terminal(reason: str) -> bytes:
    return sse(
        "message_delta",
        {
            "delta": {"stop_reason": reason, "stop_sequence": None},
            "usage": {"output_tokens": 1},
        },
    )


def anthropic_tool() -> bytes:
    return b"".join(
        [
            sse(
                "content_block_start",
                {
                    "index": 0,
                    "content_block": {
                        "type": "tool_use",
                        "id": "tool_fixture",
                        "name": "git_status",
                        "input": {},
                    },
                },
            ),
            sse("content_block_stop", {"index": 0}),
            anthropic_terminal("tool_use"),
        ]
    )


def has_tool_result(protocol: str, messages: Any) -> bool:
    for message in messages if isinstance(messages, list) else []:
        if not isinstance(message, dict):
            continue
        if protocol == "openai" and message.get("role") == "tool":
            return True
        content = message.get("content")
        if isinstance(content, list) and any(
            isinstance(block, dict) and block.get("type") == "tool_result"
            for block in content
        ):
            return True
    return False


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, _format: str, *_args: Any) -> None:
        return

    def do_POST(self) -> None:  # noqa: N802
        length = int(self.headers.get("content-length", "0"))
        payload = json.loads(self.rfile.read(length) or b"{}")
        protocol = "openai" if self.path.endswith("/chat/completions") else "anthropic"
        model = str(payload.get("model", ""))
        with _LOCK:
            ordinal = _COUNTS.get((protocol, model), 0) + 1
            _COUNTS[(protocol, model)] = ordinal
            with Path(_ARGS.log_file).open("a", encoding="utf-8") as handle:
                handle.write(
                    json.dumps(
                        {
                            "protocol": protocol,
                            "model": model,
                            "ordinal": ordinal,
                            "has_tool_result": has_tool_result(
                                protocol, payload.get("messages")
                            ),
                        },
                        separators=(",", ":"),
                    )
                    + "\n"
                )

        mode = model.split("-", 1)[1]
        if protocol == "openai":
            partial = openai_text("fixture partial", None)
            table = {
                "normal": ([openai_text("fixture normal", "stop")], 0),
                "max-tokens": ([openai_text("fixture partial", "length")], 0),
                "unknown": ([openai_text("fixture partial", "content_filter")], 0),
                "eof": ([partial], 0),
                "decode-error": ([partial, b"\xff"], 0),
                "transport-error": ([partial], 64),
            }
            if mode == "tool-continuation":
                segments, extra = ([openai_tool()], 0) if ordinal == 1 else ([partial], 0)
            else:
                segments, extra = table[mode]
        else:
            partial = anthropic_text("fixture partial")
            table = {
                "normal": ([anthropic_text("fixture normal"), anthropic_terminal("end_turn")], 0),
                "max-tokens": ([partial, anthropic_terminal("max_tokens")], 0),
                "unknown": ([partial, anthropic_terminal("pause_turn")], 0),
                "eof": ([partial], 0),
                "decode-error": ([partial, b"\xff"], 0),
                "transport-error": ([partial], 64),
            }
            if mode == "tool-continuation":
                segments, extra = ([anthropic_tool()], 0) if ordinal == 1 else ([partial], 0)
            else:
                segments, extra = table[mode]

        actual = sum(len(segment) for segment in segments)
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Connection", "close")
        self.send_header("Content-Length", str(actual + extra))
        self.end_headers()
        for segment in segments:
            self.wfile.write(segment)
            self.wfile.flush()
            time.sleep(0.01)
        self.close_connection = True


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port-file", required=True)
    parser.add_argument("--log-file", required=True)
    global _ARGS
    _ARGS = parser.parse_args()
    Path(_ARGS.log_file).write_text("", encoding="utf-8")
    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    Path(_ARGS.port_file).write_text(str(server.server_address[1]), encoding="utf-8")
    server.serve_forever()


if __name__ == "__main__":
    main()
