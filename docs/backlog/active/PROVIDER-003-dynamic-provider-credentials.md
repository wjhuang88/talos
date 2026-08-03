# PROVIDER-003: Dynamic Provider Authentication And Credentials

| Field | Value |
|---|---|
| Story ID | PROVIDER-003 |
| Type | Architecture / Provider Authentication Epic |
| Priority | P2 |
| Status | Refinement — child Stories are defined but unclaimed and unscheduled |
| Source | [GitHub Issue #132](https://github.com/wjhuang88/talos/issues/132) |
| Selected Iteration | None |
| Depends On | ADR-013; ADR-023; ADR-057 / TOOL-023; I085 provider setup; sealed provider-request boundary |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #132 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-03 |
| Handoff / Release Condition | Materialize and independently select one bounded child Story before product-code work. |

## Identity / Goal / Value

Talos provider authentication is currently coupled to static `api_key` / `api_key_env` values.
That excludes providers whose request authority is obtained, refreshed, or calculated dynamically,
including:

- GitHub Copilot OAuth device flow followed by provider-token exchange;
- AWS Bedrock request signing with SigV4;
- GCP/Vertex bearer tokens obtained from Application Default Credentials;
- corporate gateways using short-lived or externally acquired tokens;
- bearer-token sessions that must refresh before expiry or after an authentication failure.

Issue #132 is the remote intake for this broader program. PROVIDER-003 is the architecture and
decomposition Epic, not one testable implementation Story. The proposal in the Issue is input,
not an accepted schema, trait design, provider contract, or implementation authorization.

Goal: define and deliver bounded authentication capabilities so Talos can obtain the current
authorization material required by a configured provider, apply it at the final provider-request
boundary, refresh it safely, and never expose secrets through configuration, logs, diagnostics,
Debug output, transcript data, or UI projections.

The initial researched example remains GitHub Copilot. The models.dev catalog lists a
`github-copilot` provider with 25 models (verified 2026-07-03 against the live api.json), so this is
a real provider surface rather than a hypothetical extension point.

## Researched Protocol Facts (2026-07-03)

Verify each fact against primary provider documentation and exercised fixtures before accepting
an ADR. Existing OpenCode behavior may be used as implementation research, not as authority.

### GitHub Copilot device flow

1. Device flow start: `POST https://github.com/login/device/code` with `client_id` and
   `scope=read:user` returns `device_code`, `user_code`, `verification_uri`, and `interval`.
2. Poll: `POST https://github.com/login/oauth/access_token` with
   `grant_type=urn:ietf:params:oauth:grant-type:device_code` returns a GitHub OAuth token.
3. Exchange: `GET https://api.github.com/copilot_internal/v2/token` with the OAuth token and
   required client-identification headers returns a short-lived Copilot token and expiry data.
4. Chat requests use the exchanged token and required provider headers.
5. Clients refresh ahead of expiry and reconcile authentication failures without logging token
   material.

**Unverified / policy-sensitive**: open third-party clients commonly use the client id
`Iv1.b507a08c87ecfe98`. Whether Talos may ship another application's registered OAuth client id is
a policy and ToS question that MUST be resolved in the ADR. No client id may be hardcoded before
that decision.

### AWS, GCP, and external token acquisition

- AWS SigV4 signs the complete request, so it cannot be modeled as a static header value detached
  from method, URI, headers, payload hash, region, service, credentials, and signing time.
- GCP bearer tokens may come from Application Default Credentials, with explicit acquisition,
  timeout, cache-lifetime, cancellation, and error-redaction behavior.
- A generic command-token mechanism is code execution and credential handling, not merely config
  parsing. It requires the normal permission, timeout, environment-scrubbing, and secret-redaction
  boundaries.
- Refresh-on-401 must be bounded and idempotent; it must not create an unbounded retry loop or
  replay a non-replayable request body without a sealed request policy.

## Bounded Child Story Decomposition

These identities reserve separate review, claim, rollback, iteration, and completion boundaries.
They are not implementation authorization. Before a child becomes Ready or Active, create its own
owner document with acceptance tests, collaboration claim, exact dependencies, and selected
iteration.

| Child Story | Bounded outcome | Depends On | Explicit non-goal |
|---|---|---|---|
| PROVIDER-003-A | Authentication capability ADR, threat model, compatibility rules, and accepted decomposition. | ADR-013; ADR-023; provider request architecture research | No provider product code or credential persistence. |
| PROVIDER-003-B | Credential resolver lifecycle: acquisition result, expiry, single-flight refresh, invalidation, shutdown, storage boundary, and redaction. | PROVIDER-003-A | No provider-specific OAuth, ADC, SigV4, or command execution. |
| PROVIDER-003-C | Final-request authorization contract and bounded authentication-failure replay policy, including non-replayable bodies. | PROVIDER-003-A; sealed request-plan/request boundary; PROVIDER-003-B for bearer refresh | No provider-specific acquisition flow. |
| PROVIDER-003-D | GitHub Copilot device flow, OAuth-token exchange, interactive setup, and provider-specific headers. | PROVIDER-003-A/B/C; I085 setup UX; approved OAuth client-id policy | No AWS, GCP, or generic command-token behavior. |
| PROVIDER-003-E | AWS Bedrock credential-chain resolution and SigV4 signing of the exact final request. | PROVIDER-003-A/C; accepted signing/time/clock-skew rules | No bearer-token or OAuth abstraction claim. |
| PROVIDER-003-F | GCP/Vertex Application Default Credentials and bounded bearer-token resolution. | PROVIDER-003-A/B/C | No arbitrary external command execution. |
| PROVIDER-003-G | Bounded external command-token acquisition with permission, timeout, environment, stderr, cancellation, and redaction policy. | PROVIDER-003-A/B/C; TOOL-023; ADR-057; permission review | No implicit shell, unbounded process, or generic provider signing. |

Dependency order is not a mandate to implement every child. PROVIDER-003-A must be accepted first.
After that, select only one independently reviewable child at a time; provider-specific children
must not silently broaden the shared contracts owned by A/B/C.

## Epic Scope

This Epic owns only program-level architecture and decomposition:

1. authentication capability taxonomy and backward-compatible configuration boundary;
2. shared credential lifecycle, storage, refresh, cancellation, and redaction questions;
3. final-request mutation/signing and bounded replay questions;
4. interactive versus headless acquisition boundaries;
5. child ownership, dependency order, and residual placement;
6. conformance requirements across provider-specific implementations.

Implementation behavior belongs to the selected child Story, not directly to PROVIDER-003.

## Exclusions At Refinement Stage

- No accepted `AuthMethod` enum or trait shape yet; names and variants in Issue #132 are proposals.
- No hardcoded OAuth client id or undocumented impersonation headers.
- No plaintext token persistence in `config.toml`.
- No browser automation.
- No unbounded command execution, token-refresh loop, or retry-on-401 loop.
- No claim that one generic header callback is sufficient for request-signing providers.
- No implementation inside I169 / PR #131; this synchronization is governance-only.
- No child may be claimed, activated, or marked complete through this Epic document alone.

## Dependencies

- PROVIDER-003-A requires a new ADR before any implementation. ADR-013 limits current provider
  openness to the schema/config boundary, and ADR-023 covers the existing inline static-key
  boundary.
- I085 `/connect` provider setup is the natural UX carrier; provider-specific children must not
  fork a competing setup experience.
- TOOL-023 / ADR-057 constraints apply to external command-token acquisition.
- Permission and secret-handling reviews are mandatory before enabling command execution or
  durable refresh material.
- Provider request sealing and retry semantics must be accepted before 401-triggered replay or
  SigV4 integration.

## Decision Links And Constraints

- ADR-013 (provider config schema boundary): executable authentication mechanisms require a
  separate accepted decision.
- ADR-023 (inline API-key boundary): masking and storage behavior remain mandatory, but dynamic
  refresh material may need a stricter successor decision.
- ADR-057 / TOOL-023: external command acquisition inherits bounded process, environment, and
  timeout constraints.
- AGENTS.md Hard Constraint #3: no secrets in build scripts, source, fixtures, logs, or sample
  configurations.

## Uncertainty And Validation Path

Before selecting an implementation child:

- verify each provider flow against primary documentation and a controlled live or protocol-level
  fixture;
- decide whether authentication is modeled as credential resolution, final-request mutation,
  request signing, or separate capabilities rather than forcing all mechanisms behind one trait;
- define single-flight refresh, cancellation, shutdown, 401 replay, and non-replayable-body rules;
- decide durable storage boundaries and migration behavior;
- perform threat modeling for token theft, command injection, environment leakage, clock skew,
  stale refresh overwrites, and diagnostic disclosure;
- materialize the selected child owner document and choose a bounded iteration.

## State / Status Owners

- Epic scope, decomposition, and status: this file.
- Backlog row: `docs/backlog/PRODUCT-BACKLOG.md`.
- Remote intake: GitHub Issue #132.
- Child implementation status: the future child owner document, never this Epic by implication.

## User-Facing Documentation

When a child implementation is selected:

- README provider configuration documents only supported authentication mechanisms and never
  exposes credentials;
- `docs/reference/config.reference.toml` documents only accepted schema;
- setup/auth commands document interactive versus headless behavior and recovery paths.

## Required Reads

- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `docs/decisions/057-windows-powershell-process-boundary.md`
- `docs/iterations/I085-model-catalog-modernization.md`
- `docs/backlog/active/TOOL-023-cross-platform-shell-and-timeout.md`
- `crates/talos-config/src/types.rs`
- `crates/talos-config/src/credentials.rs`
- provider request construction and retry paths in `talos-provider` / `talos-agent`
- primary GitHub, AWS, and GCP authentication documentation during ADR and child research

## Acceptance For Epic Governance

- [ ] PROVIDER-003-A owner document and ADR/threat-model review are accepted before product code.
- [ ] Every selected child has its own owner, collaboration claim, iteration, exact base, validation
      matrix, and residual destination.
- [ ] Shared capability changes occur only in A/B/C and are not smuggled through a provider-specific
      child.
- [ ] Issue #132 remains open while any selected child or separately owned residual remains.

## Acceptance For Epic Completion

- Selected children are Complete with provider-specific and cross-provider conformance evidence.
- Existing static-key providers remain backward compatible.
- Dynamic credentials are acquired and refreshed without exposing secret material.
- Concurrent requests share bounded refresh ownership and cannot replace newer credentials with a
  stale result.
- Authentication failures produce bounded, redacted diagnostics and do not cause unbounded replay.
- SigV4 or equivalent signing covers the exact request that is sent.
- Interactive and headless flows fail closed when required acquisition capability is unavailable.
- Unselected provider mechanisms remain Refinement rather than being implied complete.

## Residual Destination

Partial implementation remains owned by the corresponding child Story. New provider mechanisms,
credential stores, OS keychain support, browser automation, or broader request-replay semantics
require separate owners or an explicit update to the decomposition before implementation.
