# I217 Runtime Finalizer Report Migration — Next Minor

**Status**: Implemented source migration note; release/version assignment remains unselected
**Owner**: RUNTIME-005-C / I217
**Applies to**: the first minor release containing the I217 implementation

I217 completes the ADR-063 shutdown report with a fixed ordered projection for Talos-owned runtime
finalizers. It does not change the workspace version, create a tag, publish a crate, or choose the
next release number. Release governance must place the additive public report and error types in a
minor release rather than an unrelated patch release.

## Report Inspection

Existing callers that only use `ShutdownReport::is_complete()` need no source change. That method
now also returns false when any configured finalizer failed, panicked, timed out, or could not start
before the global deadline.

Callers that need stage detail can inspect the fixed projection:

```rust,ignore
for finalizer in report.finalizers() {
    eprintln!(
        "{}: {:?}",
        finalizer.identifier().as_str(),
        finalizer.outcome()
    );
}
```

`ShutdownFinalizerOutcome` is non-exhaustive. Downstream matches must retain a fallback arm. The
identifier constructor is private: values come only from reviewed Talos runtime composition, never
from caller input, finalizer errors, prompts, tools, paths, or credentials.

## Registration Boundary

I217 does not add a public callback or plugin registration API. The initial frozen registry accepts
only Talos-owned finalizers installed by reviewed composition code before runtime construction.
The current default composition installs none, so `report.finalizers()` is empty. A future
third-party extension requires a separate panic, identifier, semver and cancellation-containment
decision.

## Compatibility Wrapper

The consuming `RuntimeHandle::shutdown()` entrypoint remains source-present. It continues to join
the first accepted structured plan and returns `RuntimeError::ShutdownIncomplete` if durable
reconciliation or any configured finalizer is incomplete. Actor join failures remain
`RuntimeError::ActorJoin`.

The external fixture at `tests/fixtures/runtime-sdk-external` compiles the report accessor and the
existing non-exhaustive `RuntimeError` fallback outside the workspace package graph.
