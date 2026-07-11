# Long-Running Task: High-Risk Governance Gate

> Status: Complete (2026-07-11; governance packets reconciled, residual features transferred to owner docs)
> Created: 2026-06-27
> Gate owner: project maintainer / architect approval boundary
> Executor rule: any agent or person may update this task only by following the gates below; this
>   document does not grant the current executor a special personal identity or unilateral approval
>   authority.
> Scope source: user request to plan and execute the roadmap areas that require explicit
>   high-risk governance
> Baseline rule: this task orders high-risk work. It does not approve release tagging,
> destructive data operations, network spending, new runtime dependencies, or permission-boundary
> changes without the gates named below.

## Startup Contract

### Outcome

Turn the backlog items that require direct senior/architectural oversight into an ordered,
auditable execution track, then execute them in dependency order without weakening Talos'
permission, prompt-cache, storage, release, or dependency boundaries.

This task is a governance track, not a replacement for the owning backlog or iteration records.
Each implementation slice still needs its own owner document, acceptance criteria, validation
evidence, and status synchronization. References to "direct review" mean review through this
project gate, not a claim that the current agent is the sole reviewer or architect.

### In Scope

- Close or explicitly block release-readiness gates for I047/I056/I057 before any `v0.2.0` tag.
- Prepare and execute SKILL-002 only after context ownership, reference loading, and prompt-cache
  invalidation rules are explicit.
- Keep permission-sensitive work under direct review: PERM-001, TOOL-010, and SCHED-001.
- Keep context-sensitive work under direct review: MEM-007 and any interaction with memory prompt
  injection, tool output history, raw transcript export, or stable prompt prefixes.
- Keep network/document/exploration ingestion under direct review: WEBFETCH-001 follow-ups that
  fetch, classify, save, or ingest remote or local documents. WEBFETCH-001 Phase 2+ planning belongs
  inside the holistic tool-set design audit rather than as a standalone tool expansion.
- Keep extension/protocol work under direct review: PLUGIN-001, MODEL-003, WEB-001, REMOTE-001,
  and any ADR they require.
- Keep embeddable runtime/API productization under direct review: RUNTIME-001 defines public SDK
  boundaries for other Rust projects and must not be delegated as routine crate plumbing.
- Use `docs/tasks/2026-06-28-architect-owned-high-risk-work-group.md` as the explicit grouping of
  work that needs direct senior/architectural handling.
- Treat WEB-001 as a product differentiation research track, not a far-future novelty. It must study
  the omp.sh/EXT-002 browser control surface reference before defining a loopback-only Talos MVP.
- Record checkpoints before moving from one risk packet to the next.

### Out Of Scope

- No `v0.2.0` tag, GitHub Release, published installer, or release-history mutation without
  explicit architect approval.
- No destructive cleanup, retention apply path, or deletion of user data outside test fixtures.
- No new native/runtime dependency for WASM, document conversion, web server, remote transport,
  local models, vector stores, or reasoning providers before the required Spike/ADR.
- No hidden auto-approval path for tools, scheduled tasks, plugin commands, web actions, or remote
  clients.
- No background daemon, marketplace, network crawler, or browser automation.
- No parallel same-worktree agent execution for overlapping files.

### Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| T0 | Establish oversight baseline | This task record exists; non-terminal work is inventoried; Board/iteration index points to this track. | User request | Governance validation passes after docs sync. | Keep this as planning-only if validation fails. | Complete |
| T1 | Release gate disposition | I047/I056/I057 have an explicit release/no-release disposition; `v0.2.0` remains blocked or is approved by architect evidence. | T0 | Owner docs and Board agree; no tag unless approval is recorded. | Keep Review/Blocked state with exact missing evidence. | Complete |
| T2 | SKILL-002 readiness and implementation | Context/cache owner, Level 1/2 loading policy, path confinement, and request-preview evidence are implemented or the story remains blocked with exact gaps. | T1 release disposition or explicit deferral | Real `talos` binary/request-preview proves activated Skill content reaches provider context and invalidation is deterministic. | Keep Level 1/2 disabled; preserve Level 0 only. | Complete |
| T3 | Search/tool-design research packet | TOOL-004 runs before TOOL-007; TOOL-007 includes WEBFETCH-001 Phase 2+ in the holistic tool family, permission, and progressive-loading design. | T2 | Search engine recommendation exists; tool family design names WEBFETCH boundaries and follow-up stories. | Keep current grep and WEBFETCH Phase 0/1 only. | Complete |
| T4 | Web control surface differentiation packet | WEB-001 studies omp.sh/EXT-002 patterns and defines a loopback-only embedded web MVP without permission/auth bypass. | T2 or explicit priority override | MVP spec/ADR names server lifecycle, auth, permissions, embedded assets, and relationship to TUI/RPC/governance views. | Keep WEB-001 Research. | Complete |
| T5 | Permission-sensitive execution packet | PERM-001, TOOL-010, and SCHED-001 are split into safe slices with permission-pipeline tests before write/execute/scheduled behavior ships. | T2/T3 or explicit priority override | Deny/ask/allow regressions prove no bypass for batch files, scheduled injections, Guardian, or exec DSL. | Keep features disabled or research-only. | Complete |
| T6 | Context compression packet | MEM-007 gets a Spike/prototype decision, cache-stability proof, deterministic compression tests, and raw-output preservation design. | T2/T3 where shared tool-history code is stable | Stable prefix hash unchanged with compression on/off; `/export` retains raw output; token-savings evidence recorded. | Reject compression strategy and keep MEM-005 compaction only. | Complete |
| T7 | Protocol and extension ADR packet | PLUGIN-001 and MODEL-003 have accepted ADRs/specs before implementation; REMOTE-001 stays research unless loopback/auth boundaries are proven. | T0 and relevant Spikes | ADR/spec accepted; no runtime dependency added before decision; governance validation passes. | Keep items Research/ADR-needed. | Complete |
| T8 | Embeddable runtime boundary packet | RUNTIME-001 API audit classifies SDK/public/internal surfaces and decides facade placement before implementation. | T2 or explicit priority override | ADR/proposal accepted; no CLI/TUI assumptions leak into the runtime API; event/config cleanup stories are named. | Keep RUNTIME-001 Planned and do not add facade code. | Complete |
| T9 | Final synchronization | Backlog, iterations, Board, README/user docs, ADR index, and residuals match actual delivered behavior. | T1-T8 | Workspace gates and governance harness pass; final checkpoint names residual owners. | Mark task Partial with exact unfinished owners. | Complete |

### Dependencies And Prerequisites

- I047 is Review; `v0.1.2` tag has been pushed, but release workflow evidence is still required
  before I047 can be Complete.
- I056 and I057 are Review; `v0.2.0` required explicit architect approval before release
  execution.
- I049-I055 and I019/I020 are Review with implementation evidence; do not rewrite their baselines.
- SKILL-002 is Review after I058; context/cache ownership and real-binary request-preview evidence
  are resolved.
- MODEL-003 is ADR-needed; PLUGIN-001, WEB-001, REMOTE-001, MEM-007, and WEBFETCH follow-ups are
  not direct implementation authority.
- ADR-006, ADR-008, ADR-009, ADR-010, ADR-013, ADR-016, ADR-017, ADR-020, ADR-021, ADR-023 remain
  binding where applicable.

### Non-Terminal Inventory And Disposition

| Item | Current State | Disposition |
|---|---|---|
| I011 | Paused | Do not reopen provider plugin architecture under this task. |
| I018 | Planned | Historical baseline preserved; OBS slice already landed through I047. |
| I019 | Review | Keep Review until final release/readiness disposition; do not rewrite execution history. |
| I020 | Review | Keep Review; vector/graph residual stays under RES-001/STORE-001. |
| I028 / SCHED-001 | Planned / In Progress | Must go through T3 permission-sensitive packet before implementation proceeds. |
| I047 | Review | T1 decides whether evidence is enough for Complete or records blocker. |
| I048 | Planned | Foundation baseline preserved; user-facing continuation already delivered in I049. |
| I049-I057 | Review | Preserve execution records; append corrections only. |
| SKILL-002 | Review | T2/I058 implemented explicit Skill activation and bounded references; keep review/closure under SKILL-002 and I058. |
| PERM-001 | Deferred | T3 may produce ADR/design only unless explicitly activated. |
| TOOL-010 | Refinement | T3 must prove per-file permission semantics before code lands. |
| MEM-007 | Research | T4 must produce Spike/prototype evidence before selection. |
| WEBFETCH-001 follow-ups | Mixed: Phase 0/1 complete; Phase 2+ blocked | Plan Phase 2+ through T3/TOOL-007 before activating only approved Rust-native slices. |
| PLUGIN-001 | Research | T6 spec/ADR only before runtime dependency. |
| MODEL-003 | ADR-needed | T6 ADR before provider/session/TUI implementation. |
| WEB-001 / REMOTE-001 | Research | WEB-001 moves through T4 as a product-differentiation Spike; REMOTE-001 remains T7 research unless loopback/auth/API boundaries are accepted. |
| RUNTIME-001 | Planned | T8 API audit/ADR first; facade implementation only after the public SDK boundary is accepted. |

### Artifacts And State Owners To Update

- This long-running task record.
- Owning backlog files for each item touched.
- Owning iteration records if an item is selected into or changes iteration state.
- `docs/iterations/README.md` for execution-round visibility.
- `docs/BOARD.md` after owner docs are updated.
- ADRs under `docs/decisions/` when a boundary decision is made.
- README/user docs only when user-visible behavior changes.
- `EVOLUTION.md` only for reusable lessons from failed validation, user correction, or
  non-obvious execution hazards.

### Validation And Acceptance Evidence

Every implementation packet must run:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
scripts/validate_project_governance.sh .
```

Planning-only packets must run at minimum:

```sh
scripts/validate_project_governance.sh .
git diff --check
```

Packet-specific evidence is mandatory:

- Release: version/tag/release-workflow evidence and explicit architect approval if tagging.
- Skills: real-binary/request-preview proof and cache invalidation tests.
- Permissions: allow/ask/deny regressions for every write/execute/network path touched.
- Compression: stable-prefix hash, determinism, raw-output export, token-savings measurement.
- Web/document: permission, size/time budget, content classification, save-path, citation tests.
- Protocols: accepted ADR/spec before runtime dependency or public protocol changes.

### Branch, Worktree And Checkpoint Plan

- Default branch/worktree remains the current workspace unless the user requests a branch.
- Do not run overlapping same-worktree agents on files that can conflict.
- Prefer one packet per commit if commits are requested.
- Append a checkpoint before moving from one task item to the next.
- Push only when explicitly requested or already approved for that exact action.

### Allowed Permissions And External Actions

Allowed by this task:

- Edit repository files in the workspace.
- Run local build, test, lint, governance, and runtime smoke checks.
- Create planning, ADR, backlog, iteration, and documentation updates.

Not allowed without separate explicit approval:

- Tagging or publishing releases.
- Pushing commits.
- Adding major runtime/native dependencies.
- Network calls that require credentials, spend money, or publish state.
- Destructive data operations outside temporary test fixtures.
- Force-push, history rewrite, or moving existing tags.

### Destructive Or Irreversible Operations

No destructive or irreversible production operation is authorized by this task. Cleanup,
retention, release, and publish actions must remain dry-run, test-fixture-only, or explicitly
approved in their owner documents.

### Time, Cost And Resource Limits

- Monetary spend: zero unless explicitly approved.
- Network: avoid unless a required official/current source cannot be evaluated locally.
- Retry deterministic gates at most twice after concrete fixes before recording a blocker.
- Prefer deferring optional polish over weakening permission, storage, cache, or release gates.

### Failure, Retry And Fallback Policy

- If release evidence is incomplete, do not tag; record the exact blocker.
- If Skill activation cannot be cache-safe, keep Level 1/2 disabled.
- If a permission-sensitive feature cannot prove deny behavior, keep it disabled or research-only.
- If active compression changes stable-prefix bytes or loses raw output, reject the strategy.
- If document/network ingestion needs a native/heavy dependency, stop for Spike/ADR.
- If plugin/reasoning/web/remote work changes public protocols, stop for ADR before code.

### Default Decisions For Foreseeable Ambiguity

- Prefer read-only/status/reporting before write-capable actions.
- Prefer explicit user commands over background or scheduled behavior.
- Prefer config-disabled defaults for new memory/context/provider behavior.
- Prefer bounded deterministic local logic over provider/network-dependent tests.
- Prefer ADR/spec first for dependency, protocol, permission, or persistence changes.

### Residual-Work Destination

- Release blockers: I047/I056/I057 and REL-001.
- Skill activation residuals: SKILL-002 and a new iteration when selected.
- Permission residuals: PERM-001, PERM-002 follow-ups, TOOL-010, SCHED-001.
- Compression residuals: MEM-007/MEM-005/MEM-003.
- Web/document residuals: WEBFETCH-001, RES-001, STORE-001.
- Protocol residuals: PLUGIN-001, MODEL-003, WEB-001, REMOTE-001, DIST-001.
- Runtime SDK residuals: RUNTIME-001 and any follow-up iteration created from its boundary ADR.

## Checkpoints

### T0 — Oversight Baseline Created (2026-06-27)

Created this task record after inventorying the current Board, Product Backlog, I047-I057,
SKILL-002, PERM-001, TOOL-010, SCHED-001, MEM-007, WEBFETCH-001, PLUGIN-001, MODEL-003, WEB-001,
and REMOTE-001.

Current state:

- This task is the execution owner for sequencing and oversight only.
- Existing backlog and iteration owners remain authoritative for scope and acceptance.
- Release publication, pushing, network use, destructive data operations, and new runtime
  dependencies remain unapproved.

Next task item:

- T1 release gate disposition: inspect I047/I056/I057 state and either record the missing
  evidence/approval or prepare a safe completion path without tagging.

Recovery/resume instruction:

- Resume from this file. Start with T1, then update owner docs before editing `docs/BOARD.md`.

### T1 — Release Gate Disposition Recorded (2026-06-27)

Completed task items:

- T0 oversight baseline.
- T1 release gate disposition.

Current state and artifacts:

- I047 remains Review. Local `v0.1.2` tag exists and I047 records that the tag was pushed, but
  release workflow evidence and post-release install smoke are still not recorded.
- I056 remains Review. It states `v0.2.0` is ready for tag only upon architect approval.
- I057 remains Review. It records that workspace version was still `0.1.2`, no `v0.2.0` tag,
  GitHub Release, or version bump had been performed at that checkpoint.
- No tag, push, release workflow mutation, or version bump was performed by this task.
- 2026-06-27 release execution note: the user explicitly requested completing a release, which
  supplies the approval required by this gate for the `v0.2.0` version bump and tag after
  validation.

Commands/checks and actual results:

- `git tag --list 'v0.*'` showed `v0.1.2` and earlier alpha tags.
- Release-state search across I047/I056/I057/REL-001/Board confirmed the same gating language:
  I047 needs release workflow evidence; `v0.2.0` needs architect approval.

Open risks or deviations:

- I047 cannot move to Complete until release workflow evidence is recorded.
- I056/I057 cannot move to release publication without architect approval for `v0.2.0`.

Next task item:

- T2 SKILL-002 readiness and implementation. First step: resolve context/cache ownership and
  activation policy before editing code.

Recovery/resume instruction:

- Resume from this file at T2. Do not reopen release publication unless architect approval or
  release workflow evidence is provided.

### T2a — SKILL-002 Readiness Baseline Created (2026-06-27)

Completed task items:

- T2 readiness/design portion only. Implementation remains pending.

Current state and artifacts:

- Created `docs/iterations/I058-explicit-runtime-skill-activation.md` as the SKILL-002
  implementation carrier.
- Updated `docs/backlog/active/SKILL-002-explicit-runtime-activation.md` from Refinement to Ready
  and recorded the context/cache ownership decision:
  - `talos-cli::skill_runtime` owns runtime SkillManager state, budgets, path confinement, and
    diagnostics.
  - `talos-agent` owns model-visible activated Skill context.
  - Activated Skill content enters the cacheable stable prefix after activation; activation
    invalidates `cached_stable_prefix`.
  - Command handling must route through typed runtime/session operations and must not append full
    Skill content to chat history or scrollback.
- Updated `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, and `docs/BOARD.md` to
  point at I058.

Commands/checks and actual results:

- Read SKILL-002 required docs and inspected `crates/talos-skill/src/{lib,loader,manager,types}.rs`,
  `crates/talos-agent/src/{prompt,lib}.rs`, `crates/talos-cli/src/skill_runtime.rs`, and the current
  `/skills` command path in `talos-conversation`.
- Code inspection found existing Level 0 discovery and prompt injection, existing cache
  invalidation primitives, and no current owner for active Level 1/2 runtime state.

Open risks or deviations:

- I058 implementation still needs code and tests. This checkpoint does not claim SKILL-002 shipped.
- If the live command/session seam cannot mutate the active agent safely, I058 must stop and route
  a prerequisite through CMD/SESSION before shipping activation.

Next task item:

- Continue T2 by implementing I058 or explicitly blocking it with compile/test evidence.

Recovery/resume instruction:

- Resume from I058. Start with agent prompt support for activated Skill context and runtime skill
  state path-confinement tests before wiring user commands.

### T2b — I058 Implementation Checkpoint (2026-06-27)

Completed task items:

- Implemented the first SKILL-002 slice under I058:
  - Activated Skill context in `talos-agent`, rendered as cacheable provider context.
  - Typed `SessionOp::SetSkillContext` for session-owned activation state.
  - Runtime activation/reference loading in `talos-cli::skill_runtime` with byte budgets and path
    confinement.
  - `/skills activate <name>` and `/skills reference <path>` typed command routing in
    `talos-conversation`.
  - TUI bridge handling that applies activation to session context without dumping full Skill or
    reference content into visible output.
  - Inline mode handling for the same Skill activation/reference commands, enabling deterministic
    real-binary request-preview validation.
- Updated user-facing README behavior for English and Chinese docs.

Commands/checks and actual results:

- `cargo check -p talos-agent -p talos-conversation -p talos-cli -p talos-tui`
  - Result: passed.
- `cargo test -p talos-agent -p talos-conversation -p talos-cli skill -- --nocapture`
  - Result: passed.
- `cargo test -p talos-agent set_skill_context_reaches_request_preview -- --nocapture`
  - Result: passed.
- `cargo test -p talos-cli conversation_loop_routes_skill_activation_to_session_op -- --nocapture`
  - Result: passed.
- `cargo clippy -p talos-core -p talos-agent -p talos-conversation -p talos-cli -p talos-tui -- -D warnings`
  - Result: passed.
- `cargo fmt --all -- --check`
  - Result: passed after applying `cargo fmt --all`.
- `cargo check --workspace`
  - Result: passed.
- `cargo clippy --workspace -- -D warnings`
  - Result: passed.
- `cargo test --workspace`
  - Result: passed.
- `scripts/validate_project_governance.sh .`
  - Result: passed with 0 warnings.
- `git diff --check`
  - Result: passed.
- `cargo test -p talos-cli --test skill_runtime_e2e -- --nocapture`
  - Result: passed.
  - Added real `talos --inline --mock` binary proof: create workspace Skill, run
    `/skills activate review`, run `/mock-request`, and verify the activated Skill body reaches the
    provider request preview.

Open risks or deviations:

- T2 no longer has a missing request-preview proof. The real-binary proof uses inline mode because it
  can be scripted deterministically; TUI bridge behavior remains covered by targeted bridge tests.
- Workspace-level planned validation is green.

Next task item:

- Treat I058/SKILL-002 as Review. Continue R27 with the next high-risk packet only after selecting
  it explicitly through the task gates.

### T3-T9 — Evidence Reconciliation And Closure (2026-07-11)

R27 closes as a governance sequencing track. Closure means every high-risk packet produced its
required decision, boundary evidence, safe fallback, or explicit residual owner. It does not claim
that every future feature named by the packet has shipped, and it grants no new permission to
release, publish, relax permissions, add dependencies, or expose remote/write-capable surfaces.

#### Non-Terminal Iteration Inventory At Closure

| State | Iterations | Disposition |
|---|---|---|
| Planned | I018, I019, I020, I028 | Preserve published baselines. I019 remains dependent on I018; I020 remains dependent on I019; I028 remains deferred under SCHED-001 permission gates. |
| Planned, superseded remainder | I081, I082, I083 | Preserve as historical shells; the 2026-07-03 product-hardening plan replaced their unexecuted path. |
| Paused | I085 | Preserve the MC107 real-terminal `/connect` walkthrough residual; R27 closure does not reactivate it. |
| Planned | I086, I087, I088, I089 | Preserve as the next product-hardening sequence; none is activated by this closure. |
| Review | I106, I107, I108, I109 | Preserve review state and non-qualifying REL-002 classification. No `v1.0.0` claim follows from R27 closure. |

#### Packet Closure Evidence

| Packet | Closure Evidence | Residual Owner |
|---|---|---|
| T3 search/tool design | TOOL-004 and TOOL-007 are Complete; ADR-025 selected the Rust library-backed search direction; TOOL-012/TOOL-013 established progressive disclosure and multi-resource permission boundaries. | WEBFETCH-001 owns bounded Phase 2+ document capture; PDF/Office/OCR/browser scope stays excluded. |
| T4 web differentiation | ADR-031 accepted the default-on, loopback-only, read-only dashboard boundary; I077 T112/T113 completed browser/security review and hardening. | WEB-001 owns any later read-only expansion. Remote access, approvals, writes, browser automation, and session mutation require a new security gate. |
| T5 permission-sensitive work | I092's autonomy permission matrix records deny/ask/allow behavior for scheduled, batch, Guardian, and exec-style paths. ADR-038 plus PERM-004's first implementation add explicit Git-workspace trust only for repo-contained file writes, with Deny precedence and strict non-Git/out-of-repo behavior. Bash/exec trust remains prohibited. | PERM-001, TOOL-010, and SCHED-001 retain feature residuals. PERM-005 is the P1 security owner for touched-path/unknown-access evidence and repo-boundary enforcement before any bash/exec trust broadening. |
| T6 context compression | MEM-007 A10 proved default-off deterministic bash compression, stable-prefix preservation, raw UI/export output, and bounded token/line savings. | MEM-007 retains read/grep/git-diff, cross-turn, representative-session, and durable raw-output residuals. |
| T7 protocol/extensions | ADR-027/028/029/030/032 bound the plugin model and first read-only WASM slice; MODEL-003 is Complete; REMOTE-001 remains Research. | PLUGIN-001 owns local explicit plugin residuals; REMOTE-001 requires a future auth/permission decision before implementation. |
| T8 runtime boundary | ADR-024 accepted the facade boundary and RUNTIME-001 is Complete as a pre-1.0 embeddable runtime; later ARCH-033 work converged session event semantics without introducing CLI/TUI ownership into the facade. | REL-002 owns stable-1.0 qualification and remains NO-GO. |
| T9 synchronization | R27 status and task table are reconciled with current owner docs; Board moves R27 out of Now; residuals remain with the owners above. | Normal Board/backlog/iteration governance continues without this umbrella remaining artificially active. |

#### Final Validation And Recovery

- `./scripts/release_preflight.sh`: pass under the pinned Rust 1.97 toolchain after rerunning with
  loopback permission for the dashboard bind tests.
- `cargo test --locked -p talos-permission`: pass (64 unit tests plus 1 doctest).
- `scripts/validate_project_governance.sh .`: pass with 0 warnings.
- `git diff --check`: pass.
- No production code, dependency, permission policy, tag, or release state changed during closure.
- If a residual is selected later, start from its owner backlog/iteration document and current ADR;
  do not reopen R27 or treat this closure as implementation authority.
