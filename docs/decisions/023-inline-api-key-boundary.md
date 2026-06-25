# Inline API Key Storage and Display Boundary

## Context

Talos users need to store provider API keys to authenticate with LLM providers.
Two storage mechanisms exist: inline `providers.<name>.api_key` in
`~/.talos/config.toml`, and environment variables via `api_key_env`. I045
briefly introduced `skip_serializing` on the `api_key` field, which silently
dropped keys during `config save()`, causing data loss. The fix reverted
`skip_serializing`, but the security boundary for where keys may appear was
not formally recorded.

Users legitimately want to write `api_key = "sk-..."` directly in their local
config file. The file lives in their home directory (`~/.talos/config.toml`).
At the same time, keys must never appear in CLI display output, logs, debug
prints, exported transcripts, or repository files.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| No secrets in build, source, or distribution | Hard | AGENTS.md #3 | No |
| User may store `api_key` in local `~/.talos/config.toml` | Soft | User requirement (I045) | No |
| `api_key` is `skip_serializing = false` (persisted in TOML) | Soft | I045 data-loss fix | No |
| CLI display, logs, debug, export must mask `api_key` | Hard | Security | No |
| Sample/default config files must not contain real keys | Hard | AGENTS.md #3 | No |

## Reasoning

The simplest approach satisfying all Hard constraints:

1. **Persist** `api_key` as a normal serializable field in `ProviderConfig`.
   This is the I045 fix — `skip_serializing` caused data loss and is rejected.

2. **Mask** `api_key` in every non-file-persistence output surface:
   - `talos config list` — `mask_secrets()` replaces `api_key = ***`
   - `talos config get providers.X.api_key` — `is_secret_key()` prints `***`
   - `talos config set providers.X.api_key=...` — echoes `***` on confirmation
   - `Debug` impls on `ProviderConfig`, `Credentials`, `CredentialResponseData`
     render `***` instead of the raw key
   - Provider HTTP debug snapshots use `redact_secret()` (first4 + **** + last4)

3. **Do not** introduce a `Secret<T>` wrapper type. It would change the field
   type, break serde round-trips, and add complexity for a boundary that custom
   `Debug` impls already enforce.

4. **Do not** replace TOML inline keys with an OS keychain in this iteration.
   That is a future enhancement (credential store, OS keyring integration).

## Decision

- `ProviderConfig.api_key` is a normal `Option<String>` field with
  `skip_serializing = false`. It persists to `~/.talos/config.toml`.
- Custom `Debug` implementations mask `api_key` on `ProviderConfig`,
  `Credentials`, and `CredentialResponseData`.
- `Config` derives `Debug` but its `providers: HashMap<String, ProviderConfig>`
  field cascades through `ProviderConfig`'s masked `Debug`.
- `config_get_dotted` supports `providers.<name>.api_key` but
  `run_config_get` masks the value via `is_secret_key()` before printing.
- Sample config files (`docs/reference/config.reference.toml`) use placeholder
  values (`sk-...`), never real keys.
- `${ENV_VAR}` substitution remains supported for users who prefer env-var
  credentials without hardcoding.

## Reversal Trigger

- If an OS keychain integration (macOS Keychain, Linux secret-service,
  Windows Credential Manager) is implemented, this boundary should be
  revisited to determine whether inline keys are still needed or become
  a fallback only.
- If a new output surface (RPC method, MCP tool, export format) is added
  that could echo config, the masking audit must be re-run.
