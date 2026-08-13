# Iteration I195: Dashboard Read-Only Visual Shell

> Document status: Planned
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
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-5.6 Sol — Dashboard governance session 2026-08-13 |
| Work Slice | Implement only WEB-001-A / I195 Dashboard-wide IA and the first cohesive read-only visual shell over the existing GET-only loopback `/status`, `/history`, `/governance`, `/config`, and `/extensions` surfaces. Preserve `127.0.0.1`, current config masking, output redaction, HTML escaping and JSON/plain-text negotiation; introduce no write/control/remote/session-mutation capability or duplicated shared business logic. |
| Claimed At | 2026-08-13 |
| Source Issue | None |
| Governance Claim PR | #212 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent natural-person exact-head review is required before claim merge and before implementation merge. If repository operations use the shared `@wjhuang88` account, the reviewer must explicitly disclose their natural-person identity. This proposed `Claimed` record remains ineffective until PR #212 reaches `main`. |
| Implementation PR | Not started |
| Last Updated | 2026-08-13 |
| Handoff / Release Condition | Pass exact-head governance/CI checks, obtain independent natural-person approval, repeat merge-time CAS against current `main` and all three lanes, and merge PR #212 to `main`; only then create the isolated implementation worktree/branch from the claim merge commit or a later current `main`. |

The `Claimed` record above is proposed on PR #212. It is not effective ownership until that exact
record reaches `main`; no Dashboard implementation branch or worktree is authorized beforehand.

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

- `main` advanced after planning to
  `0459b8afb1626783f21b54dbaf55a0ef84393cd7` after I193/SESSION-008-B claim merge #210 and its
  target-branch synchronization commit.
- #212 was synchronized by pure two-parent merge commit
  `0af41981cb3bebb8725a0ebd9ba20edf7637ab01` whose parents are the previously reviewed #212 head
  `556200a2bedd5397a0015d426bdeb0d55d5a2082` and `main@0459b8af...`.
- The pure merge intentionally took current-main truth for shared indexes; subsequent governance-only
  commits reapply I195 derived-index rows and refresh this owner on top of the synchronized baseline.
- I194/Desktop remains an unmerged parallel proposal and is not imported as target-branch truth.

### Current Non-Terminal Iteration Inventory

After synchronizing to `main@0459b8af...`, there remains no Active or Review iteration in the
current operating set. The following non-terminal target-branch iterations remain independent:

| Iteration | State | I195 Disposition |
|---|---|---|
| I159 | Blocked | Keep blocked; TUI-037 disposition remains its own gate. No Dashboard implementation is used to satisfy it. |
| I160 | Blocked | Keep blocked on I159 Complete. No overlap. |
| I161 | Blocked | Keep blocked on I160 Complete plus its security-review plan. No overlap. |
| I162 | Blocked | Keep blocked on I161 Complete plus release-readiness authorization. No overlap. |
| I164 | Paused | Preserve the superseded startup-inline target; do not resume. |
| I188 | Planned / Claimed | Keep unactivated; TOOL-024-A decision-only process/permission work is independent. |
| I189 | Planned / Claimed | Keep unactivated; PERM-006-A permission-foundation scope is independent. |
| I193 | Planned / Claimed | Claim #210 is effective on `main`; keep I193 unactivated and independent. I195 consumes no SESSION-008-B implementation authority. |

No existing target-branch owner authorizes I195 implementation. This table must be refreshed again
immediately before claim merge, implementation branch creation and implementation merge.

### Parallel Three-Track Coordination

- Runtime/session lane: PR #210 / I193 has now merged to `main`; I193 is target-branch truth and
  remains Planned / Claimed pending its own explicit activation. Dashboard does **not** reuse or
  implement I193 scope.
- Desktop lane: PR #211 / proposed I194 remains unmerged as of this synchronization. Dashboard does
  **not** use I194 and does not import its proposed owner/index state into current-main truth.
- I195 remains the Dashboard lane owner candidate and is still ineffective until #212 itself reaches
  `main` after new-head review and merge-time CAS.
- Archival recovery PRs #120/#121 remain immutable provenance and are not implementation authority.

If any parallel claim, branch or `main` update changes these facts, refresh the governance proposal
and repeat validation/review. An earlier clean snapshot never authorizes merging against a changed
target.

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
   overlapping PRs and lane branches; repeat merge-time CAS.
6. Merge claim to `main` through the authorized review path.
7. Refresh `main`; record the exact effective claim merge-or-later SHA as implementation base.
8. Only then create a dedicated implementation worktree and branch:
   `feat/dashboard-I195-read-only-shell`.
9. Implementation PR targets `main`, receives exact-head validation and independent natural-person
   review, then repeats merge-time CAS against the then-current three-line baseline.
10. After implementation merge, governance closure records `Completion Commit: <pre-existing SHA>`;
    no state-only commit self-certifies completion.

## Verification Evidence

- Governance planning branch base observed at creation:
  `c4bd9606c8bae63cb9bf11becd45846bf0805982`.
- Common three-track base:
  `23e4174bcfb036602ce2145026b872ec5c517289`.
- Runtime-line I193 collision was initially avoided based on PR #210; #210 later merged and #212 was
  synchronized to `main@0459b8afb1626783f21b54dbaf55a0ef84393cd7` by pure merge
  `0af41981cb3bebb8725a0ebd9ba20edf7637ab01`.
- Desktop-line I194 remains an unmerged parallel proposal as of this synchronization.
- Independent review comment `5277157606` on former exact head `556200a2...` returned NEEDS CHANGES
  solely for main drift/mergeability; its positive content findings do not carry forward as approval
  of the new head.
- Governance claim PR: #212.
- New exact-head validator/CI/review and merge-time CAS evidence: pending.

## Completion Evidence

No completion evidence. No production implementation exists under I195. A governance/status commit
cannot serve as `Completion Commit`.

## Variance And Residuals

- WEB-001 SSE/log, writes, approvals, session actions, WebSocket and remote/LAN concerns remain
  separately governed residuals.
- TUI-037 remains independent.
- Any shared API requirement discovered during I195 is deferred to a separately governed mainline
  owner/claim before Dashboard consumption.
- Internationalization of Dashboard UI text is not silently added to this first slice; if product
  requirements later require localized in-page UI, it receives explicit scope/acceptance rather
  than being inferred from bilingual README documentation.

## Retrospective

Pending activation and execution.
