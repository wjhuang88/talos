# Iteration I323: Azure OpenAI transport and shape routing

> Document status: Active (proposed; effective only on #918 merge)
> Published plan date: 2026-11-11
> Planned objective: complete PROVIDER-008-C / #914, child of #724.
> Baseline rule: preserve this objective; changed objectives require a new iteration.
> MVP deliverable: executable stateless Azure image/tool Agent rounds using Chat and Responses.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | PROVIDER-008-C Azure transport and per-model Chat/Responses routing only |
| Claimed At | 2026-11-11 |
| Source Issue | #914 |
| Governance Claim PR | #918 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User explicitly authorized continuous #724 implementation, testing, independent review, merge and child closure. Distinct human approver unavailable; independent Agent protocol/auth/API review, exact-head CI, validators and CAS remain mandatory. Shared GitHub identity is not independent natural-person approval. |
| Implementation PR | Not started |
| Last Updated | 2026-11-11 |
| Handoff / Release Condition | Close only after merged implementation evidence and all published acceptance; retain failures and resume gate. |

## Published scope and acceptance

- Reuse existing Chat and Responses message builders, native/Compat/TalosStrict tool modes,
  ordered image handling, reasoning, usage and streaming parsers behind a dedicated Azure transport.
- Add explicit Azure protocol/profile selection, model-local shape and deployment mapping.
  Keep logical model/reasoning identity distinct from deployment; deployment defaults to the selected
  model only when no mapping was configured. No hostname guessing or cross-wire retry.
- Versioned Chat uses `/openai/deployments/{deployment}/chat/completions?api-version={version}`;
  v1 Chat/Responses use `/openai/v1/chat/completions` or `/openai/v1/responses`, with deployment
  in body `model`. Reject unsupported version/body combinations before credential acquisition.
- API keys use `api-key`; explicit externally supplied Entra tokens use Bearer. Token mode reads
  its named environment source at request dispatch, observing changes within the running process environment only; a separate shell cannot rotate it; no automatic login,
  SDK credential chain, CLI subprocess, metadata discovery or inferred account authority.
- Require an explicit HTTPS resource origin (HTTP loopback for controlled fixtures); reject URL
  credentials, query/fragment, ambiguous paths and unsafe deployment/version values. Disable
  redirects; never forward credentials through a model endpoint replacement. Redact secrets from
  Debug, config display, errors and previews. No secret/token in catalog or fixtures.
- Provider/model explicit choices win; omission is distinct from explicit Chat. Only frozen,
  reviewed Azure provider/model shape evidence plus an explicit Azure resource/profile can infer
  routing. npm/name/shape alone cannot establish a credential destination. Unknown and ambiguous
  signals stay conservative; runtime builds never fetch a catalog. Separate review for generated data.
- Controlled HTTP tests run actual permission-gated Agent rounds for each body, including two
  distinguishable ordered images, tools/results, reasoning, usage, errors and terminal EOF behavior.
  Cover shape/deployment/endpoint precedence, old TOML, serialization/schema and visible provenance.
- Existing Chat/Messages/Responses/Gemini paths and B private custody are regression boundaries.
  Full C acceptance requires implementation, not just a decision or fixture description.

## Validation and documentation

Use Rust 1.99.0 (recheck toolchain on resume), --locked check/clippy/workspace tests and standard
`./scripts/release_preflight.sh`; property tests for stream/chunk boundaries and sourced fixtures.
Exact-head native CI and independent protocol/auth/security/API review precede merge CAS.
README en/zh, PROVIDER-PROTOCOLS, RUNTIME-SDK-CONTRACT, ARCHITECTURE and unreleased notes must explain
configuration, deployment aliases, auth limitations and next pre-1.0 minor API migration.
Catalog snapshots need revision/hash/provenance and exhaustive routing audit; never reuse 2026-10-20
counts as current evidence. No live Azure service success or release/tag is assumed.

## Non-goals and dependencies

A is Complete (#806, 5d366db2fbb63301cdce83369fc63fedd1f28c5a); B is Complete (#838 implementation
merge dbcf6163f34d73def923484ad409814fd4a98039, #913 closure 55617ad75c84db75718f287d6c05b84cfe55085e).
Reuse their shared resolver/factory and body adapters without reopening their completed scope.
ADR-103 defines C; ADR-013/021/023/034/050/051/062/093/094 remain applicable.
No Bedrock, cohere, built-in vendor tools, stateful sessions, active probing, general MODEL-014
capability inference or #132 credential-chain closure. Explicit supplied Entra token is not an
implemented automatic OAuth refresh flow. D requires its own claim/iteration/ADR after C.

## Iteration inventory and disposition — main 55617ad7, 2026-11-11

| Iteration | Current state | Disposition |
|---|---|---|
| I249 | Planned | Defer independent dependency-upgrade pilot; no Azure dependency changes authorized here. |
| I277 | Review | Retain deferred human Desktop acceptance with its owner. |
| I302 | Review | Retain Windows attribution/evidence and #717 residual; do not erase historical failures. |
| I318 | Review | Retain physical terminal/IME acceptance; no Azure overlap. |
| I319 | Active | Retain search owner / #907; shared lockfile changes require rebase/locked validation if relevant. |
| I320 | Review | Retain ARCH-035 E0/E1; E2/E3 remain gated by storage work. |
| I322 | Active | Retain STORE-002 C / #912; no storage/host composition edits in C. |
| I164 | Paused | Retain existing pause; not selected. |

All iteration status headers were inspected. I162 is terminal Complete (its header mentions a
historical review outcome), not an additional Review item. I310 and I317 are Complete; stale open
#861 remains a search-owned candidate, not Azure authority. Open PRs #861/#907/#912 and all fetched
remote branches contained no C/D claim or implementation. Matching Azure/Bedrock Issue searches
found only parent/A/B and adjacent #132/#362, not a duplicate child. Explicit user instruction to
complete #724 authorizes this independently owned non-overlapping child alongside other owners.
Parent/derived summary drift about B is repaired in this governance batch; historical records remain.

## Long-running execution contract

- Outcome: C merged and owner-first Complete; then separately claim D before parent #724 closure.
- Work mode: Standard. Deferred validation tracker: None.
- Ordered items: T1 governance claim+ADR acceptance; T2 local transport/config/body reuse;
  T3 controlled Agent/stream/auth tests and catalog audit; T4 user/SDK docs and locked preflight;
  T5 stable exact-head CI plus independent technical/security/API review and CAS;
  T6 merge and evidence-bearing owner/Issue closure. Each item depends on the prior acceptance gate.
- Artifacts: child owner, this iteration, ADR-103, parent summaries, backlog/Board/index, #914/#724.
- Branch/checkpoint plan: governance/provider008c-azure-i323; implementation branch only from its
  merge or later main. Converge locally, preserve reviewed stable stages remotely, append exact
  head/test/failure/next-gate receipts here. Never overwrite old dirty directories or force main.
- Permissions: user authorized edit/test/network/commit/push/review/merge/close within #724.
  No production credentials, paid vendor calls, publication/tag, destructive migration or deployment.
- Resource policy: isolated build with two jobs, debug/incremental disabled. Bounded HTTP deadlines,
  no unbounded retries; diagnose failed evidence before retry. Missing tooling is installed.
- Defaults: current documented v1 for new profiles; explicit versioned Chat compatibility;
  explicit API-key or externally supplied Bearer credentials; fail closed on ambiguity.
- Residual destination: incomplete C rows remain here/#914, D under #724 and shared auth under #132.
  A/B historical failures remain untouched. Missing required review/CI means no merge or Complete.

## Verification and recovery checkpoint — 2026-11-11

Fresh clean clone at 55617ad75c84db75718f287d6c05b84cfe55085e; GitHub main independently agrees.
No C/D effective claim found. Rust installation started in isolated local build directory.
Claim PR #918 is backfilled. Governance validation and independent review are in progress. Implementation
has not begun. Next: address review findings, validate/review the final exact head, CAS and merge before T2.
Completion Commit: Pending.

#915 intake registration was inspected at main 6c81390a; it is the same unclaimed owner, not a competing claim. #918 supersedes intake only upon reviewed merge.

## Implementation checkpoint — 2026-11-12

Claim #918 merged f61a15cd3f0be81489e69f92d374fc8ee41eddec. Implementation worktree
`talos-azure`, branch `feat/provider008c-azure`, starts from main 2f5da14d.
Initial config types and endpoint/deployment validation are local work in progress; not yet wired
into ProviderConfig/factory, no delivered Azure runtime. Focused locked tests started; dependency
index acquisition pending, no test pass claimed. Next: integrate config precedence and shared
Chat/Responses transport dispatch, complete tests/docs/local convergence before candidate push.

Config checkpoint: ProviderConfig.azure merge and model-local azure_deployment/azure_shape are
implemented locally. First integration compile failed E0063 on existing exhaustive config literals;
updated builtins, OpenCode conversion and test fixtures with explicit absent fields. Subsequent
`cargo test --locked -p talos-config azure` passed 3 tests (273 library tests filtered; integration
suites filtered to zero). This is focused config evidence only, not workspace/runtime acceptance.
Runtime/factory routing, catalog activation, SDK migration and full acceptance remain pending.

## Recovery checkpoint — 2026-11-13

Workspace again restored to config-only source at 2f5da14d. Later transport, Agent/HTTP tests,
security fixes and logs from the previous execution are absent. Historical conversation results
are not validation of this recovered tree. Preserve this configuration stage as a recovery commit;
this is not a stable merge candidate or Completion Commit. Restore runtime work only after durable
checkpointing. Review previously found two required boundaries: Azure Chat capability identity must
include resource/deployment; Responses opaque replay must be bound to the same identity. Also
restore stream-error redaction and isolated token/redirect negative tests. C/D and #724 remain open.

Recovery continuation: verified remote backup ef94abf12337a716e83f61eacde8fefa1809416f
has the same tree as local 143dc9c1. Rebased configuration onto main f2e30d99 without
conflicts. Restored model body/deployment resolution, endpoint-override refusal and token-env
name validation. `cargo test --locked -p talos-config azure`: 4 passed, 273 library tests
filtered; all integration suites filtered to zero. This does not validate runtime transport.

Transport recovery: restored private AzureConnection request/header/deployment construction,
redacted previews and endpoint/deployment identity hashing, with a focused unit test authored.
Not yet wired into adapters/factory; redirect policy, replay binding and stream redaction remain
required integration work. Locked provider compilation started; no current test pass claimed.

Recovered from remote 8421f1c6 into talos-azure-restored and rebased onto main a07b466d.
Restored private Chat/Responses Azure dispatch, redirect refusal, HTTP error-body suppression,
redacted previews and Chat capability identity binding. Factory exposure remains intentionally
pending replay binding and stream-error safety integration. Compilation is in progress; this is
an incomplete recovery checkpoint, not a reviewed implementation candidate.
