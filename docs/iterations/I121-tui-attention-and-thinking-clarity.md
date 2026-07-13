# Iteration I121: TUI Attention And Thinking Clarity

> Document status: Planned — blocked on I120 Complete
> Published plan date: 2026-07-13
> Planned objective: Make approval requests unmistakable and thinking previews concise without
> changing permission or reasoning-storage semantics.
> Baseline rule: preserve this target after publication; changed targets use a new iteration ID.
> MVP deliverable: approval and thinking preview behavior passes narrow-buffer tests and one native
> terminal walkthrough.

## Published Baseline

### Selected Stories

| Story | Owner | Outcome |
|---|---|---|
| F110-F111 | TUI-008 | Prominent approval overlay and unchanged approval decisions |
| F112 | TUI-024 | Standalone-bold thinking title extraction with `thinking` prefix |
| F113 | I121 | Native-terminal acceptance and docs |

### Scope

- Center or inline the existing approval presentation, preserve its event owner and keyboard actions,
  and test 80-column plus narrow terminal buffers.
- Parse the most recent standalone bold heading from transient accumulated thinking. Keep the
  existing fallback when no valid heading exists and retain the animated `thinking` label style.
- Add semantic render tests for placement, styles, clipping, title updates, and export exclusion.

### Non-Goals

- No permission-policy change, auto-approval, new popup framework, persisted title/reasoning,
  provider request change, collapsible reasoning panel, TUI-025/026/027, or broad TUI refactor.

### Acceptance

- Approval is visible and operable at 80 columns and the documented narrow minimum.
- Existing Allow/Ask/Deny routing and keys are unchanged.
- A standalone `**Title**` block yields `thinking: Title`; inline bold does not.
- Default copy/export and session persistence remain unchanged under ADR-034.
- A named terminal/viewport walkthrough records observed results without secrets.

### Validation And Docs

- Targeted `talos-tui` tests, default export regression, native-terminal packet, and the standard
  validation ladder. Update TUI-008, TUI-024, README/help if user-visible wording changes.

### Risks And Fallback

- Layout collision: prefer bounded overlay placement and clipping; do not rebuild input layers.
- Platform rendering variance: assert ratatui semantic buffers and retain one real-terminal record.

## Execution Record

Not started. Do not activate until I120 is Complete.
