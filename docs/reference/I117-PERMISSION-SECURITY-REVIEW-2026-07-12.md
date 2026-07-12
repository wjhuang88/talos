# I117 Permission Security Review — 2026-07-12

**Reviewer**: Codex GPT-5, independent of the glm-5.2 implementation commits.
**Reviewed range**: `697b66e..951a635` plus the follow-up corrections in this review worktree.
**Decision**: Accepted only for the conservative diagnostic slice described below.

## Accepted Boundary

- `AccessEvidence` is diagnostic data, not authority.
- `evaluate_command_with_evidence()` cannot produce `Allow` from evidence or workspace trust.
- Existing explicit permission rules and Deny precedence remain authoritative.
- Bash and direct exec remain per-command Ask/Deny unless an existing explicit rule applies.
- Evidence is collected for bash, exec single-command, sequential/parallel steps, and pipe steps.
- No OS-sandbox, network, credential, push, publish, release, or destructive gate is relaxed.

## Escape-Vector Review

| Vector | Evidence | Result |
|---|---|---|
| `..` and nonexistent-path traversal | canonical/lexical boundary tests in `access_evidence.rs` | No authority impact; outside/unknown remains diagnostic |
| Symlink escape | existing-path canonicalization | No authority impact; diagnostic only |
| Shell control/substitution/redirection/globs | classifier returns Unknown | Accepted |
| Mutating read-tool flags | `sed -i`, `find -delete/-exec`, `rg --pre`, awk process access tests | Accepted for diagnostics; no Allow |
| Child process/network intent | Spawn/Network/Unknown classes and permission tests | Existing permission decision unchanged |
| Exec multi-step and pipes | production-input collector tests cover every step | Accepted |
| Explicit Deny precedence | permission-engine tests | Preserved |
| Missing or malformed evidence | empty diagnostic list/Unknown; permission profile still runs | Fail-closed with respect to evidence |
| Logging leakage | debug diagnostic includes command/path metadata but not file contents or credentials | Accepted; debug logs remain sensitive operational metadata |

## Required Re-Review Triggers

A new security review is mandatory before evidence can alter an Allow/Ask/Deny result, before
runtime observation is introduced, or before an OS sandbox/default permission policy changes.
The current acceptance must not be cited as approval for command auto-allow.

## Validation

- `cargo test --locked -p talos-permission`
- `cargo test --locked -p talos-agent access_evidence_tests`
- `scripts/validate_project_governance.sh .`
- canonical release preflight; loopback-only runner limitation must be recorded separately
