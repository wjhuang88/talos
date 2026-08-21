# I216 Runtime Shutdown Migration — Next Minor

**Status**: Implemented source migration note; release/version assignment remains unselected
**Owner**: RUNTIME-005-B / I216
**Applies to**: the first minor release containing the I216 implementation

I216 adds bounded shared shutdown to `talos-runtime`. It does not change the workspace version,
create a tag, publish a crate, or choose the next release number. Release governance must place this
change in a minor release rather than a patch release because the previously exhaustive public
`RuntimeError` enum becomes `#[non_exhaustive]` and gains variants.

## Existing shutdown calls

The consuming compatibility entrypoint remains source-present:

```rust,ignore
runtime.shutdown().await?;
```

It now joins the runtime's first accepted shutdown plan. When it establishes the plan, it uses
`Interrupt` with one 30-second total deadline. It still returns `RuntimeError::ActorJoin` for an
actor join failure. Any other incomplete cleanup returns `RuntimeError::ShutdownIncomplete` rather
than incorrectly returning `Ok(())`.

## Exhaustive error matches

Downstream code that exhaustively matched `RuntimeError` must add a fallback arm when moving to the
minor release that contains I216:

```rust,ignore
fn classify(error: &talos_runtime::RuntimeError) -> &'static str {
    match error {
        talos_runtime::RuntimeError::RuntimeClosing => "closing",
        talos_runtime::RuntimeError::ShutdownIncomplete { .. } => "shutdown_incomplete",
        _ => "other_runtime_error",
    }
}
```

The fallback is required because the enum is now non-exhaustive. The independent fixture at
`tests/fixtures/runtime-sdk-external` compiles this exact downstream shape outside the workspace
root and exercises the structured API.

## Structured shutdown

Hosts that need policy choice or a report should migrate to a borrowing structured entrypoint:

```rust,ignore
use std::time::Duration;
use talos_runtime::{ActiveTurnPolicy, ShutdownOptions};

let report = runtime
    .shutdown_with(ShutdownOptions::new(
        Duration::from_secs(20),
        ActiveTurnPolicy::FinishCurrent {
            grace: Duration::from_secs(5),
        },
    )?)
    .await?;
```

Alternatively, obtain `runtime.shutdown_controller()` and clone that shutdown-only controller into
host components. The first valid request closes admission and fixes the policy and absolute
deadline; later callers cannot replace either and receive the same cached report. Invalid options
fail during construction before touching the runtime or consuming the primary handle.

The report is deliberately redacted: it contains fixed typed states, bounded counts and durations,
not prompts, reasoning, messages, tool data, provider payloads, paths, credentials, or arbitrary
error text.
