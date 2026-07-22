# Long-Running Task: Current Product Closeout And Adaptive History Visibility

| Field | Value |
|---|---|
| Task ID | `2026-07-22-product-closeout-and-history-visibility-long-task` |
| Owner | Maintainer-dispatched frontline developer; no subagent delegation |
| Created | 2026-07-22 |
| Status | Planned — execution starts only when the maintainer dispatches a phase |
| Branch | `main` only; direct commits, no feature branches |
| SOP | `docs/sop/LONG-RUNNING-TASK.md` |
| Relationship | Successor closeout package for the active 2026-07-20 provider/multimodal program; TUI-034 is a new, independent follow-on objective. |

## Outcome

Close the remaining code-owned work without converting unperformed human acceptance into a false
Complete claim:

1. Finish MODEL-008-B / I148's mock-proven discovery → selection → immediate session activation
   path, while retaining manual model-ID fallback.
2. Refresh I153's automated evidence sufficiently to decide whether I154 can be activated, then
   implement the separately scoped MODEL-009-E / I154 `read_image` tool only through its existing
   capability, permission, validation, digest, and provider-boundary controls.
3. Refine and then implement TUI-034 so history and retained tool-result lines use the actual
   viewport display width rather than fixed character caps, without weakening vertical output
   limits or tool-summary boundaries.
4. Deliver a precise maintainer walkthrough packet for the outstanding real-terminal gates.

This task does **not** authorize a release. REL-002 remains NO-GO. No tag, GitHub Release,
crates.io publish, Pages deployment, real provider credential, or paid API call is permitted.

## Non-Negotiable Disposition Of Existing Work

| Owner | Current state | Disposition in this task |
|---|---|---|
| I145 / TUI-026 | Review | Do not change to Complete without maintainer terminal evidence. Include its walkthrough in the final packet only. |
| I146 / TUI-033 | Review | Do not change to Complete without maintainer terminal evidence. No feature changes unless a regression is found while executing another phase. |
| I147 / MODEL-008-A | Review | Do not change to Complete without maintainer terminal evidence. |
| I148 / MODEL-008-B | Review | P1 code closure is complete; only the maintainer terminal walkthrough remains. |
| I150 / MODEL-009-B | Review | Preserve status; run regressions when affected by I154. |
| I151 / I152 | Review, code-level security acceptance | Preserve status pending real-terminal evidence; I154 must reuse their proven controls rather than recreate a parallel path. |
| I153 | Review | Refresh automated evidence and checklist; it remains Review until maintainer walkthrough evidence exists. |
| I154 / MODEL-009-E | Planned / Blocked | May activate only after Phase P2 records that its code-level prerequisites and evidence-refresh gate are satisfied. |
| TUI-034 | Refinement | Must pass its rendering-boundary refinement gate before a new I155 implementation baseline is created. |

## In Scope

- I148 closeout using mock HTTP/provider fixtures only.
- I153 evidence refresh needed to safely activate I154; no release action.
- I154 / MODEL-009-E, including security review because it combines agent tools, external paths,
  image validation, permissions, session continuation, and provider adapters.
- TUI-034 refinement plus a new I155 only after TUI-034 becomes Ready.
- Focused README/site/config/ADR/iteration/backlog/Board synchronization for changed user-visible
  behavior, and a final human acceptance packet.

## Out Of Scope

- New provider protocols, arbitrary headers/request JSON, OAuth, remote image URLs, audio/video,
  OCR, clipboard ingestion, and automatic reading of paths found in normal user text.
- Any expansion of tool-summary eligibility, TUI-015's 30-line threshold, or its 3/3 head-tail
  policy.
- Wrapping tool-call or approval arguments; TUI-025's one-line policy remains binding.
- New configuration options, fullscreen/pager output viewers, new `unsafe`, or new native/C
  dependencies without a separate ADR and security review.
- Marking any Review item Complete based only on tests, or treating a mock as a real provider
  acceptance.

## Ordered Task Items

| ID | Task | Expected output | Depends on | Completion gate | Fallback | Status |
|---|---|---|---|---|---|---|
| P0 | Establish this successor task and reconcile scope | This owner record, original-program change-control link, Board entry, and current-state evidence | None | Governance + diff checks; explicit inventory above remains true | If current owner docs conflict, stop and report file/line evidence | Complete on creation |
| P1 | I148 discovery activation closeout | Mock-proven discover → select → `apply` model → rebuild current session → status/picker reflects the active identity; failure retains current session/config and exposes manual fallback | P0 | Two protocol fixtures; picker/bridge/session lifecycle tests; atomicity/redaction tests; full locked ladder | Keep discovery persistence and manual fallback; leave I148 Review with its terminal-only gate | Complete — evidence commits `23db287`, `187f13d`, `4d5f8d7`, `834400b`, `a01edc5` |
| P2 | I153 prerequisite/evidence refresh and I154 activation decision | Append-only evidence update stating whether I154's code prerequisites are met; an I154 activation record only if they are | P1, I151/I152 accepted code state | Security-boundary inventory, I153 regression replay, no unresolved critical path; no false real-terminal Complete claim | Keep I154 Blocked and provide exact missing condition | Planned |
| P3 | I154 `read_image` tool | Supported-only registered tool; exact-path approval; shared image ingestion/digest revalidation; provider-neutral continuation artifact; two adapter fixtures; safe history/provenance; unchanged text `read` | P2 | Tool registry/presentation, permission Allow/Ask/Deny, symlink/replacement/decoder adversarial, agent-session continuation, OpenAI/Anthropic, text-only/history regression tests; security review; full locked ladder | Do not expose tool; amend ADR-050 with evidence and retain explicit `/attach` only | Planned |
| P4 | TUI-034 rendering refinement | Fixed-cap inventory, active-vs-legacy `ToolCallBubble` reachability evidence, chosen continuation-row representation, width/height contract, and TUI-034 changed to Ready or explicitly left Refinement | P0; must not overlap an Active I154 | Actual `Buffer`/`InlineFrame` or active-renderer spike at 80/120/160 columns; CJK/emoji/newline observations; no terminal-autowrap assumption | Keep TUI-034 Refinement and record the smallest unresolved rendering boundary | Planned |
| P5 | I155 adaptive history implementation | New append-only I155 baseline followed by viewport-width-aware tool history rendering; preserved TUI-015/TUI-025 behavior; updated user docs if behavior is described publicly | P4; P3 must be Complete or explicitly stopped so only one iteration is active | Active-renderer tests at 80/120/160; CJK/emoji/newline and former-120-boundary tests; TUI-015/TUI-025 regressions; two-terminal manual packet; full locked ladder | Revert only the uncommitted phase changes; leave TUI-034 Ready with refinement evidence | Planned |
| P6 | Integrated closeout and maintainer evidence packet | Owner-doc/status sync, issue-sync check, docs synchronization, residual register, and short real-terminal checklist for I145-I153/I154/I155 | P3 and P5, or explicit stop record for either | Full locked ladder, governance, diff check, clean tree after push; no unauthorized state promotion or release | Record partial completion and exact next phase in this task | Planned |

## Technical Requirements By Implementation Phase

### P1 — I148: do not confuse persistence with activation

- `discover_provider_models` remains bounded (timeout, response-byte limit, model-count limit) and
  never logs an API key.
- A successful discovery may persist discovered IDs atomically only with the selected provider
  update; a failed discovery must not corrupt or partially replace config.
- Selecting a discovered model must travel through the same structured identity and model
  lifecycle path as `/model`, not reconstitute a slash-command string.
- The active session must rebuild exactly once after selection. On failure, retain the old active
  model/session and show an actionable error; never claim the new model is active merely because it
  was written to config.
- Tests must cover OpenAI-compatible and Anthropic-compatible discovery fixtures, selection,
  rebuild success/failure, manual-ID fallback, duplicate/update behavior, and credential redaction.

### P3 — I154: `read_image` is a distinct tool, never a text-path side effect

- Expose `read_image` to the model only when `ImageInputCapability::Supported`; it is absent for
  `Unknown` and `Unsupported`.
- It may read only the path the model explicitly supplies through this tool. A normal message,
  `read` tool call, or pasted string that resembles a file path must not trigger image ingestion.
- Reuse, rather than duplicate, exact-path permission evaluation and approval semantics. The
  permission/tool identity must remain precise; an approval for another tool or another path must
  not authorize `read_image`.
- Reuse validation and the content digest/TOCTOU guard. Re-authorize/canonicalize and verify the
  digest at the byte-read boundary. Fail closed on path retargeting, replacement, invalid format,
  limits, decoder error/panic, or capability change.
- The text tool result and TUI history contain only a bounded safe summary/provenance. Image bytes,
  data URLs, and canonical full paths never appear in scrollback, exports, logs, or persistence.
- Carry a Talos-owned artifact to the *next* provider request once, mapped only inside the OpenAI
  and Anthropic adapters. Preserve ordered text/image parts and existing text-only request shape.

### P4/P5 — TUI-034: width is a layout concern, not a data-release policy

- Inventory and classify fixed limits such as the known 120-character retained-result cap and the
  legacy 200-character bubble cap before changing code.
- Replace only active history/tool-result fixed per-line limits with live display-cell width.
  Explicit newlines form logical lines; width overflow creates renderer-accounted continuation rows
  with stable tool-result styling, not terminal-autowrap-dependent rows.
- Use display width for CJK, emoji, and combining text. Never split UTF-8 or overflow the area.
- Keep summary-eligible tools summarized; keep the TUI-015 30-line decision and 3 head/3 tail
  retention; keep TUI-025 arguments and approval arguments one line with width-aware ellipsis.
- Verify live history layout with the active renderer, not only a string helper. If a legacy widget
  is unreachable, remove it only with reachability evidence and focused regressions; otherwise
  bring it into the same classified policy.

## Validation And Acceptance Evidence

Every code phase runs and records all of:

```text
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
git diff --check
```

Before each phase begins, run `git switch main && git pull --ff-only origin main && git status -sb`.
Review only explicitly staged paths with `git diff --cached`; scan the staged diff for secrets.

The developer may use mock HTTP/provider fixtures and local test assets. It must not use a real
provider key or paid endpoint. A browser/terminal walkthrough supplied by a maintainer is evidence,
not permission to change unrelated scopes.

## Branch, Commit, Push, And Stop Plan

- Work directly on `main`; no feature branch, rebase, force-push, `git reset --hard`, `git add .`,
  or `git add -A`.
- Each completed phase is one logical commit, after its completion gate, followed immediately by
  `git push origin main`.
- Use conventional messages with the story/iteration ID and model marker, for example:
  `feat(cli): close discovered-model activation (#MODEL-008-B #I148) [model:gpt-5]`.
- After pushing P1, P2, P3, P4, P5, or P6, append a checkpoint here and **stop for maintainer
  instruction**. Do not start the next phase merely because the previous phase is green.
- A docs-only P0 commit is permitted and is the only action authorized by creation of this plan.

## Checkpoint Template

```markdown
## Checkpoint <phase> — YYYY-MM-DD

- Completed task items:
- Commit pushed:
- Changed owner artifacts:
- Commands and exit results:
- Acceptance evidence / remaining human gate:
- Open risks or deviations:
- Next task item:
- Resume: `git switch main && git pull --ff-only origin main`; read this checkpoint and the next owner doc.
```

## Checkpoints

### Checkpoint P0 — 2026-07-22

- Completed task items: P0 — successor scope, owner/doc disposition, and phase gates published.
- Commit pushed: `e7754bc` on `origin/main`.
- Changed owner artifacts: this task record; the 2026-07-20 program change-control entry; the
  derived Board row.
- Commands and exit results:
  - `scripts/validate_project_governance.sh .` → exit 0, 0 warnings.
  - `git diff --check` → exit 0.
- Acceptance evidence / remaining human gate: P1 is fully mock-testable and may be dispatched;
  I145/I146/I147/I150-I153 still require the maintainer walkthrough listed below before any
  Complete status.
- Open risks or deviations: no GitHub issue is mapped to TUI-034. I154 and I155 remain unstarted.
- Next task item: P1 — I148 discovery → selection → immediate activation closeout.
- Resume: `git switch main && git pull --ff-only origin main`; read this checkpoint, then
  `docs/iterations/I148-model-discovery-manual-fallback-activation.md` and its MODEL-008-B story.

### Checkpoint P1 — 2026-07-22

- Completed task items: P1 — I148 discovery → selection → immediate activation closeout.
- Completion Commit: `a01edc5` — final P1-fix4 review closure; the preceding implementation and
  evidence commits are `23db287`, `187f13d`, `4d5f8d7`, and `834400b`.
- Commit pushed: `23db287` on `origin/main` (P1 tests), followed by `187f13d`,
  `4d5f8d7`, `834400b`, and `a01edc5` to close provider-identity, bridge,
  lifecycle, semver, and test-safety review findings.
- Changed owner artifacts: I148 iteration doc (execution record appended, status → Review);
  iterations README (I148 row → Review); BOARD (program row updated).
- Commands and exit results:
  - `cargo fmt --all -- --check` → exit 0.
  - `cargo check --workspace --locked` → exit 0.
  - `cargo clippy --workspace --locked -- -D warnings` → exit 0.
  - `cargo test --workspace --locked` → exit 0, 0 failures.
  - `scripts/validate_project_governance.sh .` → exit 0, 0 warnings.
  - `git diff --check` → exit 0.
- Acceptance evidence / remaining human gate: 7 new P1 tests prove the closed loop at the data
  level (discovery → all_models visibility → structured identity → activation). The code path
  (handle_register_custom_provider → atomic Config::save → /model picker → UserInput::SwitchModel
  → SessionLifecycleRequest::ModelSwitch → handle_session_model → session rebuild) was already
  implemented in the R9 rework; these tests add missing coverage. Real-terminal walkthrough of
  the discovery → selection → activation flow remains a human gate for I148 Review → Complete.
- Open risks or deviations: no code changes were needed — the R9 rework already implemented the
  closed loop. This commit is test-only. No GitHub issue is mapped to MODEL-008-B.
- Next task item: P2 — I153 evidence refresh and I154 activation decision. Must not start without
  maintainer instruction.
- Resume: `git switch main && git pull --ff-only origin main`; read this checkpoint, then the
  P2 task description in this file.

## Hard Stops

Stop, append the checkpoint, and request maintainer direction when any applies:

1. I148 requires real credentials, a new provider protocol, arbitrary headers, or custom request
   JSON to close its mock-proven path.
2. I154 requires a new `unsafe`, native/C dependency, broader permission behavior, automatic
   text-path reading, or cannot safely carry a one-turn image artifact through both adapters.
3. TUI-034 requires an unapproved change to summary/head-tail policy, a global layout redesign, or
   cannot obtain accurate row accounting in the active renderer.
4. Locked validation fails twice consecutively for a root cause outside the phase scope.
5. Working-tree ownership is uncertain, an API key is encountered, a release/deploy is requested,
   or an existing Review item can only be closed by a human terminal check.

## Residual Work And Issue Sync

- A new product idea goes to `docs/proposals/`; an implementable residual goes to the relevant
  backlog owner with status and acceptance.
- Record reusable failures/lessons through `docs/sop/EVOLUTION-FEEDBACK.md`.
- Before P6, inspect the selected owner docs for a source GitHub issue. If one exists, comment with
  the new status, commit, and summary; close it only when the owner story is Complete or Cancelled.
  No GitHub issue is currently mapped to TUI-034.

## Unified Review Closure Packet (Not Delegable To Automation)

Run the following in one clean, disposable Talos profile after building the current binary. A
single observed defect keeps only its owning iteration in Review; it must not block recording
independent passing cases. Record terminal, platform, binary commit, each case result, and any
redacted screenshot/transcript in the owner iteration documents. A later status-only commit may
cite this packet only after an already-existing evidence or repair commit is named.

### Setup

1. Start from clean `main`, build `target/debug/talos`, and use a temporary HOME/config; never put
   a real production credential in a transcript.
2. Use the built-in/mock path for deterministic checks. A disposable local compatible endpoint is
   acceptable for discovery; a paid provider is not required.
3. Before testing, confirm the status bar has a known active model. After testing, remove the
   temporary profile and any generated fixture image.

### Cases and owning Reviews

| Case | Expected observable result | Owner(s) |
|---|---|---|
| Steering queue | During a tool-running turn, enqueue at least seven messages. FIFO preview appears, `+N more` is accurate, and it disappears after drain. | I145 |
| Parameterless menus | Bare `/model` and `/connect` open their menus. Argument-bearing forms show a correction and do not mutate config/session. Tab/Enter leave no trailing argument space; Escape cancels. | I146 |
| Wizard | `/connect` → Add custom provider runs Name → Protocol → Base URL → API key → Confirm; key remains masked, Esc causes no save, and duplicate name shows the explicit update path. | I147 |
| Discovery and activation | A mock/disposable provider discovers IDs. Select an ID containing `/` or `@`; the provider/model status changes and the current session rebuilds once. Discovery failure retains the old active session and permits manual configuration. | I148 |
| Capability and attachment | Unsupported and Unknown models reject `/attach` before file access. A Supported model accepts a valid local PNG/JPEG/GIF/WebP; an invalid image is rejected. External path asks for exact-path approval; list/detach/cancel work; history shows only a safe summary; a text-only turn still works. | I150, I151, I152 |
| Provider mapping | With a maintainer-owned disposable configured endpoint, one accepted image produces the expected compatible request or a safe, actionable provider failure; no path/data URL/API key appears in TUI history. | I152, I153 |

Do not run I154 `read_image` steps here: it remains Planned/Blocked and is a distinct future
iteration.

## Maintainer Real-Terminal Packet (Not Delegable To Automation)

The final report must ask the maintainer to execute and record:

1. I145: enqueue several steering messages during a turn; verify FIFO preview, `+N more`, and
   disappearance after drain.
2. I146/I147: bare and parameterized `/model`/`/connect`, search/cancel, custom-provider wizard,
   update/cancel behavior, and no visible secret.
3. I148: use a disposable/mock provider endpoint; discover, choose a model, verify immediate
   status/session transition, then verify a discovery failure retains the old session and offers
   manual entry.
4. I151/I152/I154: Supported/Unknown/Unsupported gating, in-workspace and external-path approval,
   invalid image rejection, attach/remove/cancel/send, explicit model `read_image`, and assurance
   that a normal pasted image path is never auto-read.
5. I155: long `bash`/diagnostic output at narrow and wide widths in Alacritty plus a second terminal;
   inspect ASCII, CJK, emoji, and head-tail output for usable width and correct rows.
