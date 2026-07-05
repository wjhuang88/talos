# Permission Experience Reference Study — 2026-07-05

## Purpose

This study is the design gate for `PERM-003`. It compares current permission and command-execution
patterns in adjacent coding agents, then records the Talos-specific constraints that must shape the
permission redesign.

The goal is not to copy a permissive default from another project. Talos must reduce repeated
approval prompts while preserving its hard rule that write-capable and shell-capable tools remain
permission-gated.

## Sources Checked

| Project | Evidence | Notes |
|---|---|---|
| Claude Code | `https://code.claude.com/docs/en/settings` | Checked settings scopes, permission rules, permission modes, and bash sandbox settings on 2026-07-05. |
| Codex CLI | `https://developers.openai.com/codex/config-reference` | Checked `approval_policy`, `sandbox_mode`, workspace-write roots, and managed permission profile knobs on 2026-07-05. |
| OpenCode | `https://opencode.ai/docs/permissions` | Checked permission actions, auto mode, granular rules, external directories, defaults, and `always` semantics on 2026-07-05. |
| Aider | `https://aider.chat/docs/config/options.html` and `https://aider.chat/docs/usage/commands.html` | Checked `--yes-always`, `--auto-commits`, `/run`, `/test`, and `/git` command surfaces on 2026-07-05. |
| Talos | `docs/backlog/active/PERM-001-guardian-exec-policy.md`, `docs/backlog/active/PERM-002-operation-scoped-permissions.md`, `crates/talos-tools/src/bash_tool.rs` | Local source and backlog state. |

## Comparison

| Project | Verified Behavior | Permission Reuse Pattern | Talos Takeaway |
|---|---|---|---|
| Claude Code | Permission rules support `allow`, `ask`, and `deny`; rule examples include tool-specific forms such as `Bash(npm run *)`, `Bash(git push *)`, `Read(./.env)`, and `WebFetch(domain:example.com)`. Permission modes include `default`, `acceptEdits`, `plan`, `auto`, `dontAsk`, and `bypassPermissions`; project/local settings cannot silently enable some dangerous modes. Bash sandboxing has separate filesystem and network controls. | Reuse is expressed through explicit rule syntax and scoped settings layers. Sandbox settings are separate from approval decisions. | Talos should keep approval scope visible and typed. The permission store should distinguish command approval from sandbox/write/network capability. |
| Codex CLI | `approval_policy` controls when command execution pauses, with values such as `untrusted`, `on-request`, `never`, and granular toggles for sandbox approval, rules, MCP elicitations, permission requests, and skill approval. `sandbox_mode` separately controls filesystem/network access with `read-only`, `workspace-write`, and `danger-full-access`; workspace-write can add writable roots and network access. | Reuse is largely profile/config driven: approval policy and sandbox policy are separate levers, and managed configs can restrict available modes/profiles. | Talos should separate approval prompts from execution containment. A long task can preflight a permission plan, but that plan must not imply broader filesystem or network access unless explicitly granted. |
| OpenCode | Permission actions are `allow`, `ask`, and `deny`; `--auto` approves requests that are not explicitly denied. Rules can be global or tool-specific, including bash command patterns and edit path patterns. External directories are separate permission subjects. When asked, the UI offers `once`, `always`, and `reject`; `always` applies future requests matching tool-provided suggested patterns for the current session. | Reuse is session-scoped by default for `always`; tools provide the suggested reusable pattern, and explicit denies still apply. | This is the closest model for Talos: `always` should approve a displayed reusable scope, not a hidden broad `bash` allow. Talos needs stronger deny precedence and safer default suggested patterns. |
| Aider | `/run` runs a shell command and may add output to chat; `/test` runs a shell command and adds output on non-zero exit; `/git` runs a git command with output excluded from chat. `--yes-always` always answers yes to confirmations; `--auto-commits` defaults on for LLM changes. | Reuse can be broad through a global yes mode. This improves flow but is too coarse for Talos' permission boundary. | Talos should not adopt a global yes mode as the normal answer to approval fatigue. Any unattended mode must be backed by a concrete plan of allowed operations. |
| Talos Current | PERM-002 provides operation-scoped permissions; recent bash permission profiles were tightened so identical command/cwd/classification can share a resource without sharing unrelated subcommands. Write approvals are intended to be directory-scoped. | Reuse exists, but permission UX still lacks a complete taxonomy, reference-backed prompt copy, and long-task preflight model. | PERM-003 should finish the taxonomy and UX before any broader bash, exec, or unattended permission expansion. |

## Talos Permission Taxonomy

| Scope | Intended Use | Default Lifetime | Persistent Config Eligible? | Required Display In Prompt |
|---|---|---|---|---|
| `exact_command` | Repeat the same normalized command in the same working directory and risk class. | Session | No by default | Full command, working directory, risk class. |
| `command_template` | Low-risk validation families such as `cargo test <filter>` or `npm test -- <filter>` after project type detection. | Session | Yes, only from explicit config editing | Template, allowed arguments, project type, denied argument classes. |
| `directory_write` | Edit/write/delete within one approved directory subtree. | Session | Yes, explicit directory path only | Directory root, operations covered, exclusions. |
| `remote_network` | Fetch or push to one host/service or use a configured provider endpoint. | Session | Yes, host/service scoped | Host, method/action family, credential boundary. |
| `long_task_preflight` | Pre-approve a bounded ordered task plan for unattended work. | Session | No | Task ID, ordered operations, max retries, stop conditions. |
| `internal_service` | In-process Talos capabilities such as internal governance validation. | Session not required when read-only | N/A | Capability name and whether host tools are excluded. |
| `host_tool_adapter` | Ecosystem-specific tools such as Cargo, npm, pytest, make, or project scripts. | Session | Yes, adapter-specific | Project type, command adapter, unavailable-tool behavior. |

## Required Design Decisions For PERM-003

- `deny` rules must override every runtime/session/configured allow. This includes `always`,
  command templates, directory write grants, auto/preflight grants, and adapter grants.
- `always` must show the exact reusable scope before the user approves it. The stored resource
  should be derived from that scope, not from a broad tool name.
- Bash command reuse should start from `exact_command`. `command_template` must be opt-in and
  reserved for audited low-risk validation families.
- Write approvals should be directory-scoped with explicit operations, not single-file-only and not
  whole-workspace by accident.
- Long-running tasks should gather likely permissions up front through a preflight plan. The plan
  is only a batch of normal scoped permissions; it is not a permission-bypass mode.
- Internal Talos capabilities should replace host shell usage where practical. Host commands remain
  adapters with ecosystem metadata and unavailable-tool behavior.
- Project-type detection must happen before host-tool adapter instructions are injected. Cargo
  guidance belongs to Rust projects only; npm, pytest, Go, Java, and other guidance must be
  similarly conditional.

## Acceptance Matrix For Implementation

| Requirement | Gate |
|---|---|
| Repeated identical bash command in the same cwd asks once, then reuses the displayed exact scope. | Unit test in `talos-tools` plus CLI approval integration test. |
| Similar but materially different bash subcommand does not reuse an exact-command approval. | Unit test proves normalized resource differs. |
| Directory write `always` covers sibling files under the approved directory but not parent or unrelated directories. | Permission evaluator unit test. |
| Deny wins over session and config allows. | Permission evaluator unit test with deny after an `always` grant. |
| Long-task preflight never creates a broad `bash` allow. | Design test or snapshot of preflight plan resources. |
| Host-tool adapter guidance is injected only after project type detection. | Validation service/project detection test. |
| Tool prompt displays full arguments when they fit and truncates only on actual line overflow. | TUI rendering regression test under `TUI-025`. |

## Residual Work Owners

- `PERM-003`: permission taxonomy, prompt copy, reusable-scope storage, and security review.
- `VALIDATION-001`: project-type detection and host-tool adapter instruction injection.
- `TOOL-017`: multi-command/pipe execution only after the permission taxonomy is implemented.
- `GIT-001`: replace runtime host-`git` fallback in governance status with internal/gix behavior.
