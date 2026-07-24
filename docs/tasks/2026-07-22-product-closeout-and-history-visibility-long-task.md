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
| I145 / TUI-026 | Complete — maintainer terminal acceptance (2026-07-22) | Completion Commit: `1039430`. FIFO preview, `+N more`, queue drain/clear, terminal-growth repaint, and Ctrl+C preview cleanup were retested and accepted. |
| I146 / TUI-033 | Complete — maintainer slash-prefix retest (2026-07-22) | Completion Commit: `7f6972a`. Tab completes a bare command without execution; Enter opens its direct-command menu; `/mo` shows only `/model`. |
| I147 / MODEL-008-A | Complete — maintainer terminal acceptance (2026-07-22) | Completion Commit: `1c843b2`. Wizard rendering, cursor targeting, and visible protocol choices were retested and accepted. |
| I148 / MODEL-008-B | Complete — maintainer terminal acceptance (2026-07-22) | Completion Commit: `f89313c`. Discovery → selection → immediate activation and post-switch text submission passed. |
| I150 / MODEL-009-B | Complete — maintainer combined terminal acceptance (2026-07-22) | Completion Commit: `b3cc943`. |
| I151 / MODEL-009-C | Complete — maintainer terminal acceptance (2026-07-22) | Completion Commit: `17e3fef`. I154 must reuse its proven controls rather than recreate a parallel path. |
| I152 / MODEL-009-D | Review, code-level security acceptance | The local terminal packet passed except for the maintainer-owned live Anthropic-compatible Provider check, which is unavailable in the current environment. |
| I153 | Review | Refresh automated evidence and checklist; it remains Review until maintainer walkthrough evidence exists. |
| I154 / MODEL-009-E | Review — maintainer GO (2026-07-24) | P2 inventoried code-level prerequisites and accepted ADR-051. P3 implementation complete (commits `6d4677e`–`faa5464`); 40 new tests pass; ADR-051 Implementation Facts recorded. Maintainer independently verified all gates and contracts. I152/I153's live Anthropic review gate remains independent. |
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
| P2 | I153 prerequisite/evidence refresh and I154 activation decision | Append-only evidence update stating whether I154's code prerequisites are met; an I154 activation record only if they are | P1, I151/I152 accepted code state | Security-boundary inventory, I153 regression replay, no unresolved critical path; no false real-terminal Complete claim | Keep I154 Blocked and provide exact missing condition | Complete — Completion Commit: `ba90c02` |
| P3 | I154 `read_image` tool | Supported-only registered tool; exact-path approval; shared image ingestion/digest revalidation; provider-neutral continuation artifact; two adapter fixtures; safe history/provenance; unchanged text `read` | P2 | Tool registry/presentation, permission Allow/Ask/Deny, symlink/replacement/decoder adversarial, agent-session continuation, OpenAI/Anthropic, text-only/history regression tests; security review; full locked ladder | Do not expose tool; amend ADR-050 with evidence and retain explicit `/attach` only | Review — maintainer GO (2026-07-24) |
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
  I150-I153 still require the maintainer walkthrough listed below before any
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

### Checkpoint P2 — 2026-07-22

- Completed task items: P2 — I153 code-prerequisite evidence refresh and the I154 activation
  decision. Completion Commit: `ba90c02` (`docs(agent): ready I154 image-read implementation
  (#I154) [model:gpt-5]`).
- Commit pushed: `ba90c02` on `origin/main`.
- Changed owner artifacts: ADR-051 and its index; I154 and MODEL-009-E implementation contract;
  MODEL-009 parent/backlog/iteration index; derived Board; this long-task owner.
- Commands and exit results:
  - `git diff --check` → exit 0.
  - `scripts/validate_project_governance.sh .` → exit 0, 0 warnings.
  - I153's full locked validation was already replayed by the v0.5.0 release preflight at
    `3eec574`; P2 changes documentation only and did not alter Cargo/code.
- Acceptance evidence / remaining human gate: code inventory confirmed the existing normal tool
  authorization can bind the new `read_image` identity and exact canonical path; provider adapters
  retain final digest revalidation. ADR-051 defines the missing one-shot continuation without
  persistence or a side channel. I152/I153 remain Review solely for the unavailable
  maintainer-owned live Anthropic-compatible provider check; fixture evidence is not misreported
  as that check.
- Open risks or deviations: P3 will make additive public API changes and must include the ADR-051
  semver migration documentation. It must implement and fixture-test Anthropic tool-result/image
  coalescing; no code has begun in P2.
- Next task item: P3 — I154 `read_image` implementation. It is ready for a maintainer-dispatched
  frontline developer; do not begin it automatically.
- Resume: `git switch main && git pull --ff-only origin main`; read this checkpoint,
  `docs/decisions/051-one-shot-multimodal-tool-continuation.md`, and
  `docs/iterations/I154-agent-mediated-image-read-tool.md` before editing code.

### Checkpoint P3 — 2026-07-23

- Completed task items: P3 — I154 Agent-Mediated Image Read Tool (Steps A-F + tests + docs + NO-GO rework B1-B7).
- Commits pushed (chronological):
  - `6d4677e` — Step A: `ToolExecutionOutput` + `execute_authorized_with_output` in `talos-core/tool.rs`.
  - `ad46eba` — Step C: Image validation migrated to shared `talos-tools/src/image_validation.rs`.
  - `9009096` — Step B: `ReadImageTool` implemented in `talos-tools/src/read_image_tool.rs`.
  - `5eeb8e1` — Step D: `execute_with_output` trait method + permission wrapper overrides +
    turn-loop continuation overlay in `talos-agent`.
  - `2270f21` — Step F: `ReadImageTool` registered behind permission wrappers; `image_input_supported`
    capability gate on `Agent`; `set_image_input_capability` helper wired into all agent construction sites.
  - `36d987c` — Tests: 7 `ReadImageTool` unit tests.
  - `29c95fc` — Docs: I154 iteration doc updated.
  - `dfab8bb` — P3 checkpoint appended.
  - `4a0616a` — NO-GO rework B1/B2/B4/B5/B6: permission_profile with path facet, Anthropic
    consecutive user-message coalescing, batch limit (max 1 image per batch), execution-boundary
    capability gate, path sanitization in error messages.
  - `9ecca94` — NO-GO rework B3: 3 agent continuation integration tests (one-shot, consumed,
    not-persisted).
  - `bc38112` — R1/R2: atomic quota rejects 2nd read_image before execution; 3-call one-shot
    test + batch-limit test.
  - `13bc157` — R3: OpenAI continuation fixture + 4 permission chain tests (auto-allow, deny,
    path mismatch, headless Ask).
  - `1749333` — R4: ADR-051 Implementation Facts + site capabilities docs (EN + zh-CN).
  - `06b25f4` — T1/T2: TUI Ask→approve test + attach_image/read authorization isolation tests +
    ingestion regression tests (text file, FIFO).
  - `bacf292` — T3: provider failure continuation consumed test + safe summary persistence test.
- Step E: Provider adapter wire mapping — existing `Message::Multimodal` handling in both
  adapters covers the continuation overlay. Anthropic coalescing added in B2 rework.
- Changed owner artifacts: I154 iteration doc (execution record appended, status → Active);
  iterations README (I154 row → Active); README EN/zh-CN (read_image tool documented);
  BOARD.md (I154 → Active); MODEL-009-E story (→ Active); ADR-051 (Implementation Facts);
  site capabilities.html EN+zh-CN (read_image); this long-task owner (P3 checkpoint updated).
- Commands and exit results:
  - `cargo fmt --all` → clean.
  - `cargo clippy --workspace --locked -- -D warnings` → exit 0, 0 warnings.
  - `cargo test --workspace --locked` → exit 0, 0 failures across all suites.
  - `scripts/validate_project_governance.sh .` → exit 0, 0 warnings.
  - `git diff --check` → exit 0.
- Acceptance evidence / remaining human gate: All initial P3 steps (A-F) + all 7 NO-GO blockers
  (B1-B7) + all 4 rework items (R1-R4) + all 5 second-rework items (T1-T5 docs) + all 5
  third-rework items (F1-F5) + all 4 fourth-rework items (G1-G4) + all 4 fifth-rework items
  (H1-H4) addressed.
  Test totals: 16 ReadImageTool unit tests + 9 agent integration tests + 7 permission chain
  tests + 4 TUI scrollback tests + 1 export test + 1 provider TOCTOU guard test +
  1 Anthropic coalescing fixture + 1 OpenAI continuation fixture = 40 new tests.
  Maintainer independently verified all gates (fmt/clippy/test/governance/diff) and ADR-051
  contracts; GO received 2026-07-24.
  The I152/I153 live Anthropic-compatible provider check remains a separate human gate.
- Open risks or deviations: copy/resume safe projection covered indirectly via
  `history_message_parts` (export has direct file-level assertion). I152/I153 live
  Anthropic provider check is a separate human gate.
- Next task item: P4 — TUI-034/I155 long-output display. **Must not start without maintainer instruction.**
- Resume: `git switch main && git pull --ff-only origin main`; read this checkpoint, then the P4 task
  description in this file and `docs/iterations/I154-agent-mediated-image-read-tool.md`.

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
| Steering queue | **Complete.** During a tool-running turn, FIFO preview and `+N more` were accurate; the preview disappeared after drain and no stale text remained after viewport growth or Ctrl+C. Completion Commit: `1039430`. | I145 |
| Parameterless menus | **Complete.** Bare `/model` and `/connect` open their menus. Argument-bearing forms show a correction and do not mutate config/session. Tab completes without execution or trailing space; Enter opens the menu; Escape cancels. `/mo` shows only `/model`. Completion Commit: `7f6972a`. | I146 |
| Wizard | `/connect` → Add custom provider runs Name → Protocol → Base URL → API key → Confirm; key remains masked, Esc causes no save, and duplicate name shows the explicit update path. | I147 |
| Discovery and activation | **Complete.** A custom-provider discovery → selection → immediate activation flow and post-switch text submission passed. Failure retains manual configuration fallback. Completion Commit: `f89313c`. | I148 |
| Capability and attachment | **I150/I151 Complete.** Unsupported and Unknown models reject `/attach` before file access. A Supported model accepts a valid local PNG/JPEG/GIF/WebP; an invalid image is rejected. External path asks for exact-path approval; list/detach/send work; history shows only a safe summary; a text-only turn still works. I152 remains Review only for its live Anthropic-compatible Provider check. | I150, I151, I152 |
| Provider mapping | With a maintainer-owned disposable configured endpoint, one accepted image produces the expected compatible request or a safe, actionable provider failure; no path/data URL/API key appears in TUI history. | I152, I153 |

Do not run I154 `read_image` steps here until P3 is dispatched: I154 is Active with an accepted
implementation contract, but it remains a distinct future implementation phase.

## Maintainer Real-Terminal Packet (Not Delegable To Automation)

The final report must ask the maintainer to execute and record:

1. I145 is Complete after maintainer acceptance (Completion Commit: `1039430`).
2. I146 is Complete after the `/mo` slash-prefix retest (Completion Commit: `7f6972a`); Tab is
   intentionally non-executing while Enter opens the menu. I147 is Complete after maintainer
   acceptance (Completion Commit: `1c843b2`).
3. I148 is Complete after maintainer acceptance (Completion Commit: `f89313c`).
4. I152: a maintainer-owned live Anthropic-compatible Provider check remains. I154 is still
   Planned/Blocked and must not be included in this terminal packet; normal pasted paths remain
   non-readable unless a future explicit tool is authorized.
5. I155: long `bash`/diagnostic output at narrow and wide widths in Alacritty plus a second terminal;
   inspect ASCII, CJK, emoji, and head-tail output for usable width and correct rows.
