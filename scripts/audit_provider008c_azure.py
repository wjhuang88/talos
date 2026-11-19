#!/usr/bin/env python3
"""Offline source integrity and runtime allowlist audit for I323."""
import hashlib
import json
from pathlib import Path
import re
import tomllib

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "scripts/fixtures/provider008c-azure-20261115/audit.json"

def main():
    raw = FIXTURE.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == "9220ffddda1a970bab16eb3b546fc73cffc65d572d3083f0850abc6239b8fb80"
    data = json.loads(raw)
    assert data["revision"] == "888c250133b38839c2018d66fe687cb03c03a8f2"
    translator = FIXTURE.with_name("translate.ts.txt").read_bytes()
    assert hashlib.sha256(translator).hexdigest() == "c53a1b314008e47b098c5eb800c5b99505036a9bd5571d64730deea96dce5b7e"
    for provider in data["providers"].values():
        assert hashlib.sha256(provider["source"].encode()).hexdigest() == provider["sha256"]
    confirmed = []
    ids = set()
    for row in data["models"]:
        assert row["model"] not in ids
        ids.add(row["model"])
        assert hashlib.sha256(row["source"].encode()).hexdigest() == row["sha256"]
        source = tomllib.loads(row["source"])
        assert source.get("api", {}) == row["api"]
        if "responses" in row["api"]:
            assert next(iter(source["api"])) == "responses"
            assert not source["api"]["responses"].get("base_url")
            assert not source["api"]["responses"].get("sdk")
            confirmed.append(row["model"])
    assert len(ids) == 99 and len(confirmed) == 27
    runtime = (ROOT / "crates/talos-config/src/azure_catalog.rs").read_text()
    block = runtime.split("const RESPONSES_MODELS: &[&str] = &[", 1)[1].split("];", 1)[0]
    assert re.findall(r'"([^"\n]+)"', block) == confirmed
    packaged = tomllib.loads((ROOT / "crates/talos-config/src/models.toml").read_text())
    admitted = [r for r in packaged["models"] if r["provider"] == "azure" and r["id"] in confirmed and "protocol" not in r and "api_base_url" not in r]
    assert len(admitted) == 22
    print("Azure audit passed: 99 source models, 27 confirmed Responses defaults, 22 packaged exact pairs")

if __name__ == "__main__":
    main()
