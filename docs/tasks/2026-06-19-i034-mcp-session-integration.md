# Long-Running Task: I034 MCP Session Integration

> Status: In Progress
> Created: 2026-06-19
> Owner iteration: I034 after prerequisite closure
> Baseline rule: this confirmed task inventory is preserved; unrelated work goes to residuals.

## Startup Contract

### Outcome

Restore trustworthy governance/iteration state, close I033 with real binary evidence, then deliver
the published I034 MCP Session Integration MVP: a configured local MCP tool is discovered at
session startup, exposed to the model, executed through normal permission/display paths, and shown
with provenance/status without crashing when its server is unavailable.

### In Scope

- GOV-002 semantic status/supersession repair for I010/I012/I016/I017.
- I033 Review closure and deterministic binary-facing Level 0 Skill evidence.
- Root-cause repair for the observed mock request-preview path if required for I033 evidence.
- I034/MCP-001 startup discovery across normal runtime composition paths.
- MCP tool registration, permission routing, provenance/status, failure behavior, and prompt-cache
  semantics.
- Automated tests, local MCP fixture runtime evidence, README and governance synchronization.
- Atomic commits after each completed phase/Story.

### Out Of Scope

- SKILL-002 Level 1/2 activation.
- CMD-001 registry redesign beyond the minimum `/plugins` integration already required by I034.
- MCP prompt-to-command conversion or a third Command definition origin.
- WASM plugins, remote MCP package installation, provider plugins, marketplace, release, tag,
  deployment, or migration of user data.
- SESSION-001 lifecycle implementation.

### Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T1 | Repair GOV-002 legacy iteration state | I010/I012/I016/I017 preserve baselines and have explicit dispositions; Manifest can return conformant | Confirmation | Shell and PowerShell governance validators pass with semantic owner review | If historical evidence conflicts, keep degraded and record the exact unknown; do not guess | Complete |
| T2 | Close I033 runtime evidence gap | Real `talos` binary output proves workspace Skill Level 0 reaches the provider request | T1 | Deterministic binary/integration test plus I033/README/status sync | If mock diagnostics cannot safely prove it after two approaches, leave I033 Review and stop before I034 | Planned |
| T3 | Activate I034 baseline | Published baseline gains activation record, MVP, docs list, and prerequisite disposition without target rewrite | T1, T2 | I034, MCP-001, iterations index, and Board agree on Active state | If prerequisites remain unresolved, keep I034 Planned and stop | Planned |
| T4 | Inventory and centralize MCP startup composition | One bounded startup integration path replaces mode-specific duplication where needed | T3 | Targeted tests and no `rmcp` DTO leakage/public API break | Preserve existing adapters; register architectural residual instead of broad refactor | Planned |
| T5 | Discover/register MCP tools before first turn | Configured local MCP tools enter the live ToolRegistry in supported CLI/TUI paths | T4 | Integration tests prove model-visible definitions before first provider call | Restrict first runnable slice to startup-stable local stdio servers and record unsupported modes | Planned |
| T6 | Enforce permission, provenance, and status routing | MCP calls use normal permission/display flow; `/plugins` reports session MCP state/provenance | T5 | Read/write fixture tests, provenance assertions, no bypass of ADR-006/permission gates | Disable unsupported capability with visible diagnostic | Planned |
| T7 | Define unavailable-server and cache behavior | Startup/mid-session failures degrade visibly; session-stable tool set/cache rules documented | T5 | Failure-path tests and prompt/cache assertions | Default non-strict mode skips failed server; strict-mode work is residual unless already configured | Planned |
| T8 | End-to-end runtime acceptance | Actual `talos` binary invokes a local MCP fixture and records observable result/provenance | T6, T7 | Binary command exits 0; fixture call/result and status evidence recorded in I034 | Retry twice; if environment-only bind/process restriction occurs, use approved local fallback and record limitation | Planned |
| T9 | Full closure and delivery | Workspace green, docs/status/retrospective synchronized, residuals owned | T8 | fmt, check, clippy `-D warnings`, workspace tests, both governance validators, diff check | Do not mark Complete; leave Review/Partial with checkpoint and exact failing gate | Planned |

### Dependencies And Prerequisites

- Current HEAD includes commits `c8ed259`, `07c174e`, and `954cda6`; worktree is clean.
- Rust stable toolchain and existing Cargo dependencies are available.
- Local fixture/process execution is allowed; no remote MCP credentials are required.
- ADR-006 single-consumer event boundary, ADR-009 provenance, and ADR-021 tool protocol remain
  binding.
- I034 activation requires T1 and T2 completion. CMD-001 remains a separate In Progress Story.

### Artifacts And State Owners To Update

- Code: `talos-cli`, `talos-mcp`, `talos-agent`, `talos-conversation`, and tests only as required.
- Owners: GOV-002, I033, I034, MCP-001, CMD-001 relationship, Product Backlog, iterations index,
  Board, Manifest, README, and EVOLUTION when a reusable lesson appears.
- Task checkpoints: this file after every task item/phase boundary.

### Validation And Acceptance Evidence

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace` outside the restricted sandbox when local listener tests require it
- `scripts/validate_project_governance.sh .`
- `pwsh -NoProfile -File scripts/validate_project_governance.ps1 .`
- `git diff --check`
- Real `talos --mock` request-preview proof for I033
- Real `talos --mcp-server-fixture ...` proof for I034

### Branch, Worktree And Checkpoint Plan

- Recommended branch: `feature/I034-mcp-session-integration` from current HEAD.
- Recommended worktree: current workspace on that branch; no second worktree unless isolation is
  needed after a blocking conflict.
- Commit after T1, T2, the coherent I034 implementation slices, and final closure.
- Commit format follows AGENTS with `[model:gpt-5]`.
- Do not merge, force-push, rebase published history, or modify unrelated user changes.

### Allowed Permissions And External Actions

Proposed authorization:

- Read/edit repository files; run format, build, tests, local fixture processes, and governance
  scripts.
- Create the recommended branch and make local commits after gates pass.
- Use network only if Cargo must fetch an already-declared dependency; do not add a dependency
  without ADR/dependency review.
- Push the feature branch only if explicitly confirmed below.
- No release, tag, deployment, remote service mutation, paid API, or external account action.

### Destructive Or Irreversible Operations

None authorized. No force push, history rewrite, user-session deletion, database migration, release,
or deployment. Temporary test files/processes must be isolated and cleaned up.

### Time, Cost And Resource Limits

- Suggested unattended window: up to 6 hours wall time.
- Monetary spend: zero.
- Retry a failing deterministic command at most twice after a concrete fix or environment change.
- Keep test output/files bounded; do not download optional models, plugins, or large assets.

### Failure, Retry And Fallback Policy

- Fix root causes within the confirmed scope; do not weaken tests or permissions to obtain green.
- After two failed implementation approaches for the same blocker, record evidence and stop that
  dependency chain.
- Optional work is deferred to the named backlog owner; required gate failure leaves the task
  Partial/Blocked.
- Stop before public API breaking changes, new `unsafe`, new runtime dependency, permission model
  changes, destructive actions, credentials, external cost, or contradictory requirements unless
  an existing ADR clearly authorizes the exact action.

### Default Decisions For Foreseeable Ambiguity

- Prefer Rust-native/existing project abstractions and local stdio MCP fixtures.
- Prefer startup-stable discovery; do not add mid-session dynamic tool mutation.
- Non-strict server failure is visible and non-fatal; unknown write capability is permission-gated.
- Choose the smallest reversible implementation that delivers the I034 MVP.
- Preserve published baselines and route unrelated findings to residual backlog items.

### Residual-Work Destination

- Command architecture: CMD-001.
- Explicit Skill bodies/references: SKILL-002.
- Session switching: SESSION-001 children.
- WASM/plugin command protocol: PLUGIN-001.
- Unresolved architecture/security decisions: a new focused backlog Story and ADR when required.

## Consolidated Confirmation

Confirmed by the user on 2026-06-19 with: `按推荐方案执行`.

Approved contract:

- complete GOV-002 and I033 Review before activating I034;
- use `feature/I034-mcp-session-integration` from the current HEAD;
- edit, test, commit, and push stable checkpoints on that feature branch;
- do not merge, release, deploy, migrate data, force-push, or perform destructive operations;
- allow local MCP fixture/subprocess/listener tests with zero monetary spend;
- use a six-hour ceiling and at most two concrete repair approaches per blocker.

## Checkpoints

### Checkpoint 0 - Start

```text
Completed task items: consolidated confirmation recorded
Current state and artifacts: main is clean and three commits ahead of origin; task record created
Commands/checks and actual results: governance validator passed with the expected degraded warning
Open risks or deviations: GOV-002 and I033 Review block I034 activation
Next task item: T1 GOV-002 evidence audit
Recovery or resume instruction: open this record, verify the feature branch exists at 954cda6,
then audit I010/I012/I016/I017 without rewriting their published baselines
```

### Checkpoint 1 - T1 GOV-002 Complete

```text
Completed task items: T1
Current state and artifacts: I010 Complete; I012/I016/I017 Superseded with original baselines preserved;
GOV-002 Complete; Manifest conformant
Commands/checks and actual results: Git history and I025/I026 evidence reviewed;
`scripts/validate_project_governance.sh .` and PowerShell validator both passed with 0 warnings;
`git diff --check` passed
Open risks or deviations: none for T1; I033 remains Review
Next task item: T2 repair mock request-preview evidence path and close I033
Recovery or resume instruction: run both governance validators, commit T1, then inspect Agent debug
preview handling before changing Skill runtime code
```

Incidental confirmed fact requested by the user: the built-in `grep` tool is not based on ripgrep
and does not invoke host `rg`. It uses Rust `regex`, `walkdir`, `glob`, and
`std::fs::read_to_string` in `crates/talos-tools/src/search_tools.rs`. Any performance/ignore-rule
upgrade belongs to TOOL-001, not I034.

User-requested residual created: TOOL-004 is a timeboxed Research Spike evaluating embedded
ripgrep crates, external `rg`, and the current engine. No grep implementation changes are in this
long-running task.
