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

### Local API implementation note (2026-09-30)

The local I287 candidate adds `ArtifactObservation` to the evaluator's fresh context. Observations
are distinct from successful validation: only Behavior criteria can cite observed file contents
or observed deletion. Technical/Validation criteria still require actual validation records.
The Runtime registry registers only matching, permission-allowed, successful built-in
write/edit/delete calls; it reads those paths with confined handles when evaluation is requested.
Unregistered workspace contents are not added to the model request. File text is untrusted data,
not instructions. Capture limits are 32 registered artifacts, 64 KiB per file and 256 KiB total
UTF-8 content. Unsupported or unreadable artifacts fail closed; these limits are not a claim
that every task can be assessed from file contents.

The Desktop host uses its fixed trusted built-in tool composition. SDK embedders attaching the
registry must preserve that trust and authorize evaluator disclosure: matching a tool name is
not authentication of arbitrary plugin implementations. Likewise, public evaluator observations
are caller-authenticated input, not proof that an untrusted caller executed a tool.

Migration: existing evaluator methods retain their signatures and send no artifact observations.
Serialized `EvaluatorRequest` accepts an absent `artifact_observations` field as an empty list.
Rust callers constructing this public struct must add `artifact_observations: Vec::new()` (or
their explicitly authorized observations). This source compatibility change requires the
repository's pre-1.0 compatibility release policy; it must not be shipped as an unnoticed patch.
The new observation API and Runtime producer still require independent security/API review.

### Required checks

- unit tests for deterministic revision, subject/session mismatch, stale revision and unavailable
  evidence;
- locked Runtime/Desktop tests covering Delivery gating and restart behavior;
- native H6 acceptance using an exact candidate build and a real Runtime-produced evidence record;
- independent security/API review before marking I287 complete.
