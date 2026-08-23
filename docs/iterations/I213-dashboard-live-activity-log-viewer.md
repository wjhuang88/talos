# Iteration I213: Dashboard Live Activity And Log Viewer

> Document status: Active / Claimed — activation PR #363 merged as `e578f4196092d0884ab7dd3321fb62acb3b88257`
> Published plan date: 2026-08-20
> Proposed activation date: 2026-08-22
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
| Executing Agent | ChatGPT / GPT-5.6 Sol — Dashboard live activity governance/implementation session 2026-08-22 |
| Work Slice | Implement WEB-001-B / I213 as one bounded default-loopback GET/read-only Dashboard Live Activity workspace: project safe semantic Session/Agent activity from the existing CLI bridge, expose bounded existing-log observations over SSE with deterministic reconnect/reset/memory/client limits, and preserve current auth/redaction/Session/permission/logging authorities. No write/control/remote/token-delivery/WebSocket/global-bus or new Session/runtime/persistence authority. |
| Claimed At | 2026-08-20 |
| Source Issue | #366 |
| Governance Claim PR | #327 |
| Activation PR | #363 — merged as `e578f4196092d0884ab7dd3321fb62acb3b88257` |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #327 exact head `a50a43ab8f34db046c9bc369c03b7413a0e6bbd9` passed CI `32347020700`, independent GLM-5.3 / OhMyOpenCode claim review, merge-time inventory/CAS, and merged as `667472145ffa7644a7f049472d7389876b8aaaf9`. The maintainer authorized bounded I219/I213 parallel execution on 2026-08-22; activation PR #363 exact head `5835a97e053e2fdfb4a13f93136af5641ff20d8b` passed CI `32572881761` and merge-time CAS under the recorded single-maintainer activation path, then merged as `e578f4196092d0884ab7dd3321fb62acb3b88257`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-23 |
| Handoff / Release Condition | Preserve all controlled pairwise exceptions recorded in the appended checkpoints. Before implementation merge, obtain I213 exact-head CI and independent security review, refresh open-PR/file-overlap inventory, and repeat merge-time CAS. |

The Collaboration Claim is effective on `main` through merge `66747214`; implementation activation
is effective through #363 merge `e578f419`. Issue #366 coordinates execution only and creates no
second owner or wider authority. The activation transfers no authority from any permission, Session,
runtime, logging, security or remote-access owner.

## Published Baseline

### Fresh Inventory — 2026-08-20

Planning target:

`main@15a3d4248d13d3951c823628454a2629398a9d48`

This is newer than the handoff's historical `ec7945156d2f72eb96af58d8cd9155f3bc5f37f1`
baseline. The activation inventory below supersedes it for execution authority.

Open PR disposition at planning time:

| PR | Scope | I213 disposition |
|---|---|---|
| #326 | I211 / VALIDATION-002 claim | Non-overlapping evidence/validation lane; recheck before activation. |
| #120 / #121 | archival recovery Drafts | Do-not-merge provenance; no Dashboard authority. |
| #327 | this governance-only claim | Owns only the proposed WEB-001-B/I213 claim; ineffective until merged. |

Fresh search found no prior `WEB-001-B`, `I213`, or overlapping Dashboard claim/implementation PR.

Relevant owner disposition at planning time:

| Owner | State | I213 disposition |
|---|---|---|
| WEB-001 | Partial | Parent explicitly retains live log/SSE and live activity residual. |
| WEB-001-A / I195 | Closed historical shell | Consume shell only; do not reuse authorization. |
| SEC-002 | Refinement / Unclaimed | Token delivery/auth redesign stays separate. |
| OBS-001 | Delivered logging baseline | Consume bounded logging baseline only. |
| CONF-001 | Partial | Config writes excluded. |
| SESSION-009 | Refinement / Unclaimed | Multi-client replay/control excluded. |
| SERVER-001 | Intake | Serve/connect/interactive architecture excluded. |
| PERM-006-A / I189 | Planned / Claimed, unactivated | Permission foundation remains independent/protected. |
| I197/I198/I200/I201/I210/I212 | Review / Claimed | Deferred-human rows are non-overlapping and do not authorize I213. |
| I206/I207/I208 | Planned / Unclaimed | Unrelated TUI steering work; not activated/bypassed by I213. |
| I211 | Planned/Unclaimed on baseline; claim #326 open | If Active at I213 activation, wait or obtain explicit non-overlap authorization. |

The long-running requirements task explicitly excludes Dashboard work. Deferred Human Validation Mode
does not waive I213's own claim, review, security, CI, CAS, or runtime acceptance.

### Activation Inventory — 2026-08-22

Activation target:

`main@781bb1122d2c323854d5d65aed354d35d045e383`

Fresh inventory immediately before creating the activation branch established:

- I211 / VALIDATION-002 is Complete / Closed; the former evidence lane no longer competes for an
  Active slot.
- I219 / PERM-006-B has an effective claim/activation through PR #359 merge `781bb112` and is the
  current permission-pipeline Active work slice.
- the only open PRs are archival recovery Drafts #120/#121; no open implementation PR owns I213 or
  I219 production files at activation-branch creation time.
- I213 remains effective Claimed through #327 merge `66747214`, with no implementation PR or
  implementation branch created before this activation branch.
- the maintainer explicitly authorized I213 to run in parallel on 2026-08-22 with a requirement to
  enforce parallel protections.

If PR #363 merges, the START-ITERATION one-active rule is overridden **only** for the explicit
I219/I213 non-overlapping pair. This is not a general multi-Active waiver and does not authorize a
third Active iteration.

### Parallel Protection Contract — I219 ↔ I213

I219 owns permission semantics and mutation authority. I213 owns Dashboard observation only.

Hard authority boundaries:

- I213 must not modify `crates/talos-permission/**`, scoped-grant/proposal/store semantics, permission
  policy precedence, approval authority, permission persistence, or the public permission schema/API
  migration owned by I219 / ADR-066.
- I213 must not change `talos-runtime` permission behavior, CLI/TUI permission wrappers, approval
  decisions, `/auto`, sandbox/fallback policy, or any PERM-006 owner state.
- I213 may display a permission/sandbox fact only when an existing authoritative safe fact is already
  available; it must not synthesize, infer or cache new permission truth.
- I219 must not be treated as a dependency that can be silently consumed to widen I213. If I213
  discovers that a permission/runtime API change is required, that portion stops and returns to
  governance rather than crossing the owner boundary.

File-overlap protection:

- I213 should keep production work primarily inside `crates/talos-dashboard/**` and additive,
  Dashboard-specific CLI observation glue. It does not reserve generic CLI/runtime files against
  I219.
- before the first I213 production commit that changes a shared `talos-cli` file, and again before
  implementation PR merge, refresh open PR/branch changed-file inventory for I219. If the same
  production file is being modified by I219, I213 must rebase on the newer `main` and either move the
  Dashboard integration behind a new Dashboard-specific module/boundary or pause the overlapping
  portion. Do not resolve an authority collision by editing both implementations in one PR.
- any change under `crates/talos-runtime/**`, `crates/talos-permission/**`, permission/approval modules,
  or an I219 owner document is a stop condition for I213 unless separately governed.
- shared derived governance files (`docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`,
  `docs/iterations/README.md`, manifest/current-state views) use union semantics only: refresh from
  current `main`, preserve every I219 fact, and append/update only I213 truth.

Merge/CAS protection:

- implementation must branch from the #363 activation merge or a later fresh `main`.
- every I213 implementation exact head requires its own full CI plus independent security-focused
  review; the claim review does not approve code.
- immediately before implementation merge, re-fetch `main`, open PRs and changed-file inventory. If
  I219 (or any later Active owner) overlaps an I213 production file or public authority boundary,
  merge is blocked until the overlap is eliminated or explicitly re-governed.
- merge uses expected-head CAS; any I213 head change invalidates the exact-head implementation review.

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `WEB-001-B` | `WEB-001` | Ready / effective Claimed via #327 | WEB-001-A/I195 Complete; ADR-031; ADR-006; OBS-001/ADR-014; current TUI bridge | Local read-only Live Activity workspace with safe semantic activity + bounded live logs over SSE |

### Readiness

**Ready for activation merge. Not yet implementation-active.**

The slice has one runnable/testable deliverable, explicit authoritative sources, fixed SSE/memory
bounds, explicit security exclusions, no current Dashboard implementation owner, and a rollback that
leaves I195 intact. Implementation authority begins only if #363 reaches `main`, and remains bounded
by the parallel protection contract above.

## Activation Checkpoint — 2026-08-23

This checkpoint appends current execution facts after the preserved Published Baseline above. It
does not rewrite the historical selection, inventory, readiness or conditional activation record.

### Effective #363 Activation

| Evidence | Recorded Fact |
|---|---|
| Exact head | `5835a97e053e2fdfb4a13f93136af5641ff20d8b` |
| Target base | `main@0def395440de8e102040f0d99c34e918c8dfd052` |
| Exact-head CI | Run `32572881761` completed successfully for the governance-only diff. |
| Review / authorization | #363 had no independent exact-head reviewer; it used the recorded `Single-maintainer merge` activation path because no independent reviewer was assigned or available. The independent #327 claim review remains claim evidence only and is not represented as #363 or implementation review. |
| Merge-time CAS | Fresh target/open-PR/owner/file/dependency CAS was recorded as passed in the #363 merge evidence. |
| Merge commit | `e578f4196092d0884ab7dd3321fb62acb3b88257` |

The claim and activation are therefore effective on `main`. I213 is `Active / Claimed`. The future
SSE/CSP implementation exact head still requires its own full CI and independent security-focused
review; neither the claim review nor the single-maintainer activation authorizes implementation
merge.

### Controlled Parallel Exceptions

Only these two bounded exceptions are recorded for I213:

1. **I219 ↔ I213**: the existing permission/Dashboard parallel exception. I219 owns permission,
   grant, compiler, store, wrapper and permission-runtime authority; I213 owns only GET/read-only
   Dashboard observation. Existing authority, production-file and merge-CAS stop gates remain.
2. **I220 ↔ I213**: I220 is limited to ADR-067 decision documentation and current-path/migration
   analysis. It includes no Rust, Cargo, Dashboard, Runtime, permission implementation or behavior
   modification, and it grants no I221 implementation authority.

I220 must not modify `crates/talos-dashboard/**`, this I213 owner, or Dashboard implementation files.
I213 must not modify I220/PERM-006-C owners or permission implementation files. Any authority or
production-file overlap pauses the affected work immediately and returns it to governance. Shared
derived governance files use owner-first union semantics.

I220 and I213 retain separate evidence lanes. Each lane's applicable exact head requires its own
review, CI and merge-time CAS; evidence from one lane cannot approve the other.

## I222-B Parallel Scheduling Checkpoint — 2026-08-23

Maintainer Issue #366 comment `5386904546` supersedes the earlier serial-pause disposition in
comment `5386854800` and authorizes only the bounded I213 ↔ I222/TOOL-024-B pair to run in
parallel. This scheduling checkpoint does not change I213's published objective, acceptance,
17-file product boundary or Dashboard-only authority.

- I213 retains its existing Dashboard/live-activity candidate, including
  `crates/talos-dashboard/**`, Dashboard-specific `talos-cli` observation glue, bilingual README
  updates and its WEB/I213 owners.
- I222-B is limited to `talos-core`, `talos-agent`, `talos-tools`, necessary additive
  `talos-runtime` finalizer integration, focused tests and directly affected API/runtime
  documentation. It must not modify `crates/talos-cli/**`, `crates/talos-dashboard/**`, README
  files, I213/WEB owners, Dashboard authority or the I213 implementation branch.
- Shared Board, backlog, iteration-index and manifest changes use owner-first union semantics and
  do not create production authority for either lane.
- Before each stable push and merge, both lanes compare exact changed-file inventories. Any same
  production file or authority overlap pauses only the overlapping work and requires fresh
  governance disposition.
- Each lane requires its own exact-head CI, applicable independent review and merge-time CAS;
  neither lane may reuse the other's evidence.
- This authorization does not extend to TOOL-024-C/D. Those children must recompute overlap and
  obtain their own scheduling disposition.

I213 may resume from fresh `main` under its existing effective claim and GOV-008 local-convergence
gates. I222-B may begin implementation only after its own finalized claim reaches `main`.

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
- Parallel I219 collision -> enforce the authority/file/CAS guards above; stop overlapping work rather
  than broadening either owner.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-20 | Planning | Fresh inventory at `main@15a3d4248d13d3951c823628454a2629398a9d48`; no prior WEB-001-B/I213 owner or Dashboard PR overlap. |
| 2026-08-20 | Claim proposal | Draft PR #327 opened from governance-only planning head. |
| 2026-08-20 | Claim effective | #327 exact head `a50a43ab` passed CI `32347020700`, independent claim review and merge-time CAS, then merged as `66747214`. |
| 2026-08-22 | Activation inventory | Refreshed at `main@781bb112`; I211 Complete/Closed; I219 active permission lane identified; only archival PRs #120/#121 open; no Dashboard implementation owner. |
| 2026-08-22 | Parallel authorization | Maintainer explicitly approved I213 running in parallel with the active non-overlapping lane, conditioned on parallel protection. I219/I213 authority, file-overlap and merge-CAS guards recorded above. |
| 2026-08-22 | Activation proposal | Draft PR #363 opened as governance-only activation. It is ineffective for implementation while open. |

## Verification Evidence

- Claim exact-head CI: `32347020700` success at `a50a43ab8f34db046c9bc369c03b7413a0e6bbd9`.
- Independent claim review: GLM-5.3 / OhMyOpenCode APPROVE bound to `a50a43ab`; shared-account
  identity limitation disclosed.
- Claim merge-time CAS: passed; #327 merged as `667472145ffa7644a7f049472d7389876b8aaaf9`.
- Activation inventory: `main@781bb1122d2c323854d5d65aed354d35d045e383`; only #120/#121 open,
  both archival Drafts; I219 is the explicit parallel Active owner.
- Activation governance CI/validators: pending #363 exact head.
- Runtime evidence: not started; no production code belongs to the activation branch.

## Completion Evidence

- Completion Commit: Pending
- Future closeout must cite an already-existing implementation/merge SHA; it may not self-certify.

## Variance And Residuals

SEC-002, SESSION-009, SERVER-001, PERM-006, ACP/future AG-UI, History Explorer, governance UI,
Dashboard i18n, remote/LAN and structured-log evolution remain separately owned residuals.

The explicit pairwise parallel authorizations are scheduling variances only. They do not change any
owner boundary, expire when either named pair is no longer eligible, and must not be reused for
another iteration.

## Retrospective

- Outcome: pending.
- Documentation: pending.
- Lessons: pending.
