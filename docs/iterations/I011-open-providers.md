# I011: "Open Providers"

**User can**: Point `talos` at any OpenAI-compatible LLM gateway via config alone
— no Rust changes, no provider recompile. Foundation for a future provider plugin
architecture.

## Status: ACTIVE — first slice (S1) landed 2026-06-02 🛠️

> **Origin.** Drove this iteration to let the team run `talos` against Bailian's
> OpenAI-compatible endpoint using the existing `bailian-token-plan` API key from
> the cluster secret store, without committing any secret to the repo and without
> adding a `Provider::Bailian` variant. The S1 slice is the minimal plumbing to make
> that work; S2 and beyond are documented in
> [docs/proposals/provider-plugin-architecture.md](../proposals/provider-plugin-architecture.md).

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
