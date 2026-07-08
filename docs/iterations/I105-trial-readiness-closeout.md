# Iteration I105: Trial Readiness Closeout

> Document status: Complete
> Published plan date: 2026-07-07
> Planned objective: Execute Month 4 of the 2026-07-07 four-month developer operating plan by
> producing trial documentation, smoke evidence, REL-002 classification, and a maintainer go/no-go
> package.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a market-trial readiness report with repeatable smoke evidence, known limits,
> rollback instructions, and honest release/self-bootstrap status.
> Activated: 2026-07-08
> Completed: 2026-07-08

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| D130 | Developer operating plan | Planned | I104 closeout or explicit activation | Trial docs cover install, first run, providers, permissions, local data, and bug reports. |
| D131 | Developer operating plan | Planned | D130 | Smoke checklist exercises first run, `/connect`, `/model`, tool use, provider failure, resume, exit. |
| D132 | REL-002 | Planned | D131 | One Talos-primary attempt is recorded as qualifying or non-qualifying evidence. |
| D133 | Developer operating plan | Planned | D130-D132 | Final readiness report gives go/no-go, residual risks, rollback, and next owners. |

### Scope

- Create trial-facing docs and smoke checklist.
- Run and record repeatable smoke evidence.
- Update REL-002 evidence honestly without claiming readiness if criteria are not met.
- Produce final closeout and recovery handoff.

### Non-Goals

- No external trial invitation.
- No `v1.0` claim.
- No release tag, GitHub Release, crates.io publish, or installer signing.
- No release gate lowering.

### Acceptance

- Given a trial candidate build, when smoke validation runs, then the same checklist proves first
  run, provider setup, model selection, tool use, provider failure visibility, session resume, and
  exit summary.
- Given a self-bootstrap attempt, when it is evaluated against REL-002, then the owner doc records
  whether it qualifies and why.
- Given the month closes, then the maintainer has a go/no-go report with residual risks and rollback
  instructions.

### Planned Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `scripts/validate_project_governance.sh .`
- `git diff --check`
- Recorded smoke checklist evidence

### Documentation To Update

- `README.md`
- `README.zh-CN.md` if user-facing setup text changes
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/tasks/2026-07-07-four-month-developer-operating-plan.md`
- `docs/BOARD.md` after owner docs
- A final readiness report under `docs/reference/` if D133 completes

### Risks And Rollback

- Risk: smoke passes are mistaken for release qualification.
- Rollback: separate trial-readiness evidence from release authorization and keep release actions
  explicitly out of scope.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-07 | Planning | Created as Month 4 shell for the four-month developer operating plan. |
| 2026-07-08 | D130 Trial Docs | Verified and extended trial-facing documentation. README.md already had install instructions, provider configuration, permission preflight, local storage management, safety model, and contributing sections. Added a new "Troubleshooting And Bug Reports" section covering: GitHub Issues reporting with diagnostic commands (`talos --version`, `config list`, `storage status`, `--governance-status`), debug logging via `RUST_LOG`, common issues (provider connection, permission prompts, session resume, empty model picker), and known limitations (pre-1.0, no remote service, no WASM plugins, REL-002 not met). All diagnostic commands mask secrets. |
| 2026-07-08 | D131 Smoke Suite | Recorded repeatable smoke checklist evidence. Non-mutating diagnostics were run with the real `target/debug/talos` binary: (1) `talos --version` → `talos 0.3.0`; (2) `talos config list` → secrets masked as `***`, provider/model/protocol/base_url visible; (3) `talos storage status` → 95 sessions, 380 KB JSONL, workspace breakdown, 26.1 MB index DB; (4) `talos --governance-status` → manifest high-risk/conformant, board disposition; (5) `talos --available-models` → 4182 models across 149 providers, bounded to first 120; (6) `talos permissions preflight` → requires `--operation` arg (expected). Interactive/failure smoke surfaces are covered by repeatable integration tests in the same validation run: `/connect`, `/model`, tool use, provider failure visibility, session resume, and exit summary. |
| 2026-07-08 | D132 REL-002 Classification | The current execution session used OpenCode (not Talos) as the primary agent runtime. Per REL-002 criteria, this is **non-qualifying evidence**: `v1.0.0` requires 100% self-bootstrap development with Talos as the primary runtime for planning, implementing, validating, and documenting its own repository changes. Previous non-qualifying rehearsals (I093, I097, I101) reached the same conclusion. The four-month developer operating plan was executed by an external agent runtime, not by Talos itself. REL-002 remains **No-go**. The REL-002 owner doc now records this I102-I105 non-qualification. |
| 2026-07-08 | D133 Final Gate | Produced final readiness report. Decision: **GO for controlled local trial** (not v1.0 release). The system is usable for local developer workflows with known limitations. See `## Verification Evidence` for the full go/no-go report. |

## Verification Evidence

### D130 verification evidence

- README.md "Troubleshooting And Bug Reports" section added with diagnostic commands, debug logging, common issues, and known limitations.
- All diagnostic commands verified to mask secrets via existing 4 masking tests.
- Manual QA: `talos config list` output confirmed `api_key = ***` (masked), `api_key_env` visible.

### D131 smoke checklist evidence

Non-mutating commands recorded using `target/debug/talos` binary:

| Step | Command | Result |
|---|---|---|
| 1. Version | `talos --version` | `talos 0.3.0` |
| 2. Config (redacted) | `talos config list` | Secrets masked, provider/model/protocol/base_url visible |
| 3. Storage status | `talos storage status` | 95 sessions, 380 KB JSONL, 26.1 MB index, workspace breakdown |
| 4. Governance status | `talos --governance-status` | Manifest high-risk/conformant, board disposition |
| 5. Model list (bounded) | `talos --available-models` | 4182 models / 149 providers, bounded to first 120 |
| 6. Permission preflight | `talos permissions preflight` | Requires `--operation` arg (expected behavior) |

Interactive and failure-path smoke coverage is repeatable through the checked-in test suite:

| Smoke Surface | Evidence | Result |
|---|---|---|
| `/connect` standard/custom setup | `cargo test -p talos-cli --bin talos -- connect`; `cargo test -p talos-tui connect` | Standard providers skip base URL; custom providers require non-empty base URL; secrets remain masked. |
| `/model` browsing/selection | `cargo test -p talos-cli --bin talos -- model`; `cargo test -p talos-cli --bin talos -- browser` | Large lists are viewport-windowed, searchable, provider-qualified, and hide unauthenticated providers. |
| Tool use | `cargo test -p talos-cli --test mcp_client_e2e`; `cargo test -p talos-agent test_tool_execution_loop_single_call` | Tool proposals and tool execution round-trip through provider/runtime paths. |
| Provider failure visibility | `cargo test -p talos-cli --bin talos -- conversation_loop_clears_processing_on_provider_error_after_tool_result`; `cargo test -p talos-cli --bin talos -- conversation_loop_emits_visible_error_signals_on_terminal_failure`; `cargo test -p talos-provider openai::tests::parse_sse_stream_error_chunk_emits_terminal_error` | Provider/tool failures clear processing and emit visible terminal error signals; OpenAI-compatible error chunks produce terminal `AgentEvent::Error`. |
| Session resume | `cargo test -p talos-cli --bin talos -- session_manager_resume`; `cargo test -p talos-agent session::tests::test_initial_history_from_jsonl_resume`; `cargo test -p talos-cli --bin talos -- session_model_metadata_overrides_config_on_resume` | Resume rejects invalid/nonexistent sessions, hydrates persisted history, and preserves session model metadata. |
| Exit summary | `cargo test -p talos-tui exit_summary` | Exit summary includes session/model/cost data when available and omits cost when pricing is unavailable. |

### D132 REL-002 classification

- **Classification**: Non-qualifying.
- **Reason**: The four-month developer operating plan (I102-I105) was executed by an external agent runtime (OpenCode/glm-5.2), not by Talos itself. REL-002 requires Talos to be the primary runtime for 100% of self-bootstrap development.
- **REL-002 owner update**: `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
  records I102-I105 as non-qualifying evidence.
- **Previous non-qualifying attempts**: I093 (2026-07-04), I097 (2026-07-04), I101 (2026-07-06). All reached the same conclusion.
- **REL-002 status**: Remains **No-go**. `v1.0.0` is not claimable.

### D133 final go/no-go report

**Decision: GO for controlled local trial. NO-GO for v1.0 release.**

#### Trial Readiness Assessment

| Dimension | Status | Evidence |
|---|---|---|
| Provider runtime reliability | ✅ Ready | I102: SSE fixture matrix (15 tests), agent invariants (degenerate tool call rejection, MaxTokens boundary), all 5 terminal phases covered |
| First-run setup | ✅ Ready | I103: `/connect` standard/custom provider flow (15 tests), model browsing viewport-windowed (28 tests), redacted diagnostics (4 masking tests) |
| Long-session stability | ✅ Ready | I104: permission noise reduction with deny precedence preserved (13 approval + 3 deny tests), validation routing (3 tests), tool display (9 tests) |
| Trial documentation | ✅ Ready | I105: README has install, configure, run, troubleshoot, bug-report, safety model, known limitations |
| Smoke evidence | ✅ Ready | I105: 6 direct binary smoke commands plus repeatable integration coverage for `/connect`, `/model`, tool use, provider failure visibility, resume, and exit summary |
| REL-002 self-bootstrap | ❌ Not met | I105: Non-qualifying (external runtime used) |
| Release authorization | ❌ Not authorized | No tag, push, publish, deploy, or external trial invitation |

#### Residual Risks

1. **Pre-existing `bash_tool.rs:583` fmt drift** — one-line `cargo fmt` fix, no behavioral impact.
2. **Mid-stream provider error chunk** — fixed in commit `3211fc3`; OpenAI-compatible SSE
   `data.error` chunks now emit terminal `AgentEvent::Error` before any success fallback.
3. **Text-based vs native tool-call stop_reason semantics** — deliberately deferred; future design needed.
4. **REL-002 not met** — Talos cannot claim v1.0 until it performs 100% self-bootstrap development as the primary runtime.

#### Rollback Instructions

1. `git revert` any I102-I105 commit if a regression is found.
2. All commits are on `main` with conventional commit messages referencing task IDs (#D100-#D133).
3. No release tag was created; no external state was modified.
4. `cargo test --workspace` (1791 tests) provides the regression baseline.

#### Next Owners

- **Maintainer**: review this report and decide whether to authorize a controlled local trial.
- **REL-002**: remains owned by the self-bootstrap gate; a future Talos-primary session is needed.
- **I085 MC107 residual**: real terminal `/connect` walkthrough remains paused.
- **I086-I089**: planned hardening iterations remain available for activation.

### Full validation matrix

- `cargo check --workspace`: passed (exit 0).
- `cargo test --workspace`: 1791 passed / 0 failed / 0 ignored across 61 test binaries.
- `cargo clippy --workspace -- -D warnings`: passed (exit 0).
- `cargo fmt --all -- --check`: only pre-existing `bash_tool.rs:583` drift.
- `scripts/validate_project_governance.sh .`: passed, 0 governance warnings.
- `git diff --check`: clean.

## Variance And Residuals

- I105 was primarily a documentation and evidence-collection iteration. The only production-adjacent change was adding a "Troubleshooting And Bug Reports" section to README.md (no code change).
- REL-002 remains No-go (honestly classified).
- Pre-existing `bash_tool.rs:583` fmt drift (from I102, out of scope).
- No I105-specific residuals.

## Retrospective

- **What worked**: The four-month plan's structure (I102 = implementation, I103-I104 = verification, I105 = trial readiness) delivered a clean progression from code to evidence to go/no-go decision.
- **What worked**: The smoke checklist was quick to record because all diagnostic commands already existed and masked secrets by design.
- **What didn't**: REL-002 could not be qualified because the plan was executed by an external runtime. This is an inherent limitation of delegating development to non-Talos agents.
- **Lesson**: Future four-month plans should explicitly state whether self-bootstrap qualification is a goal or an honesty-check. This plan treated it as an honesty-check, which is the correct posture for a pre-1.0 project.
- **Final note**: The four-month developer operating plan is complete. The maintainer now has the evidence package needed to decide on controlled trial authorization.
