# Frontline Developer Execution Package: Four-Month Reliability

**Status**: Long-task in progress; I120/I121 Complete; I122 ready for activation
**Program plan**: `docs/tasks/2026-07-13-four-month-frontline-reliability-plan.md`
**Ordered iterations**: I120, I121, I122, I123
**Checkpoint owner**: assigned developer

## Outcome

Execute the four iterations in order and their F-ID stories in order. At any interruption, another
developer must be able to resume from this
file without chat history and without guessing product, security, or release decisions.

Hierarchy: this file owns the one long-running task; I120-I123 are its sequential monthly
iterations; F100-F133 are stories inside those iterations. Only iterations have an Active state.

## Gate 0: Start Here

1. Read `AGENTS.md`, this file, the owner plan, `docs/BOARD.md`, the active iteration candidate,
   and every owner story selected by it.
2. Run:

```bash
git status -sb
rustc --version
cargo metadata --locked --no-deps --format-version 1
scripts/validate_project_governance.sh .
./scripts/release_preflight.sh
```

3. Expected Rust is pinned by `rust-toolchain.toml`; `Cargo.lock` must exist. Never delete it to
   fix `--locked`.
4. Start from updated `main` on `feature/i120-dynamic-diagnostics`. If the worktree is dirty and
   the changes are not yours, stop; do not stash, reset, or overwrite them.
5. Confirm no other iteration is Active. Change I120 to Active only in the same commit that records
   the inventory disposition and Gate-0 evidence.

## Work Rules

- One story ID per logical commit. Commit format:
  `type(scope): description (#Fxxx) [model:<model-name>]`.
- Before every commit: review `git diff --cached`, scan the staged diff for secrets, run targeted
  tests, and run `git diff --check`.
- Use fixture providers, disposable HOME directories, and loopback-only dashboard tests. Never use
  real credentials in tests, screenshots, logs, or docs.
- Do not use `unwrap()` in library code. Native/process/SQLite/tree-sitter boundaries must return a
  safe error or fallback according to `AGENTS.md`.
- Update the selected owner story first, then iteration record, iteration index, backlog, Board,
  README/user docs, and originating GitHub issue if its status changes.

## File And Test Map

| Iteration | Primary files | Required focused evidence |
|---|---|---|
| I120 | `crates/talos-cli/src/diagnostics.rs`, `governance.rs`, CLI integration tests | valid JSON parse/escaping; clean/missing/malformed governance; redaction; real diagnostics command |
| I121 | `crates/talos-tui/src/widgets.rs`, `app.rs`, `scrollback.rs`, TUI tests | 80/narrow buffers; unchanged approval decisions; title parser; default export exclusion; native terminal |
| I122 | existing plugin/hook/MCP registries, command handlers, `dashboard_helpers.rs`, dashboard tests | typed snapshot; collision/trap/missing fixture; auth/redaction/no mutation routes; real command smoke |
| I123 | `scripts/install.sh`, `scripts/install.ps1`, installer fixtures, `scripts/talos_smoke.sh`, install/trial docs | fixture matrix; disposable HOME; no-network mock; second-operator replay |

Do not assume this map authorizes refactoring adjacent modules. If a change crosses a crate boundary
or public API, record the exact need and request maintainer review before coding it.

## Standard Validation Ladder

Run the smallest relevant test while editing, then at story close:

```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
./scripts/release_preflight.sh
scripts/validate_project_governance.sh .
git diff --check
```

If a command fails, record command, exit code, first actionable error, attempted fix, and next safe
step. Retry unchanged commands at most twice. Network, registry, GitHub, or platform absence is an
environment block, not permission to weaken assertions or remove `--locked`.

## Checkpoint Template

Append one row after every story or blocking discovery:

| Date/time | Story | Branch/commit | State | Validation | Changed files | Failure/retry | Next exact action |
|---|---|---|---|---|---|---|---|

States are `Not Started`, `In Progress`, `Review`, `Complete`, or `Blocked`. `Blocked` requires the
missing authority/environment/decision and the safe fallback. Never mark a story Complete from docs
status alone.

## Explicit Stop Conditions

Stop before changing permission Allow/Ask/Deny outcomes; adding remote access or a write route;
persisting new reasoning data; modifying session encoding; adding plugin host calls or executable
hooks; adding a native dependency; changing a public semver-bound API; using real credentials;
tagging, publishing, deploying, or pushing to main; destructive Git or filesystem cleanup.

## Safe Cleanup And Handoff

Temporary HOME/fixture directories may be removed only after their exact path is recorded and
evidence copied into the repo. Branch deletion and worktree removal require maintainer confirmation.
At each monthly handoff, record residuals in their owner docs, leave a clean worktree, and name the
next iteration without activating it.

## Checkpoints

| Date/time | Story | Branch/commit | State | Validation | Changed files | Failure/retry | Next exact action |
|---|---|---|---|---|---|---|---|
| 2026-07-13 | Planning handoff | `main` | Complete | `cargo check --workspace --locked`; governance 0 warnings; `git diff --check` | plan/iteration/governance docs | none | Assignee reruns full Gate 0 from updated main and activates I120 only after all checks pass. |
| 2026-07-13 | Gate 0 / I120 activation | `feature/i120-dynamic-diagnostics` @ `ac869fc` | In Progress | rustc 1.97.0; cargo metadata exit 0; governance 0 warnings; release_preflight passed; `git diff --check` clean | `docs/iterations/I120-*.md`, `docs/iterations/README.md`, execution package | none | Begin F100: owner-state fixture and dynamic diagnostics contract. |
| 2026-07-13 | F100 complete | `feature/i120-dynamic-diagnostics` (pre-commit) | Complete | 12 diagnostics tests pass; fmt/check/clippy(preflight)/tests/governance/diff-check all pass | `diagnostics.rs`, `governance.rs`, `Cargo.toml`, `Cargo.lock`, I120 iteration doc | none under the repository-standard Clippy scope | Begin F101: replace manual JSON output with `serde_json::to_string_pretty`. |
| 2026-07-13 | F101 complete | `feature/i120-dynamic-diagnostics` (pre-commit) | Complete | 7 CLI integration tests pass; release_preflight passed; governance 0 warnings | `diagnostics.rs`, `tests/diagnostics_e2e.rs`, I120 iteration doc | none | Begin F102: dynamic residual gates with safe fallback. |
| 2026-07-13 | F102 complete | `feature/i120-dynamic-diagnostics` (pre-commit) | Complete | 18 unit + 7 integration tests pass; release_preflight passed; governance 0 warnings | `diagnostics.rs`, I120 iteration doc | none | Begin F103: docs and smoke closeout. |
| 2026-07-13 | F103 / I120 closeout | `feature/i120-dynamic-diagnostics` (pre-commit) | Complete | Real binary smoke passed (JSON + text); README updated; full validation ladder pass | `diagnostics.rs`, `README.md`, I120 doc, `iterations/README.md`, `BOARD.md` | none | I120 is Complete. Next: maintainer merges feature branch; I121 activation requires fresh Gate 0. |
| 2026-07-13 | I121 Gate 0 / F110-F111 | `feature/i121-tui-attention-thinking-clarity` (pre-commit) | In Progress | 14 new approval tests pass; release_preflight passed | `scrollback.rs`, `inline_terminal.rs`, `tests.rs`, I121 doc | none | Begin F112: standalone bold thinking title extraction. |
| 2026-07-13 | F112 complete | `feature/i121-tui-attention-thinking-clarity` (pre-commit) | In Progress | 14 thinking-title tests pass; release_preflight passed | `scrollback.rs`, `app.rs`, `tests.rs`, I121 doc | none | Begin F113: native-terminal acceptance and docs. |
| 2026-07-13 | Review fixes BLK1-3 | `feature/i121-tui-attention-thinking-clarity` @ `f4a2803` | Review | BLK1: malformed→unavailable; BLK2: typed ResidualGate; BLK3: CJK display width. 18 diagnostics + 294 TUI tests pass; release preflight passed. | `diagnostics.rs`, `scrollback.rs`, `tests.rs` | Native Alacritty walkthrough remains; export test must exercise the real transcript/export path. | Maintainer removed `--all-targets` from this task's Clippy gate. I120 may close after standard gates; I121 remains Review until terminal and export evidence pass. |
| 2026-07-13 | Validation-gate decision | `feature/i121-tui-attention-thinking-clarity` | Complete | Maintainer explicitly selected repository-standard `cargo clippy --workspace --locked -- -D warnings`; `release_preflight.sh` remains authoritative. | execution package, I120/I121, Board/index | Historical test-target lint debt is outside this long task and no longer a closeout gate. | Close I120 after standard validation; retain I121 Review for its two behavior-evidence residuals. |
| 2026-07-13 | Export regression closure | `feature/i121-tui-attention-thinking-clarity` | Complete | `cargo test -p talos-session transcript --locked`: 18/18; default export excludes reasoning/title/sensitive fixture; explicit include-thinking remains intact. Diagnostics 18/18; TUI 293/293; standard Clippy passes. | `talos-session/src/transcript.rs`, diagnostics/TUI tests | Replaced the false title-only “export” test with real transcript service coverage. | Complete native Alacritty walkthrough. |
| 2026-07-13 | Native Alacritty automation attempt | `feature/i121-tui-attention-thinking-clarity` | Blocked | Alacritty app present; isolated HOME `/tmp/talos-i121-xYNB7Y`; mock/no-init/no-context; native 80×24 TUI startup observed through screen-backed session. | `/tmp/talos-i121-approval-full.txt` hardcopy; no secrets | macOS denied `osascript` keystrokes and `screencapture`; mock mode cannot issue a real tool call, so no approval/thinking result is claimed. | Human opens Alacritty with a configured real provider and asks it to create a harmless `/tmp` file, then records approval/thinking observations. |
| 2026-07-13 | Native `/export` check | `feature/i121-tui-attention-thinking-clarity` | Complete (semantic check only) | `/export /tmp/talos-i121-check.md` displayed the expected interactive-approval denial and did not write the file. | Maintainer screenshot; no secrets | `/export` is an I014 slash-command wrapper that rejects `Ask`; it has no `ToolApprovalRequest` response channel and therefore cannot validate the I121 panel. | Retain the result as unchanged-permission evidence. For panel acceptance, use a real provider-generated `write`/`bash` tool call; do not use `--mock` or `/export`. |
| 2026-07-13 | F113 / I121 closeout | `feature/i121-tui-attention-thinking-clarity` | Complete | Maintainer-supplied native Alacritty screenshot captures a real `glm-5.2 (alibaba-cn)` turn with provider reasoning, tool output, and a pending `bash` approval panel whose warning, summary, choices, and navigation help are visible. Automated semantic tests cover keyboard routing, 40/60/80/120 widths, title extraction, and export boundaries. | I121 iteration doc, Board/index, screenshot evidence; no secrets | A still image is visual evidence, not standalone proof of decision logic; closeout relies on the combined evidence packet. | I121 closed. Inventory state and run fresh Gate 0 before activating I122; do not activate it implicitly. |
