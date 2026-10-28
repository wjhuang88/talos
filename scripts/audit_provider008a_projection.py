#!/usr/bin/env python3
"""Offline, exhaustive audit of I306's frozen protocol-and-endpoint catalog projection.

Python 3.11+ standard library only. This maintenance audit neither fetches nor changes data.
The independent admission calculation intentionally checks every packaged model against the
archived input; runtime and normal catalog generation use Rust protocol_signal.rs/build.rs.
"""

import collections
import gzip
import hashlib
import json
import ipaddress
import re
from urllib.parse import urlsplit
from pathlib import Path
import tomllib


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "scripts/fixtures/provider008a-models-20261027/api.json.gz"
SOURCE_SHA256 = "4e97268dc7621205179df996f68a2c17a32284997d291cb8c6a29c43ffc33c74"
SEMANTIC_SHA256 = "acc90a95ff8f87fbf131788548066575d02cd8e1ccb8f3545a9c62e01712947d"
SDK = {
    "@ai-sdk/openai": "openai-responses",
    "@ai-sdk/openai-compatible": "openai-chat",
    "@ai-sdk/anthropic": "anthropic-messages",
}

# Credential-destination admission is frozen alongside the source. These exact tuples require
# primary vendor evidence; unsupported/unverified vendor wrappers are not inferred from SDK names.
REVIEWED_ENDPOINTS = {
    ("neon", "${NEON_AI_GATEWAY_BASE_URL}/v1", "${NEON_AI_GATEWAY_BASE_URL}/openai/v1"): "openai-responses",
    ("ofox", "https://api.ofox.ai/v1", "https://api.ofox.ai/anthropic/v1"): "anthropic-messages",
    ("zenmux", "https://zenmux.ai/api/v1", "https://zenmux.ai/api/anthropic/v1"): "anthropic-messages",
    ("freemodel", "https://cc.freemodel.dev/v1", "https://api.freemodel.dev/v1"): "openai-chat",
}


def http_endpoint(value):
    value = (value or "").strip()
    template = re.fullmatch(r"\$\{[A-Za-z0-9_]+\}(/[^$]*)?", value)
    if template:
        value = "https://catalog-template.invalid" + (template.group(1) or "")
    if any(c.isspace() or ord(c) < 32 for c in value):
        return False
    try:
        parsed = urlsplit(value)
        parsed.port  # reject malformed ports, independently of the Rust URL parser
        if not parsed.hostname or parsed.username is not None or parsed.password is not None or parsed.query or parsed.fragment:
            return False
        if parsed.scheme == "https":
            return True
        if parsed.scheme != "http":
            return False
        return parsed.hostname == "localhost" or ipaddress.ip_address(parsed.hostname).is_loopback
    except ValueError:
        return False


def normalized(value):
    return (value or "").strip().rstrip("/")


def expected(model, packaged_providers, source):
    host = source.get(model["provider"])
    upstream = host and host.get("models", {}).get(model["id"])
    if upstream is None:
        return None, "absent"
    override = upstream.get("provider") or {}
    host_sdk = host.get("npm")
    model_sdk = override.get("npm")
    if model_sdk is None:
        model_sdk = host_sdk
    host_sdk = host_sdk.strip() if host_sdk is not None else None
    model_sdk = model_sdk.strip() if model_sdk is not None else None
    if host_sdk not in SDK or model_sdk not in SDK:
        return None, "unsupported_transport"
    # Root drift is counted before model-endpoint drift; both deny admission.
    packaged_root = packaged_providers[model["provider"]].get("api_base_url")
    if normalized(host.get("api")) != normalized(packaged_root):
        return None, "provider_root_drift"
    shape = override.get("shape")
    protocol = SDK[model_sdk] if shape is None else {
        "responses": "openai-responses",
        "completions": "openai-chat",
    }.get(shape.strip())
    if protocol is None:
        return None, "unsupported_shape"
    if override.get("api") is not None:
        parent_root = host.get("api")
        if not http_endpoint(override["api"]) or not http_endpoint(parent_root):
            return None, "unsupported_transport"
        if normalized(override["api"]) != normalized(parent_root):
            reviewed = REVIEWED_ENDPOINTS.get((model["provider"], normalized(parent_root), normalized(override["api"])))
            if reviewed != protocol:
                return None, "unreviewed_model_endpoint"
    return protocol, protocol


def expected_endpoint(model, source, protocol):
    if protocol is None:
        return None
    host = source[model["provider"]]
    override = host["models"][model["id"]].get("provider") or {}
    api = override.get("api")
    return normalized(api) if api is not None and normalized(api) != normalized(host.get("api")) else None


def main():
    raw = gzip.decompress(SOURCE.read_bytes())
    digest = hashlib.sha256(raw).hexdigest()
    if digest != SOURCE_SHA256:
        raise SystemExit(f"source hash mismatch: {digest}")
    source = json.loads(raw)
    catalog = tomllib.loads((ROOT / "crates/talos-config/src/models.toml").read_text(encoding="utf-8"))
    providers = {provider["id"]: provider for provider in catalog["providers"]}
    endpoint_count = sum(model.get("api_base_url") is not None for model in catalog["models"])
    counts = collections.Counter()
    mismatches = []
    for model in catalog["models"]:
        protocol, reason = expected(model, providers, source)
        counts[reason] += 1
        endpoint = expected_endpoint(model, source, protocol)
        if model.get("protocol") != protocol or model.get("api_base_url") != endpoint:
            mismatches.append((model["provider"], model["id"], model.get("protocol"), protocol, model.get("api_base_url"), endpoint))
    # Preserve the entire original package, including provider records and all model metadata.
    for model in catalog["models"]:
        model.pop("protocol", None)
        model.pop("api_base_url", None)
    semantic = hashlib.sha256(json.dumps(catalog, sort_keys=True,
                                         separators=(",", ":")).encode()).hexdigest()
    if semantic != SEMANTIC_SHA256:
        raise SystemExit(f"non-protocol catalog changed: {semantic}")
    print(json.dumps({"source_sha256": digest, "source_bytes": len(raw),
                      "source_providers": len(source), "models": len(catalog["models"]),
                      "non_routing_sha256": semantic,
                      "endpoint_records": endpoint_count, "counts": dict(sorted(counts.items())), "mismatches": mismatches}, indent=2))
    if mismatches:
        raise SystemExit("catalog protocol projection differs from frozen source")


if __name__ == "__main__":
    main()
