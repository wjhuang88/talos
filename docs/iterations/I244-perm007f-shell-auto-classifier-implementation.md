# Iteration I244: Shell Auto Classifier Implementation

> Document status: Complete / Closed
> Planned objective: implement the accepted ADR-070 classifier contract so routine shell commands
> are model-triaged without per-command auto-approval exceptions.
> MVP deliverable: a runnable TUI/CLI flow in which `bash` command `ls -la` reaches the isolated
> classifier and can receive one `AllowOnce`, while unsafe or uncertain actions remain blocked or
> human-required.

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-007-F | PERM-007 / Issue #462 | Active / Claimed | I243 / ADR-070 | Claude-like shell classifier experience with exact binding and fail-closed evidence |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 mainline implementation session 2026-09-01 |
| Work Slice | Generic model-first shell effect classification after deterministic deny/ask, isolated tool-free context, exact action/cwd/environment/revision binding, fail-closed fallback, and CLI/TUI/Runtime/MCP equivalence. No per-command exception table; no PERM-006-D/E authority, Dashboard, Desktop, release, or publication. |
| Claimed At | 2026-09-01 |
| Source Issue | #462 |
| Governance Claim PR | #465 |
| Authorization Mode | Independent review |
| Authorization Evidence | ADR-070 Accepted through I243 closeout `be4fbcfc`; maintainer direction requests Claude-like generic model classification rather than command-by-command exceptions. Independent permission/security/API review, exact-head CI, governance validators and merge-time CAS remain mandatory; claim is ineffective until #465 merges. Shared GitHub identity provides Agent-role separation only, not natural-person identity separation. |
| Implementation PR | #468; corrective PR #473 |
| Last Updated | 2026-09-03 |
| Handoff / Release Condition | Complete. Keep the accepted ADR-070 boundary: configured/default Ask may reach bounded model assessment, while `PermissionRuleSource::Explicit` Ask remains a mandatory human checkpoint. |

The Active/Claimed state became effective when claim PR #465 merged to `main` at merge commit
`94ba2dc5ae282a786ca0001c2856cb3ccd8d927c`. Implementation started from that merge or a later
target-branch commit.

## Activation Checkpoint (2026-09-01)

- Claim PR: #465, exact reviewed head `f7b56662d0c672d5b3d62d463cec394bb29491a4`
- Claim base: `be4fbcfc0c72a6460d009eb4a1b69cd3eade1f8a`
- Exact-head CI: `33513662235` (all required jobs successful)
- Independent review: approved at exact head; issue reconciliation passed
- Merge-time CAS: passed; claim merge commit `94ba2dc5ae282a786ca0001c2856cb3ccd8d927c`
- Implementation authority: active for the bounded I244 work slice; no implementation PR exists yet.

## Activation-Time Nonterminal Inventory And Disposition

This inventory records the claim-stage baseline and is retained as historical execution evidence.
The 2026-09-03 checkpoint below is the current terminal state.

| State | Iterations | Disposition |
|---|---|---|
| Active | None | No additional implementation slice is activated by this candidate. |
| Review | I244 (this slice, claim #465 effective) | Local stable candidate owns only generic shell classifier implementation after ADR-070; exact-head CI and independent permission/security/API review remain pending. |
| Planned | I207, I208 | Preserve steering follow-ups as separate unclaimed work; no permission or shell overlap. |
| Blocked | None with an iteration-owner status | Adjacent PERM-006-D/E authority and other blocked backlog items remain separately owned. |
| Paused | I164 | Superseded; do not restore or absorb its scope. |

I243 is Complete/Closed with ADR-070 Accepted. I241, I242 and the earlier PERM-007 slices are
Complete/Closed; this inventory does not reactivate them or claim authority over Dashboard,
Desktop, release, publication, or PERM-006-D/E work.

## Planned Scope

- Route shell permission requests through the classifier after deterministic deny/explicit-ask
  evaluation while auto mode is active.
- Build an isolated, tool-free classifier request from the exact normalized action, current bounded
  user intent, trusted workspace/remotes/environment context, policy/session revisions, cwd, and
  environment identity.
- Apply accepted hard-deny, soft-deny, allow-exception, explicit-intent, and unknown-result
  precedence.
- Use parser/AST and existing `AccessEvidence` as advisory structural evidence, not per-command
  authorization or a safety proof.
- Do not add or modify authoritative public typed-effect/resource APIs owned by PERM-006-D / Issue
  #56; do not claim closure of PERM-006-E / Issue #57.
- Admit at most `AllowOnce`; recheck the authoritative permission/admission fence before execution.
- Preserve CLI, TUI, embedded Runtime, and MCP permission semantics and provide concise fallback
  reasons when human action is required.

## Exclusions

No blanket shell approval, permanent grants, classifier tool calls, repository-controlled trust
configuration, policy/sandbox override, unrestricted secrets/environment disclosure, Desktop,
release, or publication.

## Acceptance And Validation

- `ls -la` reaches model classification without adding an `ls -la` special case.
- Previously unseen commands can be classified from semantics and context rather than a command
  allowlist; unknown semantics remain human-required.
- Destructive, exfiltrating, privileged, external-target, secret-bearing, and protected-environment
  fixtures are not auto-approved.
- Exact action/cwd/environment/revision binding rejects mutation between assessment and admission.
- Model timeout/error/malformed response/cancellation and lost context fail closed.
- Focused adversarial tests, locked workspace checks, cross-surface tests, governance validators,
  real TUI acceptance, and independent permission/security/API review pass at exact head.

## User-Facing Documentation

- Update README/config reference for auto classifier behavior, fallback and rollback.
- Update `/auto` help and permission UI copy so automatic allow, human-required and hard block are
  distinguishable without exposing model reasoning or secrets.

## Execution Status

I244 is Complete / Closed. The implementation, corrective runtime path, exact-head CI, independent
permission/security/API review, merge-time CAS, and real-TUI acceptance all passed. Issue #462 may
close after this owner-first closeout reaches `main`.

## Completion Evidence

Completion Commit: `4342caa625faac3cea1ef3ff8838a203773c6a9f`,
`63358c8afa1c42083169d50a5b205686c805e21a`,
`6bcc8eca8549d6409fd60111fae17105b2efa72e`

These pre-existing commits contain the original classifier implementation and the corrective
sandbox, permission-session binding, decision-visibility, and tool-placeholder behavior. This
status-only closeout cannot serve as its own completion evidence.

## 2026-09-02 Local Convergence And Issue Synchronization Checkpoint

- Exact implementation base remains `main@94ba2dc5`; stable implementation PR #468 now carries the
  locally converged candidate.
- New open Issues #466 and #467 are reconciled through Refinement/Unclaimed intake owners CAP-001
  and CAP-001-P0, with no selected iteration, effective claim or implementation authority. They do
  not overlap I244's permission authority. Any later root Cargo or Runtime overlap requires fresh
  coordination.
- The implementation carries the exact shell command as untrusted data, bounded current user
  intent, shell structure, canonical cwd/workspace bindings, environment names and an opaque full
  environment binding, permission revisions, fixed classifier policy, and honest unavailable/empty
  configured-remote context. Network effects remain ineligible for automatic approval.
- Published 0.8.0 request/response/enum shapes remain source-compatible. Generic shell context uses
  an additive assessor entrypoint with a fail-closed default, and the legacy permission request
  struct literals are covered by a dedicated compile regression.
- The single authoritative permission evaluation now distinguishes default Ask from configured or
  explicit Ask without re-evaluating policy. Explicit Ask bypasses the assessor; one model
  `AllowOnce` still crosses the existing proposal/revision/admission fence before execution.
- Deterministically known write/mutation and package/network shell classes, secret-like input,
  background requests, composed shell syntax, non-read-only effects, model tool calls, timeout,
  malformed output, context drift and `/auto off` all return to the existing human-required or deny
  path.
- Focused local evidence passes: 22 auto-resolver tests, 12 permission-pipeline tests and the Agent
  final permission-hook regression. Locked workspace check, Clippy with warnings denied, full
  workspace tests, release preflight, both governance validators and `git diff --check` also pass.
  Real TUI acceptance, exact-head CI and independent permission/security/API review remain pending
  before closeout.

## 2026-09-02 Adversarial Re-Convergence Checkpoint

- Implementer-side adversarial review invalidated the prior `a71b4052` stable-candidate evidence:
  known external or sensitive reads, missing or truncated current intent, and secret-bearing intent
  could reach the classifier. CI run `33540527533` and review request comment `5498309737` therefore
  do not authorize merge of the corrected candidate.
- The corrected local tree fails closed before the model for missing, empty, oversized, or
  secret-bearing current intent; known secret/protected targets; URLs; home expansion; Unix
  absolute paths; Windows drive and UNC paths; lexical parent traversal; and canonical symlink
  escape. These checks are command-name independent, while an unfamiliar command with no external
  target such as `python --version` remains model-assessable.
- Focused evidence now passes: 24 `auto_resolver` tests, 12 `permission_pipeline` tests, and the
  dedicated `i244_public_api_compat` test. `./scripts/release_preflight.sh` also passed with the
  repository-pinned toolchain, locked dependency graph, full workspace check/Clippy/tests, both
  governance validators, public-site/installer validation, and CI-classifier tests. Local Cargo
  incremental compilation and debug symbols were disabled only to control disk usage; source,
  features, lockfile, test selection, and release behavior were unchanged.
- Status remains Review / Claimed. The corrected stable candidate was merged through PR #468.
  Real TUI acceptance is deferred to the validation tracker below; no completion evidence is claimed
  until that evidence is recorded.

## 2026-09-02 Post-Merge Closeout Checkpoint

- Implementation PR #468 merged at `4ba1b79f`; implementation evidence commit is
  `4342caa625faac3cea1ef3ff8838a203773c6a9f`.
- Exact-head CI `33571272200` passed 5/5. Independent permission/security/API review APPROVE is
  comment `5502603758`, bound to the exact implementation head and base.
- Merge-time CAS passed before merge. No Dashboard, Desktop, release, publication, Cargo
  manifest, lockfile, or PERM-006-D/E authority was included.
- Local locked tests, public API compatibility, full workspace validation, release preflight,
  both governance validators, and `git diff --check` passed.
- Real TUI acceptance of routine auto-allow, human-required fallback, and `/auto off` remains
  deferred under the open Issue #462 validation record. I244 therefore remains Review / Claimed and
  `Completion Commit` is intentionally unset.

## 2026-09-03 Corrective Merge And Real-TUI Acceptance Checkpoint

- Corrective PR #473 exact head `6bcc8eca8549d6409fd60111fae17105b2efa72e` was based on
  `6f4980b9673a7c11b9b29b8e9858c1d14a18f280` and merged as
  `11c6dc28be0d7c4b692e30a4f385e665dde47543` after merge-time CAS.
- Exact-head CI run `33744161881` completed 5/5 successfully after the known timing-sensitive
  Windows I226 test passed on rerun at the unchanged head. Independent permission/security/API
  review APPROVE is PR #473 comment `5525279657`, bound to the same head and base. The shared
  GitHub account establishes Agent-role separation only, not natural-person identity separation.
- Real-TUI acceptance with a configured provider passed on the exact corrective build: `/auto on`
  visibly reported model `AllowOnce` for `bash pwd`, executed once, and returned the workspace path;
  the same request in a new Session produced the same decision and result.
- A mutating `touch i244-manual-test.tmp` request was visibly excluded from automatic review,
  entered the human path, and was denied with Esc; input recovered and the file remained absent.
  `/auto off` restored the human path, and re-enabling `/auto` restored the visible model decision.
- A provider response containing only `Calling tools...` was rejected as a tool placeholder without
  a structured call; no tool executed and the input state recovered instead of hanging.
- ADR-070's `Explicit Ask` boundary maps specifically to `PermissionRuleSource::Explicit`.
  Configured/default Ask can reach bounded assessment but cannot bypass deny, exact binding,
  sandbox, revision, or admission checks.
- Review notes N2 and N3 are accepted non-blocking limits, not unfinished I244 behavior: report
  formatter variants require synchronized human-readable mappings, and a failed macOS sandbox
  health probe remains conservatively unavailable until process restart rather than retrying
  in-process.
