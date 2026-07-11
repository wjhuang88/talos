#!/usr/bin/env bash
set -euo pipefail

tag_name="${1:-}"
workspace_version="$(sed -n '/^\[workspace.package\]/,/^\[/p' Cargo.toml | sed -n 's/^version = "\([^"]*\)"/\1/p' | head -n 1)"

if [[ -z "${workspace_version}" ]]; then
  echo "release preflight: unable to read workspace version" >&2
  exit 1
fi

if [[ -n "${tag_name}" ]]; then
  expected_version="${tag_name#v}"
  if [[ "${expected_version}" != "${tag_name}" && "${expected_version}" != "${workspace_version}" ]]; then
    echo "release preflight: tag ${tag_name} does not match workspace version ${workspace_version}" >&2
    exit 1
  fi
fi

versions="$(cargo metadata --locked --no-deps --format-version 1 | jq -r '.packages[] | select(.name | startswith("talos-")) | .version' | sort -u)"
if [[ "${versions}" != "${workspace_version}" ]]; then
  echo "release preflight: Talos package versions are not synchronized:" >&2
  printf '%s\n' "${versions}" >&2
  exit 1
fi

echo "release preflight: Talos version ${workspace_version}"
cargo fmt --all -- --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
echo "release preflight: passed"
