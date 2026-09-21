#!/usr/bin/env python3
"""Check and run the independently resolved single-direct-Talos-dependency consumer."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import tomllib


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check-only", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    fixture = root / "tests/fixtures/runtime-sdk-external"
    manifest = fixture / "Cargo.toml"
    data = tomllib.loads(manifest.read_text())
    if data.get("workspace") != {}:
        raise ValueError("fixture must declare an independent, empty workspace")
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--locked", "--no-deps", "--format-version", "1",
        "--manifest-path", str(manifest),
    ], cwd=fixture))
    if Path(metadata["workspace_root"]).resolve() != fixture:
        raise ValueError("fixture resolved through parent workspace")
    package = next(p for p in metadata["packages"] if Path(p["manifest_path"]) == manifest)
    talos = [d for d in package["dependencies"] if d["name"].startswith("talos-")]
    if len(talos) != 1 or talos[0]["name"] != "talos-runtime" or talos[0]["kind"] is not None:
        raise ValueError("exactly one normal direct Talos dependency is required: talos-runtime")
    for source in (fixture / "src").rglob("*.rs"):
        imports = set(re.findall(r"\btalos_\w+\s*::", source.read_text()))
        if imports - {"talos_runtime::"}:
            raise ValueError(f"internal Talos import in {source}: {imports}")
    print("runtime SDK fixture boundary: one direct Talos dependency, independent Cargo root", flush=True)
    if not args.check_only:
        for features in ([], ["--features", "coding"]):
            subprocess.run([
                "cargo", "run", "--locked", "--manifest-path", str(manifest),
                "--target-dir", str(root / "target"), *features,
            ], cwd=fixture, check=True)


if __name__ == "__main__":
    main()
