# Iteration I291: Auto Review Locale Product Acceptance Follow-up

> Document status: Review
> Published plan date: 2026-10-03
> Parent: AUTO-UX-001 / #590 / I290
> MVP deliverable: session-level locale inference and acceptance evidence that satisfies the original product story without changing permission authority.

## Why This Follow-up Exists

I290 phase 1 is merged in PR #640 (`9bb565e0`) and its exact-head CI passed. Review found that
product acceptance still requires multi-turn session aggregation, broader extensible language
coverage, and a complete fallback/compatibility matrix. This iteration records the remaining
work; it does not invalidate the phase 1 implementation or rewrite its history.

## Scope And Non-Goals

- Observe bounded recent user-authored turns incrementally, including seeded history once per session;
  ignore model, tool, system, code-only and ambiguous short content.
- Detect language-only validated BCP-47 tags with confidence; never infer a region from script.
- Keep locale presentation-only: no changes to Allow/Ask/Deny, redaction, request identity, digest,
  eligibility, grant source, permission mode, deadlines or execution authority.
- No transcript resend, dedicated detection model call, unbounded storage, GPUI Git dependency change,
  search behavior, release/version change or unrelated translation work.

## Design Ownership

- Agent/session layer owns a bounded, deduplicated evidence tracker with session isolation.
- Detector owns documented language coverage and deterministic mixed/short/unsupported fallback.
- Config owns validated UI locale and the final fallback locale.
- Auto assessor receives only a bounded locale hint marked as presentation-only.
- Existing public APIs remain source-compatible through additive defaults or migration.

## Acceptance Matrix

- Chinese, English and at least one third language reach the real assessor prompt and human fallback.
- Multi-turn switching, history seeding, resume, duplicate approval events and session isolation pass.
- Mixed language, short text, code-only text, malformed locale, unsupported language and assessor
  failure all fall back deterministically to the configured UI locale.
- Locale changes do not change request digests or permission decisions.
- Captured prompt contains the locale hint without raw transcript content; byte/token growth is bounded.
- Warmed detection latency and full `--locked` CI evidence are recorded.
- Independent API/security review and governance validators pass before completion.

## Completion Gate

Mark AUTO-UX-001, I290 and I291 Complete only after an implementation merge commit exists,
exact-head CI passes, the acceptance matrix is evidenced, and the owner documents the final
Completion Commit. Until then the parent remains Review / Claimed.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | AUTO-UX-001 / #590: product acceptance follow-up for session locale inference |
| Claimed At | 2026-10-03 |
| Source Issue | #590 |
| Governance Claim PR | #650 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User explicitly requested execution of the product follow-up and authorized GitHub publication. Independent API/security review and exact-head CI remain required. |
| Implementation PR | #652, #662, #674, #677, #679, #680 (merged; product acceptance remains open) |
| Last Updated | 2026-10-09 |
| Handoff / Release Condition | Effective claim merge precedes implementation; no release or permission-policy change. |

## Execution Evidence And Remaining Acceptance (2026-10-04)

PR #652 merged at `9582733cca444c3ccfe70faf9daff869757c7fcf`. Exact-head CI run
[2988](https://github.com/wjhuang88/talos/actions/runs/37180797013) passed all jobs at
`cce6ca4af4a6ede0e4cb51fc1b9b0562c38c34d5`. Independent Agent API/security review approved
that head after byte-bounded observation, public context-field compatibility and localized
missing-decision-point fixes. The review covered those risk questions, not the whole product matrix.

The merged code observes up to eight recent user messages, truncates each to 4096 characters,
and caps their combined UTF-8 bytes at 16 KiB before copying. It binds observations to the
permission session lease and carries locale through additive assessor hooks. This is merged
implementation evidence, not a product Completion Commit.

| Acceptance area | Current evidence / remaining work |
|---|---|
| Permission and API boundary | Exact-head CI and independent review passed; locale excluded from digest |
| Local detection | Chinese/Japanese and short/code fallback unit fixtures exist; script heuristics still cannot establish language confidence |
| Session history | Recent bounded history is observed; incremental/deduplicated tracking, resume and session rotation tests remain required |
| Fallback/config | Environment LC_ALL/LANG fallback exists; validated UI config integration and malformed/mixed/unsupported matrix remain required |
| Real assessor and surface | Locale-aware hooks and zh/ja fixed copy exist; multilingual captured-provider prompt and failure-path tests remain required |
| Cost and user documentation | Byte bounds exist; warmed latency, prompt overhead measurement and README/user-guide behavior documentation remain required |

Resume in this owner and existing Work Slice: implement and locally test detector/config/fallback
and session tracking together, then capture multilingual assessor/surface evidence and measure cost.
Submit one converged follow-up candidate with fresh exact-head CI and independent review.
Keep AUTO-UX-001, I290, I291 Review / Claimed and #590 open until all published acceptance rows pass.

## Stage A+B+C Candidate Evidence (2026-10-04)

The first acceptance candidate changes locale aggregation from last-message-wins to a bounded
majority selection over the observed session window. Detection now records an internal confidence
level: high/medium script-majority, medium English heuristic, and low for short, code-like,
ambiguous or unsupported input. Low-confidence observations retain the configured fallback.
Supported-script fixtures cover `zh`, `en`, `ja`, `ko`, `ru`, `ar` and `hi`; mixed/unsupported
inputs fall back deterministically. Session-scoped observation tests prove that a foreign session
cannot mutate the resolver and that a resumed bounded history selects the majority locale.

Local evidence for this candidate:

- `cargo fmt --all -- --check`
- `CARGO_BUILD_JOBS=2 cargo test -p talos-agent --lib` — 433 passed
- targeted locale and session-observation tests — passed

This is an in-progress acceptance candidate. UI locale wiring, real assessor prompt capture,
provider failure-path evidence, performance measurement and user documentation remain open.


## Merge Review Correction (2026-10-06)

PR #658 original head `0dcd8b8b` passed CI run 3004 on retry, but merge review found
that the approval entrypoint could overwrite the history-majority result with a single intent,
and valid detections equal to the configured locale were excluded from voting. The candidate
now preserves the observed-history result through repeated approvals, includes all non-low
confidence votes, and retains single-intent compatibility only before history observation.
Regression fixtures cover repeated snapshots, empty-snapshot no-op behavior and foreign-session isolation.
These snapshot fixtures do not establish incremental detection or durable resume acceptance.

The earlier Stage A+B+C label describes intended scope, not completed acceptance. UI locale
configuration integration, incremental history detection, durable resume/rotation integration,
real assessor prompt/failure evidence, measurements and user documentation remain open.
Fresh exact-head CI and independent review are required for the corrected candidate.

## Corrected Candidate Merge Evidence (2026-10-06)

PR #662 merged at `1a397cfa1cbbee4cdc435cb450980b0fa6a25011` from exact head
`2ab3e3b741733548c67b5eb664c1a4b4a2468426`. Exact-head CI run 3011 passed all jobs,
and independent API/security review approved the head. The merged slice preserves session
history through repeated approvals, includes valid fallback-locale votes, keeps empty snapshots
as no-ops, and leaves locale outside permission authority and request identity.

This merge closes the corrected implementation slice, not the product story. AUTO-UX-001,
I290, I291 and #590 remain Review / Claimed while UI locale wiring, real provider prompt and
failure-path evidence, performance/prompt-overhead measurements, and user documentation remain.


## Provider Prompt Boundary Merge Evidence (2026-10-08)

PR #677 merged at `807b6bc1a0b7354e499f720a2851713df4a706c9` from corrected exact head
`6e08b6960c8c5b4ba9aece93db0816d25b8bdccf`. Exact-head CI run 3049 passed all jobs, and
independent API/security review approved the exact head. The added capture test verifies that the
bounded provider request contains the presentation locale and current bounded user intent while
excluding conversation-history text; existing provider failure handling remains human fallback.
README and config reference now document `auto.locale`, validation and fallback order.

The remaining acceptance rows are warmed detection/prompt overhead measurement and durable
incremental resume/rotation evidence. Until those rows have evidence, I291 and #590 remain Review /
Claimed and open.

## Acceptance Evidence Audit And Local Candidate (2026-10-08)

The previous statement that only measurement and resume/rotation remained was premature.
PR #677 directly supplies `zh-CN` to the assessor; it does not exercise detected multilingual
history through the resolver or an actual CLI/TUI run. Its negative history assertions do not
seed history in the fixture. Existing failure handling remains fail-closed, but technical failure
copy is still English. Do not infer complete localization acceptance from that merge.

This unsubmitted candidate adds resolver-to-provider-to-human-surface fixtures for `zh`, `en`,
and `ja`, asserting one provider call, absence of the seeded history from the captured payload,
localized decision points, and exact top-level locale JSON overhead (14 bytes for two-letter
tags). It also adds malformed-output fallback and a real TLOG close/reopen fixture that rebuilds
locale from recovered user messages. These are library integration fixtures, not binary walkthroughs;
local `cargo test --offline --locked -p talos-agent --lib` passed 440 tests with zero failures,
including these three fixtures. `cargo fmt --all -- --check`, `git diff --check`, and the
collaboration claim validator passed. Full workspace and exact-head CI are not yet recorded for
this unsubmitted candidate; project governance validation is still waiting on workspace metadata
dependencies. Independent static review found no API/security blocker but is not compiled or
exact-head merge approval.

`scripts/measure_auto_locale.py` extracts the checked-out production detector and compiles it with
the pinned Rust 1.97.0 compiler at `-O`. It adds no dependency and changes no workspace or lockfile.
On the local Linux x86_64 container: 100 warmup windows and 1000 measured eight-message windows;
32 characters/message p50 5909 ns and p95 6000 ns; 4096 characters/message p50 432123 ns and
p95 883135 ns. The larger synthetic window intentionally stresses eight full character caps and
can exceed the Agent's 16 KiB aggregate ingress bound. This measures detector CPU time only, not
snapshot copying, resolver aggregation, CLI latency, provider dispatch, or model tokenizer cost.
No claim of measured model token usage is made.

Still open at the initial local checkpoint: incremental/deduplicated detection, permission-session rotation reset, localized
technical-failure copy, malformed locale validation coverage, tokenizer measurement, and actual
binary acceptance. Keep #590, AUTO-UX-001, I290, and I291 Review / Claimed.

## Session Rotation Local Follow-up (2026-10-08)

The candidate now binds the derived locale to the Permission Session identity. Observation of
an empty snapshot after a rebind resets to the configured fallback; assessment also resets when
no new snapshot has been observed. Foreign-session observations remain rejected. Observation
binds the validated snapshot ID, not a second reading of the current ID, so a concurrent rebind
cannot relabel old history as new history. Permission identity is read outside the locale mutex.
The sequential rebind fixture verifies `en-US` fallback and later `ja` selection, including a
normal provider `human_required` report, not a panic fallback. This is library evidence, not
CLI/TUI rotation acceptance. Delayed old snapshots may still discard newer presentation history
and trigger fallback; concurrent-scheduling coverage remains open.

Independent local API/security review identified and verified corrections for the snapshot-ID
race and a mock's missing region-tag handling. No remaining local review blocker was reported;
this is not exact-head merge approval. Four pre-existing test `unwrap()` calls were changed to
descriptive `expect()` calls to satisfy the pinned Clippy policy. Current targeted Clippy with
`--offline --locked -p talos-agent --lib --tests -- -D warnings` and workspace format check pass.

The initial 441-test run passed, but after the review corrections the latest full-library runs
are not green: default parallelism yielded 437 passed / 4 session timeouts; two test threads
yielded 439 / 2; one thread yielded 435 / 6. An isolated paused-submission test passed. Do not
attribute these failures to load or to the base branch without a baseline comparison. The
project-governance validator remains blocked by offline workspace metadata download of
`arborium-c-sharp v2.18.2`; delivery-workflow validation passed 12 cases. No dependencies or
lockfile were changed. This checkpoint was held locally pending convergence; retain Review / Claimed.

Final isolated candidate rebuild: `cargo test --offline --locked -p talos-agent --lib` passed
441 / 441 with zero failures at default parallelism. The base branch's isolated session suite
also passed 35 / 35. Earlier shared-target baseline/candidate execution invalidates the targeted
57-test result (it ran the base binary); that result is not candidate evidence. The final run
used a fresh candidate-only crate build with no concurrent baseline compilation. The earlier
timeouts remain recorded, with no proven root cause. Full workspace validation, exact-head CI,
remote review, and product binary acceptance are still required before closure.

The stable slice is submitted for exact-head CI and independent remote review after local
convergence. This submission does not close the product story or waive full workspace validation.

## Incremental Locale Candidate And Recovery (2026-10-09)

PR #679 merged at `270911c071961a79c85af742a4daf62d4ad6735a` from exact head
`6024cc51b0bd8a35118601149960ac844d44fdc5`. CI run 3053
(`37772348144`) passed all jobs, including workspace tests and release preflight; independent
exact-head API/security review approved with the shared GitHub identity limitation disclosed.
This is merge evidence for the rotation/provider slice, not product closure evidence.

The next candidate reuses detector results for the latest eight bounded user messages by derived
SHA-256 digest, language and confidence. It retains no raw transcript in the cache. Each snapshot
votes only for its own window, including repeated occurrences; repeated observations never accrue
votes. Session rotation clears the cache. Changing configured fallback clears cached low-confidence
evidence and updates fallback-derived selection without overwriting a detected history language.
Hashing, bounded sampling and vote aggregation still run per snapshot; no end-to-end latency or
tokenizer improvement is claimed.

Locale normalization accepts at most 64 ASCII bytes using language[-Script][-REGION], with
validated POSIX encoding/modifier suffixes discarded. Variants, extensions and malformed tails
are rejected rather than truncated. Invalid LC_ALL falls through to valid LANG. README and config
reference describe this supported subset; it is not a complete BCP-47 implementation.

The previous unpublished local candidate passed 444 talos-agent library tests, workspace check,
Clippy and governance validators. Its full preflight first exhausted disk while linking default
debug tests; a compact debug retry reached talos-runtime with 64 passed and one host failure:
`runtime_evidence_rejects_directory_symlinks_and_socket_entries`, UnixListener::bind at
crates/talos-runtime/src/lib.rs:2035, OS error 1 PermissionDenied (Operation not permitted).
No test was skipped or permission restriction bypassed; later preflight fixtures were not reached.
The user confirmed publication for GitHub CI full acceptance on 2026-10-09.

Workspace maintenance removed that unpublished checkout and its local commits. This candidate
was reconstructed from the merged #679 base and retained implementation notes, not recovered
byte-for-byte. Prior test results do not validate the reconstructed tree. Its pure production
locale self-tests pass 6/6. Fresh `cargo test --offline --locked -p talos-agent --lib` passed
444/444, including observer cache/configuration fixtures; both governance validators passed
with zero warnings. Independent static API/security review found no blocker but is not exact-head
merge approval. Standard release preflight used compact dev/test debug information and disabled
incremental compilation as environment settings; repository build profiles are unchanged. Site,
installer, governance, claims, text-boundary, classifier and format checks passed, but workspace
check stopped compiling arborium-swift with No space left on device. The shared 32 GiB filesystem
was full; this task's target occupied about 1.4 GiB while another worktree target occupied about
26 GiB. Only this task's target was cleaned using cargo clean. Workspace check, Clippy, full tests
and SDK fixture are not green for this reconstructed tree. Per user confirmation, exact-head
GitHub CI must complete full acceptance before merge; no tests or dependency versions were
changed to bypass the host blockers.
Dependencies and Cargo.lock remain unchanged. Product residuals remain: localized technical-error
copy, actual CLI/TUI multilingual resume/rotation acceptance, concurrent delayed-snapshot evidence,
and model tokenizer measurements. I291, AUTO-UX-001 and #590 remain Review / Claimed and open.

## Incremental Merge And Technical Failure Copy (2026-10-09)

PR #680 merged at `9a7b4b8cf014058c04d6b83fa5d9a35e7236ec03` from exact head
`a9b91dc7cb2719c85c9032d0e9d66a79e3c058e0`. CI3055 run `37868494810` passed Linux/Windows
workspace tests, Clippy, standard release preflight, Desktop and governance. The Issue/owner
reconciliation job also passed after reopening #590 to match the owner's Review/Claimed state.
Independent exact-head Agent technical/API/security review approved; shared GitHub identity does
not constitute a formal approval from a distinct collaborator. Remote full tests cover the local
host blockers; local full preflight is not retroactively claimed green.

The next local slice translates fixed technical-failure explanations for configured zh/ja tags,
including region/script tags. Other configured locales use fixed English copy. This is a bounded
translation fallback, not coverage of every detected language. Valid model-produced explanations
continue using detected history locale; model failures and unverified results use configured
fallback locale. Reason codes, deadlines, report digests and permission outcomes are unchanged.
No arbitrary provider error or transcript content is interpolated into these prompts.

The slice covers missing trusted execution directory, rejected complete/sensitive inputs, missing
context, ineligible tools, incomplete assessment, mode/context changes, malformed output and
unverified request binding. New integration fixtures cover malformed/wrong-digest output with
history different from configured zh/ja/en and unsupported-language fallback, and three-language
timeouts with the original budget and human-required outcome. Exact-head remote validation and
review are pending; this slice is not completion evidence. Actual CLI/TUI acceptance,
concurrent delayed-snapshot evidence and tokenizer measurement remain open.
Independent local technical/API/security review found no blocker in this slice. Bare resolver
fallback paths add no new explanation, and the direct execution-directory-change error remains
English; neither is claimed localized by this fixed human-prompt slice. Binary acceptance must
check the actual visible surfaces rather than infer translation completeness from unit fixtures.

Local validation: pinned Rust 1.97.0, compact debug environment and disabled incremental builds;
`cargo test --offline --locked -p talos-agent --lib` passed 446/446, and the workspace test phase
also passed all 446 Agent tests. `cargo clippy --offline --locked -p talos-agent --lib --tests
-- -D warnings` passed. Standard preflight passed governance/claims, site/installers, format,
text-boundary/classifier, workspace check and workspace Clippy. Workspace tests stopped at the
existing runtime socket fixture: 64 passed / 1 PermissionDenied, OS error 1 at
crates/talos-runtime/src/lib.rs:2035. SDK fixture and later workspace packages were not reached.
No test was skipped; per existing user authorization, full exact-head CI must cover the host
restriction before merge. Cargo.lock, toolchain and dependencies remain unchanged.

The first 446-test run yielded 445 passed / one new timeout-fixture failure (elapsed 0 ns).
Without user intent, script evidence marks uncertainty and SlowAssessor's inherited adapter
returns unsupported before its delayed contextual implementation. The fixture now supplies the
current intent and asserts review_timeout before elapsed time; all three locale cases pass.
This was a fixture-path correction, not a production permission or deadline change.
