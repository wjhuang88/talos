# Accepted Dependency Baseline

Cargo manifests define build requirements and Cargo.lock defines exact resolution. The
[generated JSON snapshot](dependency-baseline.json) is the sole machine-owned accepted version
representation. The table below is derived from that snapshot, not independently maintained.
See [Dependency Upgrade](../sop/DEPENDENCY-UPGRADE.md) for refresh and advancement rules.

## Bootstrap Evidence

- Accepted source: `ce3d4cb948f0f5f1a346f87630bad282a03a1704`, an existing main ancestor.
- Acceptance timestamp: `2026-09-05T17:20:28Z` (completion of the validating CI).
- Validation: [CI 33979958221](https://github.com/wjhuang88/talos/actions/runs/33979958221), with successful Unix release preflight
  and Windows workspace check, Clippy and tests, not a reduced documentation route.
- Generated during I248 local convergence on 2026-09-06. Root Cargo files, all crate files and
  pinned toolchain have zero diff from that source to the generation tree. This is dependency-tree
  equivalence for bootstrap only, not transfer of exact-head implementation approval.
- Initial acceptance is not evidence that any dependency was upgraded. Current main may have
  other CI failures; this snapshot makes no claim about those unrelated heads.

## Generated Direct Dependency Inventory

<!-- dependency-baseline:begin -->
<!-- dependency-baseline:begin -->
| Name | Manifest | Accepted resolutions | Direct consumers | Kind | Target |
|---|---|---|---|---|---|
| anyhow | ^1.0.104 | 1.0.104 | talos-cli, talos-mcp, talos-rpc | normal | all |
| arborium | ^2.18.2 | 2.18.2 | talos-text | normal | all |
| async-trait | ^0.1.92 | 0.1.92 | talos-agent, talos-cli, talos-core, talos-evolution, talos-mcp, talos-plugin, talos-provider, talos-runtime, talos-sandbox, talos-session, talos-tools | normal | all |
| async-trait | ^0.1 | 0.1.92 | talos-rpc | normal | all |
| axum | ^0.8 | 0.8.9 | talos-dashboard | normal | all |
| base64 | ^0.23.1 | 0.23.1 | talos-provider | normal | all |
| bytes | ^1 | 1.12.1 | talos-mcp | normal | all |
| cap-std | ^4.0.3 | 4.0.3 | talos-tools | normal | all |
| chrono | ^0.4 | 0.4.45 | talos-evolution, talos-exploration, talos-memory, talos-session | normal | all |
| clap | ^4.6.6 | 4.6.6 | talos-cli | normal | all |
| crossterm | ^0.29 | 0.29.0 | talos-cli, talos-tui | normal | all |
| dirs | ^7 | 7.0.0 | talos-cli, talos-evolution | normal | all |
| futures-core | ^0.3.34 | 0.3.34 | talos-dashboard | normal | all |
| futures-util | ^0.3.34 | 0.3.34 | talos-agent, talos-mcp, talos-plugin, talos-provider | normal | all |
| futures | ^0.3.34 | 0.3.34 | talos-cli, talos-conversation, talos-rpc, talos-tui | normal | all |
| gix | ^0.87.1 | 0.87.1 | talos-tools, talos-tui | normal | all |
| glob | ^0.3.4 | 0.3.4 | talos-permission, talos-tools | normal | all |
| grep-matcher | ^0.1.9 | 0.1.9 | talos-tools | normal | all |
| grep-regex | ^0.1 | 0.1.14 | talos-tools | normal | all |
| grep-searcher | ^0.1.17 | 0.1.17 | talos-tools | normal | all |
| hex | ^0.4 | 0.4.3 | talos-exploration, talos-memory, talos-session | normal | all |
| ignore | ^0.4.33 | 0.4.33 | talos-tools | normal | all |
| image | ^0.25 | 0.25.10 | talos-provider | dev | all |
| image | ^0.25 | 0.25.10 | talos-cli, talos-tools | normal | all |
| libc | ^1.0.0-alpha.3 | 1.0.0-alpha.3 | talos-sandbox, talos-tools | normal | cfg(unix) |
| mermaid-text | ^0.57 | 0.57.0 | talos-tui | normal | all |
| mockito | ^1 | 1.7.2 | talos-provider | dev | all |
| pulldown-cmark | ^0.13.4 | 0.13.4 | talos-skill | normal | all |
| ratatui | ^0.30 | 0.30.2 | talos-tui | normal | all |
| regex | ^1.13.1 | 1.13.1 | talos-tools | normal | all |
| reqwest | ^0.13.4 | 0.13.4 | talos-cli, talos-mcp, talos-provider, talos-tools | normal | all |
| rmcp | ^3.2.0 | 3.2.0 | talos-cli | dev | all |
| rmcp | ^3.2.0 | 3.2.0 | talos-cli, talos-mcp | normal | all |
| rusqlite | ^0.40.2 | 0.40.2 | talos-evolution, talos-exploration, talos-memory, talos-models, talos-session | normal | all |
| rust-websearch | ^0.1 | 0.1.2 | talos-tools | normal | all |
| schemars | ^1.2.2 | 1.2.2 | talos-agent, talos-config, talos-core, talos-dashboard, talos-memory, talos-permission, talos-session, talos-tools | normal | all |
| scraper | ^0.27 | 0.27.0 | talos-tools | normal | all |
| serde_json | ^1.0.151 | 1.0.151 | talos-config | build | all |
| serde_json | ^1.0.151 | 1.0.151 | talos-agent, talos-cli, talos-config, talos-conversation, talos-core, talos-dashboard, talos-evolution, talos-exploration, talos-mcp, talos-memory, talos-models, talos-permission, talos-plugin, talos-provider, talos-rpc, talos-runtime, talos-session, talos-tools, talos-tui | normal | all |
| serde_json | ^1 | 1.0.151 | talos-agent, talos-plugin, talos-text | dev | all |
| serde | ^1 | 1.0.229 | talos-config | build | all |
| serde | ^1 | 1.0.229 | talos-agent, talos-cli, talos-config, talos-conversation, talos-core, talos-dashboard, talos-evolution, talos-exploration, talos-mcp, talos-memory, talos-models, talos-permission, talos-plugin, talos-provider, talos-rpc, talos-session, talos-skill, talos-text, talos-tools | normal | all |
| sha2 | ^0.11 | 0.11.0 | talos-agent, talos-cli, talos-exploration, talos-memory, talos-permission, talos-provider, talos-session, talos-tools | normal | all |
| similar | ^3.2.0 | 3.2.0 | talos-tools | normal | all |
| tempfile | ^3 | 3.27.0 | talos-agent, talos-cli, talos-conversation, talos-evolution, talos-exploration, talos-mcp, talos-memory, talos-models, talos-permission, talos-provider, talos-runtime, talos-sandbox, talos-session, talos-skill, talos-tools, talos-tui | dev | all |
| tempfile | ^3 | 3.27.0 | talos-sandbox | normal | cfg(target_os = "macos") |
| thiserror | ^2.0.20 | 2.0.20 | talos-config, talos-exploration, talos-mcp, talos-memory, talos-models, talos-permission, talos-provider, talos-runtime, talos-sandbox, talos-session, talos-skill, talos-tools | normal | all |
| thiserror | ^2 | 2.0.20 | talos-agent, talos-core, talos-dashboard, talos-evolution, talos-plugin, talos-rpc | normal | all |
| tokio-stream | ^0.1.19 | 0.1.19 | talos-conversation | normal | all |
| tokio-util | ^0.7.19 | 0.7.19 | talos-agent, talos-cli, talos-provider | normal | all |
| tokio-util | ^0.7 | 0.7.19 | talos-rpc | normal | all |
| tokio | ^1.53.1 | 1.53.1 | talos-agent, talos-core, talos-dashboard, talos-evolution, talos-runtime, talos-sandbox, talos-session, talos-tools | dev | all |
| tokio | ^1.53.1 | 1.53.1 | talos-agent, talos-cli, talos-conversation, talos-core, talos-dashboard, talos-mcp, talos-rpc, talos-runtime, talos-sandbox, talos-tools, talos-tui | normal | all |
| tokio | ^1 | 1.53.1 | talos-plugin, talos-provider | dev | all |
| tokio | ^1 | 1.53.1 | talos-plugin, talos-provider | normal | all |
| toml | ^1.1.5 | 1.1.5+spec-1.1.0 | talos-config, talos-tools | build | all |
| toml | ^1.1.5 | 1.1.5+spec-1.1.0 | talos-cli, talos-config, talos-plugin | normal | all |
| tower | ^0.5 | 0.5.3 | talos-dashboard | dev | all |
| tracing-subscriber | ^0.3 | 0.3.23 | talos-plugin | dev | all |
| tracing-subscriber | ^0.3 | 0.3.23 | talos-cli | normal | all |
| tracing | ^0.1 | 0.1.44 | talos-agent, talos-cli, talos-config, talos-evolution, talos-exploration, talos-mcp, talos-memory, talos-models, talos-plugin, talos-provider, talos-rpc | normal | all |
| tui-markdown | ^0.3.9 | 0.3.9 | talos-tui | normal | all |
| unicode-width | ^0.2 | 0.2.2 | talos-tui | normal | all |
| url | ^2 | 2.5.8 | talos-permission | normal | all |
| uuid | ^1.26.0 | 1.26.0 | talos-agent, talos-cli, talos-conversation, talos-core, talos-dashboard, talos-evolution, talos-exploration, talos-memory, talos-permission, talos-provider, talos-session, talos-tools | normal | all |
| walkdir | ^2 | 2.5.0 | talos-skill, talos-tools | normal | all |
| wasmtime | ^48.0.1 | 48.0.1 | talos-plugin | normal | all |
| windows-sys | ^0.61.2 | 0.61.2 | talos-tools | normal | cfg(windows) |
| yaml_serde | ^0.10.7 | 0.10.7 | talos-skill | normal | all |
| zstd | ^0.14 | 0.14.0 | talos-config | build | all |
| zstd | ^0.14 | 0.14.0 | talos-config, talos-session | normal | all |
<!-- dependency-baseline:end -->

## Human-Maintained Risk And Exception Notes

Registry freshness is separate from acceptance. Security and deprecation coverage are unknown
unless separately investigated; a current version is not an advisory clearance. Native/parser,
persistence, permission, runtime and rendering dependencies require their domain-specific gates.
No latest-stable exception is granted by this bootstrap. Existing prerelease resolutions must be
explicitly disposed by the upcoming upgrade audit; do not treat them as a standing policy.

An audit is read-only and cannot advance this baseline. After a governed upgrade merges and its
validation is accepted, regenerate the JSON snapshot and derive this table together, retaining
human-maintained notes. The snapshot's evidence must identify that already-merged implementation.
