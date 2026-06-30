# I011: "Open Providers"

**User can**: Point `talos` at any OpenAI-compatible LLM gateway via config alone
— no Rust changes, no provider recompile. Foundation for a future provider plugin
architecture.

## Status: Complete (2026-06-30 closure) — S1 delivered; S2 superseded by I015

> 2026-06-30 closure: S1 (OpenAI-compatible `base_url` override) shipped and remains valid. S2
> (provider plugin architecture foundation) was superseded by I015, which delivered the typed
> provider/model schema, built-in defaults, opencode import, and ADR-013 (PROV-001 Complete).
> Dynamic provider loading remains a separate future-ADR item outside I011's scope. I011 is closed.

> 2026-06-06 note: I015 supersedes the S1 top-level `base_url` config shape with named
> provider/model config. Keep this file as historical S1 evidence; use
> `docs/iterations/I015-provider-schema.md` for the current config format.

> **Origin.** Drove this iteration to let the team run `talos` against Bailian's
> OpenAI-compatible endpoint using the existing `bailian-token-plan` API key from
> the cluster secret store, without committing any secret to the repo and without
> adding a `Provider::Bailian` variant. The S1 slice is the minimal plumbing to make
> that work; S2 and beyond are documented in
> [docs/proposals/provider-plugin-architecture.md](../proposals/provider-plugin-architecture.md).
> As of 2026-06-03, I011 is paused so R1 can close I008/I009 review drift and I010 R2 can
> become the next mainline implementation slice. S2 remains valid backlog work and should resume
> only after R1/I010 or an explicit priority change.

## Story Status

| Story | Library | Runtime-integrated | Notes |
|-------|---------|--------------------|-------|
| S1: OpenAI-compatible `base_url` override | ✅ | ✅ | `Config.base_url` + `Config::base_url()` getter + `OPENAI_COMPAT_API_KEY` env var fallback + wired into `talos-cli::build_provider` (OpenAI only). 6 new unit tests in `talos-config`. |
| S2: Provider plugin architecture (foundation) | ⏳ | ⏳ | Backlog-only. Opencode-style `[[providers]]` schema imported from JSON or hand-authored TOML. Depends on S1. See proposal. |

## Plan (executed for S1 on 2026-06-02)

1. **`talos-config`**: add `base_url: Option<String>` to `Config` (`#[serde(default)]`,
   no migration needed for existing users). Add `Config::base_url() -> Option<&str>`
   getter.
2. **`talos-config::Config::api_key()`**: extend the OpenAI provider's resolution
   chain to consult `OPENAI_COMPAT_API_KEY` *after* `OPENAI_API_KEY`. The
   primary env var wins; the new one is a fallback. Update the missing-key error
   message accordingly.
3. **`talos-cli::build_provider()`**: when `Provider::OpenAI` and `config.base_url()`
   returns `Some`, call `OpenAIProvider::with_base_url(...)` on the constructed
   provider. `Provider::Anthropic` is untouched (out of scope; the hard-coded
   `https://api.anthropic.com/...` stays).
4. **Tests** in `talos-config`:
   - `test_base_url_getter`, `test_base_url_default_is_none`, `test_base_url_parsed_from_toml`
   - `test_api_key_from_env_openai_compat`
   - `test_api_key_openai_prefers_explicit_env_over_compat_env`
   - `test_api_key_anthropic_does_not_check_openai_compat_env`
5. **Docs**:
   - `PRODUCT-BACKLOG.md`: new "Iteration I011" section with S1 and S2 stories.
   - `docs/proposals/provider-plugin-architecture.md`: the long-term design.
   - `docs/proposals/reasoning-thinking-field.md`: the model-level "thinking"
     support gap, deferred out of S1.
   - `README.md`: I011 line in the iteration table.
   - `iterations/README.md`: I011 row.

## What was explicitly **not** done in S1

- No new `Provider::*` variant. `provider = "openai"` is the gateway for all
  OpenAI-compatible endpoints. (Adding `Provider::Bailian` etc. is what
  provider-plugin-architecture S2 is for.)
- No `--base-url` CLI flag. The config file is the override surface; CLI flag is
  out of scope.
- No `thinking` / `reasoning` / `budgetTokens` support. Tracked in
  [proposals/reasoning-thinking-field.md](../proposals/reasoning-thinking-field.md).
- No Anthropic `base_url` override. The hard-coded Anthropic URL is fine.
- No new env vars other than `OPENAI_COMPAT_API_KEY`.

## Verification

- `cargo check -p talos-cli -p talos-config`: clean.
- `cargo clippy -p talos-cli --bin talos -p talos-config -- -D warnings`: clean.
- `cargo test --workspace`: 515 passed (was 509; +6 in `talos-config`), 0 failed.
- (No pre-existing test count regression. One flaky `talos-sandbox` test
  occasionally fails under parallel `cargo test`; passes in isolation and on
  re-run. Pre-existing; not from this change.)
- Manual smoke test (planned, not yet executed in this commit): export
  `OPENAI_COMPAT_API_KEY=<bailian-token-plan key>` and run
  `cargo run -p talos-cli -- -p "用中文回答:1+1=?" --model glm-5
  --base-url https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1`
  against Bailian. (Will be run by the user; not part of the CI gate because it
  requires a real key.)

## How a user runs this today (with a Bailian key)

```bash
# one-time: in your shell, not in any tracked file
export OPENAI_COMPAT_API_KEY="<paste your bailian-token-plan key here>"

# ~/.talos/config.toml  (home dir, outside the repo)
provider = "openai"
model = "glm-5"
base_url = "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1"

# run
cargo run -p talos-cli -- -p "用中文回答:1+1=?"
```

The key never touches the working tree, the config file, or git.

## Execution Record (appended during execution per SOP §3a)

### 2026-06-02: S1 implementation lands

- `crates/talos-config/src/lib.rs`: +`base_url` field, +`base_url()` getter,
  extended `api_key()` resolution chain, +6 unit tests.
- `crates/talos-cli/src/main.rs`: `build_provider` now consults `config.base_url()`
  for `Provider::OpenAI`.
- `docs/backlog/PRODUCT-BACKLOG.md`: new I011 section with S1 + S2.
- `docs/proposals/provider-plugin-architecture.md`: new.
- `docs/proposals/reasoning-thinking-field.md`: new.
- Verification: see "Verification" above.

### 2026-06-02: S1 bugfix — `with_base_url` now means "gateway root", not "full endpoint"

Found during real-key E2E with Bailian `glm-5`. The S1 `with_base_url` setter
*stored* the full endpoint URL (default `OPENAI_API_URL` was
`https://api.openai.com/v1/chat/completions`, with the path baked in), so users
following the S1 docs and config example and writing
`base_url = "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1"`
ended up with `talos` POSTing to `…/compatible-mode/v1` (no `/chat/completions`),
and Bailian returned HTTP 400 with
`"message": "url error, please check url！"`. Curl tests in this session
*succeeded* only because the manual curl URL was hand-extended to
`…/v1/chat/completions` — masking the bug.

Aligned the API with the OpenAI SDK convention: `with_base_url` (and the
`Config::base_url` field) now take the **gateway root only**, and the provider
appends `/chat/completions` automatically. A small private helper
`OpenAIProvider::endpoint_url()` composes the final URL and strips a trailing
`/` from the configured base.

Changes in this commit:

- `crates/talos-provider/src/openai.rs`:
  - `OPENAI_API_URL` constant changed from `…/v1/chat/completions` to `…/v1`.
  - New private `const CHAT_COMPLETIONS_PATH: &str = "/chat/completions"`.
  - New private `OpenAIProvider::endpoint_url()` — formats
    `{base}/chat/completions`, trimming a trailing `/` from `base`.
  - `make_request` now calls `self.endpoint_url()` instead of `&self.base_url`.
  - `with_base_url` docstring rewritten to document the "gateway root" semantic
    and the auto-append.
  - `openai_provider_custom_base_url` test updated to use a bare gateway root.
  - +3 new unit tests: `endpoint_url_appends_chat_completions_to_default_base`,
    `endpoint_url_appends_chat_completions_to_custom_base`,
    `endpoint_url_strips_trailing_slash_before_appending`.

Docs/config impact:

- `~/.talos/config.toml` example in this iteration doc needs **no change** —
  `base_url = "https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1"`
  is now correct (it always *was* the intended contract; the runtime was the
  one that was wrong).
- `docs/backlog/PRODUCT-BACKLOG.md` "Reference" table needs no change for the
  same reason.
- `docs/proposals/provider-plugin-architecture.md` future design is unaffected
  — S2 will reuse the same `Config::base_url` field at the schema level.

Verification (this commit):

- `cargo test -p talos-provider --lib`: 35 passed, 0 failed (was 31; +3 new
  URL-composition tests; the renamed `openai_provider_custom_base_url` is
  unchanged in count).
- `cargo test --workspace`: 322 passed, 0 failed (the 1
  `talos-sandbox::test_env_sanitization_disabled_does_not_remove_vars` flake
  passes in isolation and is unrelated to this change).
- E2E with real Bailian `bailian-token-plan` key against `glm-5` and
  `qwen3.6-plus`: full hook chain (TurnStart → BeforeProviderCall →
  OnTextDelta × N → OnTurnEnd → AfterProviderCall → TurnComplete) fires; model
  returns correct Chinese text. The `reasoning_content` field that Bailian
  injects for thinking models is silently ignored (serde `#[serde(default)]`
  on the unused fields) — recorded as a known gap in
  `docs/proposals/reasoning-thinking-field.md`.

Regression risk for other users:

- **OpenAI / Anthropic defaults**: the `OPENAI_API_URL` constant change still
  results in the same effective request URL
  (`https://api.openai.com/v1/chat/completions`) — verified by the new test
  `endpoint_url_appends_chat_completions_to_default_base`. Anthropic provider
  is untouched (out of I011 scope).
- **Anyone who *was* using `with_base_url` with a full endpoint URL** (i.e.
  pre-bugfix users) would now produce a `…/v1/chat/completions/chat/completions`
  double-path and break. Given S1 was the first public exposure of this API
  and the bug was caught the same day, this is acceptable. Note in the
  migration story below.
