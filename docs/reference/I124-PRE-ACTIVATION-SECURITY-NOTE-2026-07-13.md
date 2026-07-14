# I124 Pre-Activation Security Note: Permission and Injection Path Safety

**Date**: 2026-07-13
**Iteration**: I124 (One-Shot Scheduled Follow-Up)
**Gate**: Pre-activation security review required by the execution package Start Gate step 5.
**Verdict**: PASS — the existing permission and session architecture satisfies every safety
requirement for I124 without code changes to `talos-permission`.

## Purpose

Prove six claims before activating I124:

1. `delay`, `schedule`, and `cancel_scheduled_task` resolve as `ToolNature::Execute`, default `Ask`.
2. `list_scheduled_tasks` resolves as `ToolNature::Read`.
3. An explicit `Deny` always wins.
4. Approval of a scheduling-tool registration does not approve any future tool call.
5. Fire-time tool calls triggered by a scheduled message are evaluated through the normal
   permission pipeline.
6. No modification to `talos-permission` default decisions is required.

## Evidence Base

All evidence is from the workspace at commit `a3f17ad` on `main` (the I124 planning commit).
No source files were modified for this review.

---

## Claim 1: Mutation Tools Are Execute / Ask

### ToolNature enum

`crates/talos-core/src/tool.rs` lines 156-170 define five variants: `Read` (default), `Write`,
`Execute`, `Network`, `Internal`. The scheduling mutation tools will declare `Execute`.

### Default rule: Execute → Ask

`crates/talos-permission/src/lib.rs` lines 174-179 (`add_default_rules`):

```rust
self.rules.push(PermissionRule::new_nature(
    ToolNature::Execute,
    None,
    None,
    PermissionDecision::Ask,
));
```

### Fallback in evaluate_facet

`crates/talos-permission/src/lib.rs` lines 332-339:

```rust
match nature {
    ToolNature::Read | ToolNature::Internal => PermissionDecision::Allow,
    ToolNature::Write | ToolNature::Execute | ToolNature::Network => PermissionDecision::Ask,
}
```

### Existing Execute tools as precedent

- `BashTool`: `crates/talos-tools/src/bash_tool.rs` line 270 — `ToolNature::Execute`
- `ExecTool`: `crates/talos-tools/src/exec_tool.rs` line 583 — `ToolNature::Execute`

### Test

`crates/talos-permission/src/permission_tests.rs` lines 35-39 — `test_default_bash_tool_ask`
asserts `PermissionDecision::Ask` for an Execute tool with default rules.

**Conclusion**: `delay`, `schedule`, and `cancel_scheduled_task` will declare
`ToolNature::Execute` and receive `Ask` by default through the existing rule and fallback path.

---

## Claim 2: list_scheduled_tasks Is Read

### Tool declaration

`list_scheduled_tasks` will declare `ToolNature::Read` in its `nature()` method.

### Heuristic fallback

Even without an explicit `nature()` override, `infer_nature()` in
`crates/talos-permission/src/lib.rs` lines 458-464 classifies any tool name containing `"list"` as
`Read`:

```rust
if name_lower.contains("read")
    || name_lower.contains("list")
    || name_lower == "grep"
    || ...
{
    ToolNature::Read
}
```

### Default rule: Read → Allow

`crates/talos-permission/src/lib.rs` lines 162-167:

```rust
self.rules.push(PermissionRule::new_nature(
    ToolNature::Read,
    None,
    None,
    PermissionDecision::Allow,
));
```

### Test

`crates/talos-permission/src/permission_tests.rs` lines 7-11 — `test_default_read_tool_allowed`
asserts `PermissionDecision::Allow` for a Read tool.

**Conclusion**: `list_scheduled_tasks` resolves as `Read` and receives `Allow` by default. It
cannot mutate session execution state.

---

## Claim 3: Deny Always Wins

### Profile aggregation

`crates/talos-permission/src/lib.rs` lines 257-282 (`evaluate_profile`):

```rust
for facet in facets {
    match self.evaluate_facet(tool_name, &facet, input) {
        PermissionDecision::Allow => {}
        PermissionDecision::Ask => saw_ask = true,
        PermissionDecision::Deny(reason) => return PermissionDecision::Deny(reason),
    }
}
```

Any denied facet returns `Deny` immediately — no further facets are evaluated.

### Explicit deny rules override runtime allow

`crates/talos-permission/src/permission_tests.rs` lines 365-398
(`test_runtime_allow_rule_does_not_override_deny`) proves a config `Deny` rule for `Execute`
survives a runtime `Allow` rule with a more specific resource pattern.

### Multi-facet deny test

`crates/talos-permission/src/permission_tests.rs` lines 493-530
(`test_profile_denies_when_any_facet_is_denied`) proves a profile with one `Allow` facet and one
`Deny` facet results in `Deny`.

**Conclusion**: No scheduling registration, approval, or runtime state can suppress an explicit
`Deny`. Deny is evaluated first and short-circuits at every layer (profile, facet,
command-with-evidence).

---

## Claim 4: Registration Approval Does Not Approve Future Tool Calls

### SessionOp carries only a String

`crates/talos-core/src/session.rs` lines 17-37:

```rust
pub enum SessionOp {
    Submit { message: String },
    PreviewRequest { message: String },
    SetSkillContext { name: Option<String>, content: Option<String> },
    Interrupt,
    Shutdown,
}
```

There is no tool-call variant. A scheduling registration approval authorizes the registration
tool call itself (e.g., `delay({ message, delay_secs })`) through the normal `Ask` path. It does
not carry forward any permission state.

### The SQ is a bounded mpsc channel

`crates/talos-core/src/session.rs` lines 127-132 and `crates/talos-agent/src/session.rs` lines
60-64:

```rust
let (sq_tx, sq_rx) = tokio::sync::mpsc::channel(512);
```

Every injection path — CLI (`event_loop.rs:338`), TUI (`tui_bridge.rs:206`), embedded runtime
(`talos-runtime/src/lib.rs:269`) — sends `SessionOp::Submit { message: String }` through the same
`mpsc::Sender<SessionOp>`. A scheduled fire will clone `sq_tx` and send `Submit` — identical to a
user-typed message.

### No pre-approval mechanism exists

The `PermissionEngine` has no "pre-approved future operation" state. Each call to
`evaluate_profile` is stateless with respect to prior approvals. Runtime allow rules are scoped to
specific `(nature, resource, resource_kind)` triples and cannot be created by a tool call — they
are installed only by the interactive approval handler when the user explicitly chooses "always"
scope for a specific operation pattern.

**Conclusion**: Approving `delay({ message: "...", delay_secs: 60 })` authorizes only that
registration. When the fire occurs 60 seconds later and the injected message causes the model to
emit a tool call, that tool call receives a completely independent permission decision.

---

## Claim 5: Fire-Time Tool Calls Re-Enter the Normal Permission Pipeline

### Full evidence chain

1. **Scheduler actor fires** → sends `SessionOp::Submit { message: labeled_followup }` via
   `sq_tx.send()` (same bounded mpsc channel as user input).

2. **Session actor receives** → `crates/talos-agent/src/session.rs` line 117: the `run()` loop
   receives `SessionOp::Submit { message }` and spawns a turn task.

3. **Turn forwarding** → `crates/talos-agent/src/session/turn.rs` line 95: calls
   `agent.run_streaming(message, history, event_tx)`. The message is treated identically to a
   user-typed message.

4. **Provider responds with tool calls** → `crates/talos-agent/src/lib.rs` lines 660-803: the turn
   loop collects `AgentEvent::ToolCall { call, provenance }` events from the provider stream.

5. **Each tool call is permission-checked** → `crates/talos-agent/src/tool_execution.rs` lines
   252-342 (`execute_single_tool_with_presentation`):

   ```rust
   let profile = tool.permission_profile(&call.input);
   let decision = engine.evaluate_profile(&call.name, &profile, &call.input);
   ```

   This is called once per tool call, not once per turn.

6. **Deny blocks execution** → `tool_execution.rs` returns `ToolExecutionResult::error(...)`.

7. **Ask triggers interactive approval** → `tool_execution.rs` propagates
   `PermissionDecision::Ask` to the session event as `ApprovalRequired` (see
   `crates/talos-core/src/session.rs` lines 64-69), which the CLI/TUI layer must resolve before
   execution proceeds. In headless mode, `Ask` is treated as `Deny`.

### No bypass path

There is no `SessionOp` variant, no agent constructor, and no tool-execution code path that can
skip `execute_single_tool_with_presentation`. The scheduler actor will hold a clone of `sq_tx` and
can only produce `SessionOp::Submit { message: String }`. It has no access to the tool registry,
the permission engine, or the agent internals.

**Conclusion**: A scheduled message fire enters the exact same pipeline as a user-typed message.
Every tool call in the resulting turn is independently permission-gated.

---

## Claim 6: No talos-permission Modification Required

The existing system already provides:

| Requirement | Existing mechanism | Location |
|---|---|---|
| Execute → Ask default | `add_default_rules` | `talos-permission/src/lib.rs:174-179` |
| Read → Allow default | `add_default_rules` | `talos-permission/src/lib.rs:162-167` |
| Deny precedence | `evaluate_profile` short-circuit | `talos-permission/src/lib.rs:269-276` |
| Per-call evaluation | `execute_single_tool_with_presentation` | `talos-agent/src/tool_execution.rs:300-301` |
| No future-approval state | Stateless `evaluate_profile` | `talos-permission/src/lib.rs:257-282` |
| Name-based heuristic for "list" | `infer_nature` | `talos-permission/src/lib.rs:458-464` |

No new `PermissionRule`, no new `ToolNature` variant, no new default decision, and no new
evaluation path is needed.

---

## Composition Root Inventory

For I124 SF102, the scheduling tools must be registered at these sites (evidence from
`crates/talos-cli/src/registry.rs` and composition roots):

| Root | Function | Call sites |
|---|---|---|
| Print / Inline / RPC | `build_print_tool_registry()` | `mode_print.rs:50`, `mode_inline.rs:59`, `mode_runners.rs:100` |
| TUI | `build_tui_tool_registry()` | `mode_runners.rs:236`, `session_handlers.rs:504/734/912`, `model_lifecycle.rs:213` |
| MCP server | `build_mcp_tool_registry()` | `mode_runners.rs:658` |
| Interactive REPL | inline in `mode_interactive.rs:46-135` | `mode_runners.rs:501` |

Missing any root is a blocking product defect per the execution package.

---

## Architecture Compliance Checklist

- [x] `delay`, `schedule`, `cancel_scheduled_task`: will declare `ToolNature::Execute`, default `Ask`
- [x] `list_scheduled_tasks`: will declare `ToolNature::Read`
- [x] `Deny` always wins (evidence: `evaluate_profile` short-circuit + tests)
- [x] Registration approval does not approve future tool calls (evidence: `SessionOp::Submit` carries only `String`; `evaluate_profile` is stateless)
- [x] Fire-time tool calls re-enter normal permission pipeline (evidence: every tool call passes through `execute_single_tool_with_presentation` → `evaluate_profile`)
- [x] No modification to `talos-permission` default decisions required

## Conclusion

The existing permission and session architecture already satisfies every safety requirement for
I124. The scheduling tools will use existing `ToolNature` classification, existing default rules,
and existing per-call permission evaluation. Message injection via `SessionOp::Submit` cannot
bypass the permission pipeline because `SessionOp` carries only a `String`, and every tool call
derived from that string is independently permission-gated by `execute_single_tool_with_presentation`
→ `engine.evaluate_profile`.

I124 is cleared for activation.

## Post-Delivery Review Addendum (2026-07-14)

The pre-activation claims describe the required architecture, but the delivered I124 composition
does not satisfy Claim 1 in production. All 9 applicable roots register the raw `DelayTool`
directly. They do not use `PermissionAwareTool` or `TuiPermissionAwareTool`, which are responsible
for resolving `PermissionDecision::Ask` into an interactive prompt or headless denial. The central
agent permission path continues when it sees `Ask`, so declaring `ToolNature::Execute` is not by
itself an effective Ask gate.

Post-delivery verdict: **FAIL pending correction and re-review**. I124 remains Review and I125
remains blocked. The correction must prove that Deny and unresolved Ask cannot register or fire a
task in every supported composition family.

### Second Review Update (2026-07-14)

Commit `68c24cf` corrects the production composition: the delay tool now passes through the
appropriate CLI/TUI permission wrapper, and focused Deny plus headless-Ask tests pass. Claim 1 is
therefore satisfied for the registered production tool.

The overall post-delivery verdict remains **FAIL pending correction and re-review** because the
new fixture-provider test configures every Execute tool as Allow. It proves traversal of the real
Agent/session path, but it does not distinguish a fresh follow-up decision from inherited or
bypassed authority. The re-review requires a distinct registration Allow and follow-up Deny/Ask,
with proof that the follow-up tool does not execute.

### Third Review Update (2026-07-14)

Commit `7fe1d17` gives the follow-up `echo` resource a distinct Deny rule, while the production
wrapper regressions remain green. The test still does not prove that the scheduled follow-up turn
occurred: its only outcome assertion is that `echo` did not execute, which is also true if the
scheduler fails to fire or the second turn is never processed. It must additionally assert the
second provider/turn path and scheduled-message observation.

The post-delivery verdict therefore remains **FAIL pending correction and re-review**. I124 stays
Review and I125 stays blocked.

### Closure Update (2026-07-14)

The third-review blocker is corrected. The real Agent/session test now proves the scheduled turn
ran by asserting all four fixture responses were consumed and that a provider request observed the
`[scheduled-followup]` user message; it separately proves the resource-specific Deny prevented
`echo` execution. Production wrapper Deny and unresolved-Ask regressions remain green.

Final post-delivery verdict: **PASS**. The six security claims match the delivered production and
test evidence. I124 is Complete; I125 is unblocked but not activated.
