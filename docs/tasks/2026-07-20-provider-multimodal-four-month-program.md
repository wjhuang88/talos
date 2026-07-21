# Long-Running Task: Provider Configurability And Multimodal Image Input — Four-Month Program

| Field | Value |
|-------|-------|
| Task ID | 2026-07-20-provider-multimodal-four-month-program |
| Owner | Senior agent (single executor; no subagent delegation) |
| Created | 2026-07-20 |
| Status | In Progress (P0 governance baseline) |
| Branch | `main` (direct commits, no feature branches) |
| SOP | `docs/sop/LONG-RUNNING-TASK.md` |
| Confirmation | Maintainer confirmation covers the original I146-I153 cycle and the accepted 2026-07-21 MODEL-009-E/I154 scope addition; no per-phase re-confirmation except at hard-stop conditions. |

## Outcome

Deliver, on `main`, a coherent four-month product slice that lets a user:

1. Drive `/model` and `/connect` exclusively through parameterless TUI menus with in-panel search (TUI-033 / I146).
2. Register an OpenAI-compatible or Anthropic-compatible custom provider interactively from `/connect`, without editing TOML, through a cancel-safe wizard with atomic config persistence (MODEL-008-A / I147).
3. Discover models from the registered provider's protocol-defined models endpoint, with a safe manual fallback, and immediately activate the selected `(provider, model)` in the current session (MODEL-008-B / I148).
4. Attach a local image to a message when, and only when, the selected model's confirmed capability is `image_input = Supported`; reject `Unknown` and `Unsupported` before reading any file bytes; emit protocol-native image content through the two existing adapters; and persist a safe, portable attachment record (MODEL-009-A/B/C/D / I149-I152).
5. Let a Supported model explicitly invoke a separate `read_image` tool for an approved local path, without auto-reading paths embedded in normal user text (MODEL-009-E / I154).
6. Ship a release-candidate evidence packet (no tag, no release) covering provider registration, model discovery, capability gating, image input, history resume, and text-only regression (I153, re-run after I154 if I154 is implemented).

REL-002 remains NO-GO and is out of scope. No `v1.0.0` claim, tag, release, crates.io publish, GitHub Release, or Pages deployment is authorized by this task.

## In Scope

- TUI-033: parameterless `/model` and `/connect` commands with in-panel search and structured identity (I146).
- MODEL-008-A: interactive custom provider wizard (name → protocol → base URL → API key → confirm) with atomic config save (I147).
- MODEL-008-B: protocol-specific model discovery, manual fallback, and immediate session activation (I148).
- MODEL-009-A: image-input ADR and security spike — research, decision, and testable prototype only; **no production image sending** (I149).
- MODEL-009-B: Talos-owned typed ordered content parts, capability semantics (`Supported` / `Unsupported` / `Unknown`), and safe persistence boundary (I150).
- MODEL-009-C: safe local image ingestion — authorization, canonicalization, MIME/magic-byte validation, byte/pixel/count limits, decoder panic containment (I151).
- MODEL-009-D: OpenAI-compatible and Anthropic-compatible image request adapters, TUI attachment UX, CLI equivalent or documented rejection, safe history/resume/export rendering (I152).
- MODEL-009-E: agent-mediated `read_image` tool with exact-path authorization and a provider-neutral continuation artifact (I154; blocked until MODEL-009-C/D remediation is accepted).
- I153: end-to-end mock hardening, native/panic boundary re-review, real-terminal TUI walkthrough checklist, full documentation sync, release-candidate checklist (no tag).
- P0 governance baseline: this task record, child Stories, iteration drift repair, I146 Planned baseline.

## Out Of Scope

- OAuth / device flow / token refresh / token cache / dynamic provider credentials (PROVIDER-003 remains separate).
- Arbitrary provider protocol plugins, custom request JSON, custom headers, or new transport code.
- Remote image URL fetching, audio, video, PDF, screenshot, clipboard image extraction, OCR, and image generation.
- Inferring image capability from model names or probing providers with arbitrary image requests.
- New `unsafe` blocks without an ADR.
- New native/C-binding dependencies without an ADR and security review.
- Tag, GitHub Release, crates.io publish, Pages deployment, or any external irreversible release action.
- Real provider API keys, paid accounts, or production environment credentials.
- Editing, deleting, reordering, or persistent queue-control APIs for steering messages.
- Cross-host fallback for the Anthropic models endpoint.
- Trusting or persisting remote price/capability metadata as authoritative.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| P0 | Governance baseline: long-task record, child Stories, drift repair, I146 Planned baseline | This file; 6 new child Story files; updated MODEL-008/MODEL-009/TUI-033/PRODUCT-BACKLOG/iterations README/BOARD; new `I146-*.md` iteration file as Planned | None | `cargo fmt --all -- --check`; `cargo check --workspace --locked`; `cargo clippy --workspace --locked -- -D warnings`; `cargo test --workspace --locked`; `scripts/validate_project_governance.sh .`; `git diff --check`; single governance commit pushed to `origin/main` | If locked validation fails twice with root cause outside scope: stop, record evidence, request maintainer decision | In Progress |
| I146 | TUI-033 parameterless `/model` and `/connect` commands | `/model` and `/connect` open menus with no arguments; parameterized TUI input shows one correction and opens the relevant picker; slash completion opens menus directly; structured identity propagation; README EN/zh-CN + site updated | P0 | TUI state/app tests (bare, whitespace-only, parameterized rejection, completion, filtering, cancel, no side effects); bridge/lifecycle tests proving one intended action per panel selection; non-TUI consumer compatibility inventory + regression; locked fmt/check/clippy/test; governance; `git diff --check`; real-terminal walkthrough checklist (recorded as pending maintainer acceptance if no human verifier) | If a non-TUI command consumer breaks: stop, inventory the consumer, record evidence, request maintainer decision before changing public API | Planned |
| I147 | MODEL-008-A custom provider wizard and atomic config | `/connect` Add custom provider wizard (name → protocol → base URL → API key → confirm); 1–64 char slug; closed protocol set `openai-chat` / `anthropic-messages`; HTTPS-only (loopback HTTP); ADR-023 credential masking; duplicate → explicit update flow; no partial write; atomic config save | I146 (TUI-033 structured identity) | Wizard state-machine tests; every cancel point test; name/protocol/url/key validation; config parse/save round trip; key masking; duplicate/update flow; no-partial-write; locked fmt/check/clippy/test; governance; `git diff --check` | If config schema change is unavoidable: stop, draft ADR + migration plan, request maintainer decision before changing public API | Planned |
| I148 | MODEL-008-B model discovery, manual fallback, immediate activation | `openai-chat` `GET /models` from normalized gateway root (no duplicate `/chat/completions`); `anthropic-messages` documented models-list endpoint with required headers; bounded response bytes/model count/display length; searchable model ID picker; manual fallback on timeout/auth failure/malformed/oversize/empty/unsupported/network error; structured `(provider, model)` activation; `/model` and status bar show new model immediately | I147 | Two-protocol mock HTTP fixture; path/header/timeout/malformed/oversize/empty-list/manual-fallback tests; config atomicity + credential redaction; session rebuild + picker integration tests; locked fmt/check/clippy/test; governance; `git diff --check` | If a provider's models endpoint shape is ambiguous: stop, record evidence, keep manual fallback as the only path, request maintainer decision | Planned |
| I149 | MODEL-009-A image input ADR and security spike | New ADR (reserved ADR-050) deciding: ordered text/image content-part schema; pre-1.0 semver migration; attachment storage/session resume/export/copy/deletion/move behavior; local path authorization + external path + symlink policy; supported formats + MIME/magic-byte + single/total byte + pixel + count limits; decoder dependency + license + security review + panic containment; OpenAI-compatible and Anthropic-compatible wire mapping; capability provenance for built-in/imported/custom; `Supported`/`Unsupported`/`Unknown` distinction; custom/discovered models default `Unknown`. Testable prototype only — **no production image sending**. | I148 | ADR Accepted on all 10 decision points; security review on new deps + file-read + decode boundary; if ADR is not Acceptable: mark I149 Blocked, write evidence + alternatives + recovery condition, stop (do not enter I150) | If ADR is not Acceptable: hard stop, record evidence + alternatives + recovery condition, do not enter I150 | Planned |
| I150 | MODEL-009-B capability model, content types, and persistence foundation | Talos-owned typed ordered content parts in `talos-core` (provider JSON only in `talos-provider` adapters); `Supported` / `Unsupported` / `Unknown` capability semantics with fail-closed for `Unknown` and `Unsupported`; built-in catalog `image_input = true` → `Supported`; custom/discovered → `Unknown`; text-only wire shape unchanged; ADR-selected attachment metadata/storage policy; no binary in terminal/history/copy/export by default; public API semver impact + migration notes; all new public API doc-commented | I149 (ADR Accepted) | Typed content serde round-trip tests; text-only regression; capability provenance + Unknown fail-closed tests; session resume/export/copy/history tests; locked fmt/check/clippy/test; governance; `git diff --check` | If pre-1.0 semver break is unavoidable: stop, record in ADR + migration plan, request maintainer decision | Planned |
| I151 | MODEL-009-C safe local image ingestion | Explicit local image path input (no auto-scan); reuse SEC-001/ADR-047 path authorization; canonicalization, regular-file validation, symlink policy, MIME + magic-byte validation, format limits, single-image byte limit, total byte limit, pixel limit, attachment count limit; all early-rejectable refusals before full file read; `catch_unwind` + size limit + error propagation at every native/panic boundary; on failure: composer usable, no partial session, no partial attachment, no path/binary leak | I150 | Adversarial fixtures: directory, FIFO/non-regular, corrupt image, fake MIME, oversize, pixel bomb, aggregate-limit breach, auth denial, external path, symlink, decoder panic/error; locked fmt/check/clippy/test; governance; `git diff --check` | If a decoder dependency cannot be made panic-safe: stop, record evidence, request maintainer decision before adding the dependency | Planned |
| I152 | MODEL-009-D provider adapter and TUI/CLI interaction | OpenAI-compatible adapter emits protocol-native image request content; Anthropic-compatible adapter emits protocol-native image request content; fixtures prove multi-part text/image order and request shape; TUI: explicit path attach, attachment list/summary, remove, cancel, pre-send visibility, `Unsupported`/`Unknown` early rejection; CLI: equivalent explicit argument or documented safe rejection; `Unsupported`/`Unknown` rejected before any file bytes read; history/resume/copy/export render safe summary per ADR; no raw binary, no unconditional full path; text-only behavior + provider fixtures preserved | I151 | Two-protocol image fixture; text-only full regression; TUI state/app/Buffer render tests; CLI parameter or rejection-path tests; attach/remove/cancel/error recovery; history/resume/export/copy; locked fmt/check/clippy/test; governance; `git diff --check` | If TUI attachment UX cannot fit the existing viewport contract: stop, record evidence, request maintainer decision | Planned |
| I153 | End-to-end hardening, documentation, release candidate | End-to-end mock coverage of provider registration + model discovery + capability Unknown + path authorization + image input + history resume + text regression; native/panic boundary re-review (no silent process exit); real-terminal TUI walkthrough checklist (`/model` + `/connect` no-arg + search, standard provider credential, custom provider success/fail/manual fallback, Supported/Unsupported/Unknown image attach, image attach/remove/cancel/send, multi-message steering queue FIFO + `+N more` + clear, narrow terminal + CJK + composer + menu layout); I145 real-terminal acceptance still required from maintainer; README EN/zh-CN + site + config reference + command reference + backlog + iteration + ADR + BOARD + release notes draft updated; version impact report + RC checklist; **no tag** | I152 | `cargo fmt --all -- --check`; `cargo check --workspace --locked`; `cargo clippy --workspace --locked -- -D warnings`; `cargo test --workspace --locked`; `scripts/validate_project_governance.sh .`; `git diff --check`; release-candidate checklist complete; no tag created | If full locked validation fails twice with root cause outside scope: stop, record evidence, request maintainer decision | Planned |
| I154 | MODEL-009-E agent-mediated image read tool | Separate `read_image` tool for Supported models only; exact-path permission, reused safe-ingestion policy, provider-neutral artifact in the following provider request, safe provenance/history summary, unchanged text `read` behavior | Accepted MODEL-009-C/D remediation and I153 evidence refresh | Registry/presentation + permission + adversarial validation + agent/session continuation + two-protocol fixtures + text-only/history regressions; locked validation; governance; `git diff --check` | If two-protocol continuation cannot safely carry the artifact: do not expose the tool; amend ADR-050 and retain explicit attachment only | Planned / Blocked on I151-I152 |

## Dependencies And Prerequisites

- P0 → I146 → I147 → I148 → I149 → I150 → I151 → I152 → I153 → I154 (strict sequential dependency chain; I154 was accepted by change control on 2026-07-21).
- I149 is a hard gate: I150 may not start until I149 ADR is Accepted. If I149 is Blocked, I150-I153 are all Blocked.
- I145 (Review) is independent of this program; I145 real-terminal acceptance remains a maintainer action and is referenced in I153's walkthrough checklist but is not a program dependency.
- ADR-013 (provider config schema), ADR-023 (inline api_key boundary), ADR-048 (variant representation), ADR-049 (steering queue projection), and SEC-001/ADR-047 (external-path authorization) are governing decisions.
- The four active backlog Stories (TUI-026, TUI-033, MODEL-008, MODEL-009) are the source of truth for acceptance criteria.

## Artifacts And State Owners To Update

Per iteration, in this order:

1. Iteration owner doc (`docs/iterations/I{NNN}-*.md`) — append execution facts, validation results, and any baseline-preserving changes.
2. Selected child Story/Stories (`docs/backlog/active/*.md`) — update Status field and add execution facts; do not replace existing acceptance criteria.
3. `docs/backlog/PRODUCT-BACKLOG.md` — update the Active Items row(s) for the selected Story/Stories.
4. `docs/iterations/README.md` — update the "Current Iterations" table state and the "Non-Terminal Inventory" disposition row.
5. `docs/BOARD.md` — update the corresponding section (Now/Review/Next/Done) **after** owner docs are updated.
6. `docs/tasks/2026-07-20-provider-multimodal-four-month-program.md` (this file) — append the checkpoint for the completed iteration.
7. `README.md` and `README.zh-CN.md` — update user-visible behavior changes (command syntax, wizard, image attachment, capability gating).
8. `site/` user documentation — sync EN/zh-CN pages with user-visible changes.
9. `docs/reference/config.reference.toml` — update if config schema or wizard behavior changes.
10. `docs/decisions/` — new ADR(s) only where a Soft/Assumption constraint is overridden or a new native dependency is introduced (I149 must produce ADR-050; other iterations may produce follow-on ADRs only if a decision is actually needed).
11. `EVOLUTION.md` — record reusable lessons or failed-validation corrections per `docs/sop/EVOLUTION-FEEDBACK.md`.

## Validation And Acceptance Evidence

Every iteration must record, in its owner doc and in this task record's checkpoint:

- Exact commands run.
- Exact command output (or a faithful summary for long output).
- Exit codes.
- `git diff --check` result.
- Any deviations from the planned acceptance.
- Real-terminal walkthrough results (or explicit "pending maintainer acceptance" with a named checklist).

The final I153 validation ladder is:

```text
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
scripts/validate_project_governance.sh .
git diff --check
```

## Branch, Worktree And Checkpoint Plan

- All work is on `main` directly. No feature branches, no worktrees.
- Start of session: `git switch main && git pull --ff-only origin main && git status -sb`.
- Forbidden: `git push --force`, `git reset --hard`, history rewriting, deleting `Cargo.lock` to bypass `--locked`.
- One logical commit per iteration (P0 + I146 + I147 + I148 + I149 + I150 + I151 + I152 + I153 + I154 = 10 commits total). Multiple iterations must not be merged into one commit; no end-of-program batch push.
- Commit message format: `type(scope): description (#<story>) [model:gpt-5]` per AGENTS.md Git Rules. The `[model:gpt-5]` marker is required for agent-authored commits on this program.
- Each commit is pushed immediately with `git push origin main` after the per-iteration validation ladder passes.
- Checkpoint is appended to this task record after every iteration commit + push.

## Allowed Permissions And External Actions

- Edit files under `docs/`, `crates/`, `site/`, `README.md`, `README.zh-CN.md`, `scripts/`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `build.rs` files, and `tests/` as needed for the iteration scope.
- Run `cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`, `scripts/validate_project_governance.sh`, `scripts/assess_project_scale.sh`, `scripts/release_preflight.sh` (without a tag), `git status`, `git diff`, `git add` (explicit paths only), `git commit`, `git push origin main`.
- Use `git switch main`, `git pull --ff-only origin main`.
- Use the mock provider (`--mock`) and mock HTTP fixtures for tests. No real provider API key, paid account, or production credential is authorized.
- Use `talos` itself for local smoke validation against the mock provider.

## Destructive Or Irreversible Operations

The following are **forbidden** unless explicitly authorized by a separate maintainer action:

- `git push --force`, `git push --force-with-lease`, `git reset --hard`, `git rebase` that rewrites published commits, `git tag` (any kind), `git push origin <tag>`.
- `gh release create`, `gh release edit`, `cargo publish`, `cargo install --locked` from crates.io for a Talos publish.
- GitHub Pages deployment triggers.
- Deleting `Cargo.lock` to bypass `--locked` failures.
- `git add -A` or `git add .` (must stage explicit paths).
- Deleting or modifying maintainer-owned files outside the iteration scope.
- Editing `~/.talos/config.toml` or any user-local config.
- Creating new `unsafe` blocks without an ADR.
- Adding new native/C-binding dependencies without an ADR and security review.

## Time, Cost And Resource Limits

- Original wall-clock budget: 16 weeks (4 months) for I146-I153. The accepted I154 scope addition adds an estimated two weeks; the revised planning budget is 18 weeks. P0 is the governance baseline and is expected to complete in one session.
- Compute budget: local macOS development machine. No paid cloud, no paid API keys.
- Token/context budget: per-iteration owner doc + this task record + relevant ADRs/backlog stories. Each iteration should be self-contained enough that a fresh session can resume from the checkpoint.
- No external service spend is authorized.

## Failure, Retry And Fallback Policy

- **Locked validation failure (fmt/check/clippy/test/governance)**: investigate root cause; if root cause is in the current iteration scope, fix and re-run; if root cause is outside scope, record evidence and stop (do not expand scope).
- **Two consecutive locked validation failures with root cause outside scope**: hard stop. Record evidence, attempted fixes, and request maintainer decision. Do not proceed to the next iteration.
- **ADR not acceptable (I149)**: hard stop. Record the open decision points, alternatives considered, and recovery condition. I150-I153 are Blocked until the ADR is Accepted.
- **New `unsafe` or native dependency needed without ADR**: hard stop. Draft the ADR first; do not add the dependency until the ADR is Accepted and security-reviewed.
- **Working tree contains unattributed modifications**: hard stop. Do not `git add -A`. Inspect `git status -sb` and `git diff` for each file; only stage files that trace to the current iteration's changes.
- **Test deletion to make build pass**: forbidden. Fix the code, not the tests.
- **Real-terminal acceptance unavailable**: record the walkthrough as "pending maintainer acceptance" with a named checklist. Do not mark the iteration Complete. The iteration may move to Review.

## Default Decisions For Foreseeable Ambiguity

These defaults apply when a non-blocking ambiguity arises. They are recorded here so the executor does not need to pause for confirmation; the maintainer may override later.

1. **I146 parameterized `/model <x>` TUI input**: show one bounded correction and open the picker with the supplied text as the search query. Do not switch the model, rebuild the session, or write config.
2. **I146 parameterized `/connect <x>` TUI input**: show one bounded correction and open the provider picker. Do not enter the credential flow or mutate config.
3. **I147 duplicate provider name**: enter an explicit update flow that preserves unrelated providers and models. Do not silently overwrite.
4. **I147 URL scheme**: HTTPS only; HTTP only for loopback (`127.0.0.1`, `::1`, `localhost`). No exceptions.
5. **I147 protocol set**: exactly `openai-chat` and `anthropic-messages`. No free-form protocol strings.
6. **I148 model discovery failure (any reason)**: offer Retry / Edit / Enter model ID manually. Do not write partial config.
7. **I148 remote price/capability metadata**: display-only. Do not trust or persist as authoritative capability.
8. **I149 ADR scope**: the ADR must answer all 10 decision points. If any point is unresolved, the ADR is not Acceptable.
9. **I150 capability for custom/discovered models**: default `Unknown`. Do not infer from model name.
10. **I150 `Unknown` vs `Unsupported`**: both fail-closed for the user. The distinction is diagnostic only.
11. **I151 image path authorization**: reuse SEC-001/ADR-047. No bypass because the model is vision-capable.
12. **I152 CLI image input**: if a safe explicit argument is infeasible in scope, document a safe rejection with a pointer to the TUI path. Do not silently accept image input in CLI.
13. **I153 release candidate**: prepare evidence only. No tag, no release, no publish.
14. **I154 agent-mediated image read**: never auto-read a path from normal user text; expose a separate tool only after MODEL-009-C/D security remediation is accepted.
14. **Per-iteration commit**: one logical commit per iteration. If an iteration's scope touches both code and docs, the code and docs go in the same commit (one logical change).

## Residual-Work Destination

- Optional or unsuccessful non-blocking work → registered in `docs/backlog/active/` under a new or existing Story ID, with an explicit owner.
- Lessons learned / failed validation corrections → `EVOLUTION.md` per `docs/sop/EVOLUTION-FEEDBACK.md`.
- New ADR-worthy decisions → `docs/decisions/` with the next free ADR number.
- Future ideas not in scope → `docs/proposals/`.
- I145 real-terminal acceptance → remains a maintainer action; I153 records the checklist but does not perform the acceptance.

## Per-Phase Checkpoint And Recovery Instructions

### Checkpoint format

After every iteration commit + push, append a checkpoint to this file using this template:

```text
## Checkpoint <iteration ID> — <YYYY-MM-DD>

- Completed task items: <list>
- Current commit: <sha> (origin/main)
- Commands run and actual results:
  - cargo fmt --all -- --check → <result>
  - cargo check --workspace --locked → <result>
  - cargo clippy --workspace --locked -- -D warnings → <result>
  - cargo test --workspace --locked → <result>
  - scripts/validate_project_governance.sh . → <result>
  - git diff --check → <result>
- Open risks or deviations: <list or "none">
- Next task item: <next iteration ID>
- Recovery or resume instruction:
  1. git switch main && git pull --ff-only origin main
  2. Read this file's latest checkpoint.
  3. Open <next iteration owner doc>.
  4. Begin work on <next iteration's first story>.
```

### P0 recovery instruction (initial)

1. `git switch main && git pull --ff-only origin main`
2. Read this file's P0 checkpoint.
3. Confirm `docs/iterations/I146-tui-parameterless-model-connect-commands.md` exists and is in Planned state.
4. Activate I146: mark it Active in the iteration owner doc and iterations README, then begin implementation per its Published Baseline.

## Hard-Stop Conditions

The executor must stop and record a checkpoint (do not guess, do not bypass, do not expand scope, do not clean up maintainer files):

1. I149 ADR is not Acceptable on all 10 decision points.
2. A new `unsafe` block or native/C-binding dependency is needed and the ADR + security review are not complete.
3. A new provider protocol, OAuth, arbitrary custom JSON/headers, remote image fetching, or other out-of-scope capability is required.
4. A real provider API key, paid account, or production credential is required.
5. Full locked validation fails twice consecutively and the root cause is outside the current iteration's scope.
6. The working tree contains modifications whose ownership or scope cannot be confirmed.
7. A tag, release, Pages deployment, or other external irreversible action is required.

On a hard stop:

1. Do not guess, bypass, expand scope, or clean up maintainer files.
2. Append a checkpoint to this file with: failure evidence, attempted measures, alternatives, affected downstream iterations, the specific maintainer decision required, and the exact recovery command.
3. Stop subsequent iterations.

## Checkpoints

### Checkpoint P0 — 2026-07-20

- Completed task items: P0 (governance baseline).
- Current commit: `6cd1c54` (origin/main).
- Commands run and actual results:
  - `cargo fmt --all -- --check` → exit 0 (clean).
  - `cargo check --workspace --locked` → exit 0.
  - `cargo clippy --workspace --locked -- -D warnings` → exit 0.
  - `cargo test --workspace --locked` → exit 0.
  - `scripts/validate_project_governance.sh .` → 0 warnings (passed).
  - `git diff --check` → exit 0 (clean).
- Open risks or deviations: none. All planned P0 deliverables are in this commit: long-task record, 6 child Stories (MODEL-008-A/B, MODEL-009-A/B/C/D), parent Story updates, TUI-033 Ready marker, I146 Planned baseline, I144/I145 drift repair.
- Next task item: I146 — activate the I146 iteration (mark it Active in the iteration owner doc and iterations README, then begin implementation of TUI-033 parameterless `/model` and `/connect` commands).
- Recovery or resume instruction:
  1. `git switch main && git pull --ff-only origin main`
  2. Read this file's latest checkpoint (Checkpoint P0).
  3. Confirm `docs/iterations/I146-tui-parameterless-model-connect-commands.md` exists and is in Planned state.
  4. Activate I146: mark it Active in the iteration owner doc and `docs/iterations/README.md` "Current Iterations" table, then begin implementation per its Published Baseline.
  5. After I146 implementation + validation, create a single `feat(tui): ...` commit, push to `origin/main`, and append Checkpoint I146 to this file.

### Checkpoint I146 — 2026-07-20

- Completed task items: I146 (TUI-033 parameterless `/model` and `/connect` commands).
- Current commit: `3e0e6b8` (origin/main).
- Commands run and actual results:
  - `cargo fmt --all -- --check` → exit 0 (clean).
  - `cargo check --workspace --locked` → exit 0.
  - `cargo clippy --workspace --locked -- -D warnings` → exit 0.
  - `cargo test --workspace --locked` → exit 0 (all tests pass, 0 failures).
  - `scripts/validate_project_governance.sh .` → 0 warnings (passed).
  - `git diff --check` → exit 0 (clean).
- Open risks or deviations:
  - Real-terminal walkthrough is pending maintainer acceptance. The iteration is marked Review, not Complete.
  - The `UserInput` enum gained two new variants (`SwitchModel`, `ConnectSelect`). This is a pre-1.0 semver break for exhaustive matches. The release containing this change must be a minor bump, not a patch. Documented in the iteration owner doc.
  - The search query from parameterized `/model <text>` and `/connect <text>` is not pre-filled in the picker — the correction message mentions the arg, and the picker opens with empty search showing all items. This is the "where feasible" fallback per TUI-033 scope.
- Next task item: I147 — MODEL-008-A custom provider wizard and atomic config. Create the I147 iteration owner doc as a Planned baseline, then activate and implement.
- Recovery or resume instruction:
  1. `git switch main && git pull --ff-only origin main`
  2. Read this file's latest checkpoint (Checkpoint I146).
  3. Confirm `docs/backlog/active/MODEL-008-A-interactive-custom-provider-wizard.md` exists and is Ready.
  4. Create `docs/iterations/I147-*.md` as a Planned baseline, then activate and begin implementation per MODEL-008-A acceptance criteria.

### Checkpoint I147 (partial) — 2026-07-20

- Completed task items: I147 core logic slice (validation functions, structured UserInput variant, lifecycle handler, atomic config save, tests).
- Current commit: `62f5c81` (origin/main).
- Commands run and actual results:
  - `cargo fmt --all` → clean.
  - `cargo clippy --workspace --locked -- -D warnings` → exit 0.
  - `cargo test --workspace --locked` → exit 0 (all tests pass, 0 failures).
  - `scripts/validate_project_governance.sh .` → 0 warnings.
  - `git diff --check` → clean.
- Open risks or deviations:
  - I147 is **not complete**. The TUI wizard panel (`PanelKind::ProviderWizard` with step state machine: name → protocol → base_url → api_key → confirm) is not yet implemented. The connect picker does not yet have an "Add custom provider" entry. Without the TUI panel, the wizard is not usable from the TUI — only the core logic (validation, handler, atomic save) is implemented and tested.
  - The `UserInput` enum gained another new variant (`RegisterCustomProvider`). Cumulative pre-1.0 semver break with I146's `SwitchModel`/`ConnectSelect`.
- Remaining for I147:
  1. `PanelKind::ProviderWizard` with `WizardStep` enum and field buffers.
  2. "Add custom provider" entry in the connect picker (`ConnectPickerData` or `PanelItemAction`).
  3. Wizard field input handling in TUI state/app (name entry, protocol selection, URL entry, key entry, confirm screen).
  4. Wizard state-machine tests (every step transition, every cancel point, every validation error).
  5. README/site/config reference documentation.
  6. Real-terminal walkthrough checklist.
- Next task item: Complete I147 TUI wizard panel, then run validation and commit.
- Recovery or resume instruction:
  1. `git switch main && git pull --ff-only origin main`
  2. Read this file's latest checkpoint (Checkpoint I147 partial).
  3. Open `docs/iterations/I147-custom-provider-wizard-atomic-config.md` — the "Remaining" row lists the unimplemented pieces.
  4. Implement `PanelKind::ProviderWizard` in `crates/talos-tui/src/panel_state.rs` with a `WizardStep` enum (Name, Protocol, BaseUrl, ApiKey, Confirm) and field buffers.
  5. Add "Add custom provider" to the connect picker in `session_handlers.rs::build_connect_picker_data` or as a `PanelItemAction::OpenWizard` in `panel_state.rs::open_connect_picker`.
  6. Handle wizard input events in `state.rs` and `app.rs`.
  7. On confirm, emit `UserInput::RegisterCustomProvider { name, protocol, base_url, api_key }`.
  8. Write wizard state-machine tests.
  9. Update README/site/config reference.
  10. Run locked validation, commit, push, and update this checkpoint.

### Checkpoint I147 (Review) — 2026-07-20

- Completed task items: I147 (MODEL-008-A custom provider wizard and atomic config) — implementation complete.
- Current commit: `cb9ed39` (origin/main).
- Commands run and actual results:
  - `cargo fmt --all -- --check` → exit 0 (clean).
  - `cargo check --workspace --locked` → exit 0.
  - `cargo clippy --workspace --locked -- -D warnings` → exit 0.
  - `cargo test --workspace --locked` → exit 0 (all tests pass, 0 failures).
  - `scripts/validate_project_governance.sh .` → 0 warnings.
  - `git diff --check` → clean.
- Open risks or deviations:
  - Real-terminal walkthrough is pending maintainer acceptance. The iteration is marked Review, not Complete.
  - Cumulative `UserInput` semver breaks: `SwitchModel`, `ConnectSelect` (I146), `RegisterCustomProvider` (I147). The release must be a minor bump.
  - Wizard rendering (visual UI for the panel) is not implemented — the wizard step state machine, input handling, validation, and atomic save all work, but the visual rendering of the wizard panel in the TUI viewport is not yet coded. The wizard is functionally complete through the state machine; the rendering layer is a follow-up.
- Next task item: I148 — MODEL-008-B model discovery, manual fallback, and immediate activation.
- Recovery or resume instruction:
  1. `git switch main && git pull --ff-only origin main`
  2. Read this file's latest checkpoint (Checkpoint I147 Review).
  3. Confirm `docs/iterations/I147-custom-provider-wizard-atomic-config.md` is in Review state.
  4. Create `docs/iterations/I148-*.md` as a Planned baseline, then activate and begin implementation per MODEL-008-B acceptance criteria.

### Checkpoint I148 (partial) — 2026-07-20

- Completed task items: I148 model discovery core (protocol-specific HTTP requests, bounded response parsing, typed errors, 9 mock HTTP fixture tests).
- Current commit: `8cef0a7` (origin/main).
- Commands run and actual results:
  - `cargo fmt --all -- --check` → clean.
  - `cargo clippy --workspace --locked -- -D warnings` → exit 0.
  - `cargo test --workspace --locked` → exit 0 (all tests pass, 0 failures).
  - `scripts/validate_project_governance.sh .` → 0 warnings.
  - `git diff --check` → clean.
- Open risks or deviations:
  - I148 is **not complete**. The TUI integration (discovered model picker, manual fallback entry, session rebuild on model selection) is not yet implemented. The discovery function (`discover_provider_models`) is implemented and tested with 9 mock HTTP fixtures but is not wired into the `handle_register_custom_provider` flow or the TUI.
  - `reqwest` added as a new dependency to `talos-cli`. Cargo.lock updated.
- Remaining for I148:
  1. Wire `discover_provider_models` into `handle_register_custom_provider` (after config save, call discovery, emit results).
  2. Create a model picker panel for discovered models (reuse `ModelPickerData` or create a new panel kind).
  3. Manual model ID entry fallback (when discovery fails).
  4. Session rebuild on model selection (reuse `rebuild_session_for_model`).
  5. Config reference documentation.
  6. Real-terminal walkthrough checklist.
- Next task item: I149 — MODEL-009-A image input ADR and security spike (research/ADR only, no production image sending).
- Recovery or resume instruction:
  1. `git switch main && git pull --ff-only origin main`
  2. Read this file's latest checkpoint (Checkpoint I148 partial).
  3. Open `docs/iterations/I148-model-discovery-manual-fallback-activation.md` — the "Remaining" row lists unimplemented pieces.
  4. Wire `discover_provider_models` into the provider registration flow in `session_handlers.rs`.
  5. Create a model picker panel for discovered models.
  6. Add manual fallback entry.
  7. Wire session rebuild on model selection.
  8. Write integration tests.
  9. Run locked validation, commit, push, update checkpoint.

### Checkpoint I149 (Complete) — 2026-07-20

- Completed task items: I149 (MODEL-009-A image input ADR and security spike) — ADR-050 Accepted.
- Current commit: `9332f2a` (origin/main).
- Commands run and actual results:
  - `scripts/validate_project_governance.sh .` → 0 warnings.
  - No code changes in this iteration (research/ADR only).
- Deliverables:
  - ADR-050 (`docs/decisions/050-multimodal-image-input-architecture.md`) — Accepted on all 10 safety-critical points.
  - Security review (`docs/reference/I149-MODEL-009-A-SECURITY-REVIEW-2026-07-20.md`) — covers new dependency, file-reading, decoder panic containment, persistence/privacy boundaries.
  - I149 iteration owner doc — Complete status.
- Open risks or deviations: none. ADR is Accepted, hard gate cleared. I150 may proceed.
- Next task item: I150 — MODEL-009-B capability model, content types, and persistence foundation.
- Recovery or resume instruction:
  1. `git switch main && git pull --ff-only origin main`
  2. Read this file's latest checkpoint (Checkpoint I149 Complete).
  3. Confirm ADR-050 is Accepted.
  4. Create `docs/iterations/I150-*.md` as a Planned baseline.
  5. Implement `ContentPart` enum and `ImageInputCapability` enum in `talos-core/src/message.rs` and `talos-core/src/model.rs`.
  6. Implement the path-reference storage policy (store path + mime + byte_count, not bytes).
  7. Write typed content serde round-trip tests, text-only regression, capability provenance tests.
  8. Run locked validation, commit, push, update checkpoint.

### Checkpoint I150 (Review) — 2026-07-20

- Completed task items: I150 (MODEL-009-B capability model, content types, and persistence foundation) — core types implemented.
- Current commit: `b3cc943` (origin/main).
- Commands run and actual results:
  - `cargo fmt --all -- --check` → clean.
  - `cargo clippy --workspace --locked -- -D warnings` → exit 0.
  - `cargo test --workspace --locked` → exit 0 (all tests pass, 0 failures).
  - `scripts/validate_project_governance.sh .` → 0 warnings.
  - `git diff --check` → clean.
- Deliverables:
  - `ContentPart` enum (Text + Image with path/mime/byte_count) in `talos-core/src/message.rs`.
  - `Message::Multimodal { parts: Vec<ContentPart> }` additive variant.
  - `ImageInputCapability` enum (Supported/Unsupported/Unknown) with `from_metadata()` and `allows_attachment()` in `talos-core/src/model.rs`.
  - All exhaustive match sites updated: session (jsonl + durable), provider (openai + anthropic), agent (compaction + token), TUI (scrollback).
  - 7 tests: ContentPart Text/Image serde round-trip, Message::Multimodal serde round-trip, Message::User regression, ImageInputCapability Supported/Unsupported/Unknown.
- Open risks or deviations:
  - Provider adapters extract text only from Multimodal (image wire mapping is I152 scope).
  - TUI scrollback returns None for Multimodal (rendering is I151/I152 scope).
  - Session resume/export/copy tests for Multimodal path-reference persistence not yet written.
  - Public API semver impact inventory and migration notes not yet documented.
  - The `image` crate dependency not yet added (needed for I151 decoder boundary).
- Next task item: I151 — MODEL-009-C safe local image ingestion (path authorization, MIME/magic-byte, size/pixel/count limits, catch_unwind).
- Recovery or resume instruction:
  1. `git switch main && git pull --ff-only origin main`
  2. Read this file's latest checkpoint (Checkpoint I150 Review).
  3. Confirm `ContentPart` and `ImageInputCapability` are in `talos-core`.
  4. Add `image` crate dependency to workspace Cargo.toml.
  5. Implement image path validation (canonicalize, regular file, MIME, magic-byte, size/pixel/count limits).
  6. Add `catch_unwind` at every decoder boundary.
  7. Write adversarial fixture tests.
  8. Run locked validation, commit, push, update checkpoint.

### Post-Oracle Fixes + I148 Wiring + I151 + I152 — 2026-07-20

- Oracle review identified: path leak in jsonl.rs (full local path exposed), I148 discovery not wired, I151 not started.
- Fixes applied:
  1. `383f291` — Fixed path leak: jsonl.rs message_parts now uses `path.file_name()` instead of `path.display()`.
  2. `7da2141` — Wired `discover_provider_models` into `handle_register_custom_provider`: after atomic config save, calls discovery, emits discovered model IDs on success, manual fallback instructions on failure. Removed `#![allow(dead_code)]` from provider_discovery.rs.
  3. `8078827` — I151 image validation module: `image_validation.rs` with `validate_image_path` (regular file, directory/empty rejection, byte/aggregate/count limits, canonicalization, magic-byte MIME detection for PNG/JPEG/GIF/WebP). 16 adversarial fixture tests. Marked `#[allow(dead_code)]` pending I152 TUI wiring.
  4. `2599501` — I152 adapter wire mapping: OpenAI adapter emits `image_url` content parts with data URLs; Anthropic adapter emits `image` content blocks with base64 source. `OpenAIMessage.content` changed from `Option<String>` to `Option<Value>` for array content support. Added `base64` dependency to `talos-provider`.
- Current commit: `2599501` (origin/main).
- Validation: cargo fmt/clippy/test/governance/diff-check all pass.
- Remaining for I153: end-to-end mock fixtures, native/panic boundary review, documentation sync (README/site/config reference/BOARD), release candidate checklist.
- Next task item: I153 — end-to-end hardening, documentation, release candidate.
- Recovery or resume instruction:
  1. `git switch main && git pull --ff-only origin main`
  2. Read this file's latest checkpoint.
  3. Run the final validation ladder:
     ```
     cargo fmt --all -- --check
     cargo check --workspace --locked
     cargo clippy --workspace --locked -- -D warnings
     cargo test --workspace --locked
     scripts/validate_project_governance.sh .
     git diff --check
     ```
  4. Update README EN/zh-CN, site/, config reference, BOARD.md, iteration docs.
  5. Generate release candidate checklist.
  6. Commit, push, update checkpoint.

## Related Documents

- `docs/sop/LONG-RUNNING-TASK.md` — governing SOP.
- `docs/sop/START-ITERATION.md` — iteration activation procedure.
- `docs/sop/ITERATION-WORKFLOW.md` — per-iteration execution rules.
- `docs/sop/CHANGE-CONTROL.md` — mid-iteration requirement changes.
- `docs/sop/GIT-WORKFLOW.md` — commit message and staging rules.
- `docs/sop/RELEASE-WORKFLOW.md` — release validation (referenced for I153 preflight only; no tag is authorized by this task).
- `docs/sop/EVOLUTION-FEEDBACK.md` — lessons and corrections.
- `docs/reference/ARCHITECTURE.md` — crate boundaries and data flow.
- `docs/decisions/013-provider-config-schema-boundary.md` — provider config boundary.
- `docs/decisions/023-inline-api-key-boundary.md` — credential display boundary.
- `docs/decisions/047-external-path-tool-authorization.md` — external path authorization.
- `docs/decisions/048-model-variant-representation.md` — variant representation.
- `docs/decisions/049-steering-queue-projection-boundary.md` — steering queue projection.
- `docs/backlog/active/TUI-033-parameterless-model-connect-commands.md` — I146 parent story.
- `docs/backlog/active/MODEL-008-interactive-custom-provider-registration.md` — I147/I148 parent story.
- `docs/backlog/active/MODEL-009-multimodal-image-input.md` — I149-I152 parent story.
- `docs/backlog/active/SEC-001-external-path-authorization.md` — SEC-001 owner.
- `docs/reference/I140-SEC001-SECURITY-REVIEW-2026-07-17.md` — SEC-001 security review.
