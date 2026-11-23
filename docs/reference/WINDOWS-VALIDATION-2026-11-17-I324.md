# I324 / #919 native Windows validation — 2026-11-17

## Verdict and provenance

**Compatibility acceptance FAILED.** The available gix candidate tests pass, but
native prepared Checkout/Commit does not pass on Git 2.36.1 in the original Scoop
environment. Production preflight has not been replaced in this candidate.
Neither #919 completion nor ADR-089 amendment/security acceptance is established.

- Refreshed local `main`: `a28c1cccebad71eb5785d149129a640ceec3d098`.
- Tested Draft #928 exact head: `71c923b95e4aff29ac7a7d71703d26f8d42ff530`.
- Validation began in a detached worktree at `target/i324-windows-validation`,
  subsequently attached to `feat/i324-gix-native-preflight` for the evidence
  handoff. Production files were not edited; November 10 logs were retained.
- Native host/compiler: `x86_64-pc-windows-msvc`, Rust 1.99.0
  (`b940084d7`, LLVM 23.1.1), rustfmt 1.10.0-stable.
- System Git: `2.36.1.windows.1`, selected through
  `C:\ProgramData\scoop\shims\git.exe`.
- Original native process environment includes `GIT_INSTALL_ROOT` and `GIT_PAGER`.
  Diagnostic reruns remove **only** `GIT_INSTALL_ROOT` in the invoking process and
  restore it afterward. No persistent Git/environment configuration was changed.
  `GIT_PAGER` remains present. This is diagnosis, not an approved workaround or proof
  that `GIT_INSTALL_ROOT` is semantically inert for every selected writer.

The effective #920 claim (`067dd3d4f5fd0fac910ac7f52bd23ee7f8f5bb80`) owns
this validation. #928 and ADR review #931 were Draft/unmerged when inspected.
The accepted ADR remains in force; no runtime authority was changed here.

## Commands and results

Cargo tests used `cargo +1.99.0-x86_64-pc-windows-msvc test --locked`.
The table lists the remaining arguments. Counts are observed local results.

| Arguments / check | Environment | Result |
|---|---|---|
| `-p talos-tools --features git --lib gix_preflight_candidate_tests` | Original native | 12 passed, 0 failed, 0 ignored, 373 filtered |
| `-p talos-tools --features git --lib` | Original native | Exit 101; 272 passed, 113 failed, 0 ignored |
| `-p talos-tools --features git --lib` | Only `GIT_INSTALL_ROOT` removed | Exit 101; 305 passed, 80 failed, 0 ignored |
| `cargo +1.99.0-x86_64-pc-windows-msvc fmt --all -- --check` | Fixed candidate | Failed; one remaining formatting hunk at `gix_preflight_candidate_tests.rs:397` |
| `scripts/release_preflight.sh` with MSVC `RUSTUP_TOOLCHAIN` | Native Git Bash | Exit 127 at line 20: `jq` unavailable; workspace checks/tests were not reached |
| `scripts/validate_project_governance.sh .` | Native Git Bash | Exit 1; SQLite consumers, delivery-workflow scenarios and remote Issue owner regression Python subchecks failed |
| `scripts/validate_collaboration_claims.sh .` | Native Git Bash | Exit 0; 1 warning about unrelated SEARCH-001-D3 Unclaimed claim metadata |

`python`/`python3` resolve to Windows Store application aliases on the inspected
PATH; no usable interpreter was found there. These are failed governance checks,
not a waiver. Earlier sandbox-only attempts encountered Cargo cache write denial
and Git unsafe-repository errors; the table uses the subsequent native runs.
Sandbox-injected `GIT_CONFIG_*` safe-directory variables are not the original native
Scoop environment and were not used to characterize the product results.

The 12 passing tests cover lexical branch validity and a native Git branch-name
matrix; config/include planning, exact-read denial, captured-source serialization,
missing-source appearance and stale include detection; admitted hasconfig lookahead;
strict attribute parsing and a native Git filter-attribute differential. They are
test-only primitives, not a complete executable gix preflight implementation.

## Failure classification

The controlled 80 failures are classified by the **first visible failure symptom**,
not asserted to have one underlying cause. The machine-readable per-test list is
in the evidence archive.

| First visible symptom | Count | Boundary |
|---|---:|---|
| Attribute-path discovery unavailable | 66 | Existing CLI preflight blocks on `GIT_ATTR_SYSTEM/GLOBAL` |
| Configuration-path discovery unavailable | 5 | Separate config-source discovery failure |
| Native oracle `git var` usage response | 1 | `host_root_order_and_worktree_selection_match_native_git` |
| URL rewrite / empty push alias / tracking overlay inequality | 3 | Independent semantics differences; not attributed to attribute discovery |
| Receiver/fixture/shell failures | 5 | Details below; all reproduced in separate serial runs |

The final five are:

- `local_push_receiver_requires_path_admission_without_receiver_reads`: discovery failed.
- `pull_command::native_command_refuses_configuration_drift_before_fetch`: fixture
  `git config pull.rebase true` cannot write `.git/config` (permission denied).
- `receiver_multiple_destinations_keep_sibling_bisect_evidence_separate`: fixture
  `git worktree add` reports invalid object name `HEAD`.
- `receiver_windows_shell_noninteractive_command_does_not_source_home_startup`:
  shell discovery status failed.
- `windows_receiver_command_executes_special_path_without_shell_injection`:
  shell selection status failed.

Serial reproduction rules out a passing serial rerun, but does not establish the
root cause of these five failures. They remain open, not ignored or waived.
Direct native `git var GIT_ATTR_SYSTEM` and `git var GIT_ATTR_GLOBAL` both returned
usage. No new production probe or fallback was introduced by these diagnostics.

## Large-content acceptance and uncovered rows

`checkout_and_commit_support_tracked_content_over_16_mib` uses a 17 MiB tracked
file. It fails in the original environment on the Git environment guard, and in
the controlled environment on attribute-path discovery. A separate serial run
reproduced that failure. It therefore does **not** establish actual prepared
Checkout, Commit or branch-switch success.

`large_tracked_source_edit_after_preparation_blocks_checkout` also fails in the
original environment on the environment guard. With only `GIT_INSTALL_ROOT`
removed it **passes**, both in the full suite and separately: the test changes
bytes at offset 16 MiB, observes refusal, and verifies the branch was not created.
This is narrow diagnostic mutation-rejection evidence, not original-environment
product acceptance.

The candidate's `gix_gitdir_conditional_requires_admitted_repository_identity`,
`gix_pure_candidate_records_zero_child_git_invocations` and its isolated child
fixture are `#[cfg(unix)]`. Native Windows executes none of them. The passing
12-test result therefore supplies **no Windows process-sentinel proof** or
Windows gitdir-identity conditional test. Complete no-preflight-child proof,
selected-writer installation-default provenance, HOME/XDG and Unicode/spaces /
multiple-install coverage, executable/path replacement and full hook/transport
equivalence remain unaccepted. A newer Git installation was not tested here.

Workspace-wide locked checks/tests and full preflight did not pass; exact-head
remote CI and independent security/API review were not replaced by local tests.
The next acceptance run requires the reviewed executable implementation and
installation-source decision, Windows coverage for the Unix-only cases, resolution
of the separate failures, and both original Git 2.36.1 and newer-Git matrices.
Keep #919 Open and the existing owner Active; do not mark Complete.

## Durable evidence

See [evidence directory](../tasks/evidence/git-compat-001/windows-2026-11-17/).
`native-logs.zip` contains UTF-8 copies of native candidate/full/focused test logs,
format/preflight/governance/claim logs, classification and focused exit codes.
`manifest.json` binds archive members to SHA-256 and the tested commit. Raw captures
also remain in the local validation worktree. The report and archive are
local evidence, not an independent review or a passing compatibility receipt.
