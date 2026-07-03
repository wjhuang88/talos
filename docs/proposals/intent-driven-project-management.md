# Intent-Driven Project Management

## Problem

Talos now has explicit slash commands such as `/agile status`, a shared command registry, and a
programmatic governance status/validation path. That is necessary but not sufficient for daily
project management.

Users often express project-management intent in natural language:

- "start the next iteration"
- "close this task and sync the board"
- "record this as a future idea"
- "this requirement changed"
- "what should I work on next?"

If Talos treats these as ordinary chat, it may answer descriptively instead of executing the right
project-management workflow. If Talos treats every similar sentence as a command, it may mutate
governance state without enough certainty. Intent recognition and project-management execution
therefore need to be one integrated capability with an explicit safety boundary.

## Requirement

Talos should recognize project-management intent, classify it into a typed governance action, and
execute that action through the same GOV-003 project-management engine used by `/agile` commands.

Intent recognition is not the owner of project-management behavior. It is the trigger and routing
layer. Project-management logic remains owned by GOV-003.

## Proposed Approach

Introduce a three-stage flow:

1. **Intent detection**
   - Classify user input as ordinary chat, explicit slash command, project-management intent, or
     ambiguous.
   - Prefer deterministic rules for high-confidence phrases that are already routed by AGENTS.md and
     SOPs.
   - Use LLM interpretation only as an advisory layer, never as the sole authority for writes.

2. **Typed governance intent**
   - Convert recognized input into a small enum such as:
     - `GovernanceStatus`
     - `StartIteration`
     - `ChangeControl`
     - `RecordProposal`
     - `CloseIteration`
     - `SyncBoard`
     - `ValidateGovernance`
   - Attach confidence, required reads, target IDs, and whether the action is read-only or mutating.
   - If confidence is low or required targets are missing, return a clarification prompt instead of
     executing.

3. **Governance action execution**
   - Read-only actions may run immediately and return structured output.
   - Mutating actions produce a preview/diff and require explicit user confirmation through the
     existing permission/approval path.
   - All writes go through the same file tools or permission-gated write pipeline. Intent detection
     must never write files directly.

## Architecture Integration

### `talos-conversation`

The conversation layer should own routing from user text to typed outputs, but not domain logic.

Future shape:

```text
UserInput::Message
  -> IntentRouter
  -> IntentDecision
       OrdinaryChat
       SlashCommand
       GovernanceIntent(...)
       Clarification(...)
```

The router emits typed `UiOutput` or session lifecycle requests, matching the CMD-001 rule that
commands delegate to their real owner.

### CMD-001 Command Registry

Slash commands remain the explicit command surface. Intent recognition should reuse command metadata
where possible:

- `/agile status` and "show project status" route to the same typed action.
- `/agile validate` and "validate governance" route to the same typed action.
- The command registry remains the source of truth for explicit command availability.

Natural-language routing must not create a second hidden command catalog. It maps to the same typed
actions exposed by commands.

### GOV-003 Governance Engine

GOV-003 becomes the domain owner for project-management actions:

```text
GovernanceEngine
  - classify_workspace()
  - summarize_status()
  - validate()
  - plan_start_iteration()
  - plan_change_control()
  - plan_closeout()
  - apply_confirmed_action()
```

Read-only methods return reports. Mutating methods return planned file operations and evidence
requirements before any write happens.

### `talos-cli` and `talos-tui`

CLI/TUI remain presentation and orchestration layers:

- TUI displays intent previews, clarification prompts, and confirmation requests.
- CLI subcommands expose explicit forms for automation.
- Neither layer owns project-management rules.

### Dashboard

The dashboard should display structured governance data from the GOV-003 engine. It should not parse
Markdown independently once a shared engine exists, except as a temporary bridge.

## Safety Boundary

Hard requirements:

- Intent recognition can trigger read-only reports directly.
- Intent recognition cannot directly mutate files.
- Mutating governance actions require a typed plan, preview, and explicit confirmation.
- All writes must go through the permission pipeline.
- The same user phrase must produce deterministic routing when workspace state is unchanged.
- Ambiguous intent asks a question instead of guessing.

## Relationship To Current Work

Current implementation already has pieces of the target design:

- CMD-001 provides a shared command registry and typed command metadata.
- `/agile status` exists as a read-only governance entry point.
- Governance validation is now Rust-programmatic for interactive status paths.
- Dashboard `/governance` exposes a read-only view.

Remaining gap:

- There is no intent router that maps natural language to GOV-003 typed actions.
- Governance parsing/validation is still embedded in existing modules rather than a dedicated
  governance engine boundary.
- Mutating project-management actions are not yet represented as previewable typed plans.

## Suggested Delivery Slices

### Slice 1: Read-Only Intent Routing

- Add deterministic routing for high-confidence read-only phrases:
  - "project status"
  - "agile status"
  - "validate governance"
  - "what is active right now?"
- Route to the same report path as `/agile status`.
- Add tests proving ordinary chat is not hijacked.

### Slice 2: Governance Engine Boundary

- Extract status, board, backlog, iteration, and validation parsing into a focused Rust module or
  crate.
- Have `/agile`, `--governance-status`, and dashboard governance use that engine.
- Keep file formats human-editable.

### Slice 3: Preview-Only Mutating Intents

- Recognize intents such as "start next iteration" and "record this idea".
- Produce a structured plan and file-operation preview without writing.
- Ask for confirmation.

### Slice 4: Confirmed Project-Management Execution

- Apply confirmed plans through the permission-gated write path.
- Record validation evidence and residual work.
- Update Board/backlog/iteration owner docs only through typed actions.

## Acceptance Criteria

- Natural-language project-management requests route to typed governance intents.
- `/agile` commands and natural-language equivalents share the same implementation path.
- Read-only intents never execute workspace scripts or mutate files.
- Mutating intents produce previewable planned file operations before confirmation.
- Ambiguous requests return clarification, not silent execution.
- Dashboard, CLI, and TUI consume shared governance reports rather than duplicating parsing logic.

## Alternatives Considered

- **Slash commands only.** Simpler, but misses the user's natural project-management workflow and
  forces memorization.
- **LLM decides and edits files directly.** Flexible, but violates Talos safety and makes governance
  behavior nondeterministic.
- **Keep governance in scripts.** Easy to bootstrap, but does not integrate with command routing,
  dashboard, typed previews, or permission-gated mutation.

## Open Questions

- Should the intent router live in `talos-conversation`, or should a future governance crate expose
  the router plus engine together?
- What confidence threshold is required before read-only intent runs automatically?
- Which mutating governance actions should be allowed in the first confirmed-execution slice?
- Should project-management intent be visible in session history as a structured marker?

## Dependencies

- CMD-001 command registry and typed command ownership.
- GOV-003 built-in project governance logic.
- Permission pipeline for file writes.
- TUI confirmation UX for planned file operations.
