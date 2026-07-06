# MODEL-006: Interactive CLI Model Catalog Browser

| Field | Value |
|---|---|
| ID | MODEL-006 |
| Type | Product Story |
| Priority | P1 |
| Status | In Progress — first independent CLI browser slice implemented |
| Source | Maintainer request 2026-07-05 — command-line `--available-models` output is too large; users need a vim-like scrollable/modifiable view before entering the main Talos TUI |
| Depends on | MC-001, MODEL-005 |
| Blocks | Full model-catalog UX closeout |

## Problem

The packaged catalog now contains thousands of model rows. A plain `--available-models` dump is too
large for normal terminal use even when each row is unambiguous as `provider/model`.

Users need an interactive command-line view that behaves like a terminal browser: scrollable,
searchable, and able to apply model-related changes without dumping thousands of rows to stdout.
This happens before the main Talos conversation TUI starts, so it must be implemented as an
independent CLI mode rather than coupling to the existing session TUI state machine.

## Product Direction

Keep the script-oriented `--available-models` output bounded and filterable. Add a separate
interactive CLI browser mode for human exploration. The first implemented command surface is
`talos --available-models-browser`.

Expected interaction shape:

- Open from command-line mode, before starting the main conversation TUI.
- Show packaged providers and models, including providers that are not connected yet.
- Support vim-like navigation keys (`j/k`, `g/G`, `/` search) in addition to existing arrow keys.
- Keep group headers visible and high-contrast.
- Allow selecting an authenticated model to switch active provider/model.
- Allow selecting an unauthenticated standard provider/model to open an independent credential
  prompt that uses the catalog-defined provider endpoint and does not ask for URL input.
- Allow custom provider setup to collect a URL because no catalog endpoint exists.
- Render large model lists incrementally or by viewport/search window so opening the browser does
  not render all packaged rows at once.
- Reuse config merge semantics from `/connect`, but not the session TUI widgets or conversation state.
- Never expose or print existing API key values.

## Implementation Path

1. Extract model-catalog view data into a CLI-neutral service:
   `provider`, `model_id`, `provider/model`, auth status, context/output limit, pricing.
2. Keep `--available-models` as non-interactive bounded output with `--available-models-filter`,
   `--available-models-limit`, and `--available-models-all`.
3. Add an independent terminal browser module under `talos-cli`, using existing terminal UI
   dependencies only if they do not pull in session TUI state.
4. Add a credential prompt path that calls the same config merge helper used by `/connect`; only
   custom provider setup asks for a base URL.
5. Add incremental or viewport-windowed rendering for the full packaged catalog.
6. Add headless tests for filtering/selection/config writes and at least one terminal-render smoke
   test for navigation state.

## Acceptance Criteria

- [x] Browser can handle the full packaged catalog without dumping thousands of rows to stdout.
- [x] Search filters by provider, model id, and provider/model.
- [x] Current active model is visually identifiable.
- [x] Unauthenticated rows route to provider setup rather than appearing as selectable active models.
- [x] Credential/base URL updates use existing config merge behavior and preserve unrelated fields,
      without depending on the main session TUI.
- [x] CLI `--available-models` remains bounded/filterable for scripts and support diagnostics.
- [x] Tests cover navigation, filtering, selection, provider setup routing, and no-secret rendering.
- [ ] Standard packaged provider setup does not ask for URL; it uses the catalog-defined endpoint.
- [ ] Custom provider setup requires and validates URL input.
- [ ] Large model-list rendering is incremental or viewport-windowed and avoids all-row render on
      open/scroll.

## Implemented Slice

2026-07-05: `talos --available-models-browser` opens an independent crossterm browser before the
main conversation TUI starts. The browser:

- Uses its own CLI-local state in `crates/talos-cli/src/models_browser.rs`.
- Supports `j/k`, arrows, PageUp/PageDown, `g/G`, `/` search, `Enter` selection/setup, `c`
  provider setup, and `q`/Esc quit.
- Renders high-contrast provider headers, current-model markers, ready/setup status, context, and
  pricing.
- Keeps existing `--available-models` bounded/filterable output unchanged for scripts.
- Refuses to run without an interactive terminal.
- Saves active authenticated model selections directly.
- Prompts for API key/base URL for unauthenticated rows without printing existing API key values.

Residual hardening:

- Add a real-terminal manual walkthrough before marking `MODEL-006` Complete.
- Fix standard-provider connect/setup so only credentials are requested; URL is prompted only for
  custom providers.
- Add incremental/viewport-windowed model-list rendering to avoid full-catalog render stalls.
- Consider a subcommand alias such as `talos models browse` after the CLI command taxonomy settles.

## Current Mitigation

2026-07-05: `--available-models` now prints `provider/model`, defaults to bounded output, supports
`--available-models-filter <QUERY>`, and requires `--available-models-all` for full output.

## Required Reads

- `docs/backlog/active/MC-001-model-catalog-modernization.md`
- `docs/backlog/active/MODEL-005-interactive-model-selection.md`
- `crates/talos-cli/src/model_lifecycle.rs`
- `crates/talos-cli/src/mode_runners.rs`
- `crates/talos-cli/src/main.rs`
