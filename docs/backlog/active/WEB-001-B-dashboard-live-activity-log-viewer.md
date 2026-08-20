# WEB-001-B: Dashboard Live Activity And Log Viewer

**Status**: Ready — I213 Planned / Claimed via governance PR #327; ineffective while open
**Priority**: P1
**Type**: Product / Observability Story
**Parent Epic**: WEB-001
**Selected Iteration**: I213 — Planned / Claimed via governance PR #327; ineffective while open

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
| Handoff / Release Condition | Merge the finalized governance-only claim to `main`, refresh inventory, and activate I213 before creating any implementation branch. |

This proposed `Claimed` record is **ineffective while PR #327 is open**. It authorizes no Rust,
dependency, runtime, protocol, or UI implementation before target-branch merge and later activation.

## Goal And Value

Deliver one local, loopback-only, GET/read-only observation workspace that answers:

> **What is Talos doing now?**

The page combines safe semantic runtime activity with a bounded live log view. It consumes existing
Session/logging facts, improves the completed I195 visual foundation, and does not become a runtime,
Session, permission, persistence, logging, or control-plane authority.

## WEB-001 Residual Matrix

| WEB-001 residual | WEB-001-B disposition |
|---|---|
| Live log / SSE | **In scope** |
| Live status / activity | **In scope, bounded safe projection only** |
| Richer history exploration | Out — future WEB-001-C |
| Config editor/write | Out — CONF-001 / later write child |
| Web approvals / permission decisions | Out — PERM-006 / later interactive child |
| Prompt submission | Out — shared Session/serve/AG-UI foundation first |
| Session actions / cancel / fork / control | Out — SESSION-009 / later interactive child |
| WebSocket / richer interactive transport | Out |
| Remote / LAN / tunnel | Out — REMOTE-001 / SERVER-001 security route |
| Project / governance UI expansion | Out — future read-only child |
| Dashboard UI i18n | Out — dedicated child |
| Bearer-token delivery/auth redesign | Out — SEC-002 exclusively |

No excluded residual is reinterpreted as implementation authority.

## Bounded Product Scope

### Live Activity

Project only already-authoritative safe facts from the existing TUI Session path:

- opaque current Session ID;
- active/recent Turn lifecycle;
- current model/provider identity already known to the runtime;
- provider dispatch/retry progress;
- tool lifecycle using tool name/provenance/safe summary metadata only;
- Turn completion state;
- authoritative token `Usage` counters;
- redacted runtime errors.

The live feed is ephemeral and rebuildable. It is never durable Session/runtime truth.

The projection **must not admit** user prompt/message content, `TextDelta`, `ThinkingDelta`,
reasoning blocks/signatures, approval arguments, raw tool-call input, raw tool-result content,
authorization headers, bearer tokens, cookies, credentials, API keys, or environment secrets.
Unknown/future non-exhaustive event variants default to **drop**.

### Live Log Viewer

OBS-001 / ADR-014 remain the retained-log authority. I213 may observe bytes successfully accepted by
the existing tracing/log writer, frame them into bounded lines, reapply Dashboard redaction, and keep
only an in-memory live ring.

No second log file/database, changed retention, or structured JSON/span contract is added. The first
viewer provides plain-text filter/search over the bounded lines currently retained by the page.

### Browser Surface

- `GET /activity` — Live Activity page inside the existing Dashboard shell.
- `GET /activity/events` — bounded SSE feed carrying safe activity/log events.
- one same-origin, dependency-free static client script for the live page only.

Existing I195 pages keep their current no-script CSP. `/activity` may use a page-specific minimum
policy such as `default-src 'none'; style-src 'unsafe-inline'; script-src 'self'; connect-src 'self'`.
No remote resource, web font, analytics, client framework, Node build pipeline, or browser automation
is introduced. Dynamic data is inserted with text DOM APIs, never data-driven `innerHTML`.

## Event-Source Architecture

ADR-006 remains authoritative:

```text
AppServerSession EQ
       |
       v
existing CLI bridge / persistence owner
       |
       +----> existing conversation loop
       |
       `----> DashboardActivityProjection (named, read-only, allowlisted)
```

The Dashboard projection is attached only after the bridge has associated each event with its owning
Session. It does not create a global `EventBus` or public app-wide subscribe/publish registry.

A Dashboard-scoped feed owns fixed count/byte rings plus a lightweight sequence notifier. SSE clients
read by cursor; they do not receive unbounded private queues.

Logs use the existing tracing writer as source. The Dashboard log observer receives only successfully
written/formatted output and remains a derived in-memory presentation copy.

## SSE Reconnect And Bounds

Initial testable limits:

- activity ring: **256 events / 512 KiB**;
- log ring: **512 lines / 1 MiB**;
- projected entry: **16 KiB max**;
- concurrent SSE clients: **8 max**;
- idle heartbeat: **15 s**;
- advertised reconnect delay: **2 s**.

Each data event has a monotonic process-local ID and stream-instance identity.

- in-window `Last-Event-ID` -> replay only newer retained events;
- stale cursor -> explicit `reset` plus current bounded window;
- old process/stream instance -> reset, never fake durable continuity;
- slow client that falls behind -> same reset path, no unbounded queue;
- disconnect -> deterministic task/client-permit release.

No durable replay or SESSION-009 multi-client attachment guarantee is claimed.

## Security Boundary

- Listener remains `127.0.0.1`; no remote/LAN/tunnel bind.
- Only GET/read-only routes are added; no POST/PUT/PATCH/DELETE business action.
- Existing auth middleware remains before route/fallback disclosure when
  `[dashboard] loopback_only = false`.
- Default `loopback_only = true` is the supported browser-live-view path.
- No query token, cookie, localStorage/sessionStorage credential, clipboard handoff, token log,
  token-bearing URL, or browser opener is introduced.
- Therefore I213 does not claim to solve the auth-required browser workflow; SEC-002 remains the
  exclusive token-delivery/auth redesign owner.
- Auth-required mode is tested fail-closed with explicit HTTP Authorization headers only.
- Dashboard redaction is applied again at the observation boundary.
- Permission/sandbox facts are shown only if an already-authoritative safe source exists; no new
  PERM-006 state is synthesized.

Because I213 adds live web transport and a page-specific CSP, its exact implementation head requires
an independent security-focused review. I195's one-off review override is not reused.

## UI Information Architecture

Use the existing light-first, quiet, Nord-derived compact shell. Do not copy Desktop Mission /
Current Goal / Current Work product semantics.

Reading order:

1. Live Activity title + connection state.
2. Current Session/model/Turn facts.
3. Approximately 4–6 newest semantic activity items.
4. Latest authoritative usage facts.
5. Logs as a secondary disclosure with bounded text filter/search.
6. Reconnect/reset state only when relevant.

Ordinary UI text uses sans; Session IDs, token counts, paths, targets, and log lines use monospace.
No KPI-card wall, nested glass cards, AI glow, fake percentage/time estimates, or decorative
streaming animation. Updates must not steal focus; status is not color-only; screen-reader
announcements stay bounded/polite.

## Dependencies And Explicit Exclusions

Required/current foundations:

- WEB-001 remains Partial and owns this residual.
- WEB-001-A / I195 is Complete/Closed and supplies only the read-only visual shell.
- ADR-031 supplies the loopback/GET-only Dashboard security boundary.
- ADR-006 supplies the single-consumer / named-fan-out event boundary.
- OBS-001 / ADR-014 supply bounded logging/retention.
- the current TUI bridge supplies the authoritative Session event path.

Related owners remain **outside** this slice: SEC-002, SESSION-009, SERVER-001, PERM-006, ACP-001,
CONF-001, future AG-UI, REMOTE-001.

Explicitly excluded: config writes, approvals, prompt submission, tool execution, Session mutation,
WebSocket, remote/LAN/tunnel, token delivery, durable replay, multi-client controller semantics,
global event bus, new Session/runtime/persistence authority, new log persistence/schema, raw
prompt/thinking/reasoning/tool payload display, History Explorer, governance expansion, Dashboard
i18n, and new third-party frontend/runtime dependencies.

If implementation needs an excluded owner to change public/runtime behavior, stop that portion rather
than widening WEB-001-B.

## Readiness Decision

**Ready for governance claim; not authorized for implementation.**

Fresh inventory found no existing `WEB-001-B`, `I213`, or overlapping Dashboard PR. The deliverable
is runnable/testable in the default loopback mode; dependencies and exclusions are explicit; SSE,
memory, client, redaction, auth, CSP, reconnect, accessibility, and rollback boundaries are testable.

Activation remains gated on an effective target-branch claim and another fresh non-terminal
iteration/open-PR inventory. If another iteration is Active then, I213 waits or requires explicit
non-overlapping parallel authorization.

## Acceptance Matrix

| Area | Acceptance |
|---|---|
| Existing Dashboard | `/`, `/status`, `/history`, `/governance`, `/config`, `/extensions` retain current negotiation, masking/redaction/escaping, navigation compatibility, and CSP behavior. |
| Live page | `/activity` presents safe current/recent semantic activity plus secondary bounded logs. |
| SSE | `/activity/events` is GET-only `text/event-stream` with IDs, retry, heartbeat, reconnect/reset and bounded clients. |
| Authority | Activity comes from the existing Session bridge/current runtime facts; Dashboard is never Session/runtime truth. |
| Safe projection | Prompt/text/thinking/reasoning/approval arguments/tool input/tool result/credentials never enter live activity; unknown variants drop. |
| Logs | Derived from existing logging output, re-redacted and bounded; no durable log owner/retention change. |
| Bounds | Count, byte, per-entry, client and reconnect limits are deterministic and tested. |
| Auth | Missing credentials fail closed in auth-required mode; no token-delivery path is introduced; default loopback browser view works. |
| CSP/client | Existing CSP unchanged; live page permits only minimum same-origin script/connect; dynamic payloads are inserted as text. |
| Methods | No business mutation route, WebSocket, remote listener, permission bypass, or browser automation. |
| UI/A11y | 320×568, 768×1024, 1440×900, 200% zoom and keyboard-only use remain viable; dynamic updates do not steal focus. |
| Docs | `README.md` and `README.zh-CN.md` describe the local read-only boundary and auth-required limitation truthfully. |

## Test Matrix

| Layer | Required evidence |
|---|---|
| Projection | Characterize every current `SessionEvent` / nested `AgentEvent`; allowlist safe fields and assert forbidden payloads absent. |
| Redaction | Adversarial API key/token/password/cookie/header/query/HTML/long-value fixtures. |
| Rings | Count/byte/entry caps, eviction order, monotonic IDs. |
| SSE | Headers, IDs/types, retry, heartbeat, in-window replay, stale/restart reset, disconnect, max-client rejection. |
| HTTP/auth | Auth ordering/no route leak, valid explicit-header client, all non-GET mutations rejected. |
| CSP/client | Existing-page CSP snapshots unchanged; live page same-origin-only; no data-driven `innerHTML`. |
| Logging | Successful-write observation, partial-line framing, long-line cap, redaction, existing rotation behavior unchanged. |
| Session switch | Old EQ drains under old Session ownership before projection switches. |
| Focused | `cargo test --locked -p talos-dashboard` plus focused `talos-cli` logger/bridge tests. |
| Workspace | `cargo check --locked --workspace`; strict workspace Clippy; workspace tests; `./scripts/release_preflight.sh`. |
| Governance | both governance validators and `git diff --check`. |
| Runtime/browser | real rebuilt mock TUI: Turn/tool activity, logs, disconnect/reconnect/stale reset, no sensitive payload; responsive/zoom/keyboard/CSP acceptance. |
| Review | independent security-focused exact-head review before implementation merge. |

## State / Documentation Owners

- Parent/residuals: `docs/backlog/active/WEB-001-embedded-web-control-surface.md`.
- Child scope: this document.
- Iteration execution/completion: `docs/iterations/I213-dashboard-live-activity-log-viewer.md`.
- Token-delivery security: `docs/backlog/active/SEC-002-dashboard-token-delivery-boundary.md`.
- Derived views: `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md`, `docs/BOARD.md`
  using union semantics.
- Implementation acceptance later updates `README.md`, `README.zh-CN.md`, and the parent cross-link.

## Required Reads

`AGENTS.md`; `docs/sop/START-ITERATION.md`; `docs/sop/ITERATION-WORKFLOW.md`;
`docs/sop/AGENT-COLLABORATION.md`; `docs/sop/GIT-WORKFLOW.md`; `docs/sop/TESTING.md`; WEB-001;
WEB-001-A/I195; ADR-031; ADR-006; SEC-002; OBS-001; ADR-014; CONF-001; SERVER-001; SESSION-009;
`docs/design/talos-desktop/DESIGN.md`; current Dashboard/CLI bridge/logging/Session event code.
