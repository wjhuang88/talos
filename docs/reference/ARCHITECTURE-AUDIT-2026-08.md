# Talos Architecture Audit: 2026-08

| Field | Value |
|---|---|
| Baseline | `5ab3b0f2a42a9dbcc9737d12a675eaf07469bc28` |
| Scope | All 21 workspace crates and current governance owners |
| Behavior changes | None in the audit; the only source change is a test-only network fixture |
| Measurement harness | [`scripts/audit_architecture.py`](../../scripts/audit_architecture.py) |
| Machine inventory | [`ARCHITECTURE-AUDIT-2026-08-inventory.json`](ARCHITECTURE-AUDIT-2026-08-inventory.json) |
| Finding register | [`ARCHITECTURE-AUDIT-2026-08-findings.json`](ARCHITECTURE-AUDIT-2026-08-findings.json) |

## Executive Summary

The current workspace has 21 crates, no internal dependency cycle, 143,772 physical Rust source
lines, and 77,943 production lines under the documented measurement convention. `talos-cli` is the
expected composition root (fan-out 17, fan-in 0); `talos-core` is the dependency-free protocol
core (fan-out 0, fan-in 16). These are intentional boundaries, not findings.

The validated architecture work is concentrated in eleven bounded owners: tool-composition
documentation, CLI/TUI bridge, TUI coordinator, Todo source organization, conversation command
projection, session command workflows, agent custody reconciliation, pending-submission storage,
the `talos-core::tool` facade, native-boundary security review, and current-state documentation.
The report does not prescribe line-count-only splits. Every accepted refactor preserves existing
module paths, public APIs, event ordering, permission behavior, persistence schema, and output.

The previous network discovery timeout was a test fixture defect: the test used fixed port `1` and
could wait behind the test-only request timeout. It now starts a loopback listener, accepts one
connection, and closes it. Production timeout policy is untouched.

## Measurement Method

The harness uses `cargo metadata --locked --no-deps --format-version 1` as package authority and
the latest 200 Git commits for change hotspots. Production lines exclude `tests` directories,
`tests.rs`, `*_tests.rs`, `test_support.rs`, and the conventional trailing item-scoped
`#[cfg(test)]` module. The convention is explicit because Rust source parsing is not part of the
runtime and adding a parser dependency would broaden this audit-only change. The harness fails
clearly when Python, Cargo, or Git is unavailable; the replacement trigger is a CI environment that
does not provide the already-used `python3` standard library runtime.

`unsafe_lexical_candidates` is a locator, not a security verdict. Manual review found four
production sites in `talos-sandbox/src/hardening.rs` and one in `talos-tools/src/bash_tool.rs`;
test-only environment mutation is excluded. Native/panic review remains owned by ARCH-034-R04.

## Crate Boundaries

| Crate | Fan-out | Fan-in | Production LOC | Boundary verdict |
|---|---:|---:|---:|---|
| talos-agent | 8 | 2 | 8,074 | Runtime actor; high fan-out is intentional |
| talos-cli | 17 | 0 | 14,853 | Composition root; no downstream consumers |
| talos-config | 1 | 2 | 4,051 | Configuration/schema boundary |
| talos-conversation | 2 | 2 | 4,212 | Conversation state/projection boundary |
| talos-core | 0 | 16 | 2,539 | Dependency-free protocol core |
| talos-dashboard | 0 | 1 | 483 | Presentation surface |
| talos-evolution | 2 | 1 | 1,488 | Hook/observation extension |
| talos-exploration | 0 | 1 | 1,151 | Isolated exploration capability |
| talos-mcp | 4 | 1 | 1,730 | MCP transport/client/server boundary |
| talos-memory | 0 | 2 | 2,411 | Storage-backed memory capability |
| talos-models | 1 | 0 | 866 | Model import/store support |
| talos-permission | 1 | 7 | 1,326 | Permission policy boundary |
| talos-plugin | 2 | 7 | 1,151 | Plugin/permission integration boundary |
| talos-provider | 2 | 2 | 2,930 | Provider protocol boundary |
| talos-rpc | 2 | 1 | 682 | RPC adapter |
| talos-runtime | 9 | 0 | 602 | Downstream runtime facade |
| talos-sandbox | 1 | 3 | 758 | Process hardening boundary; security-gated |
| talos-session | 1 | 3 | 8,026 | Durable session boundary |
| talos-skill | 0 | 3 | 886 | Skill loading/runtime support |
| talos-tools | 3 | 4 | 9,749 | Permission-gated tool implementations |
| talos-tui | 3 | 1 | 9,975 | Terminal presentation and interaction |

## Extension Traces

| Scenario | Current path | Verdict |
|---|---|---|
| Tool addition | Tool implementation in owning crate -> `ToolContribution` list -> CLI profile composition -> registry validation | Implementation ownership is correct; StatusTool and scheduler are explicit composition exceptions owned by R01 |
| Provider addition | `ProviderProtocol`/config -> provider request/stream modules -> CLI model lifecycle | Configuration-driven compatible-provider extension is sound |
| Permission facet | `AgentTool::permission_profile` -> agent execution -> permission policy | Profiles are semantic, not a proven duplicate; no builder added |
| TUI command/panel | Conversation slash command or CLI handler -> `UiOutput`/panel state -> TUI coordinator/render components | Coordinator seams are large and owned by R05/R06/R07 |
| Session backend | `talos-session` persistence/repository -> agent custody/session actor -> CLI lifecycle | Storage and orchestration are separate crates; pending store source organization is R09 |
| Plugin | plugin manifest/registry -> contribution wrapper -> registry/permission gate | Source identity and permission wrapping are preserved |
| Runtime consumer | runtime facade -> agent/provider/tools/session composition | High fan-out is expected at the facade |

## Large Production Files

The complete 53-file list, raw/production counts, test boundary, and recent-commit counts are in
the machine inventory. The responsibility classification is exhaustive:

Actionable bounded seams:

- `talos-cli/src/tui_bridge.rs` -> ARCH-034-R02 (turn-loop/event-family/projection seam)
- `talos-session/src/todo.rs` -> ARCH-034-R03 (repository/model/tool adapter seam)
- `talos-tools/src/bash_tool.rs`, `talos-tools/src/git.rs`, `talos-tools/src/symbol.rs`, and
  `talos-sandbox/src/hardening.rs` -> ARCH-034-R04 (native/security boundary)
- `talos-tui/src/app.rs` -> ARCH-034-R05 (input/stream/frame coordinator seam)
- `talos-conversation/src/engine.rs` -> ARCH-034-R06 (command/projection seam)
- `talos-cli/src/session_handlers.rs` -> ARCH-034-R07 (session lifecycle workflow seam)
- `talos-agent/src/session.rs` -> ARCH-034-R08 (actor/custody seam)
- `talos-session/src/pending_submission.rs` -> ARCH-034-R09 (state/schema seam)
- `talos-core/src/tool.rs` -> ARCH-034-R10 (private source split behind stable facade)
- `docs/reference/ARCHITECTURE.md` and stale ADR/R0 status text -> ARCH-034-R11

No-change classifications for the remaining large files are evidence-based: provider stream
parsers are cohesive protocol boundaries; `scrollback.rs` is a renderer component family already
delegated from `App`; `scheduler.rs` is one bounded delivery actor; `registry.rs` is composition
root work covered by R01; `model_lifecycle.rs` is one model activation transaction; `main.rs`,
`mode_runners.rs`, `runtime/lib.rs`, and `core` protocol modules are facades/composition roots; the
remaining storage, MCP, memory, tools, and TUI files each have one dominant responsibility or are
below the next refactor threshold. A future split requires a second independent owner or a
responsibility/behavior regression, not a line-count increase alone.

## Native, Panic, and Concurrency Review

- `talos-sandbox` and `talos-tools/bash_tool.rs` contain the five production `unsafe` candidates.
  ADR-007 and the R0 record are the policy sources; implementation changes require independent
  security review.
- `gix` calls in `talos-tools/git.rs`, arborium parser calls in `symbol.rs`, SQLite in session and
  memory stores, and process spawning in tools/MCP are integration boundaries. Existing timeout,
  error, and panic containment is mixed, so the matrix and tests belong to R04.
- The workspace uses bounded Tokio channels/timeouts and no global broadcast bus. No architecture
  change is justified by adding a global event bus; the existing event boundary decision remains in
  force.
- Mutex poison handling and `expect` sites were reviewed as reliability observations, but no
  cross-crate policy violation was proven in this baseline. They remain regression-review inputs,
  not speculative refactor owners.

## Prior Finding Reconciliation

The machine register reconciles all 20 July findings. In summary: F01 remains as R01 documentation
closure; F02, F03, F07, F09, F10, and F19 are superseded/closed or folded into the security owner;
F04 and F08 are closed by locked Clippy; F11/F12/F14/F15/F16/F17/F18 are closed boundary facts;
F05/F06 were remediated by R05/R03; F13/F19 remain represented by security-gated R04. New F20-F26
capture the current source organization and documentation drift that the July/v0.4 baseline could
not see; F20-F26 are complete except for the ADR-007/R0 semantic portion retained by R04.

### Remediation Status (2026-08-08)

- I172-I179 completed the behavior-preserving R02/R03/R05-R10 source-boundary remediation using
  the implementation commits and exact-head CI recorded in each owner. The finding register now
  records those completion dispositions without rewriting the original audit-time proof or
  counterevidence.
- I180 completed the F26 current-state workspace, CLI composition, tool contribution, storage,
  plugin, and MCP documentation reconciliation in implementation merge
  `10cceec6aeb9089fe9c830355992c8fc60430d63`; exact-head CI `31238721507` passed. The original
  finding proof above remains an audit-time observation rather than being rewritten.
- ADR-007/R0 security meaning, native-boundary containment, and process-hardening conclusions remain
  outside I180 and owned by ARCH-034-R04; review-only I181 completed at `aea26ad0`, while accepted
  gaps require separately governed implementation slices.
- I171/ARCH-034-D and I172/R02 owner/claim terminal state was repaired from their already-existing
  PR #139/#147 closeout evidence; no production or security behavior changed.

## Validation Evidence

The claim PR exact-head CI passed Linux format/check/Clippy/tests, Windows workspace tests, Windows
installer fixture, and remote issue/owner reconciliation (run `31077504918`). Local evidence for
the audit baseline includes:

```text
cargo fmt --all -- --check                         PASS
cargo test -p talos-cli --locked provider_discovery::tests::discover_network_error_is_bounded -- --exact PASS
cargo check --workspace --locked                    PASS (claim head)
cargo clippy --workspace --all-targets --locked -- -D warnings PASS (claim head)
scripts/audit_architecture.py .                    PASS; 21 crates, no dependency cycles
```

The workspace full test gate is re-run after the report/owner documentation and all later bounded
remediation slices. No production refactor is claimed by I171 itself.

## Finding And Owner Index

| Owner | Scope | Dependency/security gate |
|---|---|---|
| ARCH-034-R01 | Tool contribution exception and extension docs | Existing I158 Review evidence |
| ARCH-034-R02 | CLI/TUI bridge private seam extraction | I169 bridge tests |
| ARCH-034-R03 | Todo source organization | Session API/repository tests |
| ARCH-034-R04 | Native/panic/unsafe boundary containment | I181 review Complete at `aea26ad0`; AG-4/I182 completed at `ae31242b` from independently approved exact head `4b968823`; AG-8/9/10 own its non-blocking path/decoding/notice residuals; parent remains Partial and I161 remains separate/blocked |
| ARCH-034-R05 | TUI App coordinator seams | TUI snapshots and event tests |
| ARCH-034-R06 | Conversation command/projection seams | Conversation API/output tests |
| ARCH-034-R07 | CLI session workflow seams | Session/model lifecycle tests |
| ARCH-034-R08 | Agent custody/reconciliation helpers | Structured submission tests |
| ARCH-034-R09 | Pending submission schema/state split | SQLite/restart tests |
| ARCH-034-R10 | `talos-core::tool` private source split | Public API compatibility checks |
| ARCH-034-R11 | Current architecture documentation truth | I180 DOC-CHECK/source trace; ADR/R0 semantics remain with R04 |

I171 is complete only after these owner records exist, statuses mirror the register, and the
production owners are separately claimed before implementation. This audit does not silently
activate or bypass I158-I162 dependency ordering.
