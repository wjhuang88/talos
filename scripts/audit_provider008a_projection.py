#!/usr/bin/env python3
"""Offline, exhaustive audit of I306's frozen protocol-only catalog projection.

Python 3.11+ standard library only. This maintenance audit neither fetches nor changes data.
The independent admission calculation intentionally checks every packaged model against the
archived input; runtime and normal catalog generation use Rust protocol_signal.rs/build.rs.
"""

import collections
import gzip
import hashlib
import json
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
DEFAULT_ROOT = {
    "@ai-sdk/openai": "https://api.openai.com/v1",
    "@ai-sdk/anthropic": "https://api.anthropic.com/v1",
}


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
    if override.get("api") is not None:
        parent_root = host.get("api") or DEFAULT_ROOT.get(host_sdk)
        if parent_root is None or normalized(override["api"]) != normalized(parent_root):
            return None, "distinct_model_endpoint"
    shape = override.get("shape")
    protocol = SDK[model_sdk] if shape is None else {
        "responses": "openai-responses",
        "completions": "openai-chat",
    }.get(shape.strip())
    return protocol, protocol or "unsupported_shape"


def main():
    raw = gzip.decompress(SOURCE.read_bytes())
    digest = hashlib.sha256(raw).hexdigest()
    if digest != SOURCE_SHA256:
        raise SystemExit(f"source hash mismatch: {digest}")
    source = json.loads(raw)
    catalog = tomllib.loads((ROOT / "crates/talos-config/src/models.toml").read_text(encoding="utf-8"))
    providers = {provider["id"]: provider for provider in catalog["providers"]}
    counts = collections.Counter()
    mismatches = []
    for model in catalog["models"]:
        protocol, reason = expected(model, providers, source)
        counts[reason] += 1
        if model.get("protocol") != protocol:
            mismatches.append((model["provider"], model["id"], model.get("protocol"), protocol))
    # Preserve the entire original package, including provider records and all model metadata.
    for model in catalog["models"]:
        model.pop("protocol", None)
    semantic = hashlib.sha256(json.dumps(catalog, sort_keys=True,
                                         separators=(",", ":")).encode()).hexdigest()
    if semantic != SEMANTIC_SHA256:
        raise SystemExit(f"non-protocol catalog changed: {semantic}")
    print(json.dumps({"source_sha256": digest, "source_bytes": len(raw),
                      "source_providers": len(source), "models": len(catalog["models"]),
                      "non_protocol_sha256": semantic,
                      "counts": dict(sorted(counts.items())), "mismatches": mismatches}, indent=2))
    if mismatches:
        raise SystemExit("catalog protocol projection differs from frozen source")


if __name__ == "__main__":
    main()
