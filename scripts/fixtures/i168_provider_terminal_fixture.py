#!/usr/bin/env python3
"""Deterministic local SSE provider for I168 terminal-outcome verification."""

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
_LOG_FILE: Path


def _sse(event: str | None, payload: dict[str, Any]) -> bytes:
    prefix = f"event: {event}\n" if event else ""
    return f"{prefix}data: {json.dumps(payload, separators=(',', ':'))}\n\n".encode()


def _openai_text(text: str, finish_reason: str | None) -> bytes:
    return _sse(
        None,
        {
            "choices": [
                {
                    "index": 0,
                    "delta": {"content": text},
                    "finish_reason": finish_reason,
                }
            ]
        },
    )


def _openai_tool() -> bytes:
    return _sse(
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


def _anthropic_text(text: str) -> bytes:
    return _sse(
        "content_block_delta",
        {"index": 0, "delta": {"type": "text_delta", "text": text}},
    )


def _anthropic_terminal(reason: str) -> bytes:
    return _sse(
        "message_delta",
        {
            "delta": {"stop_reason": reason, "stop_sequence": None},
            "usage": {"output_tokens": 1},
        },
    )


def _anthropic_tool() -> bytes:
    return b"".join(
        [
            _sse(
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
            _sse("content_block_stop", {"index": 0}),
            _anthropic_terminal("tool_use"),
        ]
    )


def _has_tool_result(protocol: str, messages: Any) -> bool:
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


def _response(protocol: str, model: str, ordinal: int) -> tuple[list[bytes], int]:
    mode = model.split("-", 1)[1]
    if protocol == "openai":
        partial = _openai_text("fixture partial", None)
        table = {
            "normal": ([_openai_text("fixture normal", "stop")], 0),
            "max-tokens": ([_openai_text("fixture partial", "length")], 0),
            "unknown": ([_openai_text("fixture partial", "content_filter")], 0),
            "eof": ([partial], 0),
            "decode-error": ([partial, b"\xff"], 0),
            "transport-error": ([partial], 64),
        }
        if mode == "tool-continuation":
            return ([_openai_tool()], 0) if ordinal == 1 else ([partial], 0)
        return table[mode]

    partial = _anthropic_text("fixture partial")
    table = {
        "normal": ([_anthropic_text("fixture normal"), _anthropic_terminal("end_turn")], 0),
        "max-tokens": ([partial, _anthropic_terminal("max_tokens")], 0),
        "unknown": ([partial, _anthropic_terminal("pause_turn")], 0),
        "eof": ([partial], 0),
        "decode-error": ([partial, b"\xff"], 0),
        "transport-error": ([partial], 64),
    }
    if mode == "tool-continuation":
        return ([_anthropic_tool()], 0) if ordinal == 1 else ([partial], 0)
    return table[mode]


class FixtureHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, _format: str, *_args: Any) -> None:
        return

    def do_POST(self) -> None:  # noqa: N802
        length = int(self.headers.get("content-length", "0"))
        payload = json.loads(self.rfile.read(length) or b"{}")
        protocol = "openai" if self.path.endswith("/chat/completions") else "anthropic"
        model = str(payload.get("model", ""))
        key = (protocol, model)
        with _LOCK:
            ordinal = _COUNTS.get(key, 0) + 1
            _COUNTS[key] = ordinal
            record = {
                "protocol": protocol,
                "model": model,
                "response_ordinal": ordinal,
                "has_tool_result": _has_tool_result(protocol, payload.get("messages")),
            }
            with _LOG_FILE.open("a", encoding="utf-8") as handle:
                handle.write(json.dumps(record, separators=(",", ":")) + "\n")

        segments, declared_extra = _response(protocol, model, ordinal)
        actual_length = sum(len(segment) for segment in segments)
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Connection", "close")
        self.send_header("Content-Length", str(actual_length + declared_extra))
        self.end_headers()
        for segment in segments:
            self.wfile.write(segment)
            self.wfile.flush()
            time.sleep(0.02)
        self.close_connection = True


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port-file", required=True, type=Path)
    parser.add_argument("--log-file", required=True, type=Path)
    args = parser.parse_args()

    global _LOG_FILE
    _LOG_FILE = args.log_file
    _LOG_FILE.write_text("", encoding="utf-8")
    server = ThreadingHTTPServer(("127.0.0.1", 0), FixtureHandler)
    args.port_file.write_text(str(server.server_address[1]), encoding="utf-8")
    server.serve_forever()


if __name__ == "__main__":
    main()
