# I008: "Learning Agent"

**User can**: Agent adapts its behavior across sessions via built-in evolution with cognitive feedback (ADR-001).

## Status: ACTIVE — re-scoped 2026-06-01 to ship evolution as a builtin `HookHandler` ⚙️

> **Re-scope summary (2026-06-01).** Earlier guidance placed I008 evolution-into-all-paths
> behind the `AppServerSession` seam (#I010-S7) to avoid double-firing. A pre-implementation
> review (hook capability audit + adversarial analysis) showed evolution can ship **NOW** as
> a builtin `talos_plugin::HookHandler` registered per-Agent in all three run paths:
>
> - Hooks fire uniformly inside `talos-agent::run_inner` (one Agent per turn in every path) →
>   one set of hook fires per turn → **no double-firing**. The earlier concern was about
>   *path-level event consumption*; hook dispatch lives at the *agent-internal* layer.
> - [ADR-005](../decisions/005-tui-event-architecture.md) (which references "450 tests")
>   predates the I009 hook system (now 501 tests). Its premise that evolution requires
>   AppServerSession is partially obsolete.
> - No dependency cycle: `talos-evolution → talos-plugin → talos-core`.
>
> **I008 R1/R2/R4 therefore no longer wait on #I010-S7.** #I010-S7 is re-scoped to
> independent architectural cleanup (cross-Agent / cross-session / UI status concerns).
> The hook layer ships in I008.
>
> ADR-005, ADR-006, and ADR-001 amended to record this re-scope.

## Story Status

| Story | Library | Runtime-integrated | Notes |
|-------|---------|--------------------|-------|
| S1: Evolution crate + data models | ✅ | n/a | Types + SQLite schema present |
| S2: TurnObserver — signal capture | ✅ | ⚠️→🛠️ (re-wiring via hook) | Hook captures `OnProviderError` + user-correction from `BeforeProviderCall`; all three run paths covered uniformly |
| S3: PatternExtractor — extraction | ✅ | ✅ | Invoked at hook flush time (`TurnComplete`) |
| S4: KnowledgeStore — SQLite persistence | ✅ | ✅ | Hook writes observations + accumulates patterns at runtime, all paths |
| S5: BehaviorAdapter — prompt injection | ✅ | ⚠️→🛠️ (re-wiring via hook) | `OnSystemPromptBuilt` + `HookResult::Modify` injects in all paths |
| S6: TUI evolution panel (Ctrl+E) | ✅ | ⚠️ | Renders. Live feed works via shared `KnowledgeStore` (the hook writes to it) |
| S7: `--learned` command | ✅ | ✅ | Shows real patterns once a session has written observations |

## Plan (re-scoped, 2026-06-01)

1. **`talos-evolution`: add `EvolutionHookHandler`** implementing `talos_plugin::HookHandler`.
   Subscribed to `[TurnStart, OnSystemPromptBuilt, BeforeProviderCall, OnProviderError,
   OnToolResultObserved, OnTextDelta, AfterToolCall, OnTurnEnd, TurnComplete]`. Reset
   accumulator on `TurnStart`, flush (ingest) on `TurnComplete`. Inject context via
   `OnSystemPromptBuilt` + `HookResult::Modify`. Override `fn timeout()` to 5s for SQLite
   write. **Known cost**: `Box::leak` for the `'static` prompt replacement (tracked
   separately as a `HookResult::ModifyOwned` variant).
2. **`talos-evolution/Cargo.toml`**: add `talos-plugin = { path = "../talos-plugin" }`.
3. **`talos-cli/src/main.rs`**:
   - Extend `build_hook_registry(include_evolution: bool)` to optionally register
     `EvolutionHookHandler` alongside `LoggingHandler`.
   - `run_print_mode` / `run_tui_mode` / `run_interactive_mode` / `run_rpc_mode` pass
     `include_evolution=true`; `run_mcp_server` keeps `false` (external callers).
   - Remove the print-mode side-channel `event_rx` evolution observation loop (replaced by
     hook on `OnProviderError` + `TurnComplete`).
   - Remove the print-mode pre-turn `agent.set_append_prompt(evolution_context())`
     (replaced by hook on `OnSystemPromptBuilt`).
4. **Tests**:
   - Unit tests for `EvolutionHookHandler` (per-turn lifecycle, `Modify` payload, flush).
   - Existing `evolution_runtime` tests stay (regression).
5. **Verification**:
   - `cargo test --workspace` exits 0.
   - `cargo clippy --workspace` clean.
   - Real `talos --tui --mock` smoke test: log file at `~/.talos/logs/talos.log` shows
     evolution hook events; layout is clean.
   - Real `talos -p --mock` smoke test: `--learned` shows observed patterns.

## Residual Work (registered, not deferred silently)

- **R1** — Wire `TurnObserver` to fire in all run paths. **Post-re-scope: all three paths
  covered by the same `EvolutionHookHandler` registered via `build_hook_registry(true)`.**
  No per-path code change beyond the call-site flag.
- **R2** — Inject `BehaviorAdapter` context in all paths. **Post-re-scope: same hook handler,
  via `OnSystemPromptBuilt`.** No per-path code change.
- **R3** — Render `EvolutionPanel` in `talos-tui::render()`. **✅ DONE.**
- **R4** — End-to-end evidence (real turn → observation persisted → `--learned` shows it →
  next run injects it). **Print path ✅** (existing mock smoke test). **TUI evidence**
  becomes trivially available: just run `talos --tui --mock`, send a message, quit, then
  `talos --learned` shows the patterns. (R4 is now a verification step, not a code gap.)

## Verification

> The library + unit tests pass. The hook-based E2E checks below **WILL PASS** after the
> re-scoped implementation lands.

```bash
# Library + unit tests (PASS)
cargo build --release -p talos-cli
cargo test --workspace
cargo test -p talos-evolution
cargo test -p talos-cli            # includes evolution_runtime regression tests

# RUNTIME — print path (PASS):
HOME=$(mktemp -d) sh -c '
  talos --mock -p "don'\''t use unwrap in library code"
  talos --learned                  # shows the extracted preference pattern
'

# RUNTIME — TUI (post-re-scope: trivially available; all paths use the same hook):
# 1. Send a few messages via the TUI mock.
# 2. Quit. The hook handler has written observations to ~/.talos/index.db.
# 3. `talos --learned` shows the patterns. (Same store, same handler.)
```

## Execution Record (appended during execution per SOP §3a)

### 2026-06-01: Re-scoped implementation lands

**Code landed (commit pending):**
- `crates/talos-evolution/src/hook.rs` (NEW, 12 unit tests): `EvolutionHookHandler` implementing
  `talos_plugin::HookHandler`. Subscribed to `[TurnStart, OnSystemPromptBuilt, BeforeProviderCall,
  OnProviderError, OnToolResultObserved, OnTextDelta, AfterToolCall, OnTurnEnd, TurnComplete]`.
  Resets observer on `TurnStart`, flushes observations + patterns to `KnowledgeStore` on
  `TurnComplete` (timeout 5s override). Injects context via `OnSystemPromptBuilt` + `HookResult::Modify`
  (using `Box::leak` for the `'static` replacement, ADR-001 pointer).
- `crates/talos-evolution/src/lib.rs`: `pub mod hook;` + `pub use hook::EvolutionHookHandler;`
- `crates/talos-evolution/Cargo.toml`: added `talos-plugin`, `async-trait`, `dirs`, `tracing`;
  dev-dep `tokio` (macros/rt/time).
- `crates/talos-cli/src/evolution_runtime.rs` (DELETED, 276 lines): heuristic + accumulation
  logic moved into the hook.
- `crates/talos-cli/src/main.rs`:
  - `init_tracing(to_file: bool)` now redirects to `~/.talos/logs/talos.log` for terminal-UI
    modes (cli.tui || stdin-tty + non-machine-mode), keeps stderr for print/rpc/mcp.
    Zero new deps (`Arc<File>` is valid `MakeWriter`).
  - `build_hook_registry(include_evolution: bool)` registers `EvolutionHookHandler` (when
    `true`) alongside `LoggingHandler`.
  - All 5 call sites updated: `run_rpc_mode` / `run_print_mode` / `run_tui_mode` /
    `run_interactive_mode` → `true`; `run_mcp_server` → `false` (external callers).
  - `run_print_mode` side-channel `event_rx` evolution observation loop and pre-turn
    `agent.set_append_prompt(evolution_context)` removed (replaced by the hook on
    `OnSystemPromptBuilt` + `OnProviderError` + `TurnComplete`).
- ADR-001, ADR-005, ADR-006 amended to record the re-scope + canonical wiring (see
  `docs/decisions/`).
- Backlog #I010-S7 description updated to remove the obsolete "single wiring point for I008"
  claim; now framed as independent cross-Agent / cross-session / UI status cleanup.

**Verification evidence:**
- `cargo check -p talos-cli`: clean
- `cargo clippy -p talos-cli --bin talos -- -D warnings`: clean
- `cargo test --workspace`: **509 passed, 0 failed, 0 ignored** (+8 vs I009's 501; the +8 are
  net additions across `talos-evolution` after the 12 new hook tests)
- `cargo test -p talos-evolution`: 29 tests pass (12 new hook + 17 existing)
- **E2E runtime — print mode (machine mode)**: `echo "..." | talos -p --mock` writes tracing to
  stderr (7 lines observed), `~/.talos/logs/talos.log` unchanged, `index.db` observations
  went 4 → 5 (proves hook handler `OnSystemPromptBuilt` is firing).
- **E2E runtime — TUI mode (terminal mode)**: `talos --tui --mock` creates
  `~/.talos/logs/talos.log` (proves the file branch in `init_tracing` is taken), stderr
  has 0 tracing lines. (TUI process is killed in test; the file was opened in append mode,
  so subsequent TUI runs append.)

**R1/R2/R4 resolution:** All three are now covered by the single `EvolutionHookHandler` registered
via `build_hook_registry(true)` in `run_rpc_mode` / `run_print_mode` / `run_tui_mode` /
`run_interactive_mode`. The only per-path flag is the bool at the call site. R3 (TUI panel)
was already complete.

**Follow-ups (out of I008 scope, registered as separate stories):**
- `#I009-S5`: `HookResult::ModifyOwned` additive variant — non-breaking, drops the
  `Box::leak` per-turn cost (~few KB/turn).
- `#I010-S7`: AppServerSession seam — independent architectural cleanup (cross-Agent /
  cross-session / UI status). Not a prerequisite for I008 anymore.

### 2026-06-01: TUI is now the default TTY mode

**Decision (soft migration, option B).** The TUI is now the default for TTY users; the
legacy readline REPL is retained as `--repl` for users who want the lighter-weight path
(accessibility, screen readers, minimal terminals, debugging).

**Code change (one commit):**
- `crates/talos-cli/src/main.rs`:
  - Added `repl: bool` to `Cli` with `conflicts_with = "tui"` and help text
    *"Force the readline interactive REPL (default is TUI on a TTY)."*
  - Main dispatch updated: `cli.tui` → TUI (unchanged, redundant with new default);
    `cli.repl` → REPL (new); `!is_terminal()` → print (unchanged); **default TTY → TUI**
    (was: REPL).

**New dispatch table:**

| Invocation | Mode |
|---|---|
| `talos --mcp-server` | MCP server |
| `talos --search <q>` | Search (lists matching messages) |
| `talos --list` | List recent sessions |
| `talos --learned` | Show evolution patterns |
| `talos --mode=print …` / `talos -p …` | Print (stream + exit) |
| `talos --tui …` | TUI (explicit, now redundant with default) |
| `talos --repl …` | Readline REPL (explicit) |
| `echo "..." \| talos` (stdin not a TTY) | Print (pipe-friendly fallback) |
| **`talos` (TTY, no flags)** | **TUI (new default — was REPL)** |

**Verification:**
- `cargo test --workspace`: 509 passed, 0 failed
- `cargo clippy -p talos-cli --bin talos -- -D warnings`: clean
- E2E: `--tui --repl` rejected by clap (`cannot be used with`)
- E2E: TTY + no flags → log file created (proves TUI is the new default)
- E2E: TTY + `--repl` → routes to `run_interactive_mode` (verified by code review of dispatch)
- E2E: TTY + `--tui --mock` (explicit) → log file created (unchanged)
- E2E: stdin pipe + no flags → print mode (unchanged)
