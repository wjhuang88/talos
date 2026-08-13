# WEB-001-A: Dashboard Information Architecture And Read-Only Visual Shell

**Status**: Ready — proposed claim in PR #212; ineffective before target-branch merge
**Priority**: P1
**Type**: Product Story
**Parent Epic**: WEB-001
**Selected Iteration**: I195 proposed; no implementation authority before the finalized claim reaches `main`

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-5.6 Sol — Dashboard governance session 2026-08-13 |
| Work Slice | Deliver only Dashboard-wide information architecture and the first cohesive read-only visual shell over the existing GET-only loopback `/status`, `/history`, `/governance`, `/config`, and `/extensions` surfaces. Preserve current config masking, output redaction, HTML escaping, loopback binding and JSON/plain-text negotiation. No write/control/remote/session-mutation capability. |
| Claimed At | 2026-08-13 |
| Source Issue | None |
| Governance Claim PR | #212 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent natural-person exact-head review is required before claim merge. If the shared `@wjhuang88` GitHub account is used for review, the reviewer must explicitly disclose their natural-person identity. This proposed `Claimed` record has no ownership effect before PR #212 reaches `main`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-13 |
| Handoff / Release Condition | Pass exact-head governance/CI validation, obtain independent natural-person approval, repeat merge-time CAS against current `main` plus all three lanes, and merge PR #212 to `main` before creating any Dashboard implementation branch or worktree. |

`Claim Pending` is not a valid claim state. The `Claimed` record above is a proposal on PR #212 and
becomes effective only when that exact governed record is merged to `main`; the open PR itself does
not reserve implementation scope.

## Identity / Goal / Value

### User

A Talos user running the local application who opens the embedded loopback Dashboard to understand
current runtime status, session history, governance information, masked configuration and extension
state without changing Talos state.

### Goal

Turn the currently fragmented read-only rendered pages into one coherent, navigable and visually
usable Dashboard while preserving the security and architecture boundary already established by
ADR-031 and I129.

### Value

- make the local Dashboard feel like one user-facing product surface rather than a collection of
  diagnostic endpoints;
- let users scan the five existing read-only data surfaces with predictable navigation and hierarchy;
- provide a responsive and accessible baseline that later separately governed Dashboard slices can
  extend without reopening the security boundary;
- preserve one runtime/domain/session/permission source of truth instead of copying presentation-local
  business logic.

## Scope

- One cohesive Dashboard information architecture covering:
  - `/status`;
  - `/history`;
  - `/governance`;
  - `/config` using the existing masked representation;
  - `/extensions` as a read-only rendered presentation.
- A shared visual shell for root and rendered pages with consistent navigation, page title/hierarchy,
  spacing, typography, surfaces, tables/lists, code/preformatted content and empty states.
- Responsive presentation suitable for narrow, medium and desktop browser viewports.
- Accessible semantic rendering with keyboard-operable navigation, visible focus, meaningful heading
  hierarchy and no information that depends only on hover or color.
- Explicit `Accept: text/html` rendering for all five surfaces while retaining their existing
  JSON/plain-text behavior when HTML is not explicitly requested.
- Reuse the existing `DashboardSnapshot` and existing runtime/domain/session/config/extension data
  providers; presentation may shape layout but must not create a second business-data implementation.
- User-facing documentation that describes the Dashboard as a local read-only surface and does not
  imply control-plane capability.

## Exclusions

- SSE or live-log transport.
- Configuration writes or a config editor.
- Approvals, permission decisions or tool execution.
- Session mutation, session actions or conversation control.
- WebSocket control.
- LAN, remote or tunnel access.
- Browser automation.
- TUI-037 dashboard-logo/link behavior.
- A new permission model.
- A new persistence model.
- A remote control plane.
- Dashboard UI localization/i18n in this slice; user-facing README documentation remains bilingual,
  while the rendered Dashboard copy remains English and keeps truthful `lang="en"` metadata.
- New shared runtime/domain/session APIs unless separately owned, claimed and landed through the
  mainline foundation lane first.
- JavaScript framework, Node.js build pipeline or speculative client-side state management unless a
  later separately governed decision demonstrates that the server-rendered first slice cannot meet
  acceptance.
- Any claim that WEB-001/I129 acceptance has been re-authorized or that Dashboard now provides a
  write-capable control surface.

## Dependencies

- WEB-001 parent product direction remains Partial; this child does not change or complete its
  separately owned SSE/write/approval/session/remote residuals.
- ADR-031 is authoritative for the loopback binding, read-only security posture, and its four
  approved initial routes `/status`, `/history`, `/governance`, and `/config`; this Story does not
  claim that ADR-031 §5 originally authorized `/extensions`.
- `/extensions` read-only provenance comes from WEB-001/I129-era Dashboard evolution and the current
  GET-only implementation; this child adds HTML presentation parity without expanding route privilege.
- I129 is prior implementation evidence for the existing rendered `/status`, `/history`,
  `/governance` and `/config` representations; its acceptance is not reused as this Story's
  authorization.
- Existing `talos-dashboard` snapshot composition is consumed as-is unless a separately governed
  shared-foundation change lands first.
- TUI-037 remains independent and is neither a prerequisite nor part of acceptance.

## Decision Links And Constraints

- `docs/decisions/031-web-loopback-dashboard-boundary.md`
  - retain `127.0.0.1` loopback binding;
  - retain the GET-only/read-only security posture established there for the approved initial routes;
  - do not attribute `/extensions` route provenance to ADR-031 §5;
  - retain output-boundary redaction and the existing config masking boundary;
  - do not infer remote or browser-control authorization.
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md` and I129 implementation history
  - establish the current `/extensions` GET-only snapshot surface and its JSON-only pre-I195 state;
  - I195 may add explicit HTML presentation but no write/control semantics.
- `docs/proposals/web-001-loopback-dashboard-design.md`
  - use the existing read-only loopback presentation as the first product surface;
  - keep later live/write/control concerns independently gated.
- `docs/tasks/2026-08-13-three-track-development-baseline.md`
  - Dashboard/Desktop/shared-runtime lanes integrate only through `main`;
  - no second runtime/session/permission/domain truth;
  - if this slice needs a shared API change, pause that portion and land a separate mainline owner
    before refreshing Dashboard.

## Uncertainty And Validation Path

- **Soft constraint**: keep the first slice server-rendered and dependency-light. Validate that
  semantic HTML plus CSP-compatible CSS is sufficient for the required IA, responsive behavior and
  accessibility before considering a client framework.
- **Current CSP is a hard resource constraint**: existing HTML responses send
  `default-src 'none'; style-src 'unsafe-inline'`. The first slice must therefore remain compatible
  with inline CSS and inline SVG markup and must not depend on executable script, external/web fonts,
  raster/remote images, or `data:` image resources that the current policy blocks. This makes the
  no-JavaScript first slice mechanically enforceable at runtime rather than only a planning preference.
- **Assumption**: the existing five snapshots contain enough user-facing information for a coherent
  first shell. Validate during implementation; missing shared data is not filled by duplicating
  domain logic in `talos-dashboard`.
- **UX risk**: generic recursive JSON tables can become unreadable on narrow screens. The first slice
  may improve presentation hierarchy and overflow behavior, but must not reinterpret or mutate data.
- **Security risk**: styling/rendering changes could accidentally bypass escaping or masking. Every
  rendered dynamic value remains behind existing redaction and HTML escaping, with adversarial
  regression coverage.

## State / Status Owners

- Parent product scope and residuals: `docs/backlog/active/WEB-001-embedded-web-control-surface.md`.
- This child Story: this document.
- Implementation selection/execution/completion: `docs/iterations/I195-dashboard-read-only-visual-shell.md`.
- Project derived status: `docs/backlog/PRODUCT-BACKLOG.md`, `docs/iterations/README.md`, and
  `docs/BOARD.md`; owners remain authoritative.
- TUI logo/link behavior: `docs/backlog/active/TUI-037-dashboard-logo-link.md`, unchanged.

## User-Facing Documentation

Implementation acceptance updates:

- `README.md` with the local read-only Dashboard capability and security boundary;
- `README.zh-CN.md` with the matching user-facing description;
- WEB-001 parent status only to cross-link this child and record its bounded outcome, without
  rewriting WEB-001's published residual acceptance.

Documentation must call this a local read-only Dashboard or read-only web surface, not a remote or
write-capable control plane. Bilingual README updates do not imply Dashboard UI localization in I195.

## Required Reads

- `AGENTS.md`
- `docs/sop/AGENT-COLLABORATION.md`
- `docs/sop/REQUIREMENT-INTAKE.md`
- `docs/sop/START-ITERATION.md`
- `docs/sop/ITERATION-WORKFLOW.md`
- `docs/sop/GIT-WORKFLOW.md`
- `docs/sop/TESTING.md`
- `docs/tasks/2026-08-13-three-track-development-baseline.md`
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`
- `docs/backlog/active/TUI-037-dashboard-logo-link.md`
- `docs/decisions/031-web-loopback-dashboard-boundary.md`
- `docs/proposals/web-001-loopback-dashboard-design.md`
- `docs/iterations/I129-web001-rendered-dashboard-pages.md`
- `crates/talos-dashboard/src/lib.rs`

## Acceptance For Behavior

- **Cohesive shell** — Given a Talos user opens `/` or any of the five Dashboard surfaces in a
  browser that explicitly accepts HTML, when the page renders, then the user sees one consistent
  Dashboard shell with predictable navigation, a clear current page title and content hierarchy.
- **Complete first IA** — Given the shared navigation, when the user navigates only with standard
  links and keyboard focus, then `/status`, `/history`, `/governance`, `/config` and `/extensions`
  are all reachable and return read-only rendered content.
- **Extensions parity** — Given `Accept: text/html` on `/extensions`, when the endpoint responds,
  then it returns the same safe shared HTML shell/presentation model as the other Dashboard pages;
  given no explicit HTML request, its existing JSON representation remains available. The route
  provenance is WEB-001/I129/current implementation, not ADR-031 §5.
- **Config masking** — Given configuration contains a credential or API key, when `/config` is
  rendered or requested as plain text, then the existing masked representation is preserved and no
  secret value is exposed.
- **Escaping and redaction** — Given adversarial strings containing HTML/script-like content or
  sensitive assignments/headers/URLs, when any of the five surfaces renders, then dynamic content
  is escaped and output-boundary redaction prevents secret disclosure.
- **Read-only boundary** — Given any Dashboard route in this slice, when POST, PUT, PATCH or DELETE
  is attempted, then no business write/action route exists and the request is rejected under the
  existing GET-only router behavior.
- **Loopback boundary** — Given the Dashboard server starts through the existing lifecycle, when its
  listener address is inspected, then it remains bound to `127.0.0.1` and this Story introduces no
  LAN/remote/tunnel path.
- **Responsive UX** — Given representative 320 px, 768 px and 1440 px viewport widths, when each
  Dashboard page is viewed, then navigation and primary content remain usable and readable without
  page-level accidental horizontal scrolling; intrinsically wide table/pre content may scroll only
  inside its own bounded content region.
- **Zoom/accessibility UX** — Given browser zoom at 200% and keyboard-only interaction, when the user
  traverses the shell, then navigation remains operable, focus is visibly identifiable, headings and
  landmarks preserve reading order, and no required information is available only through hover or
  color.
- **Contrast UX** — Given normal text, large text, focus indicators, borders/icons or other essential
  visual affordances, when the rendered shell is evaluated, then text meets WCAG 2.1 AA contrast
  thresholds (4.5:1 normal text and 3:1 large text) and essential non-text/focus affordances meet a
  3:1 contrast threshold against adjacent colors.
- **CSP-compatible resources** — Given the existing HTML CSP `default-src 'none'; style-src
  'unsafe-inline'`, when all five pages render, then the UX does not depend on scripts, external/web
  fonts, raster/remote images or `data:` image resources; required visuals work with semantic text,
  inline CSS and inline SVG markup under the unchanged CSP.
- **Language metadata** — Given this first slice intentionally ships English Dashboard UI copy,
  when rendered HTML is inspected, then `lang="en"` remains truthful; README.zh-CN documentation is
  not treated as authorization or acceptance for Dashboard UI i18n.
- **Single source of truth** — Given existing runtime/session/config/extension data is displayed,
  when implementation is reviewed, then `talos-dashboard` consumes existing snapshot/domain logic
  and does not introduce duplicate durable state, permission logic or session semantics.

## Acceptance For Technical / Governance Work

- [ ] Before implementation, every current Active, Review, Planned and Blocked iteration is
      re-inventoried and its Dashboard disposition is recorded in I195.
- [ ] Before the implementation branch exists, one effective `Claimed` record for this exact Work
      Slice has reached `main` through governance claim PR #212.
- [ ] The implementation worktree records its exact current-main base SHA after the effective claim.
- [ ] Implementation branch is `feat/dashboard-I195-read-only-shell` and its PR targets `main`.
- [ ] `cargo test --locked -p talos-dashboard` passes with HTML negotiation, `/extensions`, masking,
      redaction, escaping, GET-only and loopback regressions.
- [ ] `cargo check --locked --workspace` passes.
- [ ] `cargo clippy --locked --workspace -- -D warnings` passes.
- [ ] `cargo test --locked --workspace` passes.
- [ ] `scripts/validate_project_governance.sh .` passes.
- [ ] `bash scripts/validate_collaboration_claims.sh .` passes.
- [ ] `git diff --check` is clean.
- [ ] Repeatable manual browser validation records 320×568, 768×1024 and 1440×900 layouts, 200%
      zoom, keyboard-only navigation, WCAG AA contrast checks, unchanged CSP-compatible resource
      behavior and truthful English `lang` metadata without using browser automation.
- [ ] Independent natural-person review is bound to the exact implementation head; shared-account
      review explicitly discloses reviewer identity.
- [ ] Merge-time CAS confirms current `main`, no overlapping effective claim/new PR, exact reviewed
      head and still-satisfied dependencies before merge.
- [ ] Governance closure cites a pre-existing implementation/merge SHA in `Completion Commit:`; no
      status-only commit is used as completion evidence.

## Residual Destination

All excluded live/write/control/remote work remains under WEB-001 or a future separately governed
child/ADR/claim. TUI-037 remains its own owner. Any newly discovered shared runtime/domain/session
API requirement is routed to the mainline foundation lane rather than absorbed into this Story.
