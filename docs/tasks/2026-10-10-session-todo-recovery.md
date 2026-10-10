# Session Todo Recovery Checkpoint

Status: Blocked; inspection and baseline validation only. No implementation claim or activation is established by this record.

Next task / handoff: [Session Todo Handoff](2026-10-10-session-todo-handoff.md). It preserves all six original outstanding tasks and local changes, plus a seventh task for the Git permission defect blocking requested commit/push. Two git_add attempts returned `Permission denied: permission state error: permission facet 0 has no safe typed resource`; no commit or push followed. The handoff also supersedes the incomplete preflight failure diagnosis below: recovered logs show check and Clippy passed, then CLI binary/test compilation failed with allocation errors and SIGABRT during cargo test. No live build remains.

Publication follow-up: the maintainer subsequently authorized temporary bash Git operations for these two documents. Host staging and staged whitespace validation succeeded; verify the eventual commit/push in Git history and remote state. The structured Git permission defect remains unresolved. Earlier no-commit/no-push statements describe the original checkpoint.

## Requested Deliverables

The maintainer requested completion of the outstanding session todos:

- Expose validate and agile as model-callable tools.
- Correct custom-provider variant selection and context matching, with regression coverage.
- Expose command memory limits while preserving the 2 GiB default.
- Correct compaction trigger/observability and expose an Actor-owned model-callable compaction operation.

## Scope And Authorization Gates

MODEL-014 is Refinement / Unclaimed. Its capability/variant expansion must not reuse the completed MODEL-013/I212 claim. MEM-005 remains Planned; I298/MEM-005-A is Complete and expressly excludes the broader manual/model-assisted policy. Establish effective target-branch claims before implementation branches or committed implementation. This checkpoint does not change those owners or authorize protected process-hardening changes.

The current worktree is main with existing uncommitted TUI, process display and intake changes; preserve them. Do not infer completion from the session checklist.

## Inspection Findings

- `/agile` calls `governance_summary::format_governance_summary` with the bound workspace and is read-only.
- `/validate` permits the governance profile; CLI validate also exposes plan/run and workspace/I076 profiles. Reuse the shared conversation service, but do not classify host-tool execution as read-only.
- `validation::run_validation_check` currently uses synchronous `std::process::Command::output` for host checks. A model adapter must preserve permission enforcement, bounded output, cancellation and process timeout behavior rather than treating the allowlist as authorization.
- Bash, exec and Unix background launch share process hardening in `talos-tools/src/process_boundary.rs`. RLIMIT_AS is currently 2 GiB; changing parameters must cover all paths and receive process-boundary security review.
- Existing config tests explicitly assert that context-only custom catalog projection does not inherit output limits. MODEL-014 must amend authority and explicit-negative precedence independently, not silently rewrite I212 acceptance.

## Validation Evidence

- Previous TUI checkpoint: 602 unit tests, 2 integration tests and 2 doctests passed.
- `cargo test --locked -p talos-config -j 1 catalog`: 19 passed.
- `cargo clippy --workspace --all-targets --locked -j 1 -- -D warnings`: passed.
- Standard `./scripts/release_preflight.sh`: site, installers, governance, collaboration, text boundary and classifier validation passed; cargo check passed; later exited 101. Captured output did not retain a complete failure diagnostic, so no full-preflight success is claimed.
- `cargo test --workspace --locked -j 1 -- --quiet`: failed compiling the CLI test binary with `memory allocation of 2097152 bytes failed`; tests were not executed. This invocation used `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0` and `RUSTFLAGS=-C link-arg=-Wl,--threads=1`.

## Resume

First establish the required owner/claim records through the repository collaboration workflow. Obtain a command execution environment with sufficient address space to build the CLI test binary; the current model-visible exec/bash schemas do not expose a memory override. Then implement one bounded deliverable at a time, run focused regressions and locked workspace preflight, and record real implementation commit evidence before marking governed owners Complete. No release, commit, branch creation, push or remote mutation occurred in this checkpoint.