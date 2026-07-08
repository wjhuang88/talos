# Programmer Handoff: I106-I109 Talos Self-Bootstrap Plan

**Date**: 2026-07-09
**Author runtime**: glm-5.2 via zai-coding-plan (external, NOT Talos)
**Current HEAD**: `5f53ee4`
**Plan doc**: `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md`

---

## What Was Done (I106 — Month 1: Self-Bootstrap Control Plane)

I106 SBT100-SBT104 are **complete** (5 commits `81854af` → `5f53ee4`). I106 is in **Review**.

### Deliverables

| Task | Commit | Deliverable |
|---|---|---|
| SBT100 | `81854af` | Execution contract, inventory, disqualification rules. I106 activated. |
| SBT101 | `66a46a8` | Evidence schema with checkpoint template and Qualifying/Partial/Non-qualifying rubric. |
| SBT102 | `785e4af` | `scripts/talos_smoke.sh` — repeatable non-mutating runtime smoke harness (9 checks). |
| SBT103 | `d7e4964` | Governance mutation rehearsal: preview → write → validate → rollback, all proven. |
| SBT104 | `5f53ee4` | Month-1 closeout. Pre-existing `bash_tool.rs` fmt violation fixed. Full validation matrix green. |

### Validation Baseline (Verified At `5f53ee4`)

- `cargo fmt --all -- --check` — ✅ clean
- `cargo check --workspace` — ✅ passed
- `cargo test --workspace` — ✅ 1791 passed, 0 failed
- `cargo clippy --workspace -- -D warnings` — ✅ no warnings
- `scripts/validate_project_governance.sh .` — ✅ 0 warnings
- `git diff --check` — ✅ clean
- `scripts/talos_smoke.sh` — ✅ 9/9 passed

### REL-002 Classification: Non-Qualifying

The entire I106 session was executed by glm-5.2 (external runtime), not the `talos` binary.
Per REL-002 acceptance criteria, this is **non-qualifying** evidence. The artifacts (evidence
schema, smoke harness, governance mutation path) are useful for future Talos-primary sessions,
but the session itself does not prove self-bootstrap capability.

---

## What Is Next (I107 — Month 2: Talos-Primary Feature Polish)

### Activation Prerequisites

1. I106 Review must be accepted (move to Complete) or explicitly allowed to proceed.
2. Read the Required Reads in `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md` § Required Reads.
3. Read `docs/iterations/I107-talos-primary-feature-polish.md`.

### Task Queue (SBT110-SBT113)

| ID | Task | Priority |
|---|---|---|
| SBT110 | Select from the issue-audit corrective queue. **Default decision: select #18 request-dispatch timeout first** (RUNTIME-002 / PROVIDER-002). | Critical |
| SBT111 | Implement the selected corrective change using existing patterns and permission-gated tools. | Critical |
| SBT112 | Update user docs, backlog, iteration, and board in owner-first order. | High |
| SBT113 | Classify session against REL-002. | High |

### #18 Request-Dispatch Timeout — What Needs To Happen

Per `docs/backlog/active/RUNTIME-002-turn-health-and-stuck-processing.md` § 2026-07-08 Status
Correction and `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md` § 2026-07-08
Status Correction:

**Problem**: `reqwest::Client::new()` in `crates/talos-provider/src/openai.rs` and
`crates/talos-provider/src/lib.rs` has no request-level timeout. The provider `send().await` can
hang indefinitely before response headers arrive. Existing `ProviderTimeoutConfig` only has
`first_packet_timeout_secs` and `stream_idle_timeout_secs` — these protect stream parsing after
a response exists, not the `send().await` before headers.

**Required fix**:
1. Add a request-dispatch timeout for OpenAI-compatible and Anthropic providers. Either configure
   `reqwest::Client::builder().timeout(...)` carefully or wrap provider `send().await` in
   `tokio::time::timeout`.
2. **Preserve stream-idle semantics** — do not accidentally impose a total response-body timeout
   that breaks valid long streams. The dispatch timeout must cover only the request send →
   response-header wait, not the streaming body.
3. Add deterministic test proving a provider call that accepts the request but never returns
   response headers emits a terminal provider timeout/error and clears processing.

**Key files to read before starting**:
- `crates/talos-provider/src/openai.rs` (look for `Client::new()` and `send().await`)
- `crates/talos-provider/src/lib.rs` (look for `Client::new()` and `ProviderTimeoutConfig`)
- `crates/talos-provider/src/openai_request.rs` (request assembly)
- `crates/talos-core/src/provider.rs` (provider types)
- `docs/backlog/active/PROVIDER-002-response-reliability-timeout-retry.md` (full acceptance criteria)

**After #18**, per default decisions in the plan:
- Prefer **#39 dashboard transient notification** (TUI-028) next.
- Then **#24/#31 visual evidence** (TUI-028).
- **TOOL-020** or **I085 MC107 walkthrough** only after corrective queue is closed.

---

## After I107 (I108 — Month 3: Architecture-Sensitive Self-Bootstrap)

Per default decisions: prefer **ARCH-032 Single Data Flow Audit** unless maintainer selects
something else. ARCH-032 is audit-only activation — output is current-state data-flow evidence and
follow-up stories, not implementation of new health loops or multi-agent orchestration.

---

## After I108 (I109 — Month 4: REL-002 Closeout)

Final session, evidence audit against every REL-002 acceptance criterion, and v1.0 go/no-go report.
No `v1.0.0` claim unless every criterion is met.

---

## Uncommitted Working Tree State

There are **uncommitted changes NOT from this session** in the working tree:

```
 M docs/backlog/PRODUCT-BACKLOG.md
 M docs/backlog/active/PERM-004-workspace-trust-sandbox.md
?? docs/backlog/active/PERM-005-logical-tool-sandbox-enforcement.md
```

These appear to be external edits to PERM-004/PERM-005 adding workspace-trust sandbox design
content. They were present in the worktree but were **not** part of I106 and were **not committed**
by this session. Decide whether to commit, stash, or discard them before starting I107.

---

## Critical Operating Rules

1. **Work in ID order** inside the active monthly iteration.
2. **Before editing**, record the exact files expected to change and validation commands expected
   to prove the result.
3. **Use existing repository patterns** — do not introduce broad abstractions for a single packet.
4. **Record real runtime evidence** for every behavior-facing change — through the `talos` binary
   or a Talos-owned harness, not only unit tests.
5. **Update owner docs before `docs/BOARD.md`**.
6. **Never claim a command passed** unless it was run in this worktree.
7. **REL-002 honesty**: if the runtime is external (not `talos`), classify as non-qualifying.
8. **Not authorized**: push to remote, tag, release, publish, permission-default/sandbox/credential/
   dependency/storage-default changes, force-push, reset, tag deletion, or broad cleanup.
9. **Commit format**: `type(scope): description (#story-id) [model:<model-name>]`
10. **Run `scripts/talos_smoke.sh`** at the start of each session to establish a runtime baseline.
11. **Run the full validation matrix** at closeout:
    ```sh
    cargo fmt --all -- --check
    cargo check --workspace
    cargo test --workspace
    cargo clippy --workspace -- -D warnings
    scripts/validate_project_governance.sh .
    git diff --check
    scripts/talos_smoke.sh
    ```
12. If a command cannot run, record the command, failure summary, likely cause, and fallback.
13. Stop for review if the same blocker recurs after 3 materially different attempts.

---

## Start Command

```
根据 docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md 继续 I107 (SBT110-SBT113)。
首先阅读该计划文档的 Required Reads，然后阅读 I107 iteration doc，然后从 SBT110 开始：
选择 #18 request-dispatch timeout 作为最高优先级纠正项。
```