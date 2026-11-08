# AGENT-PAUSE-001: Visible Protection Stops And Post-Edit Validation

> Document status: Active / Claimed (proposed; effective only after claim merge)
> Published plan date: 2026-11-08
> Deliverable: repeated-call and tool-budget stops visibly explain why execution ended, persist the explanation, and permit legitimate validation after successful file changes.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / gpt-6 |
| Work Slice | Agent protection-stop publication/persistence and scoped post-file-change repeat tracking; Session/Conversation/TUI regression coverage and user docs; no permission/sandbox policy change |
| Claimed At | 2026-11-08 |
| Source Issue | None |
| Governance Claim PR | None |
| Authorization Mode | Independent review |
| Authorization Evidence | User requires fixing silent stop rather than asking users to continue around it; independent Agent review and exact-head CI before merge |
| Implementation PR | Not started |
| Last Updated | 2026-11-08 |
| Handoff / Release Condition | Reproductions, locked preflight, MSVC xwin tests, independent review, exact-head CI and merged implementation evidence |

## Acceptance and constraints

A repeated tool/arguments pattern without state progress still stops before the
protected invocation executes. Total tool-call budget remains bounded. Both stops
publish a clear reason exactly once to the active UI and append an assistant
message so export/replay retains it; do not duplicate ordinary streamed final text.
They must not look like silent idle or claim requested work is complete.
Successful built-in file changes permit subsequent identical command validation;
reads, failed writes, unknown external tools and command execution alone do not
clear unrelated repeat protection. Preserve permissions and process limits.

Hard: no secrets, permission bypass, sandbox change, public protocol break or
new dependency. Preserve structured event ordering, private-text projection and
raw tool records. Soft: use existing event/message channels rather than add a
public completion enum. Validate the smallest progress signal from real built-in
file mutation evidence; no blanket reset for every successful tool.

## Evidence and scope

Live session 4404450a-443c-479d-8153-f530d10faa21 / turn_1536965_1_17:
edit completed; provider request; exec starting; no exec running/result; terminal
success persisted. No child command remained. Identical fmt/test calls had each
executed twice. Repeated-call protection is the strongest explanation; exact
live stop reason was not retained. Confirmed code defect: protection branches
return Ok(explanation) without publishing/storing it; Conversation success handler
ignores final_text. Reproduce both guards deterministically before fixing.
Diagnosis saved at /tmp/talos-silent-pause-diagnosis.md; safe activity capture at
/tmp/talos-live-activity.txt. No claim that the historical raw provider reply exists.

Required reads: Agent loop/tool execution, Session turn persistence, Conversation
completion and CLI bridge; collaboration, testing and local-dev SOPs.
Docs: README.zh-CN.md, this owner, EVOLUTION.md. Residual destination: this owner.

## Inventory and isolation

Base ce44e158. I310 Active retained; I318/I302/I277 Review retain existing external
acceptance; I249 Planned deferred; I164 Paused superseded; I162 Complete retained.
No Blocked iteration found. TUI-POLL-001/TOOL-ROUTE-001 Complete retained.
TUI-ELLIPSIS-001 Active belongs to Talos and excludes this runtime scope. Open
#838 Gemini, #849 MCP qualification and #861 search helpers do not overlap.
Original README/theme/tool_display/proposal changes and ellipsis worktree remain
untouched. This task uses /tmp/talos-silent-pause and a separate implementation
branch after target claim merge. Existing governance profile unchanged.

## Validation and closure ledger

Artifacts: minimal Agent fix, production-path regression tests and docs.
Owner first, Board mirror; no Source Issue. Prove real edit-test-edit-test cycles,
failed/no-op edits and true repetition, budget stop, streamed output plus persisted
history and replay. Run pinned locked standard release_preflight.sh and cargo-xwin
MSVC --tests for affected crates; native CI remains required. Review exact head
and merge CAS. Complete only with existing implementation SHA and honest residuals.
