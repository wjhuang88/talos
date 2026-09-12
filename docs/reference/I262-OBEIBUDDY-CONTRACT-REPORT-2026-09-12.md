# I262 / #520 OBeiBuddy Contract Investigation

## Status

This is an evidence report, not a browser implementation or a new API/security acceptance.
Evidence was inspected on `main` after claim merge `9e3d07c561e9e501eba781e347d9d16061c2fc15`.

## Request A — bounded runtime finalization

The current v0.9 runtime exposes `RuntimeHandle::shutdown_with(&self, ShutdownOptions)` and a
cloneable `shutdown_controller()`. The first valid request closes admission, fixes policy/deadline,
and later callers receive the cached redacted report. The consuming `shutdown(self)` remains a
compatibility wrapper using a 30-second interrupt plan.

| Question | Evidence-backed answer |
|---|---|
| Active provider/tool, approval, continuation behavior | `Interrupt` cancels the start-committed turn through the Session cancellation path; `FinishCurrent` gives the active turn bounded grace, then uses the same interrupt. Pending work cannot start after the shared admission fence. |
| Authoritative stopped/no-new-work signal | `RuntimeClosing` rejects SDK submissions; the coordinator admission arbiter and `ShutdownReport::actor()`/`active_turn()` describe closure and containment. |
| Durable finalization | The Session actor remains sole owner. `InterruptedAndFinalized` means ADR-058 terminalization completed; `ShutdownDurableOutcome` describes pending-custody reconciliation, not a guarantee that every external side effect was undone. |
| Unknown outcome | `Unreconciled`, `Failed`, `NotRunDeadline`, or `Contained` must be treated as incomplete; host must retain/report recovery state and must not replay automatically. |
| Deadline-exhausted drop | Dropping the handle only initiates best-effort shutdown. A host may drop after an incomplete report only if it preserves its own recovery policy; Talos does not claim unknown external effects are safe. |
| v0.9 stability | Additive structured APIs and migration notes exist, but public API stability is semver-scoped. Downstreams must use non-exhaustive fallback handling and pin an exact release/commit. |

Issue #49 is closed with merged RUNTIME-005 evidence. Issue #45 remains open despite SESSION-008
implementation evidence; therefore the downstream request's blanket “both closed” assumption is
not confirmed.

## Request B — frame-aware browser contract

Disposition: **Deferred** pending a new architecture/security decision and separately governed owner.
WEB-005 is mock-only, TOOL-014 provides conditional disclosure, WEB-007 is intake, and
BROWSER-001 is an unclaimed read-only connector. No existing owner authorizes frame interaction.

The required future contract must retain opaque tab/frame/browser-generation/snapshot-generation
identity; same-origin, cross-origin, nested, delayed-loading, duplicate-name, rebuilt and stale
frame fixtures; closed-schema admission; executor-boundary revalidation; child-origin permission
isolation; bounded redacted output; explicit detached errors; and no automatic replay. It must not
add click/fill/upload/download, cookie/credential extraction, arbitrary JavaScript or default
registration. The next owner should reconcile WEB-007/#452, BROWSER-001/#508, WEB-005 and
TOOL-014, then select crate/API boundary and security review before implementation.

## Downstream guidance and residuals

Downstreams may consume current structured shutdown with the report semantics above, but should
checkpoint ambiguous outcomes and not infer rollback of provider/tool side effects. Browser-frame
integration remains a separately selectable story; no Talos release or implementation claim follows
from this report.

Validation: exact source references, `COLLABORATION_VALIDATION_BASE=origin/main bash
scripts/validate_collaboration_claims.sh .`, `bash scripts/validate_project_governance.sh .`,
authenticated `python3 scripts/validate_remote_issue_owners.py`, and `git diff --check`.
