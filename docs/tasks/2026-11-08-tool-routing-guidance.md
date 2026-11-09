# TOOL-ROUTE-001: Direct Tool Selection And Foreground Defaults

> Document status: Complete / Closed
> Published plan date: 2026-11-08
> Deliverable: shared model-visible guidance routes covered operations to dedicated tools, avoids redundant executor/interpreter wrappers, and defaults to foreground execution.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Shared system-prompt tool-routing and foreground/background guidance plus exec, shell and process descriptions; prompt assembly tests and documentation; no runtime execution or permission-policy change |
| Claimed At | 2026-11-08 |
| Source Issue | None |
| Governance Claim PR | #874 |
| Authorization Mode | Independent review |
| Authorization Evidence | User requests general analysis and optimization beyond exec-to-bash wrappers; explicitly requires foreground unless concrete parallel work or service/foreground coordination exists |
| Implementation PR | #877 |
| Last Updated | 2026-11-08 |
| Handoff / Release Condition | None; routing scope delivered, unrelated CI diagnostic observation retained below |

## Goal, scope and acceptance

Given available dedicated tools covering the operation, guidance selects them for
files, search/code structure, Git inspection/mutation, network, validation and
session-owned process control. It forbids redundant shell/exec/interpreter wrappers
for covered operations and rejects tool-switching to evade refusal, stale snapshots,
approval, rate limits or output bounds. Missing tools or actual capability gaps
allow narrow fallback within existing authorization; do not invent tools or demand
redundant approval. Preserve explicit host-shell Git approval rule.

Given an uncovered literal argv program, exec is the direct route; shell syntax
uses available bash/powershell directly. Required project scripts, interpreter
versions and application code remain valid deliverables, not prohibited wrappers.
Given serial work with no useful concurrency, foreground is default even if slow.
Background requires concrete independent work or service/client coordination;
retain job handle, cursor, observe completion, avoid polling/duplicate starts and
clean up temporary services within existing authorization. Automatic tool yielding
is not a reason to restart a command.

The rules must reach normal and custom-identity prompt assembly, native and textual
tool-call protocols, and stable-prefix construction. Local descriptions must agree.
Tests prove assembly and availability wording, not guaranteed model behavior.
No live-provider routing accuracy claim without a separate measured evaluation.

## Constraints and inventory

Hard: no execution/permission/sandbox changes; guidance is not enforcement; only
current tool definitions authorize available names. No new dependency/public API.
Soft: conditional examples plus compact global rules, rather than a hardcoded
assumption that every registered tool is always disclosed.
Required reads: prompt builder/assets/sections, tool descriptions, collaboration
and testing SOPs. Documentation: README.zh-CN.md and this owner. Residuals: here.

I310 Active retained; I318/I302/I277 Review retained with their existing deferred
acceptance; I249 Planned deferred; I164 Paused retained; no Blocked header found.
TUI-POLL-001 Active through #873 owns only projection and has separate worktree;
this task touches prompt/exec/shell/process descriptions, not TUI. Open #838/#849/
#861 concern Gemini/search and do not own this guidance. Original uncommitted
ellipsis/theme/README edits stay untouched. Base 1a2c184d; recheck overlap at merge.

## Validation and closure

Review a route matrix including file read/edit, structured Git write, URL read,
HTTP POST, download, literal argv, shell expression, missing capability, permission
refusal, stale snapshot, long serial build, independent checks and background server
with foreground browser. Run locked Agent prompt and Tools/Process tests, standard
release_preflight.sh and cargo-xwin MSVC tests for affected platform descriptions.
Independent review, exact-head CI and CAS before merge; Complete requires existing
implementation SHA. Do not claim prompts mechanically prevent every model mistake.

## Implementation and analysis checkpoint

Claim #874 merged as ad71a208 after independent Agent APPROVE for 649ba6f8
against 1a2c184d and CI42392887768 passed. Existing identity rules covered only
files/read-only Git and treated multi-step pipelines as a blanket shell exception.
Exec described no shell parsing without prohibiting redundant shell wrappers.
Fetch URL already described its HTTP-request boundary. These facts informed the
new shared tool_selection asset, injected into the existing cacheable Tools section
so default/custom identity and native/textual protocols share one copy without
adding cache markers. Removed superseded identity routing/AST duplicates.
Updated exec/shell/process descriptions; no execution branch or permission changed.

Manual scenario review: read/edit use file tools; semantic references use supported
symbol tools; Git add/commit/push use corresponding structured tools; URL content,
download and POST use fetch_url/save_url/http_request; uncovered literal argv uses
exec; shell syntax uses platform shell directly. Missing capability permits narrow
authorized fallback, while refusal/stale snapshot requires supported recovery.
Serial slow tests stay foreground; independent checks may run concurrently after
resource/dependency review; a server plus browser is valid background/foreground
coordination with cleanup. Existing project entrypoints and requested application
code remain valid. This is a wording review, not measured model-routing accuracy.

Assembly tests cover catalog presence/absence, default/custom identity, native/text
protocols, exactly-once stable-prefix inclusion and cache-range coverage.

Local preflight caught two prompt-boundary regressions before submission: added
instructions pushed the unchanged 20k compaction fixture over its sealed request
budget, and `authorization` in process description triggered the existing sensitive
text filter in Auto action evidence. Compressed the shared rules and changed that
description to concise equivalent wording; did not relax budgets, test assertions or
safety filters. The real process cancellation allow/deny/stale/foreign/replacement
fixture then passed. Recorded the reusable lesson in EVOLUTION.md. Independent
review also caught a lost shell-unavailable fallback qualification during shortening;
restored the condition that a matching shell tool must cover the task.

The full workspace run also detected the CLI model-switch fixture entering 80%
pressure after expanded descriptions. Measured 26,016 estimated tokens at 32k
capacity with an 8,192 output reserve; shortened duplicate local descriptions and
shared prose without changing the fixture or pressure thresholds. Final Agent
655/655 and CLI 410/410 tests passed, including all three budget/approval regressions.
Final cargo-xwin MSVC --tests check for agent/tools (coding feature) passed; existing
checkpoint_history test-only dead-code warning remains unrelated.

Final standard release_preflight.sh passed (locked workspace check/Clippy/tests,
doc-tests, both external SDK fixture modes and governance checks). The retained
shell-description compatibility test passed after keeping `shell command` wording.
No test budget/assertion or runtime permission/execution behavior changed.
Logs: /tmp/talos-route-preflight4.log and /tmp/talos-route-final-xwin2.log.
Exact-head independent review and remote CI remain required before merge.

## Remote verification and residual

Implementation PR #877 reviewed by /root/quality_claim_review: APPROVE for head
abd07c11cc224b0158aa80b49b0f07ba7ade97f8 against base
ae66e309204e25cc67e4f289775989efcb79a553. No blockers. Review receipt is in the
PR body; independent Agent technical review, not separate human GitHub approval.
Both final governance validators passed with zero warnings; integrated TUI tests
passed 628/628 after preserving both README sections during integration.

CI42396273664 attempt 1 passed native Windows and Linux/Windows Desktop but failed
an unchanged talos-session macOS fixture:
`fork_recovery_skips_active_lock_and_retires_only_unpublished_intent`, durable.rs:2363,
retired count 0 versus expected 1. Session sources/dependencies were unchanged and
have no dependency on the modified Agent/Tools crates. Full Linux preflight and the
isolated fixture passed. A single failed-job rerun on the same head was approved
by independent Agent assessment; no assertions/timeouts/safety paths were changed.
First-attempt cause remains unknown: the test asserted only retired count, while
recovery can skip held locks or count I/O/SQLite failures. Do not label it fixed or
proven infrastructure instability.

Residual diagnostic observation (owner: this document): preserve attempt 1 logs in
CI42396273664 and /tmp/talos-route-ci-failure.log. If reproduced, capture the full
(scanned, retired, failed, cursor) result and underlying recovery error before
choosing a separate storage/test repair; do not mechanically retry or weaken the
lock/retirement assertion. This observation is not a missing routing deliverable.

## Completion evidence

Completion Commit: 25c7769bbc68fbc47aec5f5e9c385077bb35f90c

CI42396273664 attempt 2 passed the failed macOS job on the unchanged implementation
head, including standard preflight, real CLI/TUI acceptance, language plugins and
offline dependency audit. All applicable checks were green before #877 merged.
Merge CAS verified exact head, mergeability, unchanged owner/claim and no overlap;
subsequent #878 closure and #876 ellipsis claim did not change this scope.
README.zh-CN documents the delivered rules. No implementation remains in scope.
Prompt guidance does not enforce model routing and no live-provider accuracy
measurement is claimed. The unrelated first-attempt storage diagnostic remains
explicitly unresolved above; no storage bug-fix claim is made.
