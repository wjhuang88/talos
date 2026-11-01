#!/usr/bin/env python3
"""Manual SEARCH-001-D live observation harness.

Research-only helper. It performs exactly one direct HTTPS GET against one fixed
provider endpoint and emits bounded JSON metadata without query text, raw body,
result titles/snippets, or result URLs.
"""

from __future__ import annotations

import argparse
import hashlib
import http.client
import json
import re
import socket
import ssl
import sys
from collections import Counter
from datetime import datetime, timezone
from typing import Any
from urllib.parse import urlencode, urlsplit

SCHEMA_VERSION = 1
MAX_QUERY_BYTES = 512
MAX_BODY_BYTES = 1_048_576
DEFAULT_TIMEOUT_SECONDS = 15.0
MAX_TIMEOUT_SECONDS = 30.0

PROVIDERS = {
    "mwmbl": {
        "host": "api.mwmbl.org",
        "path": "/api/v2/search/",
        "source_field": "engine",
        "known_sources": {"mwmbl", "wikipedia", "google", "user", "eusp"},
    },
    "purili": {
        "host": "puri.li",
        "path": "/api/search",
        "source_field": "source",
        "known_sources": {"crawled"},
    },
}

SHA40_RE = re.compile(r"^[0-9a-fA-F]{40}$")
PAIR_ID_RE = re.compile(r"^[A-Za-z0-9._-]{1,64}$")
SAFE_LABEL_RE = re.compile(r"^[A-Za-z0-9 ._()/-]{1,64}$")
MEDIA_TYPE_RE = re.compile(r"^[a-z0-9!#$&^_.+-]+/[a-z0-9!#$&^_.+-]+$")


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")


def query_digest(query: bytes) -> dict[str, Any]:
    return {"sha256": hashlib.sha256(query).hexdigest(), "bytes": len(query)}


def validate_metadata(args: argparse.Namespace) -> None:
    if not SHA40_RE.fullmatch(args.talos_commit):
        raise ValueError("--talos-commit must be a full 40-hex commit SHA")
    if not PAIR_ID_RE.fullmatch(args.pair_id):
        raise ValueError("--pair-id must match [A-Za-z0-9._-]{1,64}")
    for name, value in (("region", args.region), ("network-class", args.network_class)):
        if not SAFE_LABEL_RE.fullmatch(value):
            raise ValueError(
                f"--{name} contains unsupported characters or exceeds 64 characters"
            )
    if not (0 < args.timeout_seconds <= MAX_TIMEOUT_SECONDS):
        raise ValueError(
            f"--timeout-seconds must be > 0 and <= {MAX_TIMEOUT_SECONDS:g}"
        )


def read_query() -> bytes:
    if sys.stdin.isatty():
        raise ValueError("query must be piped through non-interactive stdin")
    raw = sys.stdin.buffer.read(MAX_QUERY_BYTES + 2)
    query = raw.rstrip(b"\r\n")
    if not query:
        raise ValueError("query must not be empty")
    if len(query) > MAX_QUERY_BYTES:
        raise ValueError(f"query exceeds {MAX_QUERY_BYTES} UTF-8 bytes")
    try:
        query.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise ValueError("query must be valid UTF-8") from exc
    return query


def admissible_http_url(value: Any) -> bool:
    if not isinstance(value, str):
        return False
    try:
        parsed = urlsplit(value)
    except ValueError:
        return False
    return (
        parsed.scheme in {"http", "https"}
        and bool(parsed.hostname)
        and parsed.username is None
        and parsed.password is None
    )


def bounded_sources(results: list[Any], source_field: str, known: set[str]) -> dict[str, int]:
    counts: Counter[str] = Counter()
    for item in results:
        if not isinstance(item, dict):
            counts["invalid-result"] += 1
            continue
        source = item.get(source_field)
        if isinstance(source, str) and source in known:
            counts[source] += 1
        elif source is None:
            counts["missing"] += 1
        else:
            counts["other"] += 1
    return dict(sorted(counts.items()))


def summarize_json(provider: str, body: bytes) -> dict[str, Any]:
    try:
        payload = json.loads(body)
    except (UnicodeDecodeError, json.JSONDecodeError):
        return parse_summary("malformed-json")

    if not isinstance(payload, dict):
        return parse_summary("schema-mismatch")

    results = payload.get("results")
    if not isinstance(results, list):
        if isinstance(payload.get("error"), str):
            return parse_summary("error-json")
        return parse_summary("schema-mismatch")

    provider_cfg = PROVIDERS[provider]
    admissible = sum(
        1
        for item in results
        if isinstance(item, dict) and admissible_http_url(item.get("url"))
    )
    return {
        "parse_class": "empty" if not results else "valid-json",
        "wire_result_count": len(results),
        "admissible_http_url_count": admissible,
        "provenance_counts": bounded_sources(
            results,
            provider_cfg["source_field"],
            provider_cfg["known_sources"],
        ),
    }


def parse_summary(kind: str) -> dict[str, Any]:
    return {
        "parse_class": kind,
        "wire_result_count": None,
        "admissible_http_url_count": None,
        "provenance_counts": {},
    }


def bounded_content_type(value: str | None) -> str | None:
    if value is None:
        return None
    media_type = value.split(";", 1)[0].strip().lower()
    return media_type if MEDIA_TYPE_RE.fullmatch(media_type) else "other"


def perform_direct_observation(
    provider: str,
    query: bytes,
    timeout_seconds: float,
) -> dict[str, Any]:
    cfg = PROVIDERS[provider]
    query_text = query.decode("utf-8")
    path = f'{cfg["path"]}?{urlencode({"q": query_text})}'
    response: http.client.HTTPResponse | None = None
    conn: http.client.HTTPSConnection | None = None

    try:
        # http.client does not consult HTTP(S)_PROXY/ALL_PROXY environment
        # variables and does not follow redirects. The default SSL context keeps
        # certificate and hostname verification enabled.
        conn = http.client.HTTPSConnection(
            cfg["host"],
            port=443,
            timeout=timeout_seconds,
            context=ssl.create_default_context(),
        )
        conn.request(
            "GET",
            path,
            headers={
                "Accept": "application/json",
                "User-Agent": "talos-search-provider-observation/1",
            },
        )
        response = conn.getresponse()
        body = response.read(MAX_BODY_BYTES + 1)
        if len(body) > MAX_BODY_BYTES:
            return {
                "network_classification": "body-too-large",
                "http_status": response.status,
                "content_type": bounded_content_type(response.getheader("Content-Type")),
                "body_complete": False,
                "body_bytes": len(body),
                "body_sha256": None,
                **parse_summary("not-parsed"),
            }

        return {
            "network_classification": "http-response",
            "http_status": response.status,
            "content_type": bounded_content_type(response.getheader("Content-Type")),
            "body_complete": True,
            "body_bytes": len(body),
            "body_sha256": hashlib.sha256(body).hexdigest(),
            **summarize_json(provider, body),
        }
    except socket.gaierror:
        return failure_summary("dns")
    except ssl.SSLError:
        return failure_summary("tls")
    except (TimeoutError, socket.timeout):
        return failure_summary("timeout")
    except (ConnectionError, OSError, http.client.HTTPException):
        return failure_summary("network")
    finally:
        if response is not None:
            response.close()
        if conn is not None:
            conn.close()


def failure_summary(kind: str) -> dict[str, Any]:
    return {
        "network_classification": kind,
        "http_status": None,
        "content_type": None,
        "body_complete": False,
        "body_bytes": 0,
        "body_sha256": None,
        **parse_summary("not-received"),
    }


def build_evidence(
    args: argparse.Namespace,
    query: bytes,
    observation: dict[str, Any],
) -> dict[str, Any]:
    cfg = PROVIDERS[args.provider]
    return {
        "schema_version": SCHEMA_VERSION,
        "research_only": True,
        "talos_commit": args.talos_commit.lower(),
        "observed_at_utc": utc_now(),
        "pair_id": args.pair_id,
        "operator_context": {
            "coarse_region": args.region,
            "network_class": args.network_class,
            "proxy_mode": "direct",
            "geography_certified_by_harness": False,
        },
        "provider": args.provider,
        "endpoint": {
            "scheme": "https",
            "host": cfg["host"],
            "path": cfg["path"],
        },
        "query": query_digest(query),
        "limits": {
            "timeout_seconds": args.timeout_seconds,
            "max_query_bytes": MAX_QUERY_BYTES,
            "max_body_bytes": MAX_BODY_BYTES,
            "semantic_retries": 0,
            "redirect_following": False,
        },
        "observation": observation,
    }


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Collect one bounded SEARCH-001-D provider observation. "
            "Pipe the query through stdin; query text is never emitted."
        )
    )
    parser.add_argument("--provider", choices=sorted(PROVIDERS), required=True)
    parser.add_argument("--talos-commit", required=True)
    parser.add_argument("--region", required=True)
    parser.add_argument(
        "--network-class",
        choices=("residential", "mobile", "cloud", "unknown"),
        required=True,
    )
    parser.add_argument("--pair-id", required=True)
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=DEFAULT_TIMEOUT_SECONDS,
        help=f"direct HTTPS timeout; max {MAX_TIMEOUT_SECONDS:g}s",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    try:
        args = parse_args(argv)
        validate_metadata(args)
        query = read_query()
        observation = perform_direct_observation(
            args.provider,
            query,
            args.timeout_seconds,
        )
        evidence = build_evidence(args, query, observation)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    json.dump(evidence, sys.stdout, ensure_ascii=False, sort_keys=True, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
