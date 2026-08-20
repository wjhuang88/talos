# Iteration I213: Dashboard Live Activity And Log Viewer

> Document status: Planned
> Published plan date: 2026-08-20
> Planned objective: deliver one bounded, loopback-only, read-only realtime Dashboard observation
> workspace over authoritative Talos Session/logging facts, with SSE reconnect/bounds and no
> write/control/remote/session-authority expansion.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a rebuilt Talos user on the default local loopback Dashboard can open Live
> Activity, observe safe semantic Turn/tool/usage events and bounded live logs during a real mock
> Session, disconnect/reconnect without unbounded buffering, and still has no Dashboard action route.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | Finalize the bounded WEB-001-B/I213 claim with the real governance PR number, merge it to `main`, refresh inventory, then activate only if iteration-order/parallel-work gates still permit it. |

Before activation, follow `docs/sop/AGENT-COLLABORATION.md`. This Planned document is not an
implementation authorization. `Claim Pending` is GitHub metadata only and must never be stored as an
owner claim state.

## Published Baseline

### Fresh Inventory At Planning — 2026-08-20

Target branch observed before planning:

`main@15a3d4248d13d3951c823628454a2629398a9d48`

The observed target is newer than the Dashboard handoff's historical
`ec7945156d2f72eb96af58d8cd9155f3bc5f37f1` baseline and must be refreshed again before claim
finalization, claim merge, activation, and implementation merge.

Open PR disposition at planning:

| PR | Scope | I213 disposition |
|---|---|---|
| #326 | I211 / VALIDATION-002 deferred human acceptance batch claim | Non-overlapping evidence/validation lane. It does not own Dashboard behavior. Recheck its state before I213 activation. |
| #120 | historical recovery Draft | Archival / do-not-merge provenance; no I213 authority. |
| #121 | historical recovery Draft | Archival / do-not-merge provenance; no I213 authority. |

No open Dashboard implementation/claim PR and no repository owner named `WEB-001-B` or `I213` was
found in the fresh search.

Current relevant owner disposition:

| Owner / iteration | Current state | I213 disposition |
|---|---|---|
| WEB-001 | Partial | Parent explicitly retains live-log/SSE/live-activity residual; select only this bounded residual. |
| WEB-001-A / I195 | Complete / Closed | Consume existing shell only; do not reuse completion authorization. |
| SEC-002 | Refinement / Unclaimed | Keep separate; no token delivery/auth redesign. |
| OBS-001 | Complete | Consume bounded log/rotation baseline; do not reopen. |
| CONF-001 | Partial | Config writes are excluded. |
| SESSION-009 | Refinement / Unclaimed | Multi-client replay/controller semantics excluded. |
| SERVER-001 | Intake | Serve/connect/interactive adapter architecture excluded. |
| PERM-006-A / I189 | Planned / Claimed, unactivated | Protected permission foundation remains independent. |
| I197 | Review / Claimed; human validation deferred | Non-overlapping predecessor retained in Review through Issue #302/I211. |
| I198 | Review / Claimed; implementation merged | Non-overlapping; human validation remains in Issue #302/I211. |
| I200 | Review / Claimed; human validation deferred | Non-overlapping predecessor retained in Review. |
| I201 | Review / Claimed; implementation merged | Non-overlapping predecessor retained in Review. |
| I210 | Review / Claimed; human validation deferred | Non-overlapping predecessor retained in Review. |
| I212 | Review / Claimed; implementation merged | Non-overlapping predecessor retained in Review. |
| I206 / I207 / I208 | Planned / Unclaimed | TUI steering work is unrelated and not bypassed or activated by I213. |
| I211 | Planned / Unclaimed at target; claim PR #326 open | Evidence-only cleanup. A later effective I211 claim does not authorize I213; if I211 is Active at I213 activation, wait or obtain explicit non-overlapping parallel authorization. |

The active long-running requirements task remains separately claimed and explicitly excludes
Dashboard work. Deferred Human Validation Mode explains the retained Review rows but does not waive
I213's own claim, review, security, CI, CAS, or runtime-acceptance gates.

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| WEB-001-B | WEB-001 | Ready / Unclaimed | WEB-001-A/I195 Complete; ADR-031; ADR-006; OBS-001/ADR-014; current TUI Session bridge | One local read-only Live Activity workspace with safe semantic activity + bounded live logs over SSE |

### Readiness Decision

**Ready for claim. Not Active. No Rust implementation is authorized.**

The selected slice is runnable/testable, has explicit dependencies and non-goals, has no overlapping
owner or PR, and can preserve all currently authoritative runtime/session/security boundaries.
Activation still requires the claim to be effective on `main` and a fresh iteration inventory.

## Scope

- Create the WEB-001-B read-only live observation path only.
- Add `GET /activity` to the existing Dashboard shell.
- Add `GET /activity/events` as the bounded SSE transport.
- Project safe semantic runtime activity from the existing Session bridge after authoritative Session
  ownership/persistence handling.
- Show current/recent Session/Turn/model/provider/tool lifecycle and token-usage facts only where the
  current runtime already supplies authoritative values.
- Observe existing formatted tracing/log output without changing its durable owner, rotation or
  retention contract.
- Keep fixed count/byte bounds, process-local event IDs, reconnect/reset behavior, heartbeat and
  client-count limits.
- Add only the minimum same-origin dependency-free browser script needed to consume SSE and update
  the live page; retain existing-page CSP unchanged and use a page-specific same-origin CSP for the
  live page.
- Preserve output redaction, HTML escaping/text-only DOM insertion, auth middleware, no-store and
  nosniff headers.
- Update bilingual user-facing docs truthfully.

## Architecture Contract

### Runtime Event Source

The existing AppServerSession EQ / CLI bridge remains the authority and the single canonical event
path. I213 may add one explicit named `DashboardActivityProjection` after the bridge has associated
the event with its owning Session.

It must not add an app-wide publish/subscribe abstraction. The Dashboard live feed is ephemeral
presentation state only.

Projection admission is allowlisted. Raw prompt/message text, text/thinking deltas, reasoning,
approval arguments, tool inputs/results and credentials are dropped. Unknown future event variants
drop by default.

### Logging Source

ADR-014's existing logging writer remains the retained-log authority. I213 may tee successfully
written formatted bytes into a Dashboard-only bounded line framer/ring and reapply output-boundary
redaction.

No second log file/database or structured-log schema is introduced.

### SSE Contract

Initial bounds:

- activity: 256 events / 512 KiB;
- logs: 512 lines / 1 MiB;
- entry: 16 KiB maximum;
- concurrent clients: 8;
- heartbeat: 15 seconds;
- retry: 2 seconds.

SSE IDs are monotonic only for the current process/stream instance. `Last-Event-ID` resumes within
the ring; stale/old-instance cursors receive `reset` plus current bounded state. Slow clients do not
own unbounded queues.

### Browser / CSP Contract

Existing HTML routes preserve the I195 CSP unchanged.

`/activity` may load exactly one same-origin static script and connect only to the same-origin SSE
route. No remote resources, third-party framework, Node build pipeline, token storage, analytics, or
browser automation.

### Auth-Required Mode

I213 does not solve SEC-002. The existing bearer middleware remains authoritative when
`loopback_only = false`; no token is placed in a query, cookie, storage, log, page, URL fragment, or
clipboard. The browser-live-view deliverable is the default loopback-only mode. Auth-required mode
receives regression/security coverage at the HTTP boundary but no new operator credential workflow.

## Non-Goals

- Config editor/write.
- Web approval/permission action.
- Prompt submission or tool execution.
- Session mutation/control/fork/cancel/resume/new/delete/model switch.
- Permission-pipeline behavior.
- Durable replay or multi-client Session attachment/controller semantics.
- WebSocket or AG-UI/ACP implementation.
- Remote/LAN/tunnel.
- SEC-002 token delivery or auth redesign.
- Global event bus.
- New runtime/Session/persistence authority.
- New log persistence/retention/schema.
- Raw prompt/thinking/reasoning/tool arguments/results.
- History Explorer, governance expansion, i18n.
- New external frontend/runtime dependency.

## UI Information Architecture

Dominant question:

> **What is Talos doing now?**

Reading order:

1. Live Activity title and connection state.
2. Current Session/model/Turn facts.
3. Approximately 4-6 recent semantic activity events.
4. Latest authoritative usage facts.
5. Secondary Logs disclosure with bounded plain-text filter/search.
6. Reconnect/reset state only when relevant.

Use the existing light/quiet/Nord compact-rail language. Machine facts use monospace; ordinary UI
text uses sans. No card wall, AI glow, nested glass, fake progress percentage, time estimate, or
decorative streaming motion.

## Acceptance

- Existing Dashboard pages remain behavior/content-negotiation compatible and retain their current
  security headers/CSP.
- `/activity` is a responsive, keyboard-usable page in the existing shell.
- `/activity/events` is GET-only SSE with deterministic headers, IDs, heartbeat, retry,
  disconnect/reconnect and stale-reset behavior.
- Activity uses authoritative existing facts and creates no second Session/runtime truth.
- Forbidden content categories never enter the live projection.
- Logs are derived from the existing writer, re-redacted and bounded; no durable log authority
  changes.
- Ring count/byte limits, entry limit and max-client limit are deterministic and tested.
- No POST/write/action route, WebSocket, remote listener, token-delivery path or permission bypass is
  introduced.
- Default loopback browser mode works end to end; auth-required mode retains fail-closed middleware
  semantics and no new browser credential path.
- Existing pages keep old CSP; live page allows only minimum same-origin script/connect policy.
- Dynamic browser rendering uses text insertion, never untrusted HTML.
- 320×568, 768×1024, 1440×900, 200% zoom and keyboard-only acceptance remain usable.
- README and README.zh-CN are updated.

## Planned Validation

### Focused automated

- Projection characterization for every current `SessionEvent` / nested `AgentEvent` class.
- Adversarial redaction and HTML/client injection tests.
- Ring count/byte/entry-cap and eviction tests.
- SSE ID/retry/heartbeat/reconnect/reset/disconnect/client-limit tests.
- Existing-route representation and CSP compatibility tests.
- Auth middleware ordering and no-route-leak tests.
- Logging tee/partial-line/long-line/rotation-preservation tests.
- Session-switch ownership ordering regression.
- `cargo test --locked -p talos-dashboard`.
- focused `talos-cli` logger/bridge tests.

### Workspace / governance

- `cargo check --locked --workspace`
- `cargo clippy --locked --workspace -- -D warnings`
- `cargo test --locked --workspace`
- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`

### Runtime / browser

Rebuild the real `talos` binary with the mock provider, start the normal TUI/dashboard path, trigger a
Turn and a bounded tool lifecycle, observe the Live Activity page and raw log disclosure, disconnect
and reconnect, exercise stale reset, and verify no forbidden payload is visible.

Manual browser acceptance covers 320/768/1440 widths, 200% zoom, keyboard-only navigation/focus,
filter/search, dynamic update stability, reconnect status, accessibility reading order and CSP
console behavior.

### Review

Because the implementation adds live web transport and a page-specific CSP, require independent
security-focused review of the exact implementation head before merge. I195's one-off maintainer
review override is not reused.

## Documentation To Update

- `README.md`
- `README.zh-CN.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`
- `docs/backlog/active/WEB-001-B-dashboard-live-activity-log-viewer.md`
- `docs/iterations/I213-dashboard-live-activity-log-viewer.md`
- derived `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md`, `docs/BOARD.md`

Shared derived views use union semantics: every update starts from then-current `main`, preserves all
other lanes, and changes only the I213/WEB-001-B facts required by owner truth.

## Risks And Rollback

- **Sensitive event leakage** — allowlist projection and drop unknown variants; re-redact at
  Dashboard boundary. Rollback removes the live routes/feed while leaving existing I195 pages intact.
- **Unbounded slow-client memory** — fixed rings, sequence notification and stale-reset semantics; no
  per-client unbounded queue.
- **Session ownership race** — attach observation after the existing bridge has determined event
  ownership; preserve old-EQ drain ordering during Session switches.
- **CSP regression** — existing pages keep exact current policy; only `/activity` receives the
  minimum same-origin script/connect extension.
- **Auth scope creep** — fail closed and leave `loopback_only = false` browser workflow to SEC-002.
- **Logging semantic drift** — observe existing formatted output only; do not invent structured log
  schema.
- **Iteration concurrency** — claim may be prepared while non-overlapping Review/evidence work exists,
  but activation waits for a fresh START-ITERATION inventory and any required explicit parallel
  authorization.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-20 | Planning | Fresh inventory completed at `main@15a3d4248d13d3951c823628454a2629398a9d48`; WEB-001-B/I213 not previously owned; no Dashboard PR overlap found. Claim remains ineffective/unstarted. |

## Verification Evidence

- Planning-only: repository/owner/open-PR inventory completed.
- Governance validators: pending finalized governance-claim head.
- Exact-head CI: pending finalized governance-claim head.
- Runtime evidence: not started; implementation is not authorized.

## Completion Evidence

- Completion Commit: Pending
- A future closeout must cite an already-existing implementation/merge SHA. The closeout commit may
  not self-certify completion.

## Variance And Residuals

- SEC-002 remains the sole token-delivery/auth redesign owner.
- SESSION-009/SERVER-001/PERM-006/ACP/future AG-UI remain outside this observation slice.
- History Explorer, Dashboard governance UI and i18n remain future children.

## Retrospective

- Outcome: pending.
- Documentation: pending.
- Lessons: pending.
