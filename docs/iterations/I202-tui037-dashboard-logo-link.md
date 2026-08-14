# Iteration I202: Dashboard Availability In The Logo Prefix

> Document status: Active
> Published plan date: 2026-08-14
> Planned objective: move successful local Dashboard availability from the transient tips row into
> exactly one display-only Logo-prefix line while eliminating token disclosure and preserving the
> existing Dashboard failure path.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a rebuilt Talos TUI shows one complete, copyable and token-free Dashboard address
> with its startup Logo, never persists that line, never logs the bearer token, and keeps startup
> failures as error Tips.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex — mainline session 2026-08-14 |
| Work Slice | Implement only TUI-037 / I202: replace successful Dashboard-ready Tips with one display-only Logo-prefix line; render complete token-free plain-text URLs for ordinary and token-required loopback modes, add `authentication required` for the latter, remove token-bearing startup logging, preserve failure Tips, and prove no transcript/session/export/log/primary-screen persistence. No OSC 8, Dashboard route/auth/bind, persistence, conversation protocol, Desktop or I159-I162 behavior change. |
| Claimed At | 2026-08-14 |
| Source Issue | #104 |
| Governance Claim PR | #229 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim head `9c17711c47e1db1631a80eb615d772d8eba6c4fc` passed CI `31772628731`, independent natural-person review comment `5289857825` with shared-account disclosure, and merge-time CAS comment `5289870196`; PR #229 merged as `d801c8d1f0ce37727baf49258be780baa41816f4`. Independent exact-head security review remains mandatory for the implementation PR. |
| Implementation PR | Not started |
| Last Updated | 2026-08-14 |
| Handoff / Release Condition | Implement and validate only I202, then obtain independent natural-person exact-head security review and merge-time CAS before implementation merge. |

The claim became effective through merge `d801c8d1f0ce37727baf49258be780baa41816f4`.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TUI-037 | None / Issue #104 | Ready | TUI-005, TUI-028, TUI-035, ADR-031, ADR-054; I158 Complete | One runnable display-only Dashboard Logo line with safe plain-text fallback as the current complete behavior |

### Target-Branch Baseline At Planning

- Target branch: `main`.
- Exact `origin/main` at governance branch creation:
  `f123e534ce864a89eb3cabfc68f4a1518201c2d0`.
- Governance branch: `docs/tui-I202-dashboard-logo-link-claim`.
- No implementation branch or worktree exists for I202.
- The implementation base must be the effective claim merge commit or a later current `main` after
  repeating the full inventory and CAS preflight.

### Current Non-Terminal Inventory And Disposition

| Iteration / proposal | Observed state | I202 disposition |
|---|---|---|
| I188 / PR #228 | Target owner Planned/Claimed; implementation PR in Review | Preserve unchanged while its independent process/security review remains pending; no TOOL-024 scope is imported. |
| I189 | Planned / Claimed / unactivated | Preserve unchanged; no permission-foundation scope is imported. |
| I195 | Planned / Claimed on `main` through merge `f123e534` | Preserve the independent Dashboard product lane; I202 changes only CLI-to-TUI availability projection and no Dashboard page. |
| I196 / PR #226 | Planned / proposed claim, not on `main` | Preserve the mainline P0 proposal; #104 was explicitly selected first and does not alter P0. |
| I197-I201 / PR #227 | Planned coordination proposal, not on `main` | Preserve the proposed long-task order; I202 closes before that long task begins. |
| I159-I162 | Blocked | Keep blocked under their published dependency chain; I159 waits for I202 terminal disposition. |
| I164 | Paused / superseded | Do not resume. |

There is no Active iteration. I188 remains the only implementation Review and is waiting on an
independent external gate; the maintainer explicitly selected the non-overlapping #104 correction
before beginning the proposed long task.

### Resolved Design Gates

- Current ratatui 0.30/crossterm full-frame rendering does not expose a safe hyperlink attribute in
  the cell buffer. Injecting OSC 8 bytes into cell symbols is rejected because it breaks width,
  diff and visible-selection semantics.
- I202 therefore emits no OSC 8. Ordinary loopback shows the complete token-free URL as copyable
  plain text.
- Token-required loopback shows the same token-free base URL plus `authentication required` and is
  non-clickable. It never emits, persists or logs the token.
- A later clickable hyperlink requires a separate owner after the rendering stack offers a safe
  primitive; it is not an incomplete part of I202.

### Scope

- Add one private TUI startup-availability value owned by the TUI application, not the conversation
  protocol or transcript.
- Project that value as exactly one Logo-prefix line in ordinary and token-required loopback modes.
- Remove successful Dashboard availability from `UiOutput::Tip`; retain startup failure as an error
  Tip.
- Remove the bearer token from startup tracing and prove no visible/logged/persisted secret path.
- Preserve full URL copyability and deterministic wrapping at wide and narrow widths.
- Update English and Chinese user-facing Dashboard/TUI startup documentation.

### Non-Goals

- No OSC 8 or other terminal hyperlink escape sequence.
- No Dashboard route, response, authentication, bind address, token generation or configuration
  change.
- No conversation public enum/API, transcript, session schema, export, primary-screen summary or
  persistence change.
- No browser opener/automation, Desktop/Dashboard product implementation, new dependency, Cargo
  manifest/lockfile change, `unsafe`, I159-I162, release or tag action.

### Acceptance

- Given ordinary loopback Dashboard startup succeeds, when the first and later TUI frames render,
  then exactly one Logo-prefix line contains the complete token-free URL, no Dashboard success Tip
  exists, and no raw terminal escape sequence is emitted.
- Given token-required loopback startup succeeds, when the Logo prefix renders, then the complete
  token-free base URL and `authentication required` are visible, non-clickable and contain no token.
- Given Dashboard startup fails, when the TUI renders, then the existing error Tip is visible and no
  Dashboard Logo line is created.
- Given narrow width, scroll, resize, selection/copy and first-message transitions, when frames are
  projected, then the Dashboard row behaves as part of the display-only Logo prefix without
  corrupting history, composer, status or selection geometry.
- Given session persistence, resume, export, logs and primary-screen exit output, when inspected,
  then neither the Dashboard Logo line nor bearer token appears.

### Planned Validation

- Focused CLI tests for ordinary/restricted/failure startup projection and secret-free tracing data.
- Focused TUI splash/full-frame tests at wide and narrow widths, scrolling, first-message transition,
  selection/copy and transcript/export isolation.
- Rebuilt-binary terminal matrix: Alacritty, macOS Terminal and tmux when available; record exact
  head, terminal versions, ordinary/restricted/failure cases, resize/scroll/copy and restore result.
- `cargo fmt --all -- --check`
- `cargo check --locked --workspace`
- `cargo clippy --locked --workspace -- -D warnings`
- `cargo test --locked --workspace`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`

### Documentation To Update

- `README.md`
- `README.zh-CN.md`
- `docs/backlog/active/TUI-037-dashboard-logo-link.md`
- `docs/iterations/I202-tui037-dashboard-logo-link.md`
- owner-first derived synchronization in `docs/backlog/PRODUCT-BACKLOG.md`,
  `docs/iterations/README.md` and `docs/BOARD.md`
- Issue #104 status and closure record

### Risks And Rollback

- Risk: treating availability as conversation output could persist a display-only fact.
- Risk: a token or control sequence could leak into logs, cell symbols, copied text or exports.
- Risk: adding a prefix row could break narrow layout or history/selection anchors.
- Rollback: remove the private startup availability value and Logo row while retaining the existing
  failure Tip; do not restore token-bearing logs or a successful generic Tip.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-14 | Selection | Maintainer selected #104 for complete implementation and closure before the mainline long task. Claim remains ineffective until finalized governance PR merge; no implementation branch exists. |
| 2026-08-14 | Activation | PR #229 merged as `d801c8d1f0ce37727baf49258be780baa41816f4` after exact-head CI `31772628731`, independent review comment `5289857825` and merge-time CAS comment `5289870196`. Implementation branch `feat/tui-I202-dashboard-logo-availability` starts exactly at that merge. I188 remains Review; I189/I195 remain Planned/Claimed; I196-I201 remain proposals outside `main`; I159-I162 remain Blocked and I164 Paused. |

## Verification Evidence

- Claim governance: PR #229 head `9c17711c47e1db1631a80eb615d772d8eba6c4fc` passed CI
  `31772628731`, independent review comment `5289857825` and merge-time CAS comment `5289870196`,
  then merged as `d801c8d1f0ce37727baf49258be780baa41816f4`.
- Local implementation validation on 2026-08-14 passed `cargo fmt --all -- --check`,
  `cargo check --locked --workspace`, `cargo clippy --locked --workspace -- -D warnings`, a second
  complete `cargo test --locked --workspace` run, both governance validators with zero warnings,
  and `git diff --check`. The first workspace-test run had one unrelated transient SQLite disk-I/O
  failure in `mcp_client_e2e`; its isolated rerun and the subsequent full workspace rerun passed.
- Focused CLI/TUI tests cover token-free tracing, failure Tip behavior, wide/narrow Logo rendering,
  authenticated wording, scroll-prefix ownership and transcript isolation.
- An isolated rebuilt-binary PTY smoke showed the successful plain-text Dashboard row in the Logo
  prefix and normal terminal restoration; the sandbox-denied listener run separately exercised the
  failure Tip. This is supplemental evidence, not the required real-terminal matrix.
- Finalized implementation-PR exact-head CI, independent security review, and the Alacritty /
  Terminal.app / tmux matrix remain pending.

## Completion Evidence

- No completion evidence. A status-only commit cannot certify this iteration.

## Variance And Residuals

- Clickable OSC 8 output is explicitly excluded under the current unsafe renderer capability. Any
  future hyperlink work requires a separate owner and claim.
- ADR-031 requires the opt-in bearer token to remain memory-only and absent from logs, but defines no
  supported operator token-delivery channel. I202 restores compliance by removing the violating log;
  a replacement delivery boundary is pre-existing residual work requiring a separate owner and
  security decision.

## Retrospective

- Outcome: pending implementation.
- Documentation: pending implementation and owner-first closeout.
- Lessons: pending.
