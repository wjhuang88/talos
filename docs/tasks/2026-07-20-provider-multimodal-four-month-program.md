# Long-Running Task: Provider Configurability And Multimodal Image Input — Four-Month Program

| Field | Value |
|-------|-------|
| Task ID | 2026-07-20-provider-multimodal-four-month-program |
| Owner | Senior agent (single executor; no subagent delegation) |
| Created | 2026-07-20 |
| Status | In Progress (P0 governance baseline) |
| Branch | `main` (direct commits, no feature branches) |
| SOP | `docs/sop/LONG-RUNNING-TASK.md` |
| Confirmation | Single maintainer confirmation covers the full 8-iteration cycle + P0; no per-phase re-confirmation except at hard-stop conditions. |

## Outcome

Deliver, on `main`, a coherent four-month product slice that lets a user:

1. Drive `/model` and `/connect` exclusively through parameterless TUI menus with in-panel search (TUI-033 / I146).
2. Register an OpenAI-compatible or Anthropic-compatible custom provider interactively from `/connect`, without editing TOML, through a cancel-safe wizard with atomic config persistence (MODEL-008-A / I147).
3. Discover models from the registered provider's protocol-defined models endpoint, with a safe manual fallback, and immediately activate the selected `(provider, model)` in the current session (MODEL-008-B / I148).
4. Attach a local image to a message when, and only when, the selected model's confirmed capability is `image_input = Supported`; reject `Unknown` and `Unsupported` before reading any file bytes; emit protocol-native image content through the two existing adapters; and persist a safe, portable attachment record (MODEL-009-A/B/C/D / I149-I152).
5. Ship a release-candidate evidence packet (no tag, no release) covering provider registration, model discovery, capability gating, image input, history resume, and text-only regression (I153).

REL-002 remains NO-GO and is out of scope. No `v1.0.0` claim, tag, release, crates.io publish, GitHub Release, or Pages deployment is authorized by this task.

## In Scope

- TUI-033: parameterless `/model` and `/connect` commands with in-panel search and structured identity (I146).
- MODEL-008-A: interactive custom provider wizard (name → protocol → base URL → API key → confirm) with atomic config save (I147).
- MODEL-008-B: protocol-specific model discovery, manual fallback, and immediate session activation (I148).
- MODEL-009-A: image-input ADR and security spike — research, decision, and testable prototype only; **no production image sending** (I149).
- MODEL-009-B: Talos-owned typed ordered content parts, capability semantics (`Supported` / `Unsupported` / `Unknown`), and safe persistence boundary (I150).
- MODEL-009-C: safe local image ingestion — authorization, canonicalization, MIME/magic-byte validation, byte/pixel/count limits, decoder panic containment (I151).
- MODEL-009-D: OpenAI-compatible and Anthropic-compatible image request adapters, TUI attachment UX, CLI equivalent or documented rejection, safe history/resume/export rendering (I152).
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

## Dependencies And Prerequisites

- P0 → I146 → I147 → I148 → I149 → I150 → I151 → I152 → I153 (strict sequential dependency chain).
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
- One logical commit per iteration (P0 + I146 + I147 + I148 + I149 + I150 + I151 + I152 + I153 = 9 commits total). Multiple iterations must not be merged into one commit; no end-of-program batch push.
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

- Wall-clock budget: 16 weeks (4 months) for I146-I153. P0 is the governance baseline and is expected to complete in one session.
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
