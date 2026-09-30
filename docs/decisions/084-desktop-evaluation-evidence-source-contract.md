# ADR-084: Session-Bound Desktop Evaluation Evidence Source

## Status

Accepted (2026-09-30). This decision refines ADR-083 for I287 and authorizes the production
evidence-source implementation described below.

## Decision

The shared Runtime, not Desktop, is the producer of Evaluation evidence. Each evidence record is
bound to the Runtime session and the exact `EvaluationSubject` used by the claim. Desktop may
request evaluation and render the projection, but cannot create evidence or upgrade its state.

Workspace identity is the existing `WorkspaceRevision` identity plus a bounded content revision
computed by the Runtime from the selected workspace. The revision is an opaque digest-derived
snapshot fingerprint (not a monotonic counter); it is recomputed before evaluation and after relevant artifact changes.
The algorithm must be deterministic, bounded, and fail closed when the workspace cannot be
observed safely. It is not a Git commit identity and does not require a Git repository.

Evidence producers must provide provenance, subject identity, revision, and integrity metadata.
The Runtime accepts only producer records that match the active session, claim subject, and current
workspace revision. Tool-request text, Todo state, Desktop UUIDs, fixtures, and evaluator output
are not evidence producers.

A successful content scan proves snapshot collection only, not acceptance-criterion satisfaction.
Hash-only records must not support PASS. Generated directories `.git`, `target`, and `node_modules`
are excluded from this fingerprint; this does not establish validation of their contents.

Evaluation remains explicit and ephemeral for I287. No new durable Mission/Evaluation schema or
migration is introduced. After restart, absent evidence is `EvaluationUnavailable`; an old PASS
must never be reconstructed or upgraded from transcript text. Delivery requires a current,
authoritative report for the exact subject and revision.

## Consequences

I287 can implement a real Runtime-owned evidence adapter while preserving ADR-083's authority
boundary. Persistence adapters, automatic evaluation, evaluator routing, and retention remain
future decisions. The implementation must bound snapshot work and expose unavailable/stale states
instead of blocking the UI or fabricating a verdict.

## Validation

- unit tests for deterministic revision, subject/session mismatch, stale revision and unavailable
  evidence;
- locked Runtime/Desktop tests covering Delivery gating and restart behavior;
- native H6 acceptance using an exact candidate build and a real Runtime-produced evidence record;
- independent security/API review before marking I287 complete.
