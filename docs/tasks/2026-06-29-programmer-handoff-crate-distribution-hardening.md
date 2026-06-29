# Programmer Handoff: Crate Distribution Hardening

> Status: Ready for assignment (baseline reconciled 2026-06-29)
> Created: 2026-06-29
> Applies to:
> [Crate Distribution Hardening Two-Month Plan](2026-06-29-crate-distribution-hardening-two-month-plan.md)
> Owner backlog: [ARCH-031](../backlog/active/ARCH-031-crate-publication-boundary.md)
> Publication matrix: [CRATE-PUBLICATION-MATRIX](../reference/CRATE-PUBLICATION-MATRIX.md)

## Purpose

This handoff tells implementation programmers how to continue Talos crate distribution work and the
paired feature-development tracks without accidentally publishing high-risk crates, weakening SDK
boundaries, bypassing permissions, or leaking secrets.

The current goal is not "publish everything." The goal is to make published crates usable, protect
product-only crates from accidental publication, prepare explicit gates for high-risk crates, and
deliver real product improvements that users can exercise: bounded document capture, catalog-aware
runtime limits, CLI config editing, and opt-in shared skill discovery.

## Current Baseline

- Published at `0.2.0`: `talos-core`, `talos-config`, `talos-permission`, `talos-skill`,
  `talos-session`, `talos-plugin`, `talos-memory`, `talos-exploration`, `talos-provider`,
  `talos-conversation`, and `talos-rpc`.
- Product-only with `publish = false`: `talos-cli`, `talos-tui`, `talos-evolution`.
- Gate-before-publish candidates: `talos-sandbox`, `talos-tools`, `talos-agent`,
  `talos-runtime`, and `talos-mcp`.
- `talos-runtime` is the intended SDK facade, but must remain unpublished until its dependency
  closure is safe or decoupled.
- `talos` is not available as a Cargo package name; use `talos-*` names only.
- WEBFETCH-001 has Phase 0/1 complete (`http_request`, content detection, `save_url`); Phase 2+
  needs bounded document extraction/save workflow implementation.
- MODEL-004 is not clean-slate. I045 already implemented `Config::resolve_model_limits()`, tests,
  CLI/session wiring, and compactor limit plumbing. Remaining work is catalog-aware TUI/exit
  metadata and preserving the existing runtime limit path.
- CONF-001 is not clean-slate. I045 already implemented top-level `--config-list`,
  `--config-get`, and `--config-set` flags with secret masking. Remaining work is reconciling that
  flag surface with the planned `talos config ...` subcommand UX, validating env/schema behavior,
  and hardening docs/errors.
- AGENT-002-B is research/planned for this handoff only as opt-in `~/.agents/skills` discovery.

## Baseline Reconciliation

Do not assign M1/M2 or the existing config flags as greenfield implementation work.

| Track | Actual Baseline | Assignment Meaning |
|---|---|---|
| M1 | `resolve_model_limits()` exists and is tested | Record/preserve evidence only |
| M2 | CLI/session paths pass resolved limit into `SessionConfig`; compactor consumes it | Record/preserve evidence only |
| M3 | TUI status/exit summary still use hardcoded cost/rate logic | Implement residual catalog metadata display |
| C1 | Existing flag grammar works but no formal migration design exists | Write reconciliation design |
| C2 | `--config-*` flags exist; `talos config` subcommand does not | Add subcommands or record compatibility decision with tests |
| C3 | UX hardening/TUI `/config` not started | Implement hardening or record readiness decision |
| F1-F5 | `http_request` and `save_url` exist; extraction is clean-slate | Implement bounded document extraction workflow |
| S1-S3 | Workspace skill discovery exists; no `~/.agents/skills` global path | Implement opt-in shared discovery |

## Non-Negotiable Rules

- Never run real `cargo publish` unless the assigned task explicitly authorizes that exact crate and
  version.
- Never remove `publish = false` from `talos-cli`, `talos-tui`, or `talos-evolution` without a new
  story or decision.
- Never publish `talos-sandbox`, `talos-tools`, `talos-agent`, `talos-runtime`, or `talos-mcp`
  merely because dry-run passes.
- Never make `talos-cli` or `talos-tui` a dependency of an embeddable SDK path.
- Never claim API stability beyond pre-1.0 guarantees.
- Never combine context fetching and file saving into one implicit operation.
- Never add PDF/Office/OCR/browser automation dependencies without an explicit dependency gate.
- Never let document extraction produce unbounded model-facing output.
- Never print inline `api_key` or other secret config values in plaintext.
- Never auto-load shared skill bodies from `~/.agents/skills`; use opt-in discovery and existing
  activation/budget rules.
- Never implement `~/.agents/mcp.json` import as part of the shared-skills assignment.
- Update owner docs before `docs/BOARD.md`.

## Required Reads Before Starting

Read these in order:

1. `AGENTS.md`
2. `docs/backlog/active/ARCH-031-crate-publication-boundary.md`
3. `docs/reference/CRATE-PUBLICATION-MATRIX.md`
4. `docs/tasks/2026-06-29-crate-distribution-hardening-two-month-plan.md`
5. `docs/iterations/I045-product-readiness-model-lifecycle-observability.md`
6. `docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md` for feature-track work
7. `docs/backlog/active/MODEL-004-catalog-runtime-integration.md` for model runtime work
8. `docs/backlog/active/CONF-001-config-editing.md` for config editing work
9. `docs/backlog/active/AGENT-002-dotagents-protocol-support.md` for shared skill discovery work
10. The assigned crate's `Cargo.toml` and crate-level `src/lib.rs`
11. Any ADR listed by the assigned work item

## Assignment Map

| Assignment | Recommended Owner Profile | Main Deliverable | Must Not Do |
|---|---|---|---|
| A1 Published-crate docs audit | Rust library/docs | Matrix update listing docs/metadata gaps for the 11 published crates | Do not change public APIs opportunistically |
| A2 Product-only guard | Cargo/tooling | Check or documented command proving product-only crates cannot publish | Do not remove `publish = false` |
| A3 Sandbox gate | Security-minded Rust | Sandbox publish checklist and targeted tests | Do not publish `talos-sandbox` |
| A4 Tools gate | Tools/permissions | Feature/permission gate for `talos-tools` | Do not publish `talos-tools` before sandbox decision |
| A5 Runtime SDK decision | Senior runtime/API | `talos-agent`/`talos-runtime` publish-vs-decouple decision | Do not publish `talos-runtime` just to reserve the name |
| A6 MCP gate | Protocol/runtime | `talos-mcp` support boundary and opt-in/conflict policy | Do not introduce remote/auth promises |
| A7 User docs | Technical writing/Rust | README/README.zh-CN/architecture crate distribution docs | Do not imply 1.0 API stability |
| A8 Closeout | Release/governance | Final evidence, residuals, and publish/no-publish decisions | Do not tag or release without explicit approval |
| F1 WEBFETCH design | Tools/product | Design note for bounded document capture workflow | Do not implement before scope and non-goals are recorded |
| F2 document_extract MVP | Rust tools | Read-only bounded extractor for local text/HTML/JSON/CSV/Markdown-like files | Do not add PDF/Office/OCR dependencies |
| F3 fetch/save/extract integration | Tools/permissions | Tests proving fetch, save, and extract stay separate and permission-aware | Do not auto-save or auto-inject full content |
| F4 tool presentation | Agent/tools | Registry/presentation coverage for document tools | Do not expose tools outside intended family/policy |
| F5 feature docs | Technical writing/Rust | README/README.zh-CN docs for supported formats, limits, and non-goals | Do not imply full MarkItDown/PDF/Office support |
| M1 model runtime baseline evidence | Config/agent | Evidence note preserving existing `resolve_model_limits()` precedence and call sites | Do not reimplement completed runtime behavior |
| M2 catalog-backed compaction baseline evidence | Config/agent | Evidence note preserving existing `SessionConfig.model_context_limit` and compactor wiring | Do not remove conservative fallback |
| M3 TUI metadata residual | TUI/config | Status and exit summary use catalog context/pricing where available | Do not add catalog auto-refresh |
| C1 config editing reconciliation | CLI/config | Flag-to-subcommand design, key grammar, validation, and secret masking behavior | Do not mutate TOML ad hoc |
| C2 config subcommand/compatibility | CLI/config | `talos config get/list/set` or explicit compatibility decision, through `talos-config` | Do not print secrets or break `--config-*` flags |
| C3 config UX hardening | CLI/TUI | Error messages, docs, TUI `/config` readiness decision | Do not implement partial unsafe TUI writes |
| S1 shared skills policy | Skill/runtime | Opt-in policy and precedence for `~/.agents/skills` | Do not auto-load bodies |
| S2 shared skills discovery | Skill/runtime | Optional path discovery with dedup/budget tests | Do not import MCP config |
| S3 shared skills diagnostics | CLI/TUI/runtime | Skill source diagnostics without body leakage | Do not make shared skills higher precedence than Talos-owned config |

## Validation Commands

For code or manifest changes:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
scripts/validate_project_governance.sh .
git diff --check
```

For docs-only changes:

```sh
scripts/validate_project_governance.sh .
git diff --check
```

For publication readiness:

```sh
cargo metadata --no-deps --format-version 1
cargo publish --dry-run -p <crate>
```

Real publishing is not part of normal validation.

For WEBFETCH feature work, also run focused tests relevant to your slice:

```sh
cargo test -p talos-tools document
cargo test -p talos-tools save_url
cargo test -p talos-agent tool_presentation
cargo test -p talos-permission -p talos-runtime
```

For MODEL/CONF/shared-skill work, run focused tests relevant to your slice:

```sh
cargo test -p talos-config model
cargo test -p talos-agent compaction
cargo test -p talos-cli config
cargo test -p talos-skill
cargo test -p talos-cli skill_runtime
```

## Handoff Prompt

Copy this prompt when assigning work to a programmer:

```text
You are working in the Talos repository on the crate distribution hardening plan.

Read these first, in order:
1. AGENTS.md
2. docs/backlog/active/ARCH-031-crate-publication-boundary.md
3. docs/reference/CRATE-PUBLICATION-MATRIX.md
4. docs/tasks/2026-06-29-crate-distribution-hardening-two-month-plan.md
5. docs/tasks/2026-06-29-programmer-handoff-crate-distribution-hardening.md
6. docs/iterations/I045-product-readiness-model-lifecycle-observability.md
7. docs/backlog/active/WEBFETCH-001-web-and-document-fetch-tools.md if your assignment is F1-F5
8. docs/backlog/active/MODEL-004-catalog-runtime-integration.md if your assignment is M1-M3
9. docs/backlog/active/CONF-001-config-editing.md if your assignment is C1-C3
10. docs/backlog/active/AGENT-002-dotagents-protocol-support.md if your assignment is S1-S3

Your assignment is: <ASSIGNMENT ID AND TITLE>.

Hard constraints:
- Do not run real cargo publish.
- Do not remove publish = false from talos-cli, talos-tui, or talos-evolution.
- Do not publish or mark publish-ready any high-risk crate unless its gate is complete.
- Keep talos-runtime as the SDK facade; do not make talos-cli or talos-tui part of the embeddable path.
- Treat all public APIs as pre-1.0 unless the owner docs say otherwise.
- If working on WEBFETCH, keep fetch_url/http_request context ingestion, save_url file writes, and document_extract local extraction as separate permission-aware operations.
- If working on WEBFETCH, implement bounded deterministic output and explicit unsupported-format behavior. Do not add PDF/Office/OCR/browser automation dependencies.
- If working on MODEL-004, preserve conservative fallback when catalog metadata is absent.
- If working on MODEL-004 M1/M2, treat the task as evidence preservation, not greenfield implementation.
- If working on CONF-001, route reads/writes through talos-config, mask secrets on all display surfaces, and preserve existing --config-* flags unless an owner doc explicitly replaces them.
- If working on AGENT-002-B, make shared skill discovery opt-in and never load skill bodies without explicit activation.

Expected output:
- Implement only the assigned slice.
- Update the owner doc before docs/BOARD.md.
- Record validation evidence in the owning task or backlog document.
- Run the required validation commands for the type of change.
- Commit one logical change with a conventional commit message including (#ARCH-031) [model:<model-name>].

Stop and report instead of guessing if:
- The work would require real cargo publish.
- The work changes a permission, sandbox, network, remote-control, or SDK public boundary.
- The work requires adding a new runtime dependency.
- The work contradicts the publication matrix.
- WEBFETCH work would blur network, file-write, and context-injection boundaries.
- Config editing would display a secret value.
- Shared skill discovery would change prompt contents without opt-in or activation.
```

## Completion Report Template

Use this in the PR/commit summary or handoff note:

```text
Assignment:
Files changed:
Behavior/API changed:
Publication state changed:
Validation run:
Residual work:
Blocked items:
```

## Recovery Instructions

If the work is interrupted:

1. Check `git status --short`.
2. Re-read ARCH-031 and the publication matrix.
3. Confirm whether the current assignment changed any manifest publish setting.
4. Run at minimum `scripts/validate_project_governance.sh .` and `git diff --check`.
5. Append a checkpoint to the two-month plan before handing off.
