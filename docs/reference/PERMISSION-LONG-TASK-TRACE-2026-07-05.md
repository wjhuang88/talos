# Permission Long-Task Approval Trace — 2026-07-05

## Purpose

Record a bounded measurement for `PERM-003`: selecting `always` for the same practical operation
must reduce repeated approvals without broadening bash, write, network, or destructive permissions.

## Scenario

Simulated long-task operation:

- Tool: `bash`
- Permission facet: `Execute`
- Resource kind: `command`
- Resource: `bash:read_only_inspection:template:<cwd>:cat`
- Action: same low-risk read-only command family repeated against different file objects after the
  user chooses `always`.

This trace mirrors Talos's current bash permission resource model: eligible low-risk read-only and
validation-build commands use a scoped template resource loaded from
`crates/talos-tools/src/bash_permission_policy.toml`. Different directories, non-template command
families, complex shell syntax, parent/absolute paths, network/package-manager operations, and
write/mutating commands remain distinct exact resources.

## Measurement

| Step | Engine state | Expected decision |
|---|---|---|
| Initial call: `cat src/lib.rs` | Default rules only | `Ask` |
| User chooses `always` | Runtime allow inserted before default `Ask` for `bash:read_only_inspection:template:<cwd>:cat` | `Allow` |
| Repeat 1: `cat Cargo.toml` | Same template, different object | `Allow` |
| Repeat 2: `cat README.md` | Same template, different object | `Allow` |
| Repeat 3: `cat crates/talos-tools/src/lib.rs` | Same template, different object | `Allow` |
| Repeat 4: `cat docs/BOARD.md` | Same template, different object | `Allow` |
| Repeat 5: `cat docs/backlog/PRODUCT-BACKLOG.md` | Same template, different object | `Allow` |

Measured prompt count after `always`: `0 / 5` repeated calls ask again.

## Regression Evidence

- `crates/talos-cli/src/approval.rs`
  - `test_low_risk_bash_template_reduces_different_object_prompts`
  - `test_repeated_always_approval_reduces_same_operation_to_zero_prompts`
  - `test_configured_deny_precedes_runtime_always_allow`
  - `test_always_allow_descriptions_show_reusable_scope`
- `crates/talos-tools/src/bash_tool.rs`
  - `test_bash_permission_policy_toml_parses`
  - `test_bash_read_only_template_shares_across_objects_in_same_cwd`
  - `test_bash_read_only_template_rejects_parent_and_absolute_paths`
  - `test_bash_validation_template_shares_cargo_test_filters`
  - `test_bash_template_rejects_find_exec_complex_shell_and_writes`
  - `test_bash_permission_profile_repeated_command_shares_resource`
  - `test_bash_permission_profile_different_subcommands_do_not_share_resource`
  - `test_bash_permission_profile_same_command_across_directories_is_distinct`
- `crates/talos-cli/src/permissions.rs`
  - `preflight_packet_reports_reusable_bash_template`
  - `preflight_packet_keeps_high_risk_bash_exact`
  - `render_preflight_packet_explains_no_execution_or_rule_install`

## Preflight Evidence

I098 adds a read-only preflight surface that computes the same reusable scopes before a long task
runs:

```sh
talos permissions preflight \
  --operation 'bash={"command":"cat Cargo.toml"}' \
  --operation 'bash={"command":"rm generated.txt"}'
```

Observed result:

- `cat Cargo.toml` reports `current decision: ask` and a reusable
  `bash:read_only_inspection:template:<cwd>:cat` scope.
- `rm generated.txt` reports `current decision: ask` and an exact
  `bash:write_or_mutating:exact:<hash>` scope.
- The packet states that preflight is read-only and does not execute tools or install allow rules.

## Security Boundary

- Configured deny rules still win over runtime `always` rules.
- `always` for template-eligible bash remains scoped to command class, command family, and cwd.
- `always` for non-template bash remains scoped to the exact command/cwd/class resource.
- Directory write `always` remains directory-scoped.
- No permission default is changed.
- No broad `bash = allow` mode is introduced.
