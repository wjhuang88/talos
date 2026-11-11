# Compact context admission emergency

Status: Active; local repair, not delivered.

Authorization Mode: Emergency override.
Authorizing maintainer: current user, explicit emergency repair request recorded in the session handoff. The earlier GitHub-handle attribution was not independently verified and is not used as authorization evidence.
Incident: compact_context repeatedly fails before execution with `permission facet 0 has no safe typed resource`, preventing checkpoint installation under context pressure.
Scope: classify the existing session-owned checkpoint submission as Internal rather than an unbound filesystem Write; preserve execution scope, snapshot/source validation, permission-generation fencing, bounded payload and Actor Applied receipt semantics. No global permission bypass or changes to file/process tools.
Branch: feat/bg-default-continuation (existing working tree; keep this repair separately staged from background continuation and other people's repairs).
Validation: pending focused permission-admission and checkpoint tests. Independent security review, governance reconciliation and running-binary deployment remain pending.
Rollback/containment: revert only the compact tool classification/test change if validation fails; do not weaken Actor validation. Existing running instance will not hot-load source edits. Do not repeatedly retry rejected calls.
Residual: reconcile owner claim/Board/Issue within two business days after containment per collaboration SOP; record implementation commit and deployment evidence before completion.

## Recovery integration evidence (local candidate, not deployment)

The archived CLI Actor regression exercises registered history discover/search/detail,
rejects a forged reference, submits a source-backed compact_context payload and observes
ModelCheckpointTerminal Applied with after < before. The subsequent test-provider request
contains the installed Context. All three embedded, ordinary CLI and archived CLI recovery
Actor tests passed on the current candidate, including preservation of the original durable
head when legacy set_session_paths is configured. Log: /tmp/talos-actor-no-trim-tests.log.

This supersedes admission-only evidence but does not establish real-provider summary quality,
running-host installation or independent security approval. Full preflight, independent review,
governance reconciliation and deployment remain open. No Completion Commit exists.
