# Session-local rolling history for model-assisted tool approval

## Status

Requirement intake only; not implemented, activated or claimed. Requested by the
user in this session, with an explicit request to preserve the requirement in a
repository document rather than only session todos.

## User requirement

The model that judges automatic tool-call approval should receive a small rolling
history from the current Session. Retain the most recent few approval judgments,
including what was judged, the judgment content and result, and the actual human
outcome when the request was escalated. Attach this bounded history as reference
material to subsequent model approval requests. Evict older entries rather than
accumulating a long-term memory.

## Required information

- Stable correlation between the approval request and any later human outcome.
- Tool/action and necessary argument context, bounded and sanitized.
- Model judgment explanation available from the approval response and its decision;
  do not require or store hidden chain-of-thought.
- Whether human approval was requested and its eventual result. Distinguish pending,
  allowed, denied, cancelled and unavailable outcomes instead of inventing approval.
- Ordering sufficient to identify the latest entries within the originating Session.

## Proposed safety and resource constraints

These are design requirements to review against the existing permission contract,
not claims about current implementation:

- History is reference data, not a grant, user instruction or override of current
  permission rules. A previous model allow must never become user authorization.
- Keep model decisions and human decisions separately attributed. Human denial must
  not be overwritten by an earlier model assessment.
- Isolate Sessions. Do not introduce cross-session learning or long-term retention.
- Bound both entry count and per-entry/aggregate text size. Sanitize secrets and
  unnecessary command/output data before storing or sending to the approval model.
- Treat tool arguments, explanations and recorded history as potentially untrusted
  input. Clearly delimit history in the approval request.
- Preserve current approval/error fallback behavior when history is absent or cannot
  be read; history failure must not implicitly allow a tool call.
- Concurrent requests and delayed human responses must update the matching entry,
  never an unrelated most-recent request. An expired entry must not cause unbounded
  side storage or restoration of arbitrarily old history.

## Open decisions

The user requested only a few recent entries; the exact count is not yet selected.
Choose count, byte limits, truncation and eviction rules after inspecting the code.
Decide whether the lifecycle is live-session memory only and how resume/fork/reset
behave. This request does not authorize disk persistence or cross-session copying.
Specify which available model explanation fields are retained and how sanitization
preserves enough context for useful judgments. Define pending/expired escalation
handling and whether the current request is excluded from its own reference history.

## Acceptance targets

1. A subsequent approval-model request includes the bounded recent judgment history
   with clearly attributed model and human outcomes.
2. Escalation updates the correct request after a human response; pending, denial
   and cancellation are not represented as approval.
3. Exceeding entry or text limits evicts/truncates deterministically and remains
   bounded even with oversized tool inputs or delayed responses.
4. Another Session cannot observe this history. Reset/resume/fork behavior matches
   the selected lifecycle contract and has regression tests.
5. Existing permission rules and required human gates remain authoritative despite
   malicious, conflicting or previously permissive history.
6. Empty/unavailable history preserves safe existing fallback. Secrets are not added
   to approval prompts, diagnostics or stored history by the new mechanism.

## Implementation routing and dependencies

Before implementation, inspect ADR-064 (bounded model-assisted auto permission),
the effective permission orchestration decisions and current automatic/human
approval code. Establish an authoritative Story/claim and any required successor
ADR under REQUIREMENT-INTAKE and AGENT-COLLABORATION. Permission-boundary changes
require independent security review, focused tests and exact-head validation.
Document user-visible lifecycle and bounds in the applicable auto-approval docs.

This is separate from BG-CONTINUE-001 and the pending TUI requests. Do not switch
away from background continuation merely to implement this intake. Session todo:
f260436f-82e8-41b7-9349-5bda7fd77ad5. This document is the durable intake reference;
no implementation, validation or completion evidence is claimed.

## 2026-11-11 recovered requirement and implementation preparation

The exact user request was recovered from preserved original record
`4aa554e9-45ad-4d82-a595-66107680822a`: retain a few previous judgments and results,
including results after asking a human, and send that small history to the model as
judgment material. This resolves the later handoff-summary ambiguity: a UI-only log
does not meet the requirement. No new user confirmation is necessary for that purpose.
The low-auto-rate investigation is separately requested in original record
`7b523388-9993-4603-9b90-61993e7af1b9`.

Proposed concrete lifecycle for the implementation review:

- Eight entries, at most 1024 serialized UTF-8 bytes per entry and 8192 bytes for the
  complete serialized history envelope. Evict oldest entries before insertion;
  enforce limits after escaping/serialization, never by Unicode character count alone.
- Live resolver/session memory only; no disk persistence. A different session, resume
  into a new resolver, fork or explicit auto reset starts empty. Rebinding must also
  invalidate pending update tokens, even if an older session ID is later reused.
- Each admitted review gets a monotonic local correlation token and generation.
  Capture the prior-history snapshot before inserting the current pending entry.
  Delayed human results update only that token in that generation; eviction never
  creates an auxiliary retained pending map or resurrects the entry.
- Retain a bounded sanitized tool/action description, available public effect summary
  and decision points, model decision/category, and independently attributed human
  Pending/Allowed/Denied/Cancelled/Unavailable. Do not retain hidden reasoning, raw
  provider errors, complete arguments, command output or reusable permission grants.
- Cancellation/drop must finalize the matching pending human outcome without treating
  cancellation as denial or permission. Completed model decisions remain attributable
  if later human work is cancelled. An unavailable lock/history yields no history and
  retains the existing permission/fallback behavior.
- Include the envelope as untrusted reference data in every supported auto-review
  provider request, explicitly saying it is neither instructions nor authorization.
  Existing exact-current-request digest/revision checks and explicit human checkpoints
  remain mandatory. History must not add tools, retries or new model authority.

Current integration inventory: resolver entrypoints include ordinary execution context,
file mutation, built-in invocation, network and prepared command; provider payload builders
are split across `auto_resolver.rs` and five child modules. Last-report notification alone
is insufficient: it lacks human outcome correlation and some bypass reports only notify.
The human result must be observed around the actual fallback future, and model explanations
must come from validated/sanitized public response fields. A single shell-only patch or
last-report ring would not satisfy this proposal.

Required regression matrix: each provider request family receives prior bounded history;
same-ID concurrent requests cannot cross-update; late results after eviction/rebind/reset
are ignored; cancellation differs from denial; secrets/control characters are excluded;
oversized/multibyte/JSON-escaped input remains bounded; explicit Ask and Deny retain their
existing behavior with hostile previous Allow entries; schema/digest/expiry checks remain
effective. Real fallback oneshots and captured actual provider messages must supplement
container unit tests. Independent security review and an effective owner claim are still
required before implementation delivery; this preparation does not claim activation.

## 2026-11-14 source-grounded integration map (pre-implementation)

Current primary source inspection identifies six actual provider payload construction
boundaries (ordinary/action share one builder). Each needs captured-request regression
coverage; trait defaults or last_report alone cannot prove this requirement:

| Review family | Production request boundary | Required evidence |
|---|---|---|
| Shell, context and script evidence | auto_resolver.rs::assess_payload | Prior-history envelope present with existing context/script and locale retained |
| Ordinary invocation and built-in action | action_review.rs::assess_shared_action_payload | Both entrypoints receive history; bounded_json limit remains enforced |
| File mutation | file_mutation_review.rs::assess_file_mutation_payload | Actual prepared before/after validation remains authoritative |
| Built-in invocation | invocation_review.rs::assess_invocation_payload | Existing creation evidence/digest checks cannot be overridden |
| Network | network_review.rs::assess_network_payload | Existing network binding and explicit checkpoints preserved |
| Prepared command | prepared_command.rs::assess_prepared_command_payload | Context freshness and current command identity remain mandatory |

Human-result capture must wrap actual fallback resolve/resolve_with_explanation
futures, including early evidence-unavailable branches, rather than infer a result
from report() or notify_report(). ApprovalChoice exposes ApproveOnce/AlwaysApprove/
Deny; any recorded Allowed is an attributed observed outcome, never a reusable grant.
Current ApprovalResolverError (permission_pipeline.rs) stores only message:String.
CLI TerminalApprovalResolver distinguishes unavailable/deadline/cancelled only in
message text. Do not parse those strings to fabricate reliable typed human outcomes.
Implementation must introduce a reviewed additive outcome classification (legacy
unknown errors map Unavailable) or equivalent private typed observation at the actual
adapter boundary; cancellation of the enclosing future also needs generation/token
RAII finalization. Public API compatibility and migration documentation must be
reviewed before choosing the mechanism. This design gap is not implemented here.

Reset coverage must exercise both AutoPermissionResolver::reset() and control epoch
sync_reset(), plus replacement/rebind. Concurrent old requests completing after reset
must never update new-generation entries. Locale/session binding is presentation
state, not approval-history custody. A missing or poisoned history must not alter
current permission fallback. Existing model response binding remains the authority
for which sanitized public explanation may be retained.

This mapping is preparation only; PR916 is not merged, no implementation branch or
permission/history code was created, and acceptance remains outstanding.

### Adapter attribution correction after independent review

Source inspection and independent review establish a further boundary: adding error
classification alone cannot distinguish actual human denial across all hosts.

| Adapter | Current behavior | Required regression before claiming human attribution |
|---|---|---|
| Terminal (cli approval.rs) | Failures use message-only resolver errors | Explicit user cancellation differs from unavailable transport; unknown channel closure is not proof of cancellation |
| TUI (cli registry.rs) | Send/oneshot failures become untyped errors | Actual user Deny differs from disconnected UI; capture actual fallback completion |
| Runtime SDK (runtime lib.rs) | RuntimeApprovalResolver wraps host choice in Ok | Legacy host Deny has unknown human attribution; preserve old host compatibility |
| Desktop (desktop runtime_host.rs) | Oversize input, ID exhaustion, poisoned lock, pending request and channel failures can return Deny | Each unavailable path stays fail-closed without becoming a recorded human refusal; real UI choice remains attributable |

Minimal compatible design candidate: retain ApprovalChoice and resolver signatures;
add private error classification while preserving new(message) and Display, with
legacy errors classified Unavailable. For SDK observation, evaluate an additive
default trait method delegating to the legacy scoped-with-explanation method and
marking its outcome as LegacyHost/unknown attribution. Desktop would override the
observation method; its existing choice-returning methods retain fail-closed behavior.
This is a proposed API change requiring compatibility review, not an implemented API.
If no typed SDK observation is introduced, Desktop/legacy human attribution must
remain explicitly unknown rather than claiming full coverage.

A correlation guard starts Pending only when actual human fallback begins, disarms
on completion, and finalizes dropped requests without claiming a human decision.
Explicit cancellation requires a producer signal; an unexplained oneshot closure
means Unavailable. Old-generation or evicted updates are ignored. Tests must include
real adapter channels and a legacy SDK handler, not just the history container.
This correction belongs to the existing approval-history requirement, adds no
permission authority, and does not activate implementation before effective claim.
