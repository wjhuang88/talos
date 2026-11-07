# TOOL-ROUTE-001: Direct Tool Selection And Foreground Defaults

> Document status: Active / Claimed (proposed; ineffective until target merge)
> Published plan date: 2026-11-08
> Deliverable: shared model-visible guidance routes covered operations to dedicated tools, avoids redundant executor/interpreter wrappers, and defaults to foreground execution.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Shared system-prompt tool-routing and foreground/background guidance plus exec, shell and process descriptions; prompt assembly tests and documentation; no runtime execution or permission-policy change |
| Claimed At | 2026-11-08 |
| Source Issue | None |
| Governance Claim PR | #874 |
| Authorization Mode | Independent review |
| Authorization Evidence | User requests general analysis and optimization beyond exec-to-bash wrappers; explicitly requires foreground unless concrete parallel work or service/foreground coordination exists |
| Implementation PR | Not started |
| Last Updated | 2026-11-08 |
| Handoff / Release Condition | Shared prompt assembly, description parity, focused tests and standard validation; independent review and exact-head CI before closure |

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
