# Talos Crate Publication Matrix

Created: 2026-06-29. Last reconciled: 2026-07-26 against the current workspace.

This matrix tracks crates.io publish readiness for Talos workspace crates. It is a readiness and
release-gate artifact, NOT authorization to publish. Real `cargo publish` remains blocked until the
maintainer explicitly approves.

## Versioning Concepts (keep these distinct)

| Concept | Meaning | Current value / state |
|---|---|---|
| Current workspace version | The `version.workspace = true` value in root `Cargo.toml`; source of truth for membership and version | **0.5.0** (root `Cargo.toml` `[workspace.package]`) |
| Latest published registry version | Highest version of the crate actually on crates.io | Most published crates are at **0.2.0** only |
| Compatible published version exists? | Is there a registry version that satisfies the workspace's `version = "0.5.0"` path-dep requirement? | **No** — no crate has a published 0.5.0 |
| Support classification | What the crate IS (facade / implementation / product-only / …) | Per-row below |
| Registry / readiness state | Publish-readiness posture today | Per-row below |

Pre-1.0 lockstep is still in effect: all workspace crates share one version (currently 0.5.0). The
historical 0.2.0 publishes (see Historical Evidence) are NOT compatible with the current 0.5.0
workspace; a 0.5.0 publication wave requires re-publishing the whole closure.

## Policy

- The current workspace version is read from root `Cargo.toml`; do not hardcode it here.
- Internal dependencies carry both `version = "<workspace>"` and `path = "../..."` so Cargo can
  rewrite them on publish.
- `talos-runtime` is the supported SDK facade; `talos-agent` is an implementation dependency, not a
  supported SDK (ADR-024, ADR-052).
- Product assembly and quarantined crates are not required dependencies for embedders.
- `cargo install` support is a binary distribution surface, not a library API promise. Because the
  top-level `talos` package name is unavailable, the planned shape is
  `cargo install talos-cli --bin talos` unless a later decision chooses another package name.
- Heavy/default-weight capability crates need feature-gate review before broad publication.
- Name reservation means real crates.io publication; do not reserve by publishing placeholder
  packages without maintainer approval.

## Publication Matrix (all 21 workspace members)

Classifications: **Supported SDK facade** · **Reusable pre-1.0 crate** · **Experimental,
product-oriented** · **Implementation dependency; unsupported as SDK** · **Product-only** ·
**Quarantined** · **Binary distribution surface**.

| Crate | Support classification | Registry / readiness state | Latest published | publish= false? |
|---|---|---|---|---|
| `talos-core` | Reusable pre-1.0 crate | Published (0.2.0); 0.5.0 not yet published | 0.2.0 | no |
| `talos-config` | Reusable pre-1.0 crate | Published (0.2.0); 0.5.0 not yet published | 0.2.0 | no |
| `talos-permission` | Reusable pre-1.0 crate | Published (0.2.0); 0.5.0 not yet published | 0.2.0 | no |
| `talos-skill` | Reusable pre-1.0 crate | Published (0.2.0); 0.5.0 not yet published | 0.2.0 | no |
| `talos-session` | Reusable pre-1.0 crate | Published (0.2.0); 0.5.0 not yet published; crate description still says "JSONL-based" (stale — now TLOG, ADR-037) | 0.2.0 | no |
| `talos-plugin` | Reusable pre-1.0 crate | Published (0.2.0); 0.5.0 not yet published | 0.2.0 | no |
| `talos-memory` | Reusable pre-1.0 crate | Published (0.2.0); 0.5.0 not yet published | 0.2.0 | no |
| `talos-exploration` | Reusable pre-1.0 crate | Published (0.2.0); 0.5.0 not yet published | 0.2.0 | no |
| `talos-provider` | Reusable pre-1.0 crate | Published (0.2.0); 0.5.0 not yet published | 0.2.0 | no |
| `talos-rpc` | Reusable pre-1.0 crate (local stdio transport only) | Published (0.2.0); 0.5.0 not yet published | 0.2.0 | no |
| `talos-conversation` | Experimental, product-oriented (published but NOT a general-purpose UI SDK) | Published (0.2.0); 0.5.0 not yet published | 0.2.0 | no |
| `talos-sandbox` | Reusable pre-1.0 crate (platform-sensitive) | Gate-before-publish; manifest-ready; platform behavior + escape-vector review pending | — | no |
| `talos-tools` | Reusable pre-1.0 crate (built-in tools) | Gate-before-publish; manifest-ready; **NO feature gates implemented** — heavy deps (`gix`, `arborium`, `reqwest`, `scraper`, `rust-websearch`, `image`) are hard `[dependencies]`; ADR-052 lightweight-default target is unimplemented | — | no |
| `talos-agent` | Implementation dependency; unsupported as SDK | Gate-before-publish; manifest-ready; turn-loop implementation; not the SDK entrypoint (ADR-024/052) | — | no |
| `talos-runtime` | Supported SDK facade | Gate-before-publish; manifest-ready; SDK contract in `RUNTIME-SDK-CONTRACT.md`; blocked by unpublished 0.5.0 closure | — | no |
| `talos-mcp` | Reusable pre-1.0 crate (protocol-sensitive) | Gate-before-publish; manifest-ready; MCP support boundary ADR pending | — | no |
| `talos-cli` | Binary distribution surface | Cargo-install candidate; binary-only; library API unsupported | — | **yes** |
| `talos-tui` | Product-only | Not a reusable UI library; stays product-only unless a later decision changes this | — | **yes** |
| `talos-evolution` | Product-only | Product-specific learning; no external reusable API proven | — | **yes** |
| `talos-dashboard` | Product-only | Loopback control surface (ADR-031); no crates.io publication planned | — | **yes** |
| `talos-models` | Quarantined | Non-runtime SQLite catalog store (historical/foundation only); runtime uses packaged `models.toml`; must NOT be wired into CLI/TUI/runtime; no publication planned | — | **yes** (added 2026-07-26) |

The `publish = false` guard is enforced by `scripts/check_publish_guard.sh`.

## Remaining Gate Crates — Two Orderings

ADR-052 selected **route A**: harden and publish the required dependency closure in dependency
order, rather than redesigning the runtime first.

### Logical gate order (remaining gate crates only)

```text
talos-sandbox → talos-tools → talos-agent → talos-runtime
```

This is the logical gate ordering of the four STILL-GATED crates whose gates must clear in this
sequence. It is NOT a complete, directly-executable release command sequence.

### Actual version-aligned release order

The full `talos-runtime` publication closure (from `cargo metadata`) is wider than the four gate
crates — it also includes already-published foundation crates that must be RE-published at 0.5.0:

```text
talos-core → talos-permission → talos-skill → talos-plugin
          → talos-sandbox → talos-tools → talos-memory → talos-session
          → talos-agent → talos-runtime
```

The actual release order MUST be computed from the current Cargo dependency graph and the crates.io
compatible-version availability. A 0.5.0 publish wave cannot resolve until every closure crate has a
published 0.5.0. The four-item list above is a gate-ordering shorthand, not a publish script.
Decoupling (route B) remains only a reversal path if route A produces unacceptable weight/reach.

## Publication Gates (why each gate crate is not published yet)

| Crate | Required gate before publish |
|---|---|
| `talos-sandbox` | Security review against escape vectors; platform behavior docs; ADR-007/ADR-008/ADR-020 dependency boundary check; targeted sandbox tests. Per ADR-052: route-A step; stays policy-neutral (typed availability/errors); fallback policy owned by the SDK caller. |
| `talos-tools` | Feature-gate plan for heavy/default tools (NOT yet implemented); permission profile audit; network/write/execute tool boundary docs; dry-run after `talos-sandbox`. Per ADR-052: lightweight read-only defaults target (unimplemented). |
| `talos-agent` | Publish only after sandbox/tools dependency gates clear. Per ADR-052: published as an **implementation dependency only**, not a supported SDK; crate docs must carry direct-use caveats. |
| `talos-runtime` | Publish after the full 0.5.0 dependency closure resolves. Per ADR-052: adds caller-selected `SandboxFallbackPolicy` (planned) and an explicit `RuntimePreset::coding()` (planned). |
| `talos-mcp` | MCP support boundary ADR or equivalent; server opt-in/conflict policy; transport/auth non-goals; dry-run after `talos-tools`. |
| `talos-models` | Quarantined — intentionally `publish = false`; no publication planned unless a new decision reactivates runtime catalog DB usage (superseded by MC-002). |

Product-only crates (`talos-cli`, `talos-tui`, `talos-evolution`, `talos-dashboard`) stay
`publish = false` unless a future story changes their distribution model. `talos-cli` is a binary
package candidate whose guard removal requires a dedicated install-package gate.

## Name Availability Snapshot

Checked with `cargo search <name> --limit 3` on 2026-06-29 (historical); re-checked 2026-07-26 for
the gate crates. "Available" means the exact name had no exact match at the time of checking.

| Crate name | Search result (2026-07-26) | Notes |
|---|---|---|
| `talos` | Taken | Existing unrelated `talos` crate; top-level package name unavailable. |
| `talos-core` | Published 0.2.0 | Published by Talos; no 0.5.0 yet. |
| `talos-permission` | No exact match in `cargo search` | Historical evidence records a real 0.2.0 publish (see below); search index may lag. |
| `talos-sandbox` | No exact match | Not published. |
| `talos-tools` | No exact match | Not published. |
| `talos-agent` | No exact match | Not published. |
| `talos-runtime` | No exact match | Not published. |
| Other `talos-*` | Various | See Historical Evidence for the published set. |

Note: `cargo search` is not fully authoritative and may not return all published crates. The
committed historical publish evidence (specific commit SHAs) is the source of truth for what was
actually published.

## Historical Evidence (0.2.0 baseline — NOT current policy)

The 0.2.0 publishes below are historical fact. They are NOT compatible with the current 0.5.0
workspace and do not authorize a 0.5.0 publish. Each historical failure note states WHY it failed at
the time (name absent / no compatible version / `publish = false` / closure gap).

### 2026-06-29 — first/second/integration waves (0.2.0)

- `cargo package --allow-dirty --list -p talos-core` / `-p talos-skill` succeeded.
- `cargo publish --dry-run --allow-dirty -p talos-core` / `-p talos-skill` succeeded.
- `cargo publish --dry-run --allow-dirty -p talos-config` / `-p talos-permission` / `-p talos-session`
  failed because `talos-core` was not yet in the crates.io index (closure gap — name existed in the
  registry only after the first real publish).
- Real `cargo publish -p talos-core` attempted from clean commit `30c9abc`; rejected because the
  publisher account lacked a verified email. No crate published, no name reserved.
- After email verification, real publishes from clean commit `c8884f6` succeeded:
  `talos-core 0.2.0`, `talos-skill 0.2.0`, `talos-config 0.2.0`, `talos-permission 0.2.0`,
  `talos-session 0.2.0`.
- `cargo search talos-core --limit 5` confirmed `talos-core = "0.2.0"` visible before publishing
  the core-dependent crates.
- Second-wave dry-runs succeeded for `talos-plugin`, `talos-provider`, `talos-conversation`,
  `talos-memory`, `talos-exploration`.
- Real publishes succeeded: `talos-plugin 0.2.0`, `talos-memory 0.2.0`. `talos-exploration` hit a
  new-crate rate limit, retried successfully → `talos-exploration 0.2.0`.
- Added crate-level support-boundary docs for `talos-provider`, `talos-conversation`, `talos-rpc`
  (commit `92a0c99`); `cargo test -p talos-provider -p talos-conversation -p talos-rpc` passed;
  dry-runs passed; real publishes → `talos-provider 0.2.0`, `talos-conversation 0.2.0`,
  `talos-rpc 0.2.0`.
- Classified remaining crates: `talos-sandbox`, `talos-tools`, `talos-agent`, `talos-runtime`,
  `talos-mcp` = gate-before-publish; `talos-cli`, `talos-tui`, `talos-evolution` = product-only
  (`publish = false`).

### 2026-06-30 — binary install path

- `talos-cli` classified as a binary package candidate for `cargo install talos-cli --bin talos`;
  library API unsupported. Removing `publish = false` requires a dedicated install-package gate.
- Local install smoke (`cargo install --path crates/talos-cli --bin talos`) succeeded; `talos --version`
  printed the workspace version.
- `cargo publish --dry-run --allow-dirty -p talos-cli` failed because `publish = false` blocks even
  packaging (intentional).

### 2026-07-02 — T133 publish gate packet

- T133 produced `docs/reference/PUBLISH-GATE-PACKET-2026-07-02.md`.
- `scripts/check_publish_guard.sh .` passes; `talos-dashboard` added to the guard.
- `cargo publish --dry-run -p talos-runtime` blocked by the unpublished `talos-agent` dependency
  (closure gap — `talos-agent` was and is not on crates.io).
- `cargo publish --dry-run -p talos-cli` intentionally blocked by `publish = false`.
- No crate published, no guard removed, no tag created.

### Historical gate analysis (2026-06-29, A3–A7) — retained for reference

- **A3 `talos-sandbox`**: deps `talos-core` (then-unpublished), `libc 1.0.0-alpha.3` (pre-release,
  ADR-007), `tokio`. Escape-vector checklist must be verified before publish; `libc` pre-release is a
  stability risk.
- **A4 `talos-tools`** (heaviest crate): then-current suggestion
  `default = ["file", "search", "git", "code-intelligence", "network"]` is **SUPERSEDED** by
  ADR-052's read-only default (`file-read + search`) with opt-in write/shell/git/network/image/
  code-intelligence. That target is NOT yet implemented. Deps included `gix`, `arborium` (25+ langs),
  `reqwest`, `scraper`. Permission profiles verified TOOL-013 compliant.
- **A5 `talos-agent`/`talos-runtime`**: decided (ADR-052) to publish via route A in dependency order;
  `talos-agent` is an implementation surface only (embedders use `talos-runtime`).
  - T48 dry-run (2026-07-01): `cargo publish --dry-run -p talos-runtime` failed with
    `no matching package named talos-agent found` (closure gap — `talos-agent` not on crates.io).
    The historical note listing `talos-core`/`talos-permission` as "not on crates.io" was true ONLY
    at that instant before the first wave; they ARE now published at 0.2.0. The current blocker for a
    0.5.0 publish is that no compatible 0.5.0 versions exist in the registry, not name absence.
- **A6 `talos-mcp`**: local stdio transport only; `rmcp 1.7.0` stability must be evaluated; no
  `~/.agents/mcp.json` import (gated under AGENT-002-C).
- **A7 `talos-cli` Cargo install gate**: see the install-package gate checklist in the proposal and
  PUBLISH-GATE-PACKET-2026-07-02. Three blockers (all intentional): `publish = false`; path-dep
  closure (5 gate crates remain unpublished at 0.5.0: sandbox/tools/agent/runtime/mcp); transitive
  `publish = false` on `talos-tui` and `talos-evolution` (product-only direct deps of `talos-cli`).
  `talos-cli` itself has 17 internal workspace dependencies.

## Published-Crate Docs Audit (A1, 2026-06-29)

The then-published crates had `description` and workspace-inherited `repository`/`homepage`. None
had `keywords`, `categories`, `readme`, or `documentation`. Crate-level `//!` docs existed for
`talos-permission` (comprehensive), `talos-provider`/`talos-conversation`/`talos-rpc` (support
boundary), `talos-core`/`talos-plugin` (minimal). Missing for `talos-config`, `talos-skill`,
`talos-session`, `talos-memory`, `talos-exploration`. `keywords`/`categories` are explicitly
non-mandatory per ARCH-031 acceptance.

## Product-Only Guard (A2)

`scripts/check_publish_guard.sh` verifies `publish = false` on product-only/quarantined crates and
its absence on gate crates. As of 2026-07-26 the guard covers: product-only/quarantined =
`talos-cli`, `talos-tui`, `talos-evolution`, `talos-dashboard`, `talos-models`; gate (must NOT carry
`publish = false`) = `talos-sandbox`, `talos-tools`, `talos-agent`, `talos-runtime`, `talos-mcp`.

## Name Reservation Plan

1. Completed first wave (0.2.0): `talos-core`, `talos-skill`, `talos-config`, `talos-permission`,
   `talos-session`.
2. Completed second wave (0.2.0): `talos-plugin`, `talos-memory`, `talos-exploration`.
3. Completed integration wave (0.2.0): `talos-provider`, `talos-conversation`, `talos-rpc`.
4. Keep `talos-runtime` for the SDK facade; publish only after the 0.5.0 closure is intentionally
   published or decoupled.
5. Reserve `talos-cli` for the CLI binary only after the install-package gate passes.
6. Keep `talos-dashboard`, `talos-tui`, `talos-evolution`, `talos-models` product-only/quarantined
   with `publish = false`.
7. Do not plan around the `talos` package name (taken by an unrelated crate).
8. Defer remaining high-risk names until docs, feature gates, and API support boundaries are complete.

Do not publish empty placeholder crates. Each reservation package should compile, include a clear
description, and state its pre-1.0 support boundary.
