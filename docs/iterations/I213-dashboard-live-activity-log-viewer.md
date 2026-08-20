# Iteration I213: Dashboard Live Activity And Log Viewer

> Document status: Planned
> Published plan date: 2026-08-20
> Planned objective: deliver one bounded, loopback-only, GET/read-only realtime Dashboard observation
> workspace over authoritative Session/logging facts, with SSE reconnect/bounds and no
> write/control/remote/session-authority expansion.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: in a rebuilt real Talos TUI using the mock provider, the default local Dashboard
> can show safe semantic Turn/tool/usage activity and bounded live logs, survive disconnect/reconnect,
> and still expose no Dashboard action route.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-5.6 Sol — Dashboard live activity governance session 2026-08-20 |
| Work Slice | Define and, only after target-branch claim activation, implement WEB-001-B / I213 as one bounded default-loopback GET/read-only Dashboard Live Activity workspace: project safe semantic Session/Agent activity from the existing CLI bridge, expose bounded existing-log observations over SSE with deterministic reconnect/reset/memory/client limits, and preserve current auth/redaction/Session/permission/logging authorities. No write/control/remote/token-delivery/WebSocket/global-bus or new Session/runtime/persistence authority. |
| Claimed At | 2026-08-20 |
| Source Issue | None |
| Governance Claim PR | #327 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #327 proposes this exact bounded claim. Exact-head CI, independent claim review, both governance validators, and merge-time CAS are required before target-branch effect; while #327 is open this record is not effective authority. |
| Implementation PR | Not started |
| Last Updated | 2026-08-20 |
| Handoff / Release Condition | Merge the finalized governance-only claim to `main`, refresh inventory, and activate I213 before any implementation branch is created. |

This proposed claim is ineffective while #327 is open. I213 remains Planned; no Rust implementation
is authorized.

## Published Baseline

### Fresh Inventory — 2026-08-20

Planning target:

`main@15a3d4248d13d3951c823628454a2629398a9d48`

This is newer than the handoff's historical `ec7945156d2f72eb96af58d8cd9155f3bc5f37f1`
baseline. Refresh again before claim merge, activation, implementation creation, and merge-time CAS.

Open PR disposition:

| PR | Scope | I213 disposition |
|---|---|---|
| #326 | I211 / VALIDATION-002 claim | Non-overlapping evidence/validation lane; recheck before activation. |
| #120 / #121 | archival recovery Drafts | Do-not-merge provenance; no Dashboard authority. |
| #327 | this governance-only claim | Owns only the proposed WEB-001-B/I213 claim; ineffective until merged. |

Fresh search found no prior `WEB-001-B`, `I213`, or overlapping Dashboard claim/implementation PR.

Relevant owner disposition:

| Owner | State | I213 disposition |
|---|---|---|
| WEB-001 | Partial | Parent explicitly retains live log/SSE and live activity residual. |
| WEB-001-A / I195 | Complete / Closed | Consume shell only; do not reuse authorization. |
| SEC-002 | Refinement / Unclaimed | Token delivery/auth redesign stays separate. |
| OBS-001 | Complete | Consume bounded logging baseline only. |
| CONF-001 | Partial | Config writes excluded. |
| SESSION-009 | Refinement / Unclaimed | Multi-client replay/control excluded. |
| SERVER-001 | Intake | Serve/connect/interactive architecture excluded. |
| PERM-006-A / I189 | Planned / Claimed, unactivated | Permission foundation remains independent/protected. |
| I197/I198/I200/I201/I210/I212 | Review / Claimed | Deferred-human rows are non-overlapping and do not authorize I213. |
| I206/I207/I208 | Planned / Unclaimed | Unrelated TUI steering work; not activated/bypassed by I213. |
| I211 | Planned/Unclaimed on baseline; claim #326 open | If Active at I213 activation, wait or obtain explicit non-overlap authorization. |

The long-running requirements task explicitly excludes Dashboard work. Deferred Human Validation Mode
does not waive I213's own claim, review, security, CI, CAS, or runtime acceptance.

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `WEB-001-B` | `WEB-001` | Ready / proposed Claimed via #327 | WEB-001-A/I195 Complete; ADR-031; ADR-006; OBS-001/ADR-014; current TUI bridge | Local read-only Live Activity workspace with safe semantic activity + bounded live logs over SSE |

### Readiness

**Ready for claim. Not Active.**

The slice has one runnable/testable deliverable, explicit authoritative sources, fixed SSE/memory
bounds, explicit security exclusions, no overlapping owner, and a rollback that leaves I195 intact.

## Scope

- `GET /activity` in the existing Dashboard shell.
- `GET /activity/events` bounded SSE.
- Named safe projection from the existing Session bridge after Session ownership/persistence handling.
- Current/recent Session/Turn/model/provider/tool lifecycle and authoritative token usage only.
- Bounded derived observation of existing tracing/log output; no retention/schema change.
- Activity ring 256 events / 512 KiB; logs 512 lines / 1 MiB; entry 16 KiB max; 8 SSE clients;
  heartbeat 15 s; retry 2 s.
- In-window `Last-Event-ID` resume; stale/old-instance reset; no unbounded per-client queue.
- One same-origin dependency-free live-page script; existing-page CSP unchanged; live page
  same-origin `script-src`/`connect-src` only.
- Preserve loopback bind, GET-only routes, auth middleware, redaction, no-store/nosniff, text-only DOM
  insertion, responsive/accessibility baseline.
- Bilingual user-facing documentation.

## Non-Goals

Config write/editor; approvals/permission actions; prompt/tool submission; Session mutation/fork/
cancel/resume/new/delete/model switch; WebSocket/AG-UI/ACP implementation; remote/LAN/tunnel;
SEC-002 token delivery; durable replay or multi-client Session authority; global event bus; new
runtime/Session/persistence authority; new log persistence/retention/schema; raw prompt/thinking/
reasoning/approval/tool input/result display; History Explorer; governance expansion; Dashboard i18n;
new third-party frontend/runtime dependency.

## Architecture / Security Gates

ADR-006 remains authoritative: keep the current EQ bridge as canonical consumer and add only an
explicit named Dashboard projection. Unknown event variants drop by default.

OBS-001/ADR-014 remain the log authority: observe only successfully written formatted output, line
frame it, redact it again, and keep a bounded in-memory copy.

ADR-031 remains the web boundary: `127.0.0.1`, GET/read-only, no action route. Default
`loopback_only = true` is the supported browser-live-view deliverable. `loopback_only = false`
retains current bearer middleware, but I213 adds no query/cookie/storage/clipboard/log/browser token
delivery path; SEC-002 remains authoritative.

Live transport + page-specific CSP require independent security-focused exact-head review. The I195
one-off review override is not inherited.

## UI Information Architecture

Dominant question: **What is Talos doing now?**

1. connection state;
2. current Session/model/Turn;
3. about 4–6 recent semantic activity items;
4. latest authoritative usage;
5. secondary Logs disclosure with bounded text filter/search;
6. reconnect/reset status only when relevant.

Keep light/quiet/Nord compact-rail styling; human text sans, machine facts monospace; no KPI wall,
nested cards/glass, AI glow, fake progress/time estimates, or focus-stealing live updates.

## Acceptance

- Existing six I195 pages retain representation negotiation, masking/redaction/escaping and existing
  CSP behavior.
- `/activity` is responsive/keyboard-usable and `/activity/events` is bounded GET-only SSE.
- Live facts come from existing authorities; Dashboard creates no second Session/runtime truth.
- Prompt/text/thinking/reasoning/approval arguments/tool inputs/results/credentials never enter the
  activity feed; unknown variants drop.
- Live logs are re-redacted bounded derivatives; durable logging/rotation remains unchanged.
- Count/byte/entry/client limits, heartbeat, reconnect/reset and disconnect cleanup are deterministic.
- Default loopback browser view works; auth-required mode stays fail-closed with no new token path.
- Existing pages keep old CSP; live page permits only minimum same-origin script/connect.
- Dynamic data uses text insertion, not untrusted HTML.
- No write/action route, WebSocket, remote bind, permission bypass, browser automation, or global bus.
- 320×568, 768×1024, 1440×900, 200% zoom and keyboard-only acceptance remain usable.
- `README.md` and `README.zh-CN.md` describe boundaries truthfully.

## Planned Validation

- Projection characterization across current `SessionEvent` / nested `AgentEvent` variants.
- Adversarial redaction/injection/oversize fixtures.
- Ring count/byte/entry cap, eviction and event-ID tests.
- SSE headers/type/ID/retry/heartbeat/replay/reset/restart/disconnect/max-client tests.
- Auth ordering/no-route-leak and all non-GET method rejection.
- Existing-page and live-page CSP/resource tests; no data-driven `innerHTML`.
- Logging successful-write/partial-line/long-line/redaction/rotation-regression tests.
- Session-switch old-EQ ownership-order regression.
- `cargo test --locked -p talos-dashboard`.
- focused `talos-cli` logger/bridge tests.
- `cargo check --locked --workspace`.
- `cargo clippy --locked --workspace -- -D warnings`.
- `cargo test --locked --workspace`.
- `./scripts/release_preflight.sh`.
- `scripts/validate_project_governance.sh .`.
- `bash scripts/validate_collaboration_claims.sh .`.
- `git diff --check`.
- rebuilt real mock TUI + browser acceptance for activity/logs/reconnect/redaction/responsive/zoom/
  keyboard/CSP.
- independent security-focused exact-head review.

## Documentation To Update

Implementation acceptance updates `README.md`, `README.zh-CN.md`, WEB-001 parent cross-link,
WEB-001-B/I213 owners, and union-preserving derived views in `docs/backlog/PRODUCT-BACKLOG.md`,
`docs/iterations/README.md`, and `docs/BOARD.md`.

## Risks And Rollback

- Sensitive payload leak -> allowlist/drop-default + boundary redaction; remove live routes/feed to
  roll back without touching I195 pages.
- Slow-client memory growth -> bounded rings/cursors/reset, no unbounded client queue.
- Session ownership race -> projection after existing bridge ownership, preserve old EQ drain.
- CSP regression -> existing pages unchanged; page-specific minimum policy only.
- Auth scope creep -> fail closed; SEC-002 owns auth delivery.
- Log semantic drift -> observe current formatted writer only.
- Iteration concurrency -> fresh START-ITERATION inventory before activation; wait or require explicit
  parallel authorization.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-20 | Planning | Fresh inventory at `main@15a3d4248d13d3951c823628454a2629398a9d48`; no prior WEB-001-B/I213 owner or Dashboard PR overlap. |
| 2026-08-20 | Claim proposal | Draft PR #327 opened from governance-only planning head; this proposed Claimed record is ineffective until target-branch merge. |

## Verification Evidence

- Fresh repo/open-PR/owner inventory: complete.
- Governance validators: required on finalized exact #327 head; no local clone was available in the
  connector environment, so no local pass is claimed.
- Exact-head CI/review: pending finalized claim head.
- Runtime evidence: not started; implementation remains unauthorized.

## Completion Evidence

- Completion Commit: Pending
- Future closeout must cite an already-existing implementation/merge SHA; it may not self-certify.

## Variance And Residuals

SEC-002, SESSION-009, SERVER-001, PERM-006, ACP/future AG-UI, History Explorer, governance UI,
Dashboard i18n, remote/LAN and structured-log evolution remain separately owned residuals.

## Retrospective

- Outcome: pending.
- Documentation: pending.
- Lessons: pending.
