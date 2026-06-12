# SOP: Testing

## Purpose

Define the testing strategy and requirements for the Talos project.

## Testing Layers

### Unit Tests

- Location: `#[cfg(test)] mod tests` within each source file.
- Purpose: Test individual functions, methods, and type behavior.
- Run: `cargo test -p talos-{crate}`

### Integration Tests

- Location: `tests/` directory at workspace root or per-crate.
- Purpose: Test cross-crate interactions and end-to-end flows.
- Run: `cargo test --workspace`

### Property Tests

- Use `proptest` for:
  - Protocol message parsing and serialization
  - Config file parsing and validation
  - Session storage read/write roundtrips
- Property tests verify invariants hold for arbitrary valid inputs.

### Async Tests

- Use `#[tokio::test]` for async test functions.
- Always set a timeout: `#[tokio::test(flavor = "current_thread")]` or explicit timeout.
- Use `CancellationToken` for graceful test cleanup.
- Mock external services (LLM providers, filesystem) with test doubles.

## Coverage Requirements

| Component | Minimum | Rationale |
| --- | --- | --- |
| `talos-core` (protocol types) | High | Foundation types, must be correct |
| `talos-agent` (turn loop) | High | Core logic, hard to debug in production |
| `talos-permission` | High | Security-sensitive |
| `talos-sandbox` | High | Security-sensitive, platform-specific |
| `talos-session` | Medium | Persistence correctness |
| `talos-provider` | Medium | API compatibility |
| `talos-tools` | Medium | Per-tool correctness |
| Other crates | Standard | Best effort |

## Test Naming

```rust
#[test]
fn {unit}_{scenario}_{expected_result}() { ... }

// Examples:
#[test]
fn turn_loop_with_tool_call_produces_tool_result() { ... }

#[test]
fn permission_deny_rule_overrides_allow_rule() { ... }
```

## Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p talos-agent

# Specific test
cargo test -p talos-agent turn_loop_with_tool_call

# With output
cargo test --workspace -- --nocapture

# Ignored tests (only run explicitly)
cargo test --workspace -- --ignored
```

## Governance Checks

Run these when governance documents, status owners, profile, branch mode, worktree mode, or
process rules change:

```bash
scripts/validate_project_governance.sh .
scripts/assess_project_scale.sh .
```

## Rules

- `cargo test --workspace` must exit 0 before any merge.
- No `#[ignore]` without a tracking issue reference in a comment.
- No deleting failing tests to "pass". Fix the code or fix the test.
- New public APIs must have at least one test demonstrating usage.
- Security-sensitive code (sandbox, permission) must include adversarial test cases.
