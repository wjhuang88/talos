# ADR-014: Log Retention and Rotation Boundary

- **Status**: Accepted
- **Date**: 2026-06-05
- **Iteration**: #ARCH-S8 R2 / I018

## Context

I013 centralized Talos logging initialization and added the first `[log]` config surface.
Terminal UI mode already writes logs to `~/.talos/logs/talos.log` to avoid corrupting the
alternate-screen display. That file is currently append-only and has no cleanup mechanism.

Unbounded local logs are a product and security risk: they can fill disks, retain sensitive
operational context longer than intended, and make support/debug workflows harder.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| TUI logs must not corrupt terminal UI rendering | Hard | I013 logging R1 | No |
| Local log files must be bounded by size/count/age | Hard | User decision point; operational safety | No |
| Logs must not store secrets or full sensitive tool arguments | Hard | AGENTS.md hard constraint #3 | No |
| Rotation should not require a daemon or host logrotate | Hard | Self-contained-first principle | No |
| JSON/span contracts are a later observability surface | Soft | #ARCH-S8 R3 | Yes |

## Reasoning

Host `logrotate` would violate the self-contained-first direction. A single append-only file is
too easy to forget and too hard to bound. The smallest safe design is an in-process file sink with
explicit rotation and retention policy in config.

Size-based rotation is deterministic and easy to test. Daily rotation is useful for humans but is
not sufficient by itself because a verbose debug session can grow quickly. The default should
therefore be size-bounded with a small file count. Age cleanup can be added as metadata behavior
when file naming makes it reliable.

## Decision

Talos log files must be bounded when file logging is enabled.

R2 will add:

```toml
[log.file]
enabled = true
path = "~/.talos/logs/talos.log"
max_size_mb = 16
max_files = 5
rotation = "size" # size | daily
```

Rules:

- TUI mode keeps file logging enabled by default because stderr corrupts UI output.
- Non-TUI modes default to stderr unless file logging is explicitly enabled.
- Rotation and cleanup run in-process; no dependency on host `logrotate`.
- When both `max_size_mb` and `max_files` are set, total retained bytes are bounded by their product.
- Log records must continue to avoid secrets and full sensitive arguments.
- JSON output and shared span contracts remain R3 work; R2 only makes file output bounded.

Rejected:

- **Append-only file forever**: operationally unsafe.
- **Host logrotate**: not self-contained and platform-specific.
- **Database-backed logs in R2**: unnecessary complexity before file retention is proven.

## Reversal Trigger

Revisit if production usage requires central log shipping, encrypted local logs, or a cross-process
writer shared by multiple Talos processes.

