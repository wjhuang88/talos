# Programmer Handoff: Crate Distribution Hardening

> Status: Ready for assignment
> Created: 2026-06-29
> Applies to:
> [Crate Distribution Hardening Two-Month Plan](2026-06-29-crate-distribution-hardening-two-month-plan.md)
> Owner backlog: [ARCH-031](../backlog/active/ARCH-031-crate-publication-boundary.md)
> Publication matrix: [CRATE-PUBLICATION-MATRIX](../reference/CRATE-PUBLICATION-MATRIX.md)

## Purpose

This handoff tells implementation programmers how to continue Talos crate distribution work without
accidentally publishing high-risk crates, weakening SDK boundaries, or turning product-only code
into public API.

The current goal is not "publish everything." The goal is to make published crates usable, protect
product-only crates from accidental publication, and prepare explicit gates for high-risk crates.

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

## Non-Negotiable Rules

- Never run real `cargo publish` unless the assigned task explicitly authorizes that exact crate and
  version.
- Never remove `publish = false` from `talos-cli`, `talos-tui`, or `talos-evolution` without a new
  story or decision.
- Never publish `talos-sandbox`, `talos-tools`, `talos-agent`, `talos-runtime`, or `talos-mcp`
  merely because dry-run passes.
- Never make `talos-cli` or `talos-tui` a dependency of an embeddable SDK path.
- Never claim API stability beyond pre-1.0 guarantees.
- Update owner docs before `docs/BOARD.md`.

## Required Reads Before Starting

Read these in order:

1. `AGENTS.md`
2. `docs/backlog/active/ARCH-031-crate-publication-boundary.md`
3. `docs/reference/CRATE-PUBLICATION-MATRIX.md`
4. `docs/tasks/2026-06-29-crate-distribution-hardening-two-month-plan.md`
5. The assigned crate's `Cargo.toml` and crate-level `src/lib.rs`
6. Any ADR listed by the assigned work item

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

Your assignment is: <ASSIGNMENT ID AND TITLE>.

Hard constraints:
- Do not run real cargo publish.
- Do not remove publish = false from talos-cli, talos-tui, or talos-evolution.
- Do not publish or mark publish-ready any high-risk crate unless its gate is complete.
- Keep talos-runtime as the SDK facade; do not make talos-cli or talos-tui part of the embeddable path.
- Treat all public APIs as pre-1.0 unless the owner docs say otherwise.

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
