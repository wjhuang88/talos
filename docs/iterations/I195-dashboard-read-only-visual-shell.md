# Iteration I195: Dashboard Read-Only Visual Shell

> Document status: Complete
> Published plan date: 2026-08-13
> Planned objective: deliver one coherent, accessible and responsive user-facing Dashboard shell over
> the existing GET-only loopback status/history/governance/config/extensions surfaces without adding
> any write, control, remote-access or alternate domain/runtime state capability.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a rebuilt Talos user can open the local loopback Dashboard and navigate one
> consistent read-only shell across `/status`, `/history`, `/governance`, `/config` and `/extensions`,
> with existing config masking, redaction, escaping and non-HTML response compatibility preserved.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-5.6 Sol — Dashboard governance session 2026-08-13 |
| Work Slice | Implement only WEB-001-A / I195 Dashboard-wide IA and the first cohesive read-only visual shell over the existing GET-only loopback `/status`, `/history`, `/governance`, `/config`, and `/extensions` surfaces. Preserve `127.0.0.1`, current config masking, output redaction, HTML escaping and JSON/plain-text negotiation; introduce no write/control/remote/session-mutation capability or duplicated shared business logic. |
| Claimed At | 2026-08-13 |
| Source Issue | None |
| Governance Claim PR | #212 |
| Authorization Mode | Independent claim review; maintainer override for implementation review policy |
| Authorization Evidence | Claim head `6e3cd2c5c761fc9b241daa85018b963dcb163f38` received independent natural-person approval in comment `5289651455`; PR #212 merged as `f123e534ce864a89eb3cabfc68f4a1518201c2d0`, making the bounded WEB-001-A/I195 claim effective. For implementation merge, maintainer override `5326076971` explicitly accepted independent AI exact-head technical review `5323625004` plus human manual browser acceptance `5323801564` for this non-protected read-only slice; exact-head CI `32087223234` succeeded. |
| Implementation PR | #233 — merged as `490503db905bcd2eb2ab5e3b5487b1f542873d63` from reviewed exact head `1ee4aa3786785473069c735e1985c9d720b82e2f` |
| Last Updated | 2026-08-18 |
| Handoff / Release Condition | None — I195 is Complete/Closed; residual WEB-001 capabilities remain separately governed. |

The `Claimed` record above became effective when PR #212 merged as
`f123e534ce864a89eb3cabfc68f4a1518201c2d0`. The earlier #233 branch-local activation was not
authoritative because activation had not reached `main`; that process error was corrected through
activation PR #288 before the final implementation was rebuilt and reviewed.

## Activation Baseline - 2026-08-18

- Activation PR: #288.
- Exact activation source: `main@4635ef2b4cc9c894f03c0bcbce7e7802730e56ab`.
- Effective claim: PR #212 merge `f123e534ce864a89eb3cabfc68f4a1518201c2d0`; reviewed claim head
  `6e3cd2c5c761fc9b241daa85018b963dcb163f38`; independent approval comment `5289651455`.
- Existing implementation PR #233 is deliberately retained as Draft provenance only. None of its
  pre-activation heads are merge-authoritative; implementation must be rebuilt from #288's merge or
  a later current `main` before review.
- I205/GOV-007 is concurrently Active under claim merge `fd1eaad9` with Draft PR #287. The maintainer
  explicitly directed this Dashboard development flow to continue on 2026-08-18, authorizing parallel
  non-overlapping I195 activation. I205 is evidence-only governance work and transfers no authority
  into Dashboard; I195 transfers no product/runtime/SOP/validator/CI authority into I205.
- Current non-terminal disposition at activation: I189 and I196 stay Planned/Claimed; I197-I201,
  I206-I208 and I210 stay Planned/Unclaimed; I164 stays Paused; I205 stays Active in its independent
  audit slice. I188/I203/I204/I209 remain terminal and are not reopened.
- Open PR disposition: #287 is the non-overlapping I205 audit Draft; #233 is the same-lane historical
  Dashboard candidate to rebuild; #120/#121 remain archival recovery Drafts and are not implementation
  authority. No other open PR owns WEB-001-A/I195.
- Shared derived views continue to use the established union invariant: start from then-current main,
  preserve all other lane rows, and change only the factual I195 state plus any wording that would
  otherwise falsely claim I205 is the sole Active iteration.

### Completion Record - 2026-08-18

- Activation PR #288 merged as `8f1facc8d2955fb6c9a6e01da32a62be3d7c9d40` before the final #233 rebuild.
- Final reviewed implementation head `1ee4aa3786785473069c735e1985c9d720b82e2f` passed exact-head CI
  `32087223234`; publisher/focused validation had already passed `cargo test --locked -p talos-dashboard`.
- Independent AI exact-head technical review `5323625004` approved the implementation, and human
  maintainer browser acceptance `5323801564` passed the required viewport, zoom, keyboard, CSP,
  empty/populated and security-presentation matrix.
- Maintainer override `5326076971` explicitly accepted the AI technical review for this non-protected
  read-only slice, superseding the owner-local natural-person implementation-review requirement while
  leaving protected-scope repository policy unchanged.
- Final merge-time CAS used then-current `main@8127fa579cef03a36e743a30003a682bc5f884b1`; no overlapping
  Dashboard owner or unresolved blocking feedback existed. PR #233 merged as
  `490503db905bcd2eb2ab5e3b5487b1f542873d63`, whose second parent is exactly the reviewed head.

The earlier section named `Current Synchronized Claim Baseline` below is preserved as claim-time
history. The activation and completion records above are the current execution truth and supersede
pre-activation status statements without rewriting the published planning history.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| WEB-001-A | WEB-001 | Ready / proposed claim | ADR-031; existing I129 rendered-page baseline; current `talos-dashboard` snapshot | One user-visible, runnable and testable read-only Dashboard shell with coherent IA and UX across the five existing GET-only surfaces |

### Target-Branch Baseline At Planning

- Target branch: `main`.
- `main` HEAD observed immediately before this governance branch was created:
  `c4bd9606c8bae63cb9bf11becd45846bf0805982`.
- Three-track common baseline:
  `23e4174bcfb036602ce2145026b872ec5c517289`.
- Governance branch: `docs/dashboard-I195-read-only-shell-claim`.
- Governance claim PR: #212.
- This governance branch is not an implementation branch and carries no production code.
- The eventual implementation base is deliberately **not fixed yet**: it must be the effective I195
  claim merge commit or a later current `main` after re-running baseline/CAS checks.

### Current Synchronized Claim Baseline

- `main` is `556b5a4319085bf5250bccf4920e0dec0c6646c8`. That target includes I193/SESSION-008-B
  implementation merge `1b5461cdcb03c7a896b814ccad2d93aa44010fc6` and the subsequent I193 closeout that records
  Completion Commit `404d7a4bf5b9c7dedeae479fe91fa5400b42d411`.
- #212 was synchronized to that target by pure two-parent governance merge commit
  `2656dc43730383353a8a4825896718760abc440e`, whose parents are prior Dashboard head
  `d8beefb32a6605465a867cb2229d5495c2f95bee` and `main@556b5a4319085bf5250bccf4920e0dec0c6646c8`.
- **Shared derived-file synchronization uses union semantics.** For `docs/BOARD.md`,
  `docs/backlog/PRODUCT-BACKLOG.md`, and `docs/iterations/README.md`, every synchronization starts
  from then-current `main`, preserves every existing target-branch row, and reapplies I195's own
  bounded derived row. A shared derived file must never be resolved by taking either branch wholesale;
  current-main truth and the lane-local I195 addition must both survive.
- After that synchronization, the three shared derived views each contain one I195/WEB-001-A row
  while retaining the target-branch I193 Complete and I194 Complete rows unchanged.
- I193/SESSION-008 is terminal `Complete`: B implementation is
  `404d7a4bf5b9c7dedeae479fe91fa5400b42d411`, PR #216 merged as
  `1b5461cdcb03c7a896b814ccad2d93aa44010fc6`, and exact-head CI was `31691761892`. I187 remains
  pre-I193 history. RUNTIME-005, I188 and I189 retain their separately governed gates.
- I194/DESKTOP-001-D0 is terminal `Complete`. Decision head `0a47208ce6fad23c706ebede8b3d07111b9303dc`
  merged through PR #215; ADR-059 remains Proposed, and no Desktop crate, renderer dependency,
  production UI, mock-only visual slice or runtime/session binding is authorized by I195.

### Current Non-Terminal Iteration Inventory

After synchronizing to `main@556b5a4319085bf5250bccf4920e0dec0c6646c8`, I193 and I194 are terminal
Complete and therefore excluded from this non-terminal inventory.

| Iteration | State | I195 Disposition |
|---|---|---|
| I159 | Blocked | Keep blocked; TUI-037 disposition remains its own gate. No Dashboard implementation is used to satisfy it. |
| I160 | Blocked | Keep blocked on I159 Complete. No overlap. |
| I161 | Blocked | Keep blocked on I160 Complete plus its security-review plan. No overlap. |
| I162 | Blocked | Keep blocked on I161 Complete plus release-readiness authorization. No overlap. |
| I164 | Paused | Preserve the superseded startup-inline target; do not resume. |
| I188 | Planned / Claimed | Keep unactivated; TOOL-024-A decision-only process/permission work is independent. |
| I189 | Planned / Claimed | Keep unactivated; PERM-006-A permission-foundation scope is independent. |

No existing target-branch owner authorizes I195 implementation. This table must be refreshed again
immediately before claim merge, implementation branch creation and implementation merge.

### Parallel Three-Track Coordination

- Runtime/session lane: I193/SESSION-008 is Complete on `main`; implementation
  `404d7a4bf5b9c7dedeae479fe91fa5400b42d411` reached `main` through PR #216 merge `1b5461cd...`.
  RUNTIME-005, I188 and I189 retain their own owner-defined gates. I195 imports no Runtime/Session
  implementation authority from that completed lane.
- Desktop lane: I194/DESKTOP-001-D0 is Complete. The accepted decision packet remains decision-only;
  ADR-059 stays Proposed and all renderer/dependency/platform/mock-only implementation authority is
  reserved for a separately governed later child. #212 imports none of that authority.
- I195 remains the Dashboard lane owner candidate and is still ineffective until #212 itself reaches
  `main` after new-head validation, independent natural-person review and merge-time CAS.
- Archival recovery PRs #120/#121 remain immutable provenance and are not implementation authority.

If any parallel claim, branch or `main` update changes these facts, refresh the governance proposal
and repeat validation/review. An earlier clean snapshot never authorizes merging against a changed
target. Any later synchronization of the three shared derived files must preserve the union invariant
above; current-main-only replacement or Dashboard-only replacement is invalid.

## Scope

- Implement WEB-001-A only.
- Establish Dashboard-wide information architecture and one shared visual shell.
- Render the existing five GET-only surfaces through that shell when HTML is explicitly requested.
- Add `/extensions` HTML presentation while retaining its current JSON representation otherwise.
- Preserve ADR-031's loopback/read-only security posture for its approved initial routes; treat the
  existing `/extensions` GET-only surface as WEB-001/I129/current-implementation provenance rather
  than attributing it to ADR-031 §5.
- Preserve the existing data/snapshot ownership, masking, redaction and escaping boundaries.
- Treat UX as product acceptance:
  - consistent navigation and current-page hierarchy;
  - readable typography/spacing/content surfaces;
  - useful empty states;
  - responsive narrow/medium/desktop layouts;
  - bounded overflow for inherently wide tables/preformatted content;
  - semantic landmarks/headings;
  - keyboard-operable links/navigation and visible focus;
  - usable 200% zoom behavior;
  - WCAG 2.1 AA contrast thresholds for text and essential non-text affordances;
  - unchanged CSP-compatible resource usage;
  - truthful English `lang` metadata for the intentionally English-only first-slice UI;
  - no hover-only or color-only required information.
- Update user-facing English and Chinese README documentation to describe the local read-only
  Dashboard accurately; bilingual docs do not imply in-page Dashboard localization.

## Non-Goals

- SSE/live-log transport.
- Config writes/editor.
- Approval UI, permission decision UI or tool execution.
- Session mutation/actions or conversation control.
- WebSocket control.
- LAN/remote/tunnel access.
- Browser automation.
- TUI-037 behavior.
- Dashboard UI localization/i18n in this slice.
- New permission/persistence/remote-control models.
- New shared runtime/domain/session behavior.
- A second Dashboard-owned data source of truth.
- Completing WEB-001 as a whole or reusing/re-authorizing I129 acceptance.
- A speculative client framework, Node build pipeline or new dependency unless separately governed.

## Architecture And Security Invariants

- Listener remains `127.0.0.1` loopback under ADR-031.
- ADR-031 §5's approved initial route list is `/status`, `/history`, `/governance`, `/config`; I195
  does not rewrite that historical provenance. `/extensions` is consumed from the current WEB-001/I129
  GET-only implementation and gains HTML presentation only.
- No new non-GET business route is registered.
- Existing output-boundary snapshot redaction remains upstream of rendering, including extension data.
- Existing config masking remains authoritative.
- Every dynamic HTML value remains escaped.
- Existing JSON/plain-text representations remain available when HTML is not explicitly requested.
- Existing HTML CSP remains unchanged: `default-src 'none'; style-src 'unsafe-inline'`.
- Under that CSP, required UX must not depend on executable script, external/web fonts, raster/remote
  images or `data:` image resources; semantic text, inline CSS and inline SVG markup are the allowed
  first-slice presentation primitives.
- Existing runtime/domain/session/permission/config/extension ownership is reused; Dashboard is only
  a presentation layer for this slice.
- If implementation discovers that a shared API must change, that work is removed from I195 and
  separately governed through the mainline foundation lane before I195 consumes it.

## UX Acceptance Matrix

| Area | Acceptance | Repeatable Evidence |
|---|---|---|
| IA | Root and all five rendered pages share a coherent navigation model and page hierarchy. | Browser walkthrough of `/`, `/status`, `/history`, `/governance`, `/config`, `/extensions`. |
| Narrow layout | At 320×568, navigation remains usable and primary page layout has no accidental page-level horizontal scroll. | Manual viewport walkthrough; wide table/pre regions may scroll internally. |
| Medium layout | At 768×1024, hierarchy, spacing and data regions remain readable without clipped controls/links. | Manual viewport walkthrough. |
| Desktop layout | At 1440×900, content density remains readable and the shell does not stretch data into an unusable wall of text. | Manual viewport walkthrough. |
| Zoom | At 200% browser zoom, core navigation and content reading order remain usable. | Manual zoom walkthrough. |
| Keyboard | All Dashboard navigation is reachable with keyboard focus and focus is visibly identifiable. | Keyboard-only walkthrough. |
| Semantics | Navigation/main/headings have a logical DOM/reading hierarchy; required state is not communicated only by color/hover. | Markup review plus browser accessibility inspection without automation. |
| Contrast | Normal text is at least 4.5:1, large text at least 3:1, and essential non-text/focus affordances at least 3:1 against adjacent colors. | Recorded WCAG 2.1 AA contrast measurements for the rendered palette. |
| CSP resources | Required visuals work under unchanged `default-src 'none'; style-src 'unsafe-inline'` without script, external/web fonts, raster/remote or `data:` images. | Header inspection plus browser walkthrough; inline CSS and inline SVG markup only for non-text presentation. |
| Language | First-slice Dashboard UI remains English and rendered documents keep truthful `lang="en"`; bilingual README docs do not imply UI i18n. | Markup inspection across root and five rendered pages. |
| Empty/data states | Empty snapshots remain understandable and populated snapshots remain scannable. | Existing/new fixture-backed tests plus walkthrough. |
| Security UX | Masked/redacted values are represented consistently and raw hostile HTML is shown only as escaped text. | Adversarial unit/integration fixtures. |

Browser automation is explicitly excluded; this matrix is a deterministic manual acceptance record
bound to the rebuilt implementation head.

## Behavior Acceptance

- Given the rebuilt Talos Dashboard and an HTML-capable browser, when a user opens `/` or any of the
  five allowed surfaces, then one coherent read-only shell presents predictable navigation and a
  clear page/content hierarchy.
- Given `/extensions` with `Accept: text/html`, when it responds, then it is rendered safely through
  the shared shell; given no explicit HTML request, the existing JSON response remains available.
  This presentation parity consumes WEB-001/I129/current implementation provenance and does not
  claim ADR-031 §5 originally listed `/extensions`.
- Given `/status` or `/history`, when HTML is not explicitly accepted, then existing JSON negotiation
  behavior is preserved.
- Given `/governance` or `/config`, when HTML is not explicitly accepted, then existing plain-text
  negotiation behavior is preserved.
- Given masked config input or secrets/sensitive assignments in any snapshot, when a user views any
  representation, then output-boundary redaction and existing config masking prevent disclosure.
- Given hostile strings such as `<script>`/event-like HTML or special attribute characters, when
  rendered, then the browser receives escaped text rather than executable markup.
- Given POST/PUT/PATCH/DELETE attempts against Dashboard pages, when routing occurs, then no business
  mutation/action endpoint is available under this slice.
- Given the Dashboard server is started, when its bound socket is inspected, then the listener is
  still on `127.0.0.1`; no LAN/remote/tunnel path is introduced.
- Given the unchanged CSP, when all rendered pages are inspected, then required typography/icons/
  decoration do not rely on blocked resources and no JavaScript execution is introduced.
- Given the first-slice English UI, when root and all five HTML pages are inspected, then `lang="en"`
  remains truthful and no Desktop/i18n contract is imported into this Dashboard slice.
- Given existing runtime/session/permission/config/extension state, when implementation is reviewed,
  then I195 reads the existing snapshot/domain sources and adds no alternate durable or policy state.

## Planned Implementation Boundary

Expected production/test change is intentionally narrow:

- `crates/talos-dashboard/src/lib.rs` — shared shell/IA styling, `/extensions` HTML negotiation,
  presentation helpers and focused tests.
- A presentation-only module under `crates/talos-dashboard/src/` may be introduced only if the final
  code is materially clearer than retaining the bounded helpers in `lib.rs`; no new business layer.
- `README.md` and `README.zh-CN.md` — user-visible read-only Dashboard documentation.
- Owner/status documentation is synchronized through separate governance updates; no status-only
  commit is implementation completion evidence.

No Cargo/dependency change is planned for the first slice. Presentation assets must remain compatible
with the current CSP; no script, external font/image asset pipeline or client runtime is planned.

## Planned Validation

Targeted automated checks:

```bash
cargo test --locked -p talos-dashboard
cargo check --locked --workspace
cargo clippy --locked --workspace -- -D warnings
cargo test --locked --workspace
scripts/validate_project_governance.sh .
bash scripts/validate_collaboration_claims.sh .
git diff --check
```

Focused Dashboard regression evidence must cover at least:

- `/status`, `/history`, `/governance`, `/config`, `/extensions` explicit HTML negotiation;
- preserved JSON/plain-text responses without explicit HTML negotiation;
- `/extensions` default JSON compatibility;
- output-boundary redaction across all five surfaces;
- config masking;
- HTML escaping across all five surfaces;
- POST/PUT/PATCH/DELETE rejection across the Dashboard surface;
- `127.0.0.1` binding;
- unchanged HTML CSP;
- representative empty states and shell navigation structure.

Manual acceptance on the exact rebuilt implementation head:

- 320×568 viewport;
- 768×1024 viewport;
- 1440×900 viewport;
- 200% browser zoom;
- keyboard-only navigation/focus walkthrough;
- WCAG 2.1 AA contrast measurements;
- unchanged-CSP resource walkthrough;
- truthful English `lang="en"` markup inspection;
- root plus all five rendered pages;
- no browser automation.

## User-Facing Documentation

- `README.md`: describe how the local Dashboard presents the five read-only surfaces and retain the
  loopback/read-only caveat.
- `README.zh-CN.md`: equivalent Chinese user-facing description.
- `docs/backlog/active/WEB-001-embedded-web-control-surface.md`: cross-link WEB-001-A outcome after
  implementation without rewriting the parent's separately gated live/write/control/remote scope.

## Governance And Git Execution Gate

1. Governance-only claim material lands first.
2. Draft claim PR #212 obtained the actual PR number; `Claim Pending` is not persisted.
3. The finalized proposed claim uses `Claim State: Claimed` with the exact bounded Work Slice and
   real claim PR #212, but remains ineffective until target-branch merge.
4. Run both governance validators, `git diff --check`, applicable exact-head CI and independent
   natural-person review; disclose identity if the shared GitHub account is used.
5. Immediately before merge, re-fetch current `main`, all Active/Review/Planned/Blocked owners, open
   overlapping PRs and lane branches; repeat merge-time CAS. If target synchronization touches
   `docs/BOARD.md`, `docs/backlog/PRODUCT-BACKLOG.md`, or `docs/iterations/README.md`, preserve the
   union of every then-current target row plus I195's bounded derived row; replacing a shared file
   wholesale from either side is invalid.
6. Merge claim to `main` through the authorized review path.
7. Refresh `main`; record the exact effective claim merge-or-later SHA as implementation base.
8. Only then create a dedicated implementation worktree and branch:
   `feat/dashboard-I195-read-only-shell`.
9. Implementation PR targets `main`, receives exact-head validation and independent natural-person
   review, then repeats merge-time CAS against the then-current three-line baseline.
10. After implementation merge, governance closure records `Completion Commit: <pre-existing SHA>`;
    no state-only commit self-certifies completion.

The natural-person implementation-review language above is preserved as the original execution gate.
For this non-protected I195 slice, maintainer override `5326076971` explicitly superseded that
owner-local requirement at merge time by accepting independent AI technical review `5323625004`
plus human manual acceptance `5323801564`; protected-scope repository policy was not changed.

## Verification Evidence

- Governance planning branch base observed at creation:
  `c4bd9606c8bae63cb9bf11becd45846bf0805982`.
- Common three-track base:
  `23e4174bcfb036602ce2145026b872ec5c517289`.
- Prior Runtime-line synchronization after I193 claim merge is retained as historical evidence:
  #212 was synchronized to `main@0459b8afb1626783f21b54dbaf55a0ef84393cd7` by pure merge
  `0af41981cb3bebb8725a0ebd9ba20edf7637ab01`.
- Prior synchronization after I194 claim merge is historical evidence:
  `bdc9ea8e446dc694976f43ef99ac7323f7960584` synchronized #212 to `main@f778543c...`, followed by
  inventory refresh `c2370c6bf782d0c8530a7c3f7ff6711b53214820`.
- Prior synchronization after Desktop D0 decision merge is historical evidence:
  `81e7b6433e5ef7918051e273427570c51de076ed` synchronized #212 to `main@1beaca68...`, followed by
  exact-head `eb52cc7e4ac73e380a3f775e4de4d34a651c458f` and successful CI `31690077339`.
- Prior synchronization after I194 completion/closeout cleanup is historical evidence:
  `b65a8b44f02c75c24abe7e8ef545b9f88e0b1c80` synchronized #212 to `main@bd5a755e...`.
- Current synchronization after I193 implementation and closeout is pure two-parent commit
  `2656dc43730383353a8a4825896718760abc440e` with parents prior reviewed Dashboard head
  `d8beefb32a6605465a867cb2229d5495c2f95bee` and
  `main@556b5a4319085bf5250bccf4920e0dec0c6646c8`.
- Shared derived views are union-preserved after that sync: `docs/BOARD.md`,
  `docs/backlog/PRODUCT-BACKLOG.md`, and `docs/iterations/README.md` retain current-main rows and add
  one bounded I195 row each.
- I193/SESSION-008 is Complete at implementation `404d7a4b...`; PR #216 merged as `1b5461cd...` and
  CI `31691761892` passed. I194/DESKTOP-001-D0 is Complete; ADR-059 remains Proposed.
- Independent review comment `5288964884` returned REQUEST CHANGES on former exact head
  `d8beefb32a6605465a867cb2229d5495c2f95bee`; that result does not carry forward after these fixes.
- Earlier review comments and CI runs remain historical only and do not substitute for validation or
  independent review on the new exact head.
- Governance claim PR: #212.
- Final reviewed implementation head `1ee4aa3786785473069c735e1985c9d720b82e2f` passed exact-head
  CI `32087223234`; independent AI review `5323625004` and human browser acceptance `5323801564`
  passed; maintainer override `5326076971` accepted that implementation-review path; PR #233 merged
  by expected-head CAS as `490503db905bcd2eb2ab5e3b5487b1f542873d63`.

## Completion Evidence

Completion Commit: `490503db905bcd2eb2ab5e3b5487b1f542873d63`

- Source/reviewed implementation head: `1ee4aa3786785473069c735e1985c9d720b82e2f`.
- Exact-head CI `32087223234`: SUCCESS.
- Independent AI exact-head technical review: `5323625004`.
- Human maintainer manual browser acceptance: `5323801564`.
- Maintainer review-policy override accepting the AI review for I195: `5326076971`.
- Merge parent verification: first parent `8127fa579cef03a36e743a30003a682bc5f884b1`, second parent
  exactly `1ee4aa3786785473069c735e1985c9d720b82e2f`.
- No status-only closeout commit is used as its own completion evidence.

## Variance And Residuals

- WEB-001 SSE/log, writes, approvals, session actions, WebSocket and remote/LAN concerns remain
  separately governed residuals.
- TUI-037 remains independent.
- Any shared API requirement discovered during I195 is deferred to a separately governed mainline
  owner/claim before Dashboard consumption.
- Internationalization of Dashboard UI text is not silently added to this first slice; if product
  requirements later require localized in-page UI, it receives explicit scope/acceptance rather
  than being inferred from bilingual README documentation.
- Review noted a non-blocking `Accept: text/html;q=0` nuance; any correction is a separately governed
  content-negotiation compatibility refinement and is not an I195 completion blocker.

## Retrospective

Delivered the approved light/quiet Nord-derived read-only shell without widening the Dashboard
security boundary. The key execution variance was governance recovery: activation initially existed
only on the implementation branch and was corrected through #288 before the final #233 rebuild. The
maintainer later accepted independent AI exact-head technical review while retaining a human manual
browser walkthrough. Keeping the final implementation PR to three product/documentation files made
merge-time current-main CAS straightforward and preserved unrelated lane truth.
