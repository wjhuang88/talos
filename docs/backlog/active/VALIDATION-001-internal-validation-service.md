# VALIDATION-001: Internal Validation Service

**Status**: Complete for shared internal validation service slice
**Priority**: P0
**Created**: 2026-07-04
**Source**: Maintainer correction after I095 review
**Depends on**: RUNTIME-001, GOV-003, REL-002

## Problem

I095 added `talos validate run`, but its execution model is still host-command oriented. The
evidence format is useful, yet the current `governance` and `workspace` profiles execute local
commands such as `scripts/validate_project_governance.sh .` and `cargo ...`.

That is not the right long-term boundary for Talos as a general-purpose agent runtime:

- Talos should expose validation as an internal callable capability, not primarily as a shell
  command wrapper.
- Governance validation should be implemented in Rust and callable by CLI, TUI, runtime, and future
  agent loops without shelling out to project scripts.
- Cargo is only the validation tool for this repository. Talos must not bake in a Rust-only agent
  model.
- Talos should be able to infer common project types, then inject host-tool adapter instructions
  only when that project type has been identified.
- Project-type inference must be extensible. It should use a strategy-style detector registry rather
  than a monolithic hardcoded language check.
- Host-tool validation may still exist, but it must be an adapter with explicit dependency,
  language, permission, and evidence metadata.

## Goal

Create a language-neutral internal validation service that can be called in-process by CLI, TUI,
runtime, and future governance workflows. Host tools such as Cargo, npm, pytest, make, or project
scripts are adapters, not the core abstraction.

## Scope

- Define shared validation types for:
  - profile identity;
  - check identity;
  - execution mode: internal check vs host-tool adapter;
  - language/ecosystem metadata;
  - permission decision;
  - status, exit/result code, stdout/stderr or structured diagnostic summaries;
  - evidence source and required/optional status.
- Move governance validation onto an internal callable path.
- Keep `talos validate plan/run` as CLI frontends over the internal service.
- Add a TUI-safe read-only or confirm-gated path for internal validation evidence.
- Preserve host-tool checks as explicit adapters, not hidden runtime assumptions.
- Add common project-type detection so validation can identify Rust, Node.js, Python, Go, Java, or
  mixed workspaces from project manifests before selecting host-tool adapters.
- Add governance-project detection so Talos can recognize workspaces managed by Talos governance
  docs and route governance checks through internal capabilities instead of language/toolchain
  assumptions.
- Implement project-type detection as an extensible strategy registry. New project or governance
  types must be added by registering a detector, not by growing a single all-purpose conditional.
- Add demand-driven host-tool adapter instruction injection: adapter usage guidance is loaded only
  after a matching project type is confirmed, and remains absent for unrelated ecosystems.
- Make profiles project-configurable over time without allowing arbitrary command execution.

## Non-Goals

- No generic shell runner.
- No arbitrary user-provided command execution in validation profiles.
- No Rust-only validation model.
- No hidden TUI execution of host tools.
- No scheduled execution, Guardian auto-approval, release action, tag, publish, or permission
  default relaxation.

## Acceptance Criteria

- [x] A shared internal validation API exists outside `talos-cli`.
- [x] `governance` profile can run without invoking `scripts/validate_project_governance.sh`.
- [x] CLI `talos validate plan/run` uses the shared service rather than owning validation logic.
- [x] TUI can call at least one internal validation profile without spawning host commands.
- [x] Evidence records distinguish `internal` checks from `host_tool` checks.
- [x] Host-tool adapters record language/ecosystem metadata and unavailable-tool behavior.
- [x] Common project types can be inferred from manifests before host-tool adapters are selected.
- [x] Host-tool adapter usage instructions are injected on demand only for confirmed project types.
- [x] Cargo is represented only as a Rust-project adapter for this repository, not as a Talos-wide
      assumption.
- [x] Documentation clearly explains the boundary between internal validation and host-tool
      validation.

## Candidate Design

Validation profiles should be composed from typed checks:

| Check Kind | Meaning | Examples |
|---|---|---|
| `internal` | In-process Talos logic with no host command execution. | governance manifest validation, board/iteration consistency, config schema validation |
| `host_tool` | Explicit adapter to a project toolchain. | Cargo, npm, pytest, make, project script |
| `external_service` | Future adapter requiring network or credentials. | CI status, remote policy service |

Project-type detection should be a separate discovery step before adapter selection. For example,
`Cargo.toml` may enable Rust/Cargo guidance, `package.json` may enable Node.js guidance, and
`pyproject.toml` may enable Python guidance. Mixed workspaces can expose multiple adapters, but the
validation service should not inject Cargo, npm, pytest, or similar instructions until the matching
project type is discovered.

The discovery step should follow a strategy pattern: each detector owns the markers and matching
logic for one project/governance type, and the validation service iterates a detector registry. This
keeps future ecosystems and governance schemes additive instead of forcing all detection logic into
one branch-heavy function.

Governance detection is a sibling strategy, not a Rust-specific special case. A workspace with
`.agent-governance/manifest.yaml`, `docs/sop/`, or `docs/BOARD.md` can expose internal governance
checks even when no language toolchain is available.

The first implementation should prioritize internal governance validation because it already exists
as Rust logic through `talos-conversation` governance summary/validation code. `scripts/*.sh` can
remain as compatibility wrappers, but they should not be the primary runtime path.

2026-07-05 implementation: validation logic now lives in the shared
`talos_conversation::validation` service. CLI validation is a frontend over
`collect_validation_plan`, `run_validation_plan`, and the text/JSON renderers. The conversation
engine exposes `/validate governance` as a TUI-safe internal command; it rejects host-tool profiles
from the TUI path and does not execute project scripts.

Project detection now identifies `talos_governance`, `rust`, `node`, `python`, `go`, and `java`
through a `ProjectTypeDetector` strategy registry. Adapter instructions require both a confirmed
project type and a selected host-tool check for that ecosystem, so `governance` profile output does
not inject Cargo guidance merely because a Rust manifest exists. Cargo checks remain
`execution_mode: "host_tool"` with `ecosystem: "rust"` and are blocked when a Rust manifest is not
detected.

## Relationship To I095

I095 remains valid as a transitional evidence format and bounded CLI command. This story records the
missing architecture requirement: validation must become a reusable internal service. The I095
implementation should not be treated as the final TUI/runtime boundary.

## Validation

- Unit tests for internal validation profile execution without host commands.
- CLI tests proving `talos validate run --profile governance` uses the internal service.
- TUI/conversation tests proving the TUI path does not spawn host commands for internal profiles.
- Adapter tests for host-tool unavailable behavior.
- Project-type detection tests covering Rust, Node.js, Python, and mixed workspaces.
- Instruction-injection tests proving unrelated host-tool adapter guidance is not loaded.
- Governance validation and `git diff --check`.

2026-07-05 validation evidence:

- `cargo fmt --all -- --check` passed.
- `cargo test -p talos-conversation slash_validate` passed: 3 tests.
- `cargo test -p talos-conversation validation::tests` passed: 10 tests.
- `cargo test -p talos-cli validation` passed: 2 tests.
- `cargo check -p talos-cli` passed.
- `cargo run -p talos-cli -- validate plan --profile governance --json` prints
  `project_types:["talos_governance","rust"]`, an internal governance check, and no Cargo adapter
  instruction.
- `cargo run -p talos-cli -- validate run --profile governance --json` prints a passed
  `execution_mode:"internal"` governance record.
- `scripts/validate_project_governance.sh .` passed: 0 warning(s).
- `git diff --check` passed.

## Required Reads

- `docs/iterations/I095-runtime-validation-evidence.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/GOV-003-builtin-project-governance.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `crates/talos-cli/src/validation.rs`
- `crates/talos-conversation/src/governance_summary.rs`
