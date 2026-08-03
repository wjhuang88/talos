# PROVIDER-003: Dynamic Provider Credentials (GitHub Copilot Device Flow)

Type: Product/API Story
Parent Epic: None (coordinates with MC-001 / I085)
Status: Refinement

## Identity / Goal / Value

Providers whose credentials are not static API keys — the driving example is GitHub Copilot,
which requires an OAuth device-code login and a short-lived exchanged token — currently cannot be
configured in Talos at all. `ProviderConfig` supports only `api_key` / `api_key_env` (ADR-013,
ADR-023). The models.dev catalog lists a `github-copilot` provider with 25 models
(verified 2026-07-03 against the live api.json), so this is a real provider surface users will
ask for, not a hypothetical.

Goal: a user can authenticate a dynamic-credential provider once through an interactive login
flow, and Talos transparently maintains a valid short-lived token for requests afterwards.

## Researched Protocol Facts (2026-07-03)

Verify each against the OpenCode reference implementation
(`packages/opencode/src/auth/github-copilot.ts` in sst/opencode) during design; recorded here
from library research, not yet independently exercised:

1. Device flow start: `POST https://github.com/login/device/code` with `client_id` and
   `scope=read:user` → returns `device_code`, `user_code`, `verification_uri`, `interval`.
2. Poll: `POST https://github.com/login/oauth/access_token` with
   `grant_type=urn:ietf:params:oauth:grant-type:device_code` → long-lived GitHub OAuth token.
3. Exchange: `GET https://api.github.com/copilot_internal/v2/token` with
   `Authorization: token <gh_token>` plus editor identification headers (`Editor-Version`,
   `Editor-Plugin-Version`, `Copilot-Integration-Id`, `User-Agent`) → response carries
   `{ token, expires_at, refresh_in, endpoints.api }`.
4. Chat: `POST https://api.githubcopilot.com/chat/completions` (OpenAI-compatible) with
   `Authorization: Bearer <copilot_token>` plus the same editor headers.
5. Token lifetime is short (`refresh_in` on the order of minutes); clients refresh ahead of
   expiry and re-exchange on 401.

**Unverified / policy-sensitive**: open third-party clients commonly use the client id
`Iv1.b507a08c87ecfe98`. Whether Talos may ship someone else's registered OAuth app client id is
a GitHub ToS question that MUST be resolved in the ADR (options: registered Talos app, known
open client id with documented risk, or user-supplied client id). Do not hardcode before that
decision.

## Scope

- A credential-acquisition mechanism distinct from static `api_key`/`api_key_env`: provider
  config declares an auth kind (e.g. `auth = "github-copilot-device"`); adapters request a
  current token from a credential resolver instead of reading a static string.
- Token cache with expiry in local credential storage (`~/.talos/credentials.toml` shape gains
  `expires_at` / refresh material), masked in every display surface per ADR-023 discipline.
- Interactive login: `/connect` (I085) or a `talos auth login <provider>` CLI path that runs the
  device flow, shows `user_code` + `verification_uri`, and polls to completion.
- Refresh-ahead-of-expiry and 401-triggered re-exchange at the provider boundary.

## Exclusions

- No browser automation (device flow is copy-a-code-by-design).
- No other OAuth providers in the first slice (design must not preclude them).
- No OS keychain (ADR-023 defers it).
- No proxying/reselling semantics — user's own Copilot subscription only.

## Dependencies

- New ADR required before implementation: ADR-013 explicitly limits provider openness to
  schema/config and defers new auth execution flows; ADR-023 covers static inline keys only.
- I085 `/connect` provider setup flow is the natural UX carrier; this story must not fork a
  competing setup UX.
- `github-copilot` catalog entries arrive via the I085 catalog pipeline.

## Decision links and constraints

- ADR-013 (provider config schema boundary) — constraint: schema-only today; acceptance impact:
  new auth kinds need their own decision record.
- ADR-023 (inline api key boundary) — constraint: masked display, normal serialization;
  acceptance impact: token cache must round-trip and mask identically.
- AGENTS.md Hard Constraint #3 — no secrets in build/source/sample configs; client id decision
  recorded in the ADR.

## Uncertainty and validation path

- Client id policy (above) — resolve in ADR.
- Endpoint/response shapes — validate against OpenCode reference and a live device-flow login
  before freezing the schema.

## State/status owners

- Backlog row: `docs/backlog/PRODUCT-BACKLOG.md`; this file owns story state.

## User-facing documentation

- README "Configure A Provider" section gains the dynamic-auth provider path.
- `docs/reference/config.reference.toml` gains the auth-kind field once the ADR fixes its shape.

## Required Reads

- `docs/decisions/013-provider-config-schema-boundary.md`
- `docs/decisions/023-inline-api-key-boundary.md`
- `docs/iterations/I085-model-catalog-modernization.md`
- `crates/talos-config/src/types.rs` (ProviderConfig)
- `crates/talos-config/src/credentials.rs`
- OpenCode reference: `packages/opencode/src/auth/github-copilot.ts` (sst/opencode)

## Acceptance for behavior

- Given a provider configured with the Copilot auth kind and no cached token
  When the user runs the login flow
  Then Talos displays `user_code` + `verification_uri`, completes the device flow, stores the
  exchanged token with its expiry, and never prints the token unmasked.
- Given a cached token past (or near) its refresh window
  When a chat request is issued
  Then Talos refreshes/re-exchanges before the request and the request succeeds without user
  interaction.
- Given `talos config list`
  When credentials for a dynamic provider exist
  Then all token material renders masked (`***`).

## Acceptance for technical/governance work

- [ ] New ADR accepted covering auth-kind schema, client id policy, and storage shape
- [ ] `cargo test --workspace` proves credential cache round-trip + masking
- [ ] README + config.reference.toml updated in the same slice
