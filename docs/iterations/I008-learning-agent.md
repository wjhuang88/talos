# I008: "Learning Agent"

**User can**: Agent adapts its behavior across sessions via built-in evolution with cognitive feedback (ADR-001).

## Status: REVIEW — runtime loop wired on the print path; TUI/interactive paths pending ⚠️

> **Downgraded from COMPLETE on 2026-06-01.** A post-implementation audit found that
> `talos-evolution` existed as a fully unit-tested library, but the four-phase learning loop
> (ADR-001's flagship capability) was **not wired into the running binary**. Acceptance had been
> claimed on checked boxes + isolated unit tests, which did not cover the integration path.
> This is the originating case for the end-to-end runtime acceptance gate added to
> `docs/sop/ITERATION-WORKFLOW.md`.
>
> **Update 2026-06-01 (same day):** The learning loop is now wired into the **`-p` print-mode**
> execution path via `crates/talos-cli/src/evolution_runtime.rs`, and `EvolutionPanel` now
> renders. Remaining before COMPLETE: wire the same pipeline into `run_tui_mode` and
> `run_interactive_mode`, plus TUI end-to-end evidence. Status stays REVIEW until all live
> paths are covered.

## Story Status

| Story | Library | Runtime-integrated | Notes |
|-------|---------|--------------------|-------|
| S1: Evolution crate + data models | ✅ | n/a | Types + SQLite schema present |
| S2: TurnObserver — signal capture | ✅ | ⚠️ | Wired in `run_print_mode` (objective `Error` signals + correction heuristic); `run_tui_mode` / `run_interactive_mode` pending (R1) |
| S3: PatternExtractor — extraction | ✅ | ✅ | Invoked by `evolution_runtime::ingest` on the print path |
| S4: KnowledgeStore — SQLite persistence | ✅ | ✅ | `evolution_runtime` writes observations + accumulates patterns at runtime (print path) |
| S5: BehaviorAdapter — prompt injection | ✅ | ⚠️ | Injected in `run_print_mode`; TUI/interactive paths pending (R2) |
| S6: TUI evolution panel (Ctrl+E) | ✅ | ⚠️ | `render()` now draws the panel (R3 ✅); feeding live patterns into the panel during a TUI session is pending |
| S7: `--learned` command | ✅ | ✅ | Shows real patterns once a session has written observations |

## Residual Work (registered, not deferred silently)

Remaining items before I008 can be claimed COMPLETE. Tracked as a follow-up slice (see backlog):

- **R1** — Invoke `TurnObserver` in the real turn loop. **Print path ✅** (`evolution_runtime.rs`
  observes `AgentEvent::Error` + user-correction heuristic and writes to `KnowledgeStore`).
  **Remaining:** `run_tui_mode` and `run_interactive_mode` (event_loop) paths.
- **R2** — Call `BehaviorAdapter` during system-prompt assembly. **Print path ✅** (combined with
  `--append-system-prompt`, injected before the agent runs). **Remaining:** TUI/interactive injection.
- **R3** — Render `EvolutionPanel` in `talos-tui::render()` so `Ctrl+E` shows data. **✅ DONE.**
- **R4** — End-to-end evidence (real turn → observation persisted → `--learned` shows it →
  next run injects it). **Print path ✅** (mock smoke test below). **Remaining:** TUI evidence (needs a TTY).

## Verification

> The library + unit tests pass. The print-path RUNTIME checks below now **PASS**; the TUI
> RUNTIME check remains outstanding per the end-to-end acceptance gate.

```bash
# Library + unit tests (PASS)
cargo build --release -p talos-cli
cargo test --workspace
cargo test -p talos-evolution
cargo test -p talos-cli            # includes evolution_runtime accumulation tests

# RUNTIME — print path (PASS):
# A user-correction marker is observed, persisted, and surfaced by --learned.
HOME=$(mktemp -d) sh -c '
  talos --mock -p "don'\''t use unwrap in library code"
  talos --learned                  # shows the extracted preference pattern
'

# RUNTIME — TUI (STILL OUTSTANDING — R1/R2 for the TUI path):
./target/release/talos --mock --tui        # Ctrl+E renders the panel (R3); live pattern feed pending
```
