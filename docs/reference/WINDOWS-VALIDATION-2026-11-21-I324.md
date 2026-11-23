# I324 native Windows query matrix — 2026-11-21

## Provenance and outcome

Executed the [Windows runbook](../sop/GIT-COMPAT-001-I324-WINDOWS-VALIDATION.md)
from Draft #928 at `2bbd3c3441ea14e3ac853ced8101de6424463cf8`.
Host: Windows NT 10.0.26200.0 x64, PowerShell 7.6.9, pinned Rust
1.99.0 MSVC. Selected installed Git reports `2.36.1.windows.1`.
The caller has `GIT_INSTALL_ROOT` and `GIT_PAGER` present;
`GIT_CONFIG_GLOBAL` is absent. No environment values, absolute local paths,
credentials or Git configuration contents are included here.

**Command-level diagnostics completed; product compatibility acceptance remains
FAILED.** Keep supported ADR-approved CLI. Only individual unsupported queries
are replacement candidates under the current #919 scope; this does not revive
the superseded all-gix / zero-CLI plan or accept ADR proposal #931.

## First run and bounded script correction

Both original script modes completed their matrix at the exact source SHA above,
printing `unexpected=0 status=complete`. Both nevertheless exited 1 in `finally`:
`.NET Directory.Delete` could not remove a Git object file. Git creates read-only
object files on Windows; this failure occurred after diagnostic commands finished.
The first executions are failed script runs, not retroactively successful runs.

Changed only fixture cleanup to `Remove-Item -LiteralPath ... -Recurse -Force`,
after validating the resolved target is directly below the system temporary
directory and its basename matches the generated `talos-i324-<32 hex>` shape.
This handles read-only files while bounding recursive deletion to this script's
fixture. No production Git, permission, PATH or caller-environment change.

Both modes were then rerun with that uncommitted cleanup-only delta. The official
pinned ZIP passed the existing upstream SHA-256 check before execution:
`a7a78c306dea018cc7ca3efe6a0d87c1dd7a43762705ccb0c5e31e3e44349207`.
Pinned and selected-installed runs both exited **0**, including cleanup.

## Complete classified outputs after cleanup repair

The following output is identical between modes except `pinned=True` versus
`pinned=False`; both binaries report Git 2.36.1.windows.1. Individual exit 129 is
an expected unsupported-query result, not the script's final exit status.

```text
I324_LEGACY_FIXTURE version=git version 2.36.1.windows.1 pinned=True
I324_LEGACY_ENV mode=isolated host_environment_unchanged=true
I324_LEGACY_VAR name=GIT_ATTR_SYSTEM exit=129 classification=unsupported-variable
I324_LEGACY_VAR name=GIT_ATTR_GLOBAL exit=129 classification=unsupported-variable
I324_LEGACY_VAR name=GIT_CONFIG_SYSTEM exit=129 classification=unsupported-variable
I324_LEGACY_VAR name=GIT_CONFIG_GLOBAL exit=129 classification=unsupported-variable
I324_LEGACY_VAR name=GIT_SHELL_PATH exit=129 classification=unsupported-variable
I324_LEGACY_CHECK_ATTR worktree=ok index=ok source=unsupported-option exit=129
I324_LEGACY_QUERY_MATRIX unexpected=0 status=complete
```

This supplies native per-query evidence for the config-root and shell-query
hypotheses left open on November 17; it does not prove every previous failure
has that cause. Normal worktree and `--cached` select different fixture values;
neither supplies the target-commit semantics of `--source=HEAD`.

Both matrix modes isolate child HOME/config and remove Git-specific child
environment overrides. Their matching outcomes do not establish compiled system
attribute-root equivalence, inherited Scoop wrapper semantics, or a safe blanket
allowance for `GIT_INSTALL_ROOT`. No newer-Git matrix was run on this host.

## Optional Rust rows

Used pinned `cargo +1.99.0-x86_64-pc-windows-msvc`, retaining the original caller
Git environment. Only the PowerShell cleanup delta was present; Rust source and
lockfile were the exact source SHA above. Raw output remains in local TEMP.

| Check | Result |
|---|---|
| `fmt --all -- --check` | Exit 0 |
| `test --locked -p talos-tools --features git --lib gix_preflight_candidate_tests -- --nocapture` | Exit 0; 15 passed, 0 failed, 0 ignored, 374 filtered |
| `test --locked -p talos-tools --features git --lib checkout_and_commit_support_tracked_content_over_16_mib -- --nocapture` | Exit 101; 0 passed, 1 failed, 388 filtered |

The candidate result includes the now-enabled Windows gitdir metadata admission /
stale-identity regression, and independently prints all five unsupported var
classifications and the unsupported source option. It is test-only coverage,
not runtime substitution or complete child-process/effect proof.

The 17 MiB test fails at initial `prepare_checkout`, before executing approved
Checkout or Commit, with:

```text
InvalidInput("Git environment overrides require a separately authorized Git invocation")
```

No actual Checkout/Commit success is asserted. No complete workspace test,
release preflight, independent security/API approval or exact-head remote CI pass
is inferred from these focused runs. Production preflight remains unchanged.

## Handoff

Raw initial/fixed matrix logs and Rust logs stay in local TEMP per the runbook;
they are not committed. This report records manually checked classifications,
exit codes and redacted failure text. Initial cleanup failures may have left
temporary diagnostic directories; repaired reruns clean their own directories.

The developing session should pull the feedback commit, preserve the failed
initial-run record, and use these six unsupported forms as concrete native
evidence for targeted compatibility design. Selected-writer system-default
provenance, exact admitted sources, stale-evidence and authority review remain
required before production wiring. #928/#931 remain Draft; #919 stays Open.
