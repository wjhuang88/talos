# PROVIDER-003: Dynamic Provider Authentication And Credentials

Type: Product/API Story
Parent Epic: None (coordinates with MC-001 / I085)
Status: Refinement
Source Issue: #132

## Identity / Goal / Value

Talos provider authentication is currently coupled to static `api_key` / `api_key_env` values.
That excludes providers whose request authority is obtained or refreshed dynamically, including:

- GitHub Copilot OAuth device flow followed by provider-token exchange;
- AWS Bedrock request signing with SigV4;
- GCP/Vertex bearer tokens obtained from Application Default Credentials or a bounded command;
- corporate gateways using short-lived or externally acquired tokens;
- bearer-token sessions that must refresh before expiry or after an authentication failure.

Issue #132 is the remote intake for this broader capability. This owner document is the single
planning authority for that Issue; the proposal in the Issue is input, not an accepted schema or
implementation contract.

Goal: Talos can obtain the current authorization material required by a configured provider,
apply it at the provider request boundary, refresh it safely, and never expose secrets through
configuration, logs, diagnostics, Debug output, transcript data, or UI projections.

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
- GCP bearer tokens may come from Application Default Credentials or an external command, but the
  acquisition mechanism, timeout, cache lifetime, stderr handling, and process permissions must
  be explicit and bounded.
- A generic command-token mechanism is code execution and credential handling, not merely config
  parsing. It requires the normal permission, timeout, environment-scrubbing, and secret-redaction
  boundaries.
- Refresh-on-401 must be bounded and idempotent; it must not create an unbounded retry loop or
  replay a non-replayable request body without a sealed request policy.

## Scope

This Story owns the architecture and staged delivery plan for dynamic provider authentication:

1. **Authentication contract and configuration schema**
   - distinguish static API keys, bearer-token acquisition, OAuth device flow, and request-signing
     mechanisms without embedding provider-specific behavior in generic config parsing;
   - preserve backward compatibility for existing static-key providers;
   - define capability discovery and fail-closed behavior for unsupported combinations.
2. **Credential resolver lifecycle**
   - current credential acquisition, expiry, refresh, invalidation, and bounded 401 recovery;
   - single-flight refresh so concurrent requests do not stampede or overwrite newer credentials;
   - explicit cancellation, timeout, and shutdown behavior.
3. **Provider request-boundary integration**
   - attach bearer/header authorization only after request construction is sealed;
   - allow signing mechanisms such as SigV4 to cover the complete final request;
   - keep authentication state out of transcript and model-visible content.
4. **Credential storage and redaction**
   - define which refresh material is durable, where it is stored, and which material must remain
     process-local;
   - mask every secret-bearing display and diagnostic path under ADR-023 discipline;
   - prevent secrets from `Debug`, errors, tracing fields, snapshots, fixtures, and sample config.
5. **Interactive and headless acquisition surfaces**
   - coordinate `/connect`, `talos auth`, and provider setup rather than creating competing flows;
   - distinguish interactive device flow from headless environment/credential-chain resolution.
6. **Provider-specific implementation slices**
   - Copilot OAuth/token exchange;
   - AWS Bedrock SigV4;
   - GCP/Vertex token resolution;
   - bounded external command token acquisition;
   - generic bearer refresh only where the ADR proves a stable cross-provider contract.

## Exclusions At Refinement Stage

- No accepted `AuthMethod` enum or trait shape yet; names and variants in Issue #132 are proposals.
- No hardcoded OAuth client id or undocumented impersonation headers.
- No plaintext token persistence in `config.toml`.
- No browser automation.
- No unbounded command execution, token-refresh loop, or retry-on-401 loop.
- No claim that one generic header callback is sufficient for request-signing providers.
- No implementation inside I169 / PR #131; this synchronization is governance-only.

## Dependencies

- A new ADR is required before implementation. ADR-013 limits current provider openness to the
  schema/config boundary, and ADR-023 covers the existing inline static-key boundary.
- I085 `/connect` provider setup is the natural UX carrier; this Story must not fork a competing
  setup experience.
- TOOL-023 / ADR-057 constraints apply to any external command-token process.
- Permission and secret-handling reviews are mandatory before enabling command execution or
  durable refresh material.
- Provider request sealing and retry semantics must be understood before 401-triggered replay.

## Decision Links And Constraints

- ADR-013 (provider config schema boundary): new executable auth mechanisms require a separate
  accepted decision.
- ADR-023 (inline API-key boundary): masking and storage behavior remain mandatory, but dynamic
  refresh material may need a stricter successor decision.
- ADR-057 / TOOL-023: external command acquisition inherits bounded process, environment, and
  timeout constraints.
- AGENTS.md Hard Constraint #3: no secrets in build scripts, source, fixtures, logs, or sample
  configurations.

## Uncertainty And Validation Path

Before selecting an implementation iteration:

- verify each provider flow against primary documentation and a controlled live or protocol-level
  fixture;
- decide whether authentication is modeled as credential resolution, final-request mutation,
  request signing, or separate capabilities rather than forcing all mechanisms behind one trait;
- define single-flight refresh, cancellation, shutdown, 401 replay, and non-replayable-body rules;
- decide durable storage boundaries and migration behavior;
- perform threat modeling for token theft, command injection, environment leakage, clock skew,
  stale refresh overwrites, and diagnostic disclosure;
- split provider-specific implementation into independently reviewable iterations.

## State / Status Owners

- Backlog row: `docs/backlog/PRODUCT-BACKLOG.md`.
- Remote intake: GitHub Issue #132.
- This file owns Story scope and state.

## User-Facing Documentation

When implementation is selected:

- README provider configuration documents supported authentication mechanisms without exposing
  credentials;
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
- primary GitHub, AWS, and GCP authentication documentation during ADR research

## Acceptance For Architecture And Governance

- [ ] Accepted ADR defines capability boundaries, schema compatibility, storage, refresh,
      cancellation, retries, redaction, and provider-specific signing requirements.
- [ ] Threat model covers token disclosure, command execution, stale refresh races, clock skew,
      retry loops, request replay, and shutdown.
- [ ] Issue #132 is decomposed into bounded implementation slices with explicit dependencies.
- [ ] No implementation begins until one slice has an iteration, exact base, owner, validation
      matrix, and maintainer activation.

## Acceptance For Future Behavior

- Existing static-key providers remain backward compatible.
- Dynamic credentials are acquired and refreshed without exposing secret material.
- Concurrent requests share bounded refresh ownership and cannot replace newer credentials with a
  stale result.
- Authentication failures produce bounded, redacted diagnostics and do not cause unbounded replay.
- SigV4 or equivalent signing covers the exact request that is sent.
- Interactive and headless flows fail closed when required acquisition capability is unavailable.
- Workspace tests cover cache/refresh races, masking, cancellation, shutdown, malformed responses,
  provider rejection, clock boundaries, and retry exhaustion.
