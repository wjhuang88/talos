# WEB-001-B: Dashboard Live Activity And Log Viewer

**Status**: Ready — I213 Planned / Unclaimed; governance claim PR pending
**Priority**: P1
**Type**: Product / Observability Story
**Parent Epic**: WEB-001
**Selected Iteration**: I213 — Planned / Unclaimed

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
| Handoff / Release Condition | Establish an effective target-branch claim for this exact bounded read-only observation slice before activation or implementation. |

The proposed claim remains ineffective until a finalized `Claimed` record with the real governance PR
number reaches `main`. No implementation branch, production code, dependency change, or activation is
authorized by this planning document.

## Identity / Goal / Value

### User

A Talos user running the normal local TUI who opens the embedded Dashboard and wants to understand
what Talos is doing now without taking control of the Session or changing runtime state.

### Goal

Add one bounded, loopback-only, read-only observation workspace to the existing Dashboard. The page
shows current/recent semantic runtime activity plus a bounded live log view, using existing
Session/observability facts and an SSE transport, while preserving the established permission,
Session, persistence, logging, redaction, and remote-access boundaries.

### Value

- make the completed read-only Dashboard useful while a Turn is running, not only as a startup
  snapshot;
- surface safe semantic activity before raw logs, consistent with the Desktop visual hierarchy;
- give local operators a bounded troubleshooting view without introducing a second runtime,
  Session store, permission path, logging store, or global event bus;
- defer write/control/remote protocol work until the shared Session/permission/AG-UI foundations own
  it.

## WEB-001 Residual Matrix

| WEB-001 residual | This child | Owner / disposition |
|---|---|---|
| Live log / SSE view | **In scope** | WEB-001-B / I213 |
| Live status / activity | **In scope, bounded** | WEB-001-B / I213; semantic read-only projection only |
| Richer history exploration | Out of scope | Future WEB-001-C or separately named child |
| Config editor / config write | Out of scope | CONF-001 plus future Dashboard write design |
| Web approvals / permission decisions | Out of scope | PERM-006 / future interactive Dashboard child |
| Prompt submission | Out of scope | SERVER-001 / SESSION-009 / AG-UI direction |
| Session actions / cancel / fork / control | Out of scope | SESSION-009 and future shared interactive foundation |
| WebSocket / richer interactive transport | Out of scope | Separate architecture decision if later needed |
| Remote / LAN / tunnel | Out of scope | REMOTE-001 / SERVER-001 plus security design |
| Project / governance UI expansion | Out of scope | Future read-only governance child |
| Dashboard UI i18n | Out of scope | Future dedicated i18n child |
| Opt-in bearer-token delivery | Out of scope | SEC-002 exclusively |

This matrix narrows the parent residuals; it does not reinterpret any excluded residual as authorized.

## Scope

### A. Live Activity

Add a read-only semantic activity projection using authoritative runtime events already flowing
through the TUI Session bridge.

The first slice may expose only safe, bounded facts such as:

- current Session identity as an opaque ID;
- active/recent Turn lifecycle;
- current model/provider identity already known to the TUI runtime;
- provider dispatch/retry state;
- tool lifecycle using tool name/provenance/summary metadata only;
- Turn completion state;
- token usage counters from authoritative `Usage` facts;
- runtime errors after output-boundary redaction.

The Dashboard does not persist these as a new source of truth. The live feed is an ephemeral,
rebuildable presentation projection.

The following payloads are explicitly not admitted into the live activity feed:

- user prompt/message content;
- `TextDelta` content;
- `ThinkingDelta` content;
- reasoning blocks/signatures/redacted-thinking payloads;
- approval arguments or decision payloads;
- raw tool-call input;
- raw tool-result content;
- authorization headers, bearer tokens, cookies, credentials, API keys, or environment secrets.

Unknown/future non-exhaustive event variants default to **drop**, not display.

### B. Live Log Viewer

Expose a bounded read-only view of the existing Talos tracing/log output.

The authoritative retained log contract remains OBS-001 / ADR-014. The Dashboard may receive a
derived copy of bytes successfully accepted by the existing logging writer, frame them into bounded
lines, reapply Dashboard output redaction, and retain only an in-memory ring for live presentation.

The first slice does not:

- create a second log file or log database;
- change rotation/retention configuration;
- claim structured JSON/span semantics that OBS-001 leaves residual;
- parse log text into new durable domain facts.

The UI provides plain-text filter/search over the bounded lines currently retained by the live page.
Structured target/level query semantics remain out of scope until a separately accepted structured
logging contract exists.

### C. Browser Presentation

Add one new Dashboard workspace:

- `GET /activity` — server-rendered Live Activity page inside the existing Dashboard shell;
- `GET /activity/events` — SSE stream for safe activity/log projection events;
- one same-origin, dependency-free static client script used only by `/activity` to consume SSE and
  update the bounded live view.

Existing Dashboard pages retain their current no-script CSP and representations. The new live page
may use a page-specific CSP limited to same-origin script and same-origin connection, for example
`default-src 'none'; style-src 'unsafe-inline'; script-src 'self'; connect-src 'self'`. No remote
resource, web font, analytics, image CDN, Node.js build pipeline, or client framework is introduced.

The live client must update dynamic text with DOM text APIs, not data-driven `innerHTML`.

## Event-Source Architecture

ADR-006 remains authoritative.

Current TUI flow stays:

```text
AppServerSession EQ
       |
       v
existing CLI bridge / persistence owner
       |
       +----> existing conversation loop (authoritative consumer path)
       |
       `----> named DashboardActivityProjection (read-only safe projection)
```

The Dashboard branch is a deterministic, explicitly named fan-out from the existing single-consumer
bridge after the event is associated with its owning Session. It is not a new `EventBus`, does not
publish commands, and does not become a runtime backplane.

The implementation should use one Dashboard-scoped live-feed object with:

- fixed-size / fixed-byte in-memory rings;
- monotonic process-local event IDs;
- a lightweight sequence notification primitive for subscribers;
- no unbounded per-client `mpsc` queue;
- no public app-wide subscribe/publish registry.

Logging uses the existing tracing writer as source. The Dashboard log observer receives only
successfully written/formatted output, line-frames it, redacts it again, and stores the bounded
derived copy in the same Dashboard-scoped observation facility.

## SSE Reconnect And Bounds Contract

Initial implementation constants are part of the testable contract and may be changed later only
through normal change control:

- activity ring: maximum **256 events** and **512 KiB** total retained payload;
- log ring: maximum **512 lines** and **1 MiB** total retained payload;
- single activity/log entry after projection/redaction: maximum **16 KiB**;
- concurrent SSE clients: maximum **8**;
- heartbeat comment: every **15 seconds** while otherwise idle;
- advertised reconnect delay: **2 seconds**.

Every SSE data event carries a monotonically increasing process-local `id`.

Reconnect behavior:

1. a client reconnecting with `Last-Event-ID` inside the retained window receives only newer events;
2. a cursor older than the retained ring receives an explicit `reset` event followed by the current
   bounded snapshot/window;
3. a process/server restart establishes a new stream instance identity; an old cursor is treated as
   stale and receives reset/current state rather than pretending replay continuity;
4. a slow client does not accumulate an unbounded private queue; if it falls behind the ring it uses
   the same reset path;
5. disconnect cancels the response task deterministically and releases its client permit.

No durable replay guarantee is claimed. SESSION-009 retains ownership of multi-client Session replay
and attachment semantics.

## Authentication / Security Boundary

- Listener remains `127.0.0.1`; this child adds no `0.0.0.0`, LAN, tunnel, remote mode, or browser
  opener.
- Routes remain GET/read-only. No POST/PUT/PATCH/DELETE business action is added.
- Existing auth middleware still runs before route/fallback information when
  `[dashboard] loopback_only = false`.
- The normal default `loopback_only = true` path is the supported browser-live-view path for this
  child.
- This child does **not** invent query-string tokens, cookies, localStorage/sessionStorage token
  transport, clipboard handoff, token logging, token-bearing URLs, or automatic browser launch.
- Therefore it does not claim to solve the existing operator/browser workflow for
  `loopback_only = false`; SEC-002 remains the only owner of that token-delivery/authentication
  residual.
- A valid explicitly authenticated HTTP test client may verify the SSE route in auth-required mode,
  but no new browser credential-delivery workflow is accepted here.
- Dashboard redaction is applied at the observation boundary in addition to the existing upstream
  logging/Session safety rules.
- No permission/sandbox status is synthesized if the current runtime has no already-authoritative
  safe fact for it.

Because this adds a live browser connection and a page-specific CSP, implementation review must
include an independent security-focused review even though the slice remains read-only.

## UI Information Architecture

The page answers one dominant question:

> **What is Talos doing now?**

Use the established light, quiet, Nord-derived shell and compact rail. Do not copy Desktop Mission /
Current Goal / Current Work product semantics into Dashboard.

Recommended reading order:

```text
Live Activity
  -> current Session / model / Turn state
  -> 4-6 most recent semantic activity items
  -> usage facts from the latest completed Turn
  -> Logs (secondary disclosure)
       -> text filter/search
       -> bounded live lines
       -> connection/reconnect state
```

Rules:

- semantic activity is primary; raw logs are visually secondary;
- normal explanatory text uses the UI sans stack; Session IDs, token counts, paths, targets and log
  lines use monospace;
- no KPI-card wall, nested cards, AI glow, decorative streaming animation, or percentage/time
  estimates;
- dynamic updates must not steal focus or reset keyboard position;
- newest activity may use a restrained marker; status is never color-only;
- screen-reader announcements are bounded and polite rather than replaying every raw log line;
- existing responsive 320 px / 768 px / 1440 px behavior remains compatible.

## Dependencies

Required and satisfied/current dependencies:

- WEB-001 parent remains Partial and explicitly retains live-log/live-activity residual work;
- WEB-001-A / I195 is Complete/Closed and supplies only the existing visual/read-only shell;
- ADR-031 supplies the loopback and read-only Dashboard boundary;
- ADR-006 supplies the explicit single-consumer / named-fan-out event architecture;
- OBS-001 and ADR-014 supply bounded local logging and retention;
- the current TUI Session bridge supplies the authoritative Session event path.

Related but **not selected or imported as authority**:

- SEC-002 — token-delivery/auth redesign residual;
- SESSION-009 — future multi-client Session architecture;
- SERVER-001 — future serve/connect adapter architecture;
- PERM-006 — permission-pipeline convergence;
- ACP-001 — agent-client protocol;
- future AG-UI work — interactive application transport;
- CONF-001 — configuration editing.

If implementation requires any of those owners to change public/runtime behavior, stop that portion
and create or wait for the separately governed foundation rather than widening WEB-001-B.

## Explicit Exclusions

- Any config edit/write route.
- Approval/permission action or permission-model change.
- Prompt submission.
- Session cancel/new/resume/fork/delete/model-switch action.
- Tool execution.
- WebSocket.
- Remote/LAN/tunnel exposure.
- Browser automation or browser opener.
- Token delivery/auth redesign.
- Durable event replay or multi-client controller/observer semantics.
- New global event bus or app-wide subscription registry.
- New Session/runtime/persistence authority.
- New log database, changed log retention, or structured-log contract.
- Raw prompt, thinking, reasoning, approval argument, tool input, or tool result display.
- History Explorer, governance expansion, Dashboard i18n, or interactive control-plane work.
- New third-party frontend/runtime dependency or Node.js toolchain.

## Readiness Decision

**Ready for a governance claim; not authorized for implementation.**

Why Ready:

1. WEB-001 explicitly owns this residual and no `WEB-001-B` or `I213` owner currently exists.
2. The first slice has a runnable user-visible deliverable in the normal default loopback mode.
3. ADR-031 and ADR-006 already bound the security/event architecture without requiring a new
   control-plane protocol.
4. OBS-001/ADR-014 provide a bounded logging source.
5. SEC-002, SESSION-009, SERVER-001 and PERM-006 can remain explicit exclusions rather than hidden
   dependencies.
6. Reconnect, memory, client-count, redaction, CSP, compatibility and test bounds are explicit.

Activation remains gated on an effective claim in `main` and a fresh non-terminal iteration / open-PR
inventory. If another iteration is Active at that point, I213 must wait or obtain explicit
non-overlapping parallel authorization rather than silently bypassing `START-ITERATION.md`.

## State / Status Owners

- Parent product direction/residuals: `docs/backlog/active/WEB-001-embedded-web-control-surface.md`.
- This child scope/readiness: this document.
- Selection/execution/completion: `docs/iterations/I213-dashboard-live-activity-log-viewer.md`.
- Token-delivery security residual: `docs/backlog/active/SEC-002-dashboard-token-delivery-boundary.md`.
- Shared derived views: `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md`,
  `docs/BOARD.md`; update owner first and preserve union semantics.

## User-Facing Documentation

Implementation acceptance must update:

- `README.md`;
- `README.zh-CN.md`;
- WEB-001 parent cross-link/status wording only as needed to record this bounded child;
- Dashboard navigation/help copy for the new Live Activity workspace.

Documentation must call the feature a local read-only live observation surface, not a remote control
plane, Session host, or permission UI.

## Required Reads

- `AGENTS.md`
- `docs/sop/START-ITERATION.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/AGENT-COLLABORATION.md`
- `docs/sop/GIT-WORKFLOW.md`
- `docs/sop/TESTING.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`
- `docs/backlog/active/WEB-001-A-dashboard-read-only-visual-shell.md`
- `docs/iterations/I195-dashboard-read-only-visual-shell.md`
- `docs/decisions/031-web-loopback-dashboard-boundary.md`
- `docs/decisions/006-event-architecture-boundary.md`
- `docs/backlog/active/SEC-002-dashboard-token-delivery-boundary.md`
- `docs/backlog/active/OBS-001-observability-prompt-assets.md`
- `docs/decisions/014-log-retention-and-rotation.md`
- `docs/backlog/active/CONF-001-config-editing.md`
- `docs/backlog/active/SERVER-001-serve-connect-protocol-adapters.md`
- `docs/backlog/active/SESSION-009-multi-client-session-architecture.md`
- `docs/design/talos-desktop/DESIGN.md`
- `crates/talos-dashboard/src/lib.rs`
- `crates/talos-cli/src/dashboard_helpers.rs`
- `crates/talos-cli/src/mode_runners.rs`
- `crates/talos-cli/src/logging.rs`
- `crates/talos-core/src/session.rs`
- `crates/talos-core/src/message.rs`

## Acceptance Matrix

| Area | Acceptance |
|---|---|
| Existing Dashboard | `/`, `/status`, `/history`, `/governance`, `/config`, `/extensions` retain their existing response negotiation, masking/redaction/escaping, navigation compatibility and current CSP behavior. |
| Live page | `GET /activity` is reachable from the shared Dashboard shell and presents current/recent safe semantic activity plus a secondary bounded log view. |
| SSE | `GET /activity/events` returns `text/event-stream`, emits bounded typed events with IDs/heartbeat/retry, and never registers a write/action route. |
| Authoritative facts | Activity is projected from the existing Session bridge/current model/session facts; Dashboard never becomes Session/runtime truth. |
| Safe projection | Raw prompt/text/thinking/reasoning/approval arguments/tool inputs/tool results/credentials never enter the activity payload. Future unknown event variants drop by default. |
| Logs | Live logs derive from existing logging output, are re-redacted, line/entry bounded, and do not create or alter durable log storage/retention. |
| Bounds | Activity/log rings, entry size, SSE clients and reconnect behavior obey the explicit constants above; no client can cause unbounded process memory growth. |
| Reconnect | In-window `Last-Event-ID` resumes newer events; stale/restarted cursors produce explicit reset/current-window behavior. |
| Disconnect | Client disconnect releases stream resources deterministically; heartbeat tasks do not leak. |
| Auth | Auth-required mode preserves current bearer middleware and rejects missing credentials before route information; no new token transport is added. Default loopback mode supports the browser live view. |
| Security headers | Existing pages retain existing CSP. `/activity` allows only the minimum same-origin script/connect policy required for SSE; no remote resources. |
| Client safety | Dynamic data is inserted as text, not data-driven HTML; no credential/local-storage path is introduced. |
| UI | Desktop-aligned light/quiet/Nord hierarchy, compact navigation, semantic activity first, logs secondary, no KPI/card wall or AI decoration. |
| Responsive | 320×568, 768×1024 and 1440×900 layouts remain usable; log overflow is bounded to its own region. |
| Accessibility | Keyboard/focus/reading order remain usable, dynamic updates do not steal focus, and live-region announcements are bounded. |
| Methods | POST/PUT/PATCH/DELETE provide no business mutation route; no WebSocket is registered. |
| Documentation | `README.md` and `README.zh-CN.md` describe the local read-only live observation boundary and the auth-required-mode limitation truthfully. |

## Test Matrix

| Layer | Required evidence |
|---|---|
| Projection unit tests | Exhaustive characterization of current `SessionEvent` / `AgentEvent` projection; safe variants map to bounded semantic fields, forbidden payload-bearing variants are dropped or stripped. |
| Redaction adversarial tests | API keys, bearer headers, query tokens, passwords, cookies, authorization strings, HTML/script-like strings and long payloads never survive the output boundary. |
| Ring/bounds tests | Count cap, byte cap, per-entry cap, eviction order and monotonic event IDs are deterministic. |
| SSE protocol tests | Content type, IDs, event type, retry, heartbeat, in-window replay, stale reset, restart reset, disconnect cancellation and max-client rejection. |
| HTTP/auth tests | Missing token rejection in auth-required mode, valid-header test-client access, no route leakage before auth, and all non-GET mutation attempts rejected. |
| CSP/resource tests | Existing HTML CSP snapshots unchanged; live page CSP contains only required same-origin `script-src` / `connect-src`; no external resource URLs. |
| Client rendering tests | Dynamic payloads use text insertion, filtering/search stays within the bounded client window, no `innerHTML` data path. |
| Logging tests | Existing writer behavior/rotation unchanged; live observer sees only successfully written bounded lines, handles partial writes/line framing, reapplies redaction, and does not persist a second copy. |
| Session-switch tests | Old actor events remain associated with the old owning Session until its EQ drains; Dashboard projection follows the same ownership order and does not jump early to the new Session. |
| Focused crate tests | `cargo test --locked -p talos-dashboard`; focused `talos-cli` tests for logger/bridge wiring. |
| Workspace validation | `cargo check --locked --workspace`; `cargo clippy --locked --workspace -- -D warnings`; `cargo test --locked --workspace`; `./scripts/release_preflight.sh`. |
| Governance | `scripts/validate_project_governance.sh .`; `bash scripts/validate_collaboration_claims.sh .`; `git diff --check`. |
| Runtime acceptance | Rebuilt real `talos` TUI with mock provider: open Dashboard, trigger a Turn/tool lifecycle, observe live semantic activity/logs, disconnect/reconnect, verify stale reset and absence of sensitive payloads. |
| Browser acceptance | 320/768/1440 widths, 200% zoom, keyboard-only navigation, focus, live updates, filter/search, reconnect indicator and CSP console inspection. |
| Security review | Independent review of SSE/auth/CSP/redaction/event projection before implementation merge. |

## Residual Work Destination

- searchable/paginated durable Session/tool-call history -> future WEB-001-C;
- browser actions/approvals/prompt/session control -> future interactive child after shared
  Session/permission/AG-UI foundations;
- authenticated browser token delivery -> SEC-002;
- remote/LAN -> REMOTE-001 / SERVER-001 security architecture;
- Dashboard localization -> dedicated i18n child;
- structured logs/shared span schema -> existing observability residual owner.
