# I213 Implementation Dispatch — 2026-08-22

## Purpose

This is a non-authoritative execution handoff packet for the already-effective `WEB-001-B / I213`
owner. It exists so a separate tool-equipped implementation session can pick up the work without
reconstructing the governance history. Story scope remains authoritative in
`docs/backlog/active/WEB-001-B-dashboard-live-activity-log-viewer.md`; iteration execution remains
authoritative in `docs/iterations/I213-dashboard-live-activity-log-viewer.md`.

Execution coordination Issue: #366.

## Effective Authority

- WEB-001-B claim: PR #327 merged as `667472145ffa7644a7f049472d7389876b8aaaf9`.
- I213 activation: PR #363 merged as `e578f4196092d0884ab7dd3321fb62acb3b88257`.
- I219 / PERM-006-B activation: PR #359 merged as
  `781bb1122d2c323854d5d65aed354d35d045e383`.
- The maintainer approved I213 running in parallel with I219 on 2026-08-22 only under the explicit
  non-overlap protection recorded by #363.
- At dispatch, open PRs are archival Drafts #120/#121 only; no I213 or I219 implementation PR is
  open. This is a point-in-time snapshot and must be refreshed before shared-file edits and merge.

The implementation branch must start from #363's merge commit or a later fresh `main`.

## Known Post-Merge Documentation Drift

The #363 activation proposal intentionally used conditional wording before merge. After merge,
`main` still contains some of that conditional wording in WEB-001-B/I213 and derived current-state
views. The I219 iteration header likewise still contains pre-merge wording for #359.

These are reconciliation defects, not reversals of the target-branch merge facts above.

The implementation session must, before its first push, locally reconcile I213's owner/derived-view
truth as part of the coherent candidate. It must preserve owner-first union semantics and must not
edit I219/PERM-006 owner documents from the I213 lane. If I219's stale wording still exists, report
it to that lane rather than absorbing it into I213.

Do not move I213 to `Review` until an implementation candidate and the required local evidence
actually exist.

## Deliverable

Implement one default-loopback, GET/read-only Dashboard observation workspace answering:

> **What is Talos doing now?**

Required routes:

- `GET /activity`
- `GET /activity/events` using bounded Server-Sent Events

The page combines safe semantic Session/Agent activity with a bounded live observation of existing
tracing/log output. It introduces no write/control/remote/token-delivery/Session-authority path.

### Safe activity projection

Allow only already-authoritative safe facts:

- opaque Session ID;
- Turn lifecycle;
- current model/provider identity;
- provider dispatch/retry progress;
- tool lifecycle using tool name, provenance and safe summary metadata only;
- Turn completion;
- authoritative `Usage`;
- redacted runtime errors.

Never admit prompt/message content, `TextDelta`, `ThinkingDelta`, reasoning blocks/signatures,
approval arguments, raw tool input/result, authorization headers, bearer tokens, cookies,
credentials, API keys, environment secrets, or unknown future event variants. Unknown variants drop
by default.

Preserve ADR-006 ownership:

```text
AppServerSession EQ
       |
       v
existing CLI bridge / persistence owner
       |
       +----> existing conversation loop
       |
       `----> DashboardActivityProjection
```

The projection is a named read-only Dashboard subscriber after Session association. It is not a
public/global event bus and does not become Session/runtime truth.

### Live logs

Observe only successfully written/formatted output from the existing tracing/log writer, frame it
into lines, re-redact at the Dashboard boundary and retain only a bounded in-memory presentation
copy. Do not introduce another log database/file/schema/retention owner.

### Fixed bounds

- activity ring: 256 events / 512 KiB;
- log ring: 512 lines / 1 MiB;
- projected entry: 16 KiB max;
- max concurrent SSE clients: 8;
- heartbeat: 15 s;
- advertised reconnect delay: 2 s;
- monotonic process-local event IDs plus stream identity;
- `Last-Event-ID` replay only inside the retained window;
- stale cursor, old stream instance or slow-client overrun: explicit reset plus current bounded
  window;
- deterministic disconnect/client-permit cleanup.

No durable replay or multi-client Session authority is claimed.

## Security And Browser Boundary

- listener remains `127.0.0.1`;
- add GET/read-only routes only;
- existing auth middleware remains before route/fallback disclosure when `loopback_only=false`;
- default `loopback_only=true` is the supported browser-live-view path;
- no query token, cookie, localStorage/sessionStorage credential, clipboard/opener token transfer or
  token-bearing URL/log;
- SEC-002 exclusively owns token delivery/auth redesign;
- auth-required tests use an explicit `Authorization` header only;
- existing Dashboard pages retain their existing no-script CSP;
- the live page may use only a minimum same-origin policy, e.g.
  `default-src 'none'; style-src 'unsafe-inline'; script-src 'self'; connect-src 'self'`;
- client code is dependency-free, same-origin static JS;
- dynamic payloads use text DOM insertion only, never untrusted data-driven `innerHTML`.

Because I213 adds live web transport and page-specific CSP, the exact implementation head requires
an independent security-focused review. The I195 review override does not carry forward.

## UI Acceptance

Keep the existing light-first, quiet, Nord-derived compact shell. Reading order:

1. connection state;
2. current Session/model/Turn;
3. roughly 4–6 newest semantic activity items;
4. latest authoritative usage;
5. secondary Logs disclosure with bounded text filter/search;
6. reconnect/reset state only when relevant.

Normal text is sans; machine facts are monospace. No KPI wall, nested glass cards, AI glow, fake
progress/time estimates or focus-stealing updates. Status must not be color-only and screen-reader
announcements must remain bounded/polite.

Acceptance viewports: 320×568, 768×1024, 1440×900, 200% zoom and keyboard-only.

## I219 Parallel Protection

I219 exclusively owns permission/grant/compiler/store/proposal semantics, Session grant ownership,
Once/Session authorities, pre-admission fencing, scope/provenance matching, Deny dominance,
permission wrappers and the ADR-066 public permission API/schema migration.

I213 must not:

- modify `crates/talos-permission/**`;
- modify I219/PERM-006 owner documents;
- change permission/grant/store/proposal/policy precedence or approval decisions;
- change permission Runtime APIs/wrappers, `/auto`, sandbox/fallback policy;
- synthesize, infer or cache new permission truth;
- modify `crates/talos-runtime/**` for permission behavior.

Production work should stay primarily in `crates/talos-dashboard/**`, with only additive
Dashboard-specific CLI observation glue where necessary.

Before the first shared `talos-cli` production edit, refresh I219's current implementation
PR/branch changed-file inventory. Same production-file overlap is a stop/governance gate, not an
ordinary merge conflict. Repeat the fresh main/open-PR/changed-file/owner inventory before merge.

## GOV-008 Local-Convergence Contract

The implementation session must not use GitHub CI as an edit-by-edit development loop. Before the
first implementation push it must locally converge one coherent candidate:

1. refresh `main` and create the implementation branch from #363 merge or later;
2. read the required SOPs/owners/ADRs and characterize current event/log authority;
3. implement architecture, code and tests locally;
4. run focused then full required validation locally;
5. capture applicable rebuilt mock-TUI/browser evidence locally;
6. update `README.md`, `README.zh-CN.md`, I213 owner truth and union-preserving derived views;
7. move I213 to `Review` only when that stable candidate exists;
8. inspect the diff for unrelated files, secrets, generated residue and scope drift;
9. only then push/open one stable implementation PR.

Suggested branch name, if still unused: `feat/i213-dashboard-live-activity`.

## Required Validation

At minimum:

- characterize every current `SessionEvent` / nested `AgentEvent`; safe allowlist plus forbidden
  payload absence tests;
- adversarial API-key/token/password/cookie/header/query/HTML/long-value redaction fixtures;
- ring count/byte/entry caps, eviction order and event-ID tests;
- SSE headers/types/IDs/retry/heartbeat/replay/reset/restart/disconnect/max-client tests;
- auth ordering/no-route-leak, explicit-header auth client and all non-GET rejection;
- existing-page CSP regression, live-page same-origin CSP/resource tests and no data-driven
  `innerHTML`;
- successful-write log observation, partial-line framing, long-line cap, redaction and unchanged
  rotation behavior;
- Session-switch regression proving the old EQ drains under old Session ownership before projection
  switches;
- `cargo test --locked -p talos-dashboard`;
- focused `talos-cli` logger/bridge tests when relevant;
- `cargo check --locked --workspace`;
- `cargo clippy --locked --workspace -- -D warnings`;
- `cargo test --locked --workspace`;
- `./scripts/release_preflight.sh`;
- `scripts/validate_project_governance.sh .`;
- `bash scripts/validate_collaboration_claims.sh .`;
- `git diff --check`;
- rebuilt real mock-TUI + browser acceptance for Turn/tool/usage activity, logs,
  disconnect/reconnect/stale reset, sensitive-payload absence, responsive/zoom/keyboard/CSP.

The repository toolchain is Rust 1.97.0 with `rustfmt` and `clippy`.

## Stable Candidate, Review And Merge

The implementation PR must link WEB-001-B, I213, claim #327, activation #363 and Issue #366. It
must state non-goals/residuals and carry the locally converged evidence.

The exact implementation head requires full exact-head/base CI and an independent security-focused
review covering SSE, CSP, auth ordering, redaction/data minimization, memory/client bounds,
reconnect/reset and authority boundaries, with explicit `APPROVE` or `REQUEST CHANGES`.

Any substantive head change invalidates the prior exact-head CI/review. Batch corrections locally,
rerun the complete checkpoint and then push the next stable head.

Immediately before merge, perform merge-time CAS against fresh `main`, effective owners, open
PRs/changed files, I219 overlap, dependencies, exact-head checks/review and unresolved blocking
feedback. Merge with expected-head SHA.

Do not mark I213 Complete in the implementation PR. Closure may cite only an implementation/evidence
commit already present on `main` as `Completion Commit`.

## Strict Exclusions

No config writes, approvals, prompt/session actions, tool execution, Session mutation/fork/cancel/
resume/new/delete/model switch, WebSocket, remote/LAN/tunnel, token delivery, durable replay, global
event bus, new Session/runtime/persistence authority, new log persistence/schema, raw sensitive
payload display, History Explorer, Dashboard i18n, governance expansion or new third-party
frontend/runtime dependencies.

If implementation discovers a need to cross an excluded owner or change public/runtime authority,
stop that portion and return it to governance rather than widening I213.
