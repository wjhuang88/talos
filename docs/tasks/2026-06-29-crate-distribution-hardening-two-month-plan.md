# Long-Running Task: Crate Distribution Hardening Two-Month Plan

> Status: Planned
> Created: 2026-06-29
> Planning horizon: about two months / eight one-week iterations
> Owner backlog: [ARCH-031](../backlog/active/ARCH-031-crate-publication-boundary.md)
> Publication matrix: [CRATE-PUBLICATION-MATRIX](../reference/CRATE-PUBLICATION-MATRIX.md)
> Programmer handoff:
> [Crate Distribution Programmer Handoff](2026-06-29-programmer-handoff-crate-distribution-hardening.md)
> Baseline rule: this task inventory is preserved; changed objectives use a new task record or
> change-control entry.

## Startup Contract

### Outcome

Close the crate-distribution work after the first public crates.io wave by making Talos packaging
safe, documented, and maintainable for external users, while delivering several substantial
user-facing capabilities: bounded web/document capture, model-catalog runtime integration,
configuration editing, and shared skill discovery.

The work is intentionally ordered:

1. Keep already published reusable crates trustworthy with docs, metadata, and semver hygiene.
2. Keep product-only crates impossible to publish accidentally.
3. Evaluate high-risk crates through explicit safety/API gates before any further publication.
4. Decide the `talos-runtime` SDK dependency closure without forcing embedders to depend on
   unstable product implementation details.
5. Deliver a permission-aware document capture MVP that builds on existing `http_request` and
   `save_url` without bypassing network/write permissions.
6. Wire the model catalog into runtime limits and user-visible metadata so published model data is
   no longer dead data.
7. Add practical configuration editing surfaces so users do not have to hand-edit TOML for common
   model/provider settings.
8. Add opt-in shared skill discovery from `~/.agents/skills` under the same Level 0/1/2 activation
   and prompt-budget boundaries as Talos-owned skills.
9. Finish user-facing README and architecture documentation for binary install, SDK use, and
   standalone crate use.

### Current Baseline

- 11 crates are published at `0.2.0`: `talos-core`, `talos-config`, `talos-permission`,
  `talos-skill`, `talos-session`, `talos-plugin`, `talos-memory`, `talos-exploration`,
  `talos-provider`, `talos-conversation`, and `talos-rpc`.
- `talos-cli`, `talos-tui`, and `talos-evolution` are product-only and have `publish = false`.
- `talos-sandbox`, `talos-tools`, `talos-agent`, `talos-runtime`, and `talos-mcp` are
  gate-before-publish candidates.
- `talos-runtime` remains the intended SDK facade, but it must not be published until its
  dependency closure is either safely published or decoupled.
- `talos` as a Cargo package name is already taken by an unrelated crate; Talos uses `talos-*`
  package names.

### In Scope

- Add crate-level docs and metadata for published crates where missing.
- Define release-support expectations for pre-1.0 crate users.
- Add mechanical checks that prevent accidental publication of product-only crates.
- Create safety/API gates for `talos-sandbox`, `talos-tools`, `talos-agent`, `talos-runtime`, and
  `talos-mcp`.
- Run dry-runs only after gate documents are in place.
- Publish additional crates only if their gate is satisfied and the maintainer explicitly
  authorizes the publish.
- Update README, README.zh-CN, architecture docs, and release notes for crate distribution.
- Design and implement WEBFETCH-001 Phase 2 as a bounded feature slice:
  `document_extract` for local saved documents/text-like resources, metadata-rich extraction
  summaries, explicit unsupported-format behavior, and integration with `save_url`/`http_request`.
- Add tests proving document extraction never writes files, save/download never injects full
  content automatically, and hybrid network/write permission boundaries still apply.
- Implement MODEL-004: catalog-backed context/output limit resolution, compaction limit plumbing,
  and user-visible model metadata where already displayed.
- Implement CONF-001 CLI configuration editing for `get`, `set`, and `list`, with secret masking
  and config validation. TUI `/config` may be planned or implemented as a later slice if popup
  dependencies are ready.
- Implement AGENT-002-B as opt-in shared skill discovery from `~/.agents/skills`, with Talos-owned
  skill precedence and no automatic body loading beyond existing activation/budget rules.

### Out Of Scope

- No automatic publication of remaining high-risk crates.
- No release tag, GitHub release, installer change, or binary distribution change unless separately
  approved.
- No `talos-cli`, `talos-tui`, or `talos-evolution` crates.io publication in this plan.
- No remote server semantics for `talos-rpc`; it remains local stdio.
- No new runtime dependency solely to improve packaging presentation.
- No independent per-crate versioning before a separate decision.
- No browser automation, anti-bot bypass, OCR, audio/video transcription, whole-site crawling, or
  hosted document conversion service in the WEBFETCH feature track.
- No PDF/Office parser dependency without a focused Spike and an ADR/dependency note. The MVP may
  classify unsupported document types and extract text/HTML/JSON/CSV/Markdown-like resources first.
- No automatic model catalog network refresh at startup.
- No plaintext display of inline `api_key` or other secret config values.
- No automatic loading of shared skill bodies from `~/.agents/skills`; shared skills must follow
  the same explicit activation path and token budgets as project/local skills.
- No AGENT-002 MCP import implementation in this plan; MCP remains a gated protocol/security track.

## Ordered Task Items

| ID | Week | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---:|---|---|---|---|---|
| T0 | 0 | Two-month plan and handoff committed | ARCH-031 classification | Governance validation passes | Keep as planning-only artifact | Planned |
| T1 | 1 | Published-crate docs and metadata audit | T0 | Matrix lists doc/readme/metadata gaps per published crate | Register gaps without code change | Planned |
| T2 | 2 | Product-only publication guard check | T1 | Script or documented check proves `publish = false` for product-only crates | Keep manual `cargo publish --dry-run -p talos-cli` evidence | Planned |
| T3 | 3 | Sandbox publication safety gate | T1 | Escape-vector checklist, platform behavior docs, targeted tests named | Keep `talos-sandbox` unpublished | Planned |
| T4 | 4 | Tools publication feature/permission gate | T3 | Tool-family feature plan and permission profile audit recorded | Keep `talos-tools` unpublished | Planned |
| T5 | 5 | Agent/runtime dependency-closure decision | T3; T4 | Decide publish vs decouple path for `talos-agent` and `talos-runtime` | Keep `talos-runtime` unpublished and document blocker | Planned |
| T6 | 6 | MCP publication gate | T4 | MCP support boundary, opt-in policy, and transport/auth non-goals recorded | Keep `talos-mcp` unpublished | Planned |
| T7 | 7 | User docs and crate distribution guide | T1-T6 | README/README.zh-CN/architecture explain install, SDK, standalone crates | Add docs debt with exact owner | Planned |
| T8 | 8 | Closeout and optional publish decision packet | T1-T7 | Workspace checks, governance, matrix, residuals, and publish/no-publish decisions complete | Mark blocked items with concrete gate failures | Planned |
| F1 | 1 | WEBFETCH Phase 2 design note | T0 | Extraction/save workflow, formats, permission boundary, and non-goals recorded | Keep feature in design-only state | Planned |
| F2 | 2-3 | `document_extract` MVP for local text/HTML/JSON/CSV/Markdown-like inputs | F1 | Bounded output, metadata, unsupported-format notices, tests | Keep local text-only extractor | Planned |
| F3 | 4-5 | Fetch/save/extract workflow integration | F2; TOOL-013 | `save_url` remains write-capable; extraction is read-only; no implicit save or injection | Keep manual two-step workflow | Planned |
| F4 | 6 | Tool presentation and CLI/TUI smoke coverage | F3 | Network/document tools load through the intended tool family and permission path | Keep feature hidden from default prompt policy | Planned |
| F5 | 7-8 | User docs and closeout for document capture | F3-F4 | README/README.zh-CN mention supported formats, limits, and non-goals | Register docs debt | Planned |
| M1 | 1 | MODEL-004 runtime design checkpoint | T0 | Limit resolution precedence and touched call sites recorded | Keep implementation planned | Planned |
| M2 | 2-3 | Catalog-backed limit resolution | M1 | Config/agent tests prove catalog fallback replaces hardcoded limit | Keep fallback 128k with blocker | Planned |
| M3 | 4 | Compaction and UI metadata integration | M2 | Compactor receives resolved limit; status/exit summary use catalog metadata where available | Keep UI display unchanged with blocker | Planned |
| C1 | 1 | CONF-001 CLI config editing design | T0 | Key path grammar, validation, and secret masking behavior recorded | Keep as design-only | Planned |
| C2 | 2-3 | `talos config get/list/set` MVP | C1 | Round-trip tests, env substitution preservation, secret masking tests | Keep read-only get/list first | Planned |
| C3 | 4-5 | Config editing UX hardening | C2 | Clear validation errors and docs; TUI `/config` readiness decision | Defer TUI editor with explicit residual | Planned |
| S1 | 1 | AGENT-002-B shared skills activation policy | T0 | Precedence, opt-in flag, and Level 0/1/2 cache boundary recorded | Keep research state | Planned |
| S2 | 3-4 | Opt-in `~/.agents/skills` discovery | S1 | Loader/manager tests prove opt-in, precedence, dedup, and budget behavior | Keep path disabled | Planned |
| S3 | 5 | Skill diagnostics and user docs | S2 | `/skills` or diagnostics show source without leaking bodies | Keep diagnostics internal | Planned |

## Required Reads

- `AGENTS.md`
- `docs/backlog/active/ARCH-031-crate-publication-boundary.md`
- `docs/reference/CRATE-PUBLICATION-MATRIX.md`
- `docs/reference/ARCHITECTURE.md`
- `docs/proposals/talos-crate-distribution-architecture.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
- `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md`
- `docs/backlog/active/MODEL-004-catalog-runtime-integration.md`
- `docs/backlog/active/CONF-001-config-editing.md`
- `docs/backlog/active/AGENT-002-dotagents-protocol-support.md`
- `docs/backlog/active/SKILL-002-explicit-runtime-activation.md`
- `docs/backlog/active/TOOL-012-tool-family-progressive-loading.md`
- `docs/backlog/active/TOOL-013-multi-resource-tool-permissions.md`
- ADR-007, ADR-008, ADR-020, ADR-024, ADR-025
- `Cargo.toml`
- `crates/*/Cargo.toml`

## Non-Negotiable Rules

- Do not run real `cargo publish` unless the assigned task explicitly says the gate is satisfied
  and the maintainer has approved that exact crate.
- Do not remove `publish = false` from `talos-cli`, `talos-tui`, or `talos-evolution` without a new
  story or decision.
- Do not publish `talos-runtime` until the dependency closure is safe or decoupled.
- Do not publish `talos-sandbox` or `talos-tools` before security/permission gates are complete.
- Do not make `talos-cli` or `talos-tui` required dependencies for embedders.
- Do not claim 1.0 API stability for any `0.2.0` crate.
- Owner docs must be updated before `docs/BOARD.md`.
- WEBFETCH feature work must keep context-fetching separate from file-saving.
- Document extraction must be bounded and deterministic; unsupported formats must produce metadata
  and a clear unsupported message, not a panic or silent truncation.
- MODEL-004 work must preserve conservative fallback behavior when catalog data is missing.
- CONF-001 work must mask secrets on all display surfaces and preserve `${ENV_VAR}` semantics.
- AGENT-002-B work must be opt-in and must not silently inject shared skill bodies into the prompt.

## Validation Gates

For code or manifest changes:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
scripts/validate_project_governance.sh .
git diff --check
```

For docs/governance-only changes:

```sh
scripts/validate_project_governance.sh .
git diff --check
```

For package-publication readiness:

```sh
cargo metadata --no-deps --format-version 1
cargo publish --dry-run -p <crate>
cargo search <crate> --limit 3
```

Real `cargo publish -p <crate>` is not a validation command. It is an external irreversible action
and requires explicit maintainer approval for that exact crate and version.

For WEBFETCH feature slices, add focused tests for the touched behavior:

```sh
cargo test -p talos-tools document
cargo test -p talos-tools save_url
cargo test -p talos-permission -p talos-runtime
```

For MODEL/CONF/skill feature slices, add focused tests as applicable:

```sh
cargo test -p talos-config model
cargo test -p talos-agent compaction
cargo test -p talos-cli config
cargo test -p talos-skill
cargo test -p talos-cli skill_runtime
```

## Work Assignment Notes

### A1: Published-Crate Docs And Metadata Audit

Audit the 11 published crates for crate-level docs, `description`, docs.rs readability, missing
README/category/keyword decisions, and public support caveats. Do not change APIs. Record findings
in the matrix.

### A2: Product-Only Publication Guard

Add or document a lightweight check that proves product-only crates remain unpublished:
`talos-cli`, `talos-tui`, and `talos-evolution` must keep `publish = false`. The check may be a
script, governance validator extension, or documented command evidence if a script would be
overkill.

### A3: Sandbox Safety Gate

Review `talos-sandbox` against process hardening, OS behavior, native dependency boundaries, and
escape vectors. The output is a gate checklist and targeted validation plan, not an automatic
publish.

### A4: Tools Feature And Permission Gate

Review `talos-tools` default dependencies and permission profiles. Define feature gates for heavy
or sensitive capabilities and ensure network/write/execute surfaces are documented.

### A5: Agent And Runtime SDK Decision

Decide whether `talos-agent` should ever be a direct external dependency or remain an implementation
detail behind `talos-runtime`. Then choose one path for `talos-runtime`: publish dependency closure
or decouple from unpublished implementation crates.

### A6: MCP Publication Gate

Define what `talos-mcp` promises and does not promise. Include server opt-in, tool conflict
policy, transport/auth non-goals, and relationship to `talos-rpc`.

### A7: User Documentation

Update README, README.zh-CN, and architecture docs with three distribution paths:

- binary/product install;
- embeddable SDK facade through `talos-runtime` when its gate is ready;
- standalone crates for configuration, permissions, storage, memory, exploration, provider,
  conversation, plugin, and local RPC foundations.

### A8: Closeout

Close the long task with explicit published crates, product-only crates, gate-before-publish crates,
validation evidence, and residual work. Do not tag or create a release unless separately requested.

### F1-F5: WEBFETCH Bounded Document Capture

This is the feature-development track for the plan. Build a useful bounded extraction workflow
without importing a broad document-conversion stack:

- F1 records the design under WEBFETCH-001 before implementation.
- F2 adds `document_extract` for local text/HTML/JSON/CSV/Markdown-like files with bounded output,
  metadata, and unsupported-format notices.
- F3 proves the `http_request` / `save_url` / `document_extract` workflow does not mix network
  context fetching, local writes, and model-context injection.
- F4 wires tool presentation/registry coverage only after permissions and output bounds pass.
- F5 updates user docs and closes residuals for PDF/Office/OCR as future gated work.

Do not add PDF/Office/OCR dependencies in this track unless the maintainer explicitly approves the
dependency gate.

### M1-M3: MODEL-004 Catalog Runtime Integration

Make the existing model catalog affect runtime behavior:

- M1 records the exact precedence for context/output limits and the affected call sites.
- M2 implements catalog-backed limit resolution in config/agent code with conservative fallback.
- M3 wires compaction and existing status/exit displays to catalog metadata where available.

Do not add catalog auto-refresh or new provider discovery in this track.

### C1-C3: CONF-001 CLI Configuration Editing

Deliver practical CLI configuration editing:

- C1 records key-path grammar, validation behavior, secret masking, and env-substitution rules.
- C2 implements `talos config get`, `talos config list`, and `talos config set` through
  `talos-config`, not ad hoc TOML mutation.
- C3 hardens UX and decides whether TUI `/config` is ready or should remain a follow-up.

Secrets such as inline `api_key` must never be printed in plaintext by get/list.

### S1-S3: AGENT-002-B Shared Skill Discovery

Implement only the skills portion of dotagents compatibility:

- S1 records opt-in policy, precedence, and cache/prompt ownership.
- S2 adds optional `~/.agents/skills` discovery with dedup and budget-preserving tests.
- S3 exposes diagnostics/source information without loading unrequested skill bodies.

Do not implement `~/.agents/mcp.json` import or shared MCP server startup in this track.

## Checkpoints

### T0 — Planning Baseline Created (2026-06-29)

Created this two-month task plan and the programmer handoff. No runtime behavior or additional
crates.io publication is claimed by this checkpoint.

Recovery/resume instruction: start from ARCH-031, then the publication matrix, then this plan.
Assign A1 first unless the maintainer explicitly reprioritizes a high-risk gate.
