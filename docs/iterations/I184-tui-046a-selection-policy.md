# Iteration I184: TUI-046-A Native Selection Policy

> Document status: Complete (2026-08-10)
> Published plan date: 2026-08-10
> Planned objective: decide and document a predictable native text-selection/copy interaction for Issue #134 while preserving ADR-054's Alternate Screen and application-owned transcript boundaries.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: an Accepted ADR-054 contract amendment (or replacement decision) plus a runnable cross-terminal validation plan that makes the TUI-046-B implementation boundary unambiguous.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 architecture session 2026-08-10 |
| Work Slice | Implement only I184/TUI-046-A: establish the native-selection versus mouse-capture contract, validate the causal interaction on the selected terminal matrix, and amend or replace ADR-054 with the explicit TUI-046-B gate; no Rust implementation or TUI-046-B authority. |
| Claimed At | 2026-08-10 |
| Source Issue | #134 |
| Governance Claim PR | #186 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent review `5236470750` approved exact claim head `00fc49376715fc1fc4e3bfe9e82465aea676b3bf` with no blockers and disclosed that a distinct natural-person reviewer used the shared `@wjhuang88` account. Exact-head CI `31358815361` passed all four jobs; merge-time CAS passed against `main@a403fdbae61372db4f830f2bf0c9adf2173a85ba`; PR #186 merged at `66d0f932370f679d491cb78f64dff9d84878479d`. |
| Implementation PR | #187 (decision/docs only) |
| Last Updated | 2026-08-10 |
| Handoff / Release Condition | I184 is closed by the existing #187 merge evidence; TUI-046-B requires a separate effective claim and remains responsible for implementation and full terminal acceptance. |

Before activation, follow `docs/sop/AGENT-COLLABORATION.md`. The claim is ineffective until the
finalized record is merged into `main`.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TUI-046-A | TUI-046 / Issue #134 | Ready | I183 Complete; current TerminalSession/input inventory | Accepted native-selection versus mouse-capture contract and ADR-054 amendment, with explicit TUI-046-B gate. |

### Scope

- Reproduce the current mouse-capture/native-selection conflict on the maintainer terminal and one materially different terminal/platform.
- Decide whether capture is disabled by default, explicitly configurable, gesture-overridden, or replaced by a bounded application-owned selection path.
- Amend ADR-054 or record a replacement decision, preserving Alternate Screen, transcript ownership, keyboard history navigation, privacy boundaries and `/copy` semantics.
- Define TUI-046-B's exact implementation, restoration, test and real-terminal acceptance contract.

### Non-Goals

- No Rust implementation, mouse-selection model, clipboard/export redesign or `/copy` behavior change.
- No TUI-042/#79 fix, permission/provider/session/runtime change, or abandonment of Alternate Screen.
- No TUI-046-B claim or implementation authorization.

### Acceptance

- Given the current lifecycle/input inventory, the decision identifies the causal interaction between mouse capture, native selection and wheel history without assuming Alternate Screen is the cause.
- Given the selected terminal matrix, the decision records observed default selection, override gestures, wheel behavior, redraw behavior and restoration results without overstating unsupported environments.
- ADR-054 or a replacement decision explicitly defines the default policy, keyboard history preservation, privacy boundary and the gate for TUI-046-B.
- The owner, Board and backlog state agree that TUI-046-B is Blocked until this decision is Accepted.

### Planned Validation

- Focused code/lifecycle inventory under `crates/talos-tui` and ADR-054 review.
- Recorded manual matrix on the maintainer's primary terminal and one materially different platform terminal. Every row names the exact terminal and version, OS/platform version, multiplexer and version or `none`, Talos SHA, gesture, wheel behavior, redraw condition, restoration result and copied-text observation; include Alacritty/Windows Terminal/macOS Terminal or iTerm2/tmux where applicable.
- `git diff --check`, `scripts/validate_project_governance.sh .`, `bash scripts/validate_collaboration_claims.sh .`, exact-head CI and independent architecture review.

### Documentation To Update

- `docs/decisions/054-alternate-screen-app-owned-transcript-rendering.md` or a replacement decision.
- `docs/reference/TUI-NATIVE-SELECTION-MATRIX.md` for exact causal and later implementation evidence.
- `docs/backlog/active/TUI-046-native-text-selection-copy.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, `docs/iterations/README.md`, and `.agent-governance/manifest.yaml`.
- Issue #134 with the exact decision/claim/validation links.

### Risks And Rollback

- Risk: selecting a policy that restores native selection but silently breaks wheel history or redraw stability.
- Rollback: keep ADR-054 unchanged and leave TUI-046-B Blocked until a narrower, independently reviewed contract is available.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-10 | Selection | I183/AG-7 completed at `edf903aa96574043294923ad60b0cefe9730f8c4`; no other active implementation iteration remains. TUI-046-A is selected as the P0 decision slice, pending effective claim. |
| 2026-08-10 | Activation | Review `5236470750` approved exact claim head `00fc49376715fc1fc4e3bfe9e82465aea676b3bf`; CI `31358815361` passed; merge-time CAS held against `main@a403fdba`; PR #186 merged at `66d0f932370f679d491cb78f64dff9d84878479d`. This decision branch starts from that effective claim. |
| 2026-08-10 | Inventory | `TerminalSession` unconditionally enables mouse capture; the input router consumes only wheel events and ignores pointer down/drag/up; PageUp/PageDown/Ctrl+Home/Ctrl+End independently navigate application history. The local environment identifies Alacritty 0.17.0 on macOS 26.5.2 with no multiplexer; mouse and clipboard observations still require a human terminal run. |
| 2026-08-10 | Submission | Proposed ADR-054 amendment and exact observation matrix submitted in draft PR #187. The amendment remains Proposed and the PR remains Draft until both current-baseline rows are observed. |
| 2026-08-10 | Baseline observation | On `33cc8dab23a38c387063d1265c230dfa0f8922d9`, Alacritty 0.17.0 (94e7c88) on macOS 26.5.2 (25F84), `TMUX=none`: ordinary drag produced no selection; Shift+drag selected and `Command+C`/`pbpaste` matched, but wheel scrolling did not carry the selection with projected content, edge-drag did not autoscroll, and resize cleared the selection. Native-only default is rejected; B recommendation is application-owned visible-cell selection with edge autoscroll. |
| 2026-08-10 | Second-terminal observation | On `c0fba2e92cace29fde4e2fc33fd26640058eddca`, Terminal.app 2.15 (`TERM=xterm-256color`) on macOS 26.5.2 (25F84), `TMUX=none`: ordinary and Shift+drag both failed while mouse reporting was enabled. Disabling View > Allow Mouse Reporting restored exact native cross-row selection, but wheel and edge-drag scrolled Terminal.app rather than Talos history, and repeated resize cleared the selection. `/quit` restored a clean shell without mouse escape leakage. This fills the second terminal row but not the published cross-platform validation requirement. |
| 2026-08-10 | Change control | The maintainer directed that cross-platform/manual terminal testing must occur after implementation and must not block development. This is a priority/gate correction, not added behavior: the published cross-platform matrix remains mandatory for TUI-046-B acceptance, but moves from an I184 pre-development dependency to the B implementation-PR merge gate. I184 enters Review with the application-owned selection boundary; B still requires its own effective claim before Rust work. |
| 2026-08-10 | Review correction | Review `5237634929` found a stale ADR heading, two stale derived `Active` states, and baseline matrix rows that overstated completeness despite missing restoration, fixture and failure-cleanup evidence. The correction keeps those rows as incomplete causal evidence and reserves full matrix acceptance for the exact B implementation head. After #187 merges, an evidence-only closeout must cite its real merge SHA/review/CI, mark the amendment Accepted and I184 Complete, and only then unlock the B claim. |
| 2026-08-10 | Closeout | PR #187 merged at existing decision evidence commit `f98488277803ee26180100089a48ef850939234b`; exact reviewed head `24e15db8d9df852c07fe08cc79ccc670fda36d27`, review `5237824299`, CI `31370219799` (4/4 green). ADR-054 I184 amendment is Accepted and I184 is Complete. The frozen cross-platform Planned Validation clause is recorded as an explicit deferred variance: it is a mandatory TUI-046-B acceptance gate, not satisfied by I184. Non-blocking follow-ups: matrix index entry, ADR index summary, and full B matrix procedure coverage are assigned to the B closeout/implementation owner. |

## Verification Evidence

- Claim exact-head CI `31358815361`, independent review `5236470750`, and merge-time CAS passed.
- Code inventory confirms mouse reporting is enabled during lifecycle setup while no application-owned
  arbitrary selection exists; keyboard history navigation is independent of mouse events.
- Manual baseline evidence covers Alacritty and Terminal.app on one macOS version and supports the
  causal decision. Per the recorded change-control direction, materially different platform testing
  is deferred to TUI-046-B implementation acceptance; independent review, exact-head CI and
  merge-time CAS are recorded in the Completion Evidence below.

## Completion Evidence

- Completion Commit: `f98488277803ee26180100089a48ef850939234b`
- PR #187 merged the reviewed decision evidence at that existing SHA; exact reviewed head was
  `24e15db8d9df852c07fe08cc79ccc670fda36d27`, review `5237824299`, CI `31370219799` (4/4 green).

## Variance And Residuals

- TUI-046-B is now unblocked for a separate claim; its implementation PR must complete the
  cross-platform matrix, fixture coverage, active redraw, and failure-cleanup acceptance before
  merge.
- Authorization Evidence review-state linkage is an existing validator/auditability gap owned by
  unclaimed GOV-004; I184 does not modify the collaboration validator.

## Retrospective

- Terminal-native overrides were useful causal probes but could not supply the required default
  interaction across Alacritty and Terminal.app. Separating raw observations from policy inference
  kept the matrix reusable; complete character, redraw and failure-cleanup evidence remains with B.
