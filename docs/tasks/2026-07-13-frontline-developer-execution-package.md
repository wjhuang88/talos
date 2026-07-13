# Frontline Developer Execution Package: Four-Month Reliability

**Status**: Long-task owner; ready for assignment; not started
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
cargo clippy --workspace --all-targets --locked -- -D warnings
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
| 2026-07-13 | F100 complete | `feature/i120-dynamic-diagnostics` (pre-commit) | Complete | 12 diagnostics tests pass; fmt/check/clippy(preflight)/tests/governance/diff-check all pass | `diagnostics.rs`, `governance.rs`, `Cargo.toml`, `Cargo.lock`, I120 iteration doc | pre-existing `--all-targets` clippy violations in other crates | Begin F101: replace manual JSON output with `serde_json::to_string_pretty`. |
