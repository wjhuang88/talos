# Permission Long-Task Approval Trace — 2026-07-05

## Purpose

Record a bounded measurement for `PERM-003`: selecting `always` for the same practical operation
must reduce repeated approvals without broadening bash, write, network, or destructive permissions.

## Scenario

Simulated long-task operation:

- Tool: `bash`
- Permission facet: `Execute`
- Resource kind: `command`
- Resource: `bash:read_only_inspection:trace`
- Action: same normalized operation repeated five times after the user chooses `always`.

This trace mirrors Talos's current bash permission resource model: exact normalized command, working
directory, and command class produce a stable resource. Different commands, directories, and
high-risk classes remain distinct resources.

## Measurement

| Step | Engine state | Expected decision |
|---|---|---|
| Initial call | Default rules only | `Ask` |
| User chooses `always` | Runtime allow inserted before default `Ask` | `Allow` |
| Repeat 1 | Same resource | `Allow` |
| Repeat 2 | Same resource | `Allow` |
| Repeat 3 | Same resource | `Allow` |
| Repeat 4 | Same resource | `Allow` |
| Repeat 5 | Same resource | `Allow` |

Measured prompt count after `always`: `0 / 5` repeated calls ask again.

## Regression Evidence

- `crates/talos-cli/src/approval.rs`
  - `test_repeated_always_approval_reduces_same_operation_to_zero_prompts`
  - `test_configured_deny_precedes_runtime_always_allow`
  - `test_always_allow_descriptions_show_reusable_scope`
- `crates/talos-tools/src/bash_tool.rs`
  - `test_bash_permission_profile_repeated_command_shares_resource`
  - `test_bash_permission_profile_different_subcommands_do_not_share_resource`
  - `test_bash_permission_profile_same_command_across_directories_is_distinct`

## Security Boundary

- Configured deny rules still win over runtime `always` rules.
- `always` for bash remains scoped to the exact command/cwd/class resource.
- Directory write `always` remains directory-scoped.
- No permission default is changed.
- No broad `bash = allow` mode is introduced.
