#!/usr/bin/env python3
"""I337 synthetic observation only; never contacts a provider or reads credentials."""
import argparse
import hashlib
import http.client
import json
import re
import threading
import time
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, HTTPServer

TOOLS = {"exa": "web_search_exa", "parallel": "web_search"}
QUERY = "I337 synthetic search fixture"
SCHEMA = {"type": "object", "properties": {"query": {"type": "string"}},
          "required": ["query"], "additionalProperties": False}
LIMIT = 65536
STEPS = ("initialize", "notifications/initialized", "tools/list", "tools/call")
CASES = ("json", "sse", "extra-tool", "schema-drift", "duplicate-tool", "missing-tool",
         "bad-json", "bad-sse", "wrong-id", "rpc-error", "tool-error", "bad-result",
         "partial", "oversized", "redirect", "401", "403", "429", "500",
         "proxy", "tls", "cancelled", "deadline", "budget", "invalid-url")


class Reject(Exception):
    pass


def strict_json(body):
    def unique(items):
        result = {}
        for key, value in items:
            if key in result:
                raise Reject("malformed-json")
            result[key] = value
        return result
    try:
        return json.loads(body.decode("utf-8"), object_pairs_hook=unique,
                          parse_constant=lambda _: (_ for _ in ()).throw(Reject("malformed-json")))
    except (ValueError, UnicodeError, RecursionError):
        raise Reject("malformed-json") from None


def decode(body, media, request_id):
    if media == "text/event-stream":
        if not body.endswith(b"\n\n"):
            raise Reject("malformed-sse")
        events = [part for part in body.split(b"\n\n") if part]
        if len(events) != 1:
            raise Reject("malformed-sse")
        lines = events[0].splitlines()
        if any(line != b"event: message" and not line.startswith(b"data: ") for line in lines):
            raise Reject("malformed-sse")
        data = [line[6:] for line in lines if line.startswith(b"data: ")]
        if len(data) != 1:
            raise Reject("malformed-sse")
        body = data[0]
    elif media != "application/json":
        raise Reject("media-type")
    obj = strict_json(body)
    if not isinstance(obj, dict) or obj.get("jsonrpc") != "2.0":
        raise Reject("rpc-envelope")
    if type(obj.get("id")) is not int or obj["id"] != request_id:
        raise Reject("request-id")
    if "error" in obj:
        raise Reject("rpc-error")
    if "result" not in obj:
        raise Reject("rpc-envelope")
    return obj["result"]


def fixture(provider, case, step, req):
    """Fixed synthetic server response; never interpret a discovered tool as authority."""
    expected = TOOLS[provider]
    if step == 1:
        return 202, "application/json", b"", None
    if step == 0:
        value = {"protocolVersion": "2024-11-05", "capabilities": {"tools": {}}}
    elif step == 2:
        tools = [{"name": expected, "inputSchema": SCHEMA}]
        if case == "extra-tool":
            tools.append({"name": "web_research", "inputSchema": SCHEMA})
        if case == "schema-drift":
            tools = [{"name": expected, "inputSchema": {"type": "object"}}]
        if case == "duplicate-tool":
            tools += tools
        if case == "missing-tool":
            tools = [{"name": "web_research", "inputSchema": SCHEMA}]
        value = {"tools": tools}
    else:
        if req.get("params") != {"name": expected, "arguments": {"query": QUERY}}:
            return 403, "application/json", b"{}", None
        url = "https://example.com/synthetic"
        if case == "invalid-url":
            url = "https://user:secret@example.com/synthetic"
        value = {"structuredContent": {"results": [{"url": url, "title": "synthetic"}]}}
        if case == "tool-error":
            value = {"isError": True, "content": [{"text": "synthetic secret"}]}
        if case == "bad-result":
            value = {"structuredContent": {"results": "wrong"}}
    obj = {"jsonrpc": "2.0", "id": req["id"], "result": value}
    if step == 3:
        if case in ("401", "403", "429", "500"):
            return int(case), "application/json", b'{"error":"synthetic secret"}', None
        if case == "redirect":
            return 302, "application/json", b"", None
        if case == "wrong-id":
            obj["id"] += 1
        if case == "rpc-error":
            obj = {"jsonrpc": "2.0", "id": req["id"], "error": {"code": -32000, "message": "secret"}}
        if case == "bad-json":
            return 200, "application/json", b"{", None
        if case == "bad-sse":
            return 200, "text/event-stream", b"data: {}\n", None
        if case == "oversized":
            return 200, "application/json", b"x" * (LIMIT + 1), None
        if case == "partial":
            return 200, "application/json", b"{}", 20
    body = json.dumps(obj, separators=(",", ":")).encode()
    if case == "sse":
        return 200, "text/event-stream", b"event: message\ndata: " + body + b"\n\n", None
    return 200, "application/json", body, None


def observe(provider, case, commit):
    if provider not in TOOLS or case not in CASES or not re.fullmatch(r"[0-9a-fA-F]{40}", commit):
        raise ValueError("invalid fixed fixture or commit")
    seen = []

    class Handler(BaseHTTPRequestHandler):
        def setup(self):
            super().setup()
            self.connection.settimeout(1.0)

        def log_message(self, *_):
            pass
        def do_POST(self):
            n = int(self.headers.get("Content-Length", "0"))
            if self.path != "/mcp" or not 0 < n <= LIMIT or len(seen) >= len(STEPS):
                self.send_error(400)
                return
            req = strict_json(self.rfile.read(n))
            step = len(seen)
            if req.get("method") != STEPS[step] or req.get("jsonrpc") != "2.0":
                self.send_error(400)
                return
            seen.append(STEPS[step])
            status, media, body, declared = fixture(provider, case, step, req)
            self.send_response(status)
            self.send_header("Content-Type", media)
            self.send_header("Content-Length", str(len(body) if declared is None else declared))
            if status == 302:
                self.send_header("Location", "https://example.invalid/never-contact")
            self.end_headers()
            try:
                self.wfile.write(body)
            except (BrokenPipeError, ConnectionResetError):
                pass
    count = 0
    budget = 3 if case == "budget" else 4
    cancelled = threading.Event()
    admitted = None
    outcome = "ok"
    deadline = time.monotonic() + 5.0
    # Fixed IPv4 loopback and direct HTTPConnection: no DNS, proxies, redirects or auth.
    with HTTPServer(("127.0.0.1", 0), Handler) as server:
        worker = threading.Thread(target=server.serve_forever, kwargs={"poll_interval": 0.01}, daemon=True)
        worker.start()
        try:
            for step, method in enumerate(STEPS):
                if step == 3:
                    if case in ("proxy", "tls"):
                        raise Reject(case)
                    if case == "cancelled":
                        cancelled.set()
                    if case == "deadline":
                        deadline = time.monotonic() - 1
                if cancelled.is_set():
                    raise Reject("cancelled")
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise Reject("deadline")
                if count >= budget:
                    raise Reject("budget")
                req = {"jsonrpc": "2.0", "method": method}
                if step != 1:
                    req["id"] = step + 1
                if step == 0:
                    req["params"] = {"protocolVersion": "2024-11-05", "capabilities": {},
                                     "clientInfo": {"name": "talos-synthetic", "version": "1"}}
                if step == 2:
                    req["params"] = {}
                if step == 3:
                    req["params"] = {"name": TOOLS[provider], "arguments": {"query": QUERY}}
                conn = http.client.HTTPConnection("127.0.0.1", server.server_port, timeout=remaining)
                try:
                    count += 1
                    conn.request("POST", "/mcp", json.dumps(req), {"Content-Type": "application/json",
                                 "Accept": "application/json, text/event-stream"})
                    with conn.getresponse() as response:
                        if 300 <= response.status < 400:
                            raise Reject("redirect")
                        if not 200 <= response.status < 300:
                            raise Reject("http-" + str(response.status))
                        body = response.read(LIMIT + 1)
                        if len(body) > LIMIT:
                            raise Reject("oversized")
                        if response.length not in (None, 0):
                            raise Reject("partial")
                        if time.monotonic() >= deadline:
                            raise Reject("deadline")
                        if step == 1:
                            if response.status != 202 or body:
                                raise Reject("notification")
                            continue
                        result = decode(body, response.getheader("Content-Type"), step + 1)
                finally:
                    conn.close()
                if step == 0 and (not isinstance(result, dict) or result.get("protocolVersion") != "2024-11-05"):
                    raise Reject("initialize")
                if step == 2:
                    tools = result.get("tools") if isinstance(result, dict) else None
                    if not isinstance(tools, list) or not all(isinstance(t, dict) for t in tools):
                        raise Reject("tool-schema")
                    matches = [t for t in tools if t.get("name") == TOOLS[provider]]
                    if len(matches) != 1 or matches[0].get("inputSchema") != SCHEMA:
                        raise Reject("tool-schema")
                if step == 3:
                    if not isinstance(result, dict) or result.get("isError") is True:
                        raise Reject("tool-error")
                    structured = result.get("structuredContent")
                    rows = structured.get("results") if isinstance(structured, dict) else None
                    if not isinstance(rows, list) or not all(
                        isinstance(x, dict) and set(x) == {"url", "title"}
                        and isinstance(x["title"], str) and isinstance(x["url"], str) for x in rows
                    ):
                        raise Reject("result-schema")
                    # Synthetic fixture allowlist only; never fetch result URLs.
                    admitted = sum(row.get("url") == "https://example.com/synthetic" for row in rows)
                    if admitted != len(rows):
                        raise Reject("invalid-url")
        except Reject as exc:
            outcome = str(exc)
        except (TimeoutError, http.client.IncompleteRead) as exc:
            outcome = "partial" if isinstance(exc, http.client.IncompleteRead) else "deadline"
        except (OSError, http.client.HTTPException):
            outcome = "transport"
        finally:
            server.shutdown()
            worker.join()
    return {"schema_version": 1, "synthetic_only": True, "region": "LOCAL-ONLY",
            "talos_commit": commit.lower(), "observed_at_utc": datetime.now(timezone.utc).isoformat(),
            "provider_fixture": provider, "case": case, "query_sha256": hashlib.sha256(QUERY.encode()).hexdigest(),
            "query_bytes": len(QUERY.encode()), "max_response_bytes": LIMIT, "max_requests": budget,
            "requests": count, "methods": seen, "outcome": outcome, "admissible_count": admitted,
            "injected_fault": case in ("proxy", "tls", "cancelled", "deadline", "budget"),
            "vendor_observation": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--provider", choices=TOOLS, required=True)
    parser.add_argument("--case", choices=CASES, default="json")
    parser.add_argument("--talos-commit", required=True)
    args = parser.parse_args()
    print(json.dumps(observe(args.provider, args.case, args.talos_commit), sort_keys=True))


if __name__ == "__main__":
    main()
