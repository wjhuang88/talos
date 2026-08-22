# Architecture Decision Records

## Purpose

Record significant technical decisions that affect hard boundaries, security, durable state,
public APIs, or assumptions that require explicit validation.

Each ADR file is authoritative for its own status and decision. The complete pre-closeout index is
preserved unchanged at
[`DECISIONS-INDEX-pre-I170-closeout-2026-08-01.md`](DECISIONS-INDEX-pre-I170-closeout-2026-08-01.md).

## Status Model

- **Proposed** — decision is defined but not yet accepted by current implementation/review evidence.
- **Accepted** — the decision is approved and bound to recorded evidence.
- **Superseded** — retained as history; a newer ADR owns the current boundary.
- **Deferred / Rejected** — the proposed direction is intentionally not active.

Changing an Accepted decision requires an amendment where allowed or a superseding ADR. Do not
silently rewrite the reason or boundary that governed an earlier implementation.

## Current Decision Gates

| ADR | State | Current Boundary / Gate |
|---|---|---|
| [065: Encapsulated Permission Rules And Diagnostic Provenance](065-structured-permission-rule-provenance.md) | **Accepted / I189 Complete** | Decision content commit `dae98460` became effective through PR #355 merge `9579df7a`. I189 implemented the boundary at Completion Commit `6b577d6a`; PR #356 merged as `54241bdd` after exact-head CI `32511672926`, independent Agent-role review `5376591491` and CAS. |
| [064: Bounded Model-Assisted Auto Permission Decisions](064-bounded-model-assisted-auto-permission.md) | **Accepted / I218 Complete** | Completion Commit `a289a07f`; exact-head CI `32505438495`, independent security review `5372825090`, CAS and PR #353 merge `c129d4a5` passed. Default-on means attempted assistance, never default Allow; only capability-relative atomic no-clobber creation is initially eligible. No behavior or child authority is granted. |
| [063: Bounded Runtime Shutdown And Finalizer Coordination](063-bounded-runtime-shutdown-finalization.md) | **Accepted / I214 Complete** | One SDK/actor admission-start arbiter, validated borrowing structured shutdown, first-valid-request arbitration, one total deadline, actor-owned ADR-058 reconciliation, frozen ordered finalizers and a redacted shared report are accepted at Completion Commit `6719c876`. RUNTIME-005-B later completed through I216 at `c123328d`; C is Ready/Unclaimed and TOOL-024 production remains dependency-gated. |
| [062: Typed Provider Retry Progress Boundary](062-typed-provider-retry-progress.md) | **Accepted / I210 Review** | Provider retry progress uses a defaulted additive request-local typed channel, existing non-exhaustive Agent/session events and a distinct reconnecting phase. Retry policy, persistence and dependencies remain unchanged; implementation evidence is `6efee2b8`. |
| [060: Supervised Background Command Job Lifecycle](060-supervised-background-command-jobs.md) | **Proposed / I188 Review / PR #228** | Session-owned bounded supervisor, explicit `background:` permission resource, live-only terminal event and Unix-first group cleanup are proposed. Independent exact-head review gates acceptance; production still waits for RUNTIME-005 and PERM-006-C, and Windows waits for a separate D Job Object decision. |
| [061: Canonical Work Domain And Todo Migration Boundary](061-canonical-work-domain-and-todo-migration.md) | **Proposed / I196 Active** | One Work Graph is the long-term planning authority; current `talos-session` Todo remains the sole durable authority until a separately reviewed P1 compatibility/cutover slice. No new crate, schema, dual-write or public API is authorized by P0. |
| [059: Desktop Renderer, Host, And Motion Quality Boundary](059-desktop-renderer-host-motion-boundary.md) | **Proposed / I194 Complete** | Decision-only boundary is accepted; renderer selection, dependency implementation, platform execution evidence and measured interaction performance remain separate authorization gates. |
| [058: Partial-Turn Durable Finalization Boundary](058-partial-turn-durable-finalization.md) | **Accepted / I187 Complete** | One atomic, idempotent Success/Error/Cancelled finalizer, closed display-safe partial prefixes, explicit turn outcomes and first-writer conflict semantics gate separately claimed SESSION-008-B. No B or RUNTIME-005 implementation is authorized by acceptance. |
| [008: Bundled SQLite for Local Storage](008-sqlite-bundled-storage.md) | **Accepted; amended by I183** | Exactly four runtime consumers plus quarantined non-runtime `talos-models`; locked-metadata validation rejects allowlist, version, bundled-feature, or quarantine drift. |
| [056: Transactional Steering Submission And Turn Ownership Boundary](056-transactional-steering-submission-boundary.md) | **Accepted (2026-08-06)** | Durable structured queue custody, receipt-based ownership transfer, generation-safe lifecycle, Actor user/Scheduler arbitration, transcript-before-journal finalization and exact request-plan semantics are authoritative. PR #131 merged at `685d3b4f4088a172551f8c844a89f5dee9469430`; exact Head `90165cace4625c0f27616b3e1b9871bcb6a10186`, CI `31010166558`, and rebuilt real-terminal acceptance passed. Issue #136 is a non-blocking diagnostic wording residual. |
| [057: Windows PowerShell Process Boundary](057-windows-powershell-process-boundary.md) | **Accepted (2026-08-01)** | Windows uses one native `powershell.exe -NoLogo -NoProfile -NonInteractive -Command` boundary; Unix retains `bash`/`sh -c`; child environment removal is local; timeout is one absolute direct-child boundary; reusable Windows templates use an inert-token allowlist. PR #126 merged at `592254d73a98166df48da0139a02df67e9cd2cd6`. |
| [053: Tool Registration Composition](053-tool-registration-composition.md) | Accepted; I158 Review | Current checked-contribution/outer composition boundary remains authoritative. I158 closure still requires scheduler/status exception disposition and final documentation. |
| [054: Alternate-Screen App-Owned Transcript Rendering](054-alternate-screen-app-owned-transcript-rendering.md) | Accepted; I184 amendment Accepted; I186 delivered | TUI owns full-frame alternate-screen rendering, history reflow and terminal restoration. Bounded application-owned visible-cell selection completed at `a5115f5c` after exact-code-head Alacritty and Terminal.app acceptance. |
| [052: SDK Publication And Composition Boundary](052-sdk-publication-and-composition-boundary.md) | Accepted | Runtime/SDK publication and composition work remains gated through I158-I162; no implicit publication authorization. |
| [049: Steering Queue Projection Boundary](049-steering-queue-projection-boundary.md) | Accepted; extended by ADR-056 | Engine owns queued steering state and read-only UI projection before durable Actor acknowledgement. ADR-056 owns transactional consumption, custody and later-Turn execution. |
| [042: Embedded Durable Runtime Session Boundary](042-embedded-durable-runtime-session-boundary.md) | Accepted | Durable successful turns and host-selected session binding remain the runtime/session foundation for steering persistence. |
| [039: Runtime Event Semantic Single-Flow Boundary](039-runtime-event-semantic-single-flow.md) | Accepted | Canonical ordered lifecycle, live output and actor-owned persistence remain mandatory for protocol and runtime work. |

## Accepted Core Boundaries

The following Accepted ADRs remain active unless their own files say they are superseded or amended:

| Range | Areas |
|---|---|
| [001](001-runtime-self-evolution.md)–[006](006-event-architecture-boundary.md) | evolution, storage, TUI/session event architecture and single-flow ownership |
| [007](007-process-hardening-unsafe.md)–[013](013-provider-config-schema-boundary.md) | process unsafe, local storage dependency, tool provenance, Git/search dependencies, Guardian/exec policy and provider schema |
| [014](014-log-retention-and-rotation.md)–[018](018-tui-job-control-unsafe.md) | bounded logs, embedded prompts, layered memory, exploration storage and TUI job-control unsafe |
| [020](020-tree-sitter-code-analysis.md)–[026](026-multi-resource-tool-permissions.md) | tree-sitter, tool-call protocol, shared agent config, credentials, embeddable runtime, ripgrep libraries and multi-resource permissions |
| [027](027-plugin-runtime-boundary.md)–[034](034-reasoning-thinking-boundary.md) | plugin/runtime extension boundaries, command taxonomy, loopback dashboard, wasmtime review, associative memory policy and reasoning data |
| [036](036-zstd-compression-dependency.md)–[042](042-embedded-durable-runtime-session-boundary.md) | session compression/log format, workspace trust, runtime events, logical sandbox evidence, scheduler API and durable runtime sessions |
| [045](045-transient-model-private-tool-projection.md)–[057](057-windows-powershell-process-boundary.md) | model-private edits, memory admission, external-path authorization, model variants, transactional steering, multimodal/image safety, SDK/tool composition, TUI transcript and Windows shell/process boundaries |

Use the individual ADR document—not this range summary—when making implementation or review decisions.

## Superseded / Deferred Decisions

- [011: Guardian Approval Boundary](011-guardian-approval-boundary.md) — Superseded by ADR-064;
  retained as the historical disabled-by-default/read-only first-version boundary.
- [019: TUI Splash Scrollback-Only Boundary](019-tui-splash-scrollback-boundary.md) — Superseded by amended ADR-054.
- [035: TUI Conversation History Scrollback Boundary](035-tui-history-scrollback-boundary.md) — Superseded by amended ADR-054.
- [043: Defer Persistent Task Runtime](043-defer-persistent-task-runtime.md) — Persistent task runtime intentionally not authorized.
- [044: Defer Multi-Instance Discovery](044-defer-multi-instance-discovery.md) — Automatic A2A discovery intentionally not authorized.

## I169 / ADR-056 Acceptance Evidence

ADR-056 acceptance is bound to:

- exact implementation Head `90165cace4625c0f27616b3e1b9871bcb6a10186`;
- exact-head CI run `31010166558` / CI #1233, attempt 1, all jobs successful;
- independent architecture/code review and completed remediation;
- rebuilt real-terminal A/B/C queue, restart, restoration, fork, delete and recovery walkthrough;
- release binary SHA-256 `2fe9f07679bd3f513165e849c59335ef11f47662852283c8f22051e954b2683d`;
- merged PR #131 / completion commit `685d3b4f4088a172551f8c844a89f5dee9469430`;
- maintainer acceptance and merge authorization;
- TUI-044 / I169 completion and Issue #119 closure.

Issue #136 remains Open and independently owns only direct `/delete` recovery-command wording. It
does not reopen ADR-056's accepted custody, retryability or no-false-success boundary.

## I170 / ADR-057 Acceptance Evidence

ADR-057 acceptance is bound to:

- exact implementation Head `8cfe8edb2dbda581244f583fb809591391a54298`;
- exact-head CI run `30705366763` (`CI` run 718);
- Windows walkthrough artifact `8820174164`;
- merged PR #126 / completion commit `592254d73a98166df48da0139a02df67e9cd2cd6`;
- accepted security review at `docs/reference/I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-08-01.md`;
- maintainer readiness and merge authorization.

The accepted residual remains direct-child-only timeout cleanup. Full descendant process-tree
supervision, a PowerShell grammar parser, PowerShell 7 selection and Job Object lifecycle remain
outside ADR-057/I170.

## Writing And Review Rules

Write or amend an ADR when work:

- chooses between materially different architectures;
- changes security, permission, process, persistence or public API boundaries;
- relies on an assumption whose failure changes the product contract;
- overrides an existing constraint;
- supersedes an Accepted decision.

Every proposed decision must name validation and reversal triggers. Every acceptance must point to
implementation/review evidence. Passing tests alone does not establish acceptance for security- or
architecture-sensitive work.

## History

The original detailed decision descriptions and ordering are retained at:

- [`DECISIONS-INDEX-pre-I170-closeout-2026-08-01.md`](DECISIONS-INDEX-pre-I170-closeout-2026-08-01.md)

All individual ADR documents remain in this directory and are not replaced by the compact index.
