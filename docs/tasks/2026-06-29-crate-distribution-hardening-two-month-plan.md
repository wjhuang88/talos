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
safe, documented, and maintainable for external users.

The work is intentionally ordered:

1. Keep already published reusable crates trustworthy with docs, metadata, and semver hygiene.
2. Keep product-only crates impossible to publish accidentally.
3. Evaluate high-risk crates through explicit safety/API gates before any further publication.
4. Decide the `talos-runtime` SDK dependency closure without forcing embedders to depend on
   unstable product implementation details.
5. Finish user-facing README and architecture documentation for binary install, SDK use, and
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

### Out Of Scope

- No automatic publication of remaining high-risk crates.
- No release tag, GitHub release, installer change, or binary distribution change unless separately
  approved.
- No `talos-cli`, `talos-tui`, or `talos-evolution` crates.io publication in this plan.
- No remote server semantics for `talos-rpc`; it remains local stdio.
- No new runtime dependency solely to improve packaging presentation.
- No independent per-crate versioning before a separate decision.

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

## Required Reads

- `AGENTS.md`
- `docs/backlog/active/ARCH-031-crate-publication-boundary.md`
- `docs/reference/CRATE-PUBLICATION-MATRIX.md`
- `docs/reference/ARCHITECTURE.md`
- `docs/proposals/talos-crate-distribution-architecture.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/backlog/active/DIST-001-optional-runtime-asset-distribution.md`
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

## Checkpoints

### T0 — Planning Baseline Created (2026-06-29)

Created this two-month task plan and the programmer handoff. No runtime behavior or additional
crates.io publication is claimed by this checkpoint.

Recovery/resume instruction: start from ARCH-031, then the publication matrix, then this plan.
Assign A1 first unless the maintainer explicitly reprioritizes a high-risk gate.
