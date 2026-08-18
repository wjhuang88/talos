# WEB-001-A: Dashboard Information Architecture And Read-Only Visual Shell

**Status**: Complete — I195 closed after PR #233 merge `490503db905bcd2eb2ab5e3b5487b1f542873d63`
**Priority**: P1
**Type**: Product Story
**Parent Epic**: WEB-001
**Selected Iteration**: I195 — Complete / Closed

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-5.6 Sol — Dashboard governance session 2026-08-13 |
| Work Slice | Deliver only Dashboard-wide information architecture and the first cohesive read-only visual shell over the existing GET-only loopback `/status`, `/history`, `/governance`, `/config`, and `/extensions` surfaces. Preserve current config masking, output redaction, HTML escaping, loopback binding and JSON/plain-text negotiation. No write/control/remote/session-mutation capability. |
| Claimed At | 2026-08-13 |
| Source Issue | None |
| Governance Claim PR | #212 |
| Authorization Mode | Independent claim review; maintainer override for implementation review policy |
| Authorization Evidence | Claim head `6e3cd2c5c761fc9b241daa85018b963dcb163f38` received independent natural-person approval in comment `5289651455`; PR #212 merged as `f123e534ce864a89eb3cabfc68f4a1518201c2d0`, making this bounded claim effective. For implementation merge, maintainer override `5326076971` explicitly accepted independent AI exact-head technical review `5323625004` plus human manual browser acceptance `5323801564` for this non-protected read-only slice; exact-head CI `32087223234` succeeded. |
| Implementation PR | #233 — merged as `490503db905bcd2eb2ab5e3b5487b1f542873d63` from reviewed exact head `1ee4aa3786785473069c735e1985c9d720b82e2f` |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | None — I195 is Complete/Closed; excluded live/write/control/remote work remains separately governed under WEB-001 residual owners. |

`Claim Pending` is not a valid claim state. The bounded `Claimed` record above became effective
through PR #212 merge `f123e534ce864a89eb3cabfc68f4a1518201c2d0`. Activation PR #288 moves
WEB-001-A/I195 into execution without expanding that scope. The pre-activation #233 implementation
heads remain Draft provenance only and must be rebuilt from the activation merge or later current
`main` before review.

### Activation Record - 2026-08-18

- Exact activation source: `main@4635ef2b4cc9c894f03c0bcbce7e7802730e56ab`.
- Maintainer direction in the current Dashboard session explicitly authorizes continued I195 work in
  parallel with I205/GOV-007. I205 is an evidence-only governance audit and is non-overlapping.
- #287 remains I205-owned; #233 remains the I195-owned Draft candidate to rebuild. Recovery PRs
  #120/#121 remain archival only.
- I189/I196 remain Planned/Claimed; I197-I201, I206-I208 and I210 remain Planned/Unclaimed; I164
  remains Paused. No other effective owner or open PR overlaps WEB-001-A.
- The three shared derived views retain all then-current main rows and change only I195 state plus the
  I205 wording needed to avoid falsely claiming it is the sole Active iteration.

### Completion Record - 2026-08-18

- Activation PR #288 merged as `8f1facc8d2955fb6c9a6e01da32a62be3d7c9d40` before the final #233 rebuild.
- Reviewed implementation head `1ee4aa3786785473069c735e1985c9d720b82e2f` passed exact-head CI `32087223234`.
- Independent AI exact-head technical review `5323625004` and human maintainer browser acceptance
  `5323801564` passed. Maintainer override `5326076971` explicitly accepted that review path for this
  non-protected read-only slice instead of requiring a separate natural-person technical reviewer.
- Final merge-time CAS used then-current `main@8127fa579cef03a36e743a30003a682bc5f884b1`; PR #233 merged
  as `490503db905bcd2eb2ab5e3b5487b1f542873d63`, whose second parent is exactly the reviewed head.

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
- Shared derived-status synchronization is union-based: retain every then-current `main` row and
  add or retain WEB-001-A/I195's bounded row. Never replace a shared derived file wholesale from
  either the target side or the Dashboard side.
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

- [x] Before implementation, every current Active, Review, Planned and Blocked iteration was
      re-inventoried and its Dashboard disposition was recorded in I195.
- [x] Before the implementation branch existed, one effective `Claimed` record for this exact Work
      Slice reached `main` through governance claim PR #212; activation was made authoritative via #288
      before the final implementation rebuild.
- [x] The implementation worktree recorded its exact current-main base SHA after the effective claim.
- [x] Implementation branch was `feat/dashboard-I195-read-only-shell` and PR #233 targeted `main`.
- [x] `cargo test --locked -p talos-dashboard` passed with HTML negotiation, `/extensions`, masking,
      redaction, escaping, GET-only and loopback regressions.
- [x] `cargo check --locked --workspace` passed.
- [x] `cargo clippy --locked --workspace -- -D warnings` passed.
- [x] `cargo test --locked --workspace` passed.
- [x] `scripts/validate_project_governance.sh .` passed.
- [x] `bash scripts/validate_collaboration_claims.sh .` passed.
- [x] `git diff --check` was clean.
- [x] Repeatable manual browser validation recorded 320×568, 768×1024 and 1440×900 layouts, 200%
      zoom, keyboard-only navigation, WCAG AA contrast checks, unchanged CSP-compatible resource
      behavior and truthful English `lang` metadata without using browser automation (`5323801564`).
- [x] Independent AI exact-head technical review `5323625004` was accepted by explicit maintainer
      override `5326076971`; the owner-local natural-person technical reviewer requirement was waived
      for this non-protected read-only slice.
- [x] Merge-time CAS confirmed current `main`, no overlapping effective claim/new PR, exact reviewed
      head and still-satisfied dependencies before merge.
- [x] Governance closure cites the pre-existing implementation/merge SHA in `Completion Commit:`; no
      status-only commit is used as completion evidence.

## Completion Evidence

Completion Commit: `490503db905bcd2eb2ab5e3b5487b1f542873d63`

- Reviewed implementation head: `1ee4aa3786785473069c735e1985c9d720b82e2f`.
- Exact-head CI `32087223234`: SUCCESS across Unix/macOS and Windows workspace gates plus both
  governance validators.
- Independent AI technical review `5323625004` approved the exact head; human maintainer browser
  walkthrough `5323801564` passed the viewport, zoom, keyboard, CSP, empty/populated and security
  presentation matrix.
- Maintainer review-policy override `5326076971` explicitly accepted that AI technical review for
  this non-protected I195 slice and waived the owner-local natural-person reviewer requirement.
- PR #233 merged with first parent `8127fa579cef03a36e743a30003a682bc5f884b1` and second parent
  exactly `1ee4aa3786785473069c735e1985c9d720b82e2f`, preserving the reviewed implementation tree.

## Residual Destination

All excluded live/write/control/remote work remains under WEB-001 or a future separately governed
child/ADR/claim. TUI-037 remains its own owner. SEC-002 remains the separate token-delivery residual.
Any newly discovered shared runtime/domain/session API requirement is routed to the mainline
foundation lane rather than absorbed into this Story.
