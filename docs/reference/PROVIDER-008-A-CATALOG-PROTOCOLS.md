# PROVIDER-008-A: Protocol-Only Catalog Projection

Carrier: endpoint data candidate on code PR #792 (`f43cd990`), with code-first merge required.
The prior protocol-only data PR #775 merged as `6af8654c`. Historical stacked/Draft checkpoints
remain below. This new data candidate is not independently activated; child A remains Review / Claimed.

Current source: archived [2026-10-27 snapshot](../../scripts/fixtures/provider008a-models-20261027/README.md)
from https://models.dev/api.json. Earlier dated checkpoints below describe superseded inputs. This is catalog evidence, not a live endpoint
probe or a claim that vendor-specific authentication/transports have been implemented.

- Snapshot SHA-256: `4e97268dc7621205179df996f68a2c17a32284997d291cb8c6a29c43ffc33c74`.
- Snapshot bytes: 5,955,477; 232 upstream providers.
- Preserved packaged models: 6,474, with the same IDs, providers, limits, pricing, capabilities,
  release dates and variants. Provider metadata and API roots are byte-for-byte unchanged.
- Current candidate protocol keys: 3,857 Chat, 65 Anthropic Messages, 133 Responses (4,055 total).
- Added reviewed model endpoint records: 45 (14 Responses / 27 Messages / 4 Chat).
- Withheld: 918 absent, 1,353 unsupported transports (including four WSS/Realtimes), seven
  unreviewed model endpoints (Copilot Messages), and 141 provider-root drift records.

The projection matches only exact `(provider, model)` identities already in the package. It uses the
shared `protocol_signal.rs` map: explicit `responses`/`completions` shape first, otherwise exact
supported SDK package defaults. Host and model SDKs must belong to the three already implemented
wire families. Azure, Bedrock, Vertex and unknown transports are not admitted just because an item
says `shape=responses`. Different model endpoints require an exact reviewed provider/root/wire pair;
unreviewed destinations remain withheld. Upstream provider-root drift is also withheld in this stage;
no old endpoint is silently reinterpreted using a newer root's protocol evidence.

At runtime, overriding a built-in provider's endpoint to a different root suppresses that built-in
catalog evidence. User-explicit model/provider protocols still win. Unique custom-provider model
matching remains the published ADR-093 rule. A model-level Messages choice on a gateway root is
normalized once to `/messages` by the shared adapter factory; explicit full Messages endpoints are
preserved.

Normal builds remain offline. Full `BUILD_MODELS=1` refreshes use the same shared map and admission
checks but may separately update broader upstream metadata; those changes are not included here.
The one-time protocol projection was checked by parsing both datasets and removing the optional
model `protocol` and `api_base_url` routing fields: every remaining model field and every provider
record matched the old package. Generation/routing tests exercise transport refusal, endpoint refusal, explicit overrides
and packaged catalog selection. No recorded network model responses or vendor credentials were used.

## Responses Signal Ledger

All rows below passed the supported-host/model-SDK and unchanged-endpoint checks. An absent shape
uses the exact `@ai-sdk/openai` default; these are data signals, not name-based guesses.

| Provider | Packaged model | Model SDK / inherited SDK | Shape |
|---|---|---|---|
| github-copilot | `gpt-5.3-codex` | `@ai-sdk/openai` | (SDK default) |
| github-copilot | `gpt-5.4` | `@ai-sdk/openai` | (SDK default) |
| github-copilot | `gpt-5.4-mini` | `@ai-sdk/openai` | (SDK default) |
| github-copilot | `gpt-5.4-nano` | `@ai-sdk/openai` | (SDK default) |
| github-copilot | `gpt-5.5` | `@ai-sdk/openai` | (SDK default) |
| github-copilot | `gpt-5.6-luna` | `@ai-sdk/openai` | (SDK default) |
| github-copilot | `gpt-5.6-sol` | `@ai-sdk/openai` | (SDK default) |
| github-copilot | `gpt-5.6-terra` | `@ai-sdk/openai` | (SDK default) |
| github-copilot | `grok-4.5` | `@ai-sdk/openai` | (SDK default) |
| github-copilot | `grok-4.6` | `@ai-sdk/openai` | (SDK default) |
| github-copilot | `mai-code-1.1-flash` | `@ai-sdk/openai` | (SDK default) |
| meta | `muse-spark-1.1` | `@ai-sdk/openai` | (SDK default) |
| meta | `muse-spark-1.2` | `@ai-sdk/openai` | (SDK default) |
| meta | `muse-spark-1.2-contributor` | `@ai-sdk/openai` | (SDK default) |
| neosmith | `neosmith.intelligent-basic` | `@ai-sdk/openai` | (SDK default) |
| neosmith | `neosmith.intelligent-maestro` | `@ai-sdk/openai` | (SDK default) |
| neosmith | `neosmith.intelligent-pro` | `@ai-sdk/openai` | (SDK default) |
| neosmith | `neosmith.neolite` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-4.1` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-4.1-mini` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-4o` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-4o-mini` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5-mini` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5-nano` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.1` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.1-codex-max` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.1-codex-mini` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.2` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.2-codex` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.3-codex` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.4` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.4-mini` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.4-nano` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.4-pro` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.5` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.6-luna` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.6-sol` | `@ai-sdk/openai` | (SDK default) |
| ofox | `openai/gpt-5.6-terra` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-4` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-4-turbo` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-4.1` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-4.1-mini` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-4.1-nano` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-4o` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-4o-2024-05-13` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-4o-2024-08-06` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-4o-2024-11-20` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-4o-mini` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5-mini` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5-nano` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5-pro` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.1` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.2` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.2-pro` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.3-codex` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.3-codex-spark` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.4` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.4-mini` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.4-nano` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.4-pro` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.5` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.5-pro` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.6` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.6-luna` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.6-sol` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-5.6-terra` | `@ai-sdk/openai` | (SDK default) |
| openai | `gpt-realtime-2.1` | `@ai-sdk/openai` | (SDK default) |
| openai | `o1` | `@ai-sdk/openai` | (SDK default) |
| openai | `o1-pro` | `@ai-sdk/openai` | (SDK default) |
| openai | `o3` | `@ai-sdk/openai` | (SDK default) |
| openai | `o3-mini` | `@ai-sdk/openai` | (SDK default) |
| openai | `o3-pro` | `@ai-sdk/openai` | (SDK default) |
| openai | `o4-mini` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `anthropic/claude-haiku-4-5` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `anthropic/claude-opus-4-5` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `anthropic/claude-opus-4-6` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `anthropic/claude-opus-4-7` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `anthropic/claude-sonnet-4-5` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `anthropic/claude-sonnet-4-6` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `deepseek/deepseek-v4-flash-0731` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `google/gemini-3-flash-preview` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `google/gemini-3.1-pro-preview` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `moonshot-ai/kimi-k2.7-code` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `moonshot-ai/kimi-k3` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `nvidia/nemotron-3-super-120b-a12b` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `openai/gpt-5-mini` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `openai/gpt-5.1` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `openai/gpt-5.2` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `openai/gpt-5.4` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `openai/gpt-5.5` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `perplexity/sonar` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `xai/grok-4-1-fast-non-reasoning` | `@ai-sdk/openai` | (SDK default) |
| perplexity-agent | `xai/grok-4.6` | `@ai-sdk/openai` | (SDK default) |
| sakana | `fugu` | `@ai-sdk/openai-compatible` | responses |
| sakana | `fugu-ultra` | `@ai-sdk/openai-compatible` | responses |
| sakana | `fugu-ultra-20260615` | `@ai-sdk/openai-compatible` | responses |
| vivgrid | `gpt-5.1-codex` | `@ai-sdk/openai` | (SDK default) |
| vivgrid | `gpt-5.1-codex-max` | `@ai-sdk/openai` | (SDK default) |
| vivgrid | `gpt-5.2-codex` | `@ai-sdk/openai` | (SDK default) |
| vivgrid | `gpt-5.3-codex` | `@ai-sdk/openai` | (SDK default) |
| vivgrid | `gpt-5.6-luna` | `@ai-sdk/openai` | (SDK default) |
| vivgrid | `gpt-5.6-sol` | `@ai-sdk/openai` | (SDK default) |
| vivgrid | `gpt-5.6-terra` | `@ai-sdk/openai` | (SDK default) |
| vivgrid | `kimi-k3` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5-codex` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5.1` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5.1-codex` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5.1-codex-mini` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5.2` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5.2-codex` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5.2-pro` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5.3-codex` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5.4` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5.4-mini` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5.4-nano` | `@ai-sdk/openai` | (SDK default) |
| zenmux | `openai/gpt-5.4-pro` | `@ai-sdk/openai` | (SDK default) |

## Projection Integrity

- Old catalog bytes SHA-256: `aa173e5361159000dd22a6a6bfa8b8cfa08e42e37961b736393763db5c8d02ed`.
- Current catalog bytes SHA-256: `d339625ab3d55ca47da57f526bd18d9a0e884329d474feefae52ff6fbd8ce1f3`.
- Historical 2026-10-26 projection bytes SHA-256: `b742393abbb56a8fd65230d7b0865b2d1deda31f65bf77c70def571e5889d0aa`.
- Semantic SHA-256 after removing model protocol keys: `acc90a95ff8f87fbf131788548066575d02cd8e1ccb8f3545a9c62e01712947d`.

The semantic digest serializes parsed TOML with Python JSON `sort_keys=True` and compact separators;
it is identical to the old catalog. The pinned old bytes remain reachable through archived #749
head `5acd2161`. New-source and generated-data CI are independent gates; merging this data does not
close I306 or admit withheld endpoint/transport cases.

## Candidate Validation And Residuals

Code dependency: #749 at `7fd35dcf1e66317a7523f5cfdf64168079091207`, rebased on main `2789b97b`.
Four new snapshot acceptance cases pass: actual per-model wire selection/source, unconfirmed-endpoint
refusal with explicit override, unsupported transport/root-drift refusal, and exact projected counts.
Focused strict Clippy for `catalog_protocol_snapshot` passes. Model/protocol data and provider,
configuration, CLI/binary image-tool integration cases passed in the isolated worktree run.

The first data-worktree build reused another worktree's target and failed with E0463/colliding
StableCrateId; it is not acceptance evidence. All later data validation uses this worktree's own
target directory. The initial isolated four-package test run recorded **1,653 passed / 1 failed**,
stopping in existing `highlight::tests::unavailable_grammar_keeps_existing_markdown_fallback` at
`highlight("rust", ...).is_some()`. Its exact focused rerun passed (1 test). A second full run again
recorded **1,653 passed / 1 failed** at the same assertion. Preserve both full failures; no full-suite
success or root-cause closure is claimed. The assertion and production highlighting code are
unchanged. Treat this as an open validation blocker for the data candidate, not proof that the
catalog caused or did not cause the behavior. Fresh unfiltered remote CI and full acceptance remain
required; do not merge or activate the data from the passing four snapshot cases alone.

## Terminal-Lifecycle Base Synchronization — 2026-10-26

The code dependency is now #749 at `bb31d0125d63841165ce8cbab04cb729ace0e72f`, which ends
Responses HTTP reading at the first semantic terminal instead of waiting for EOF/idle timeout.
The combined candidate preserves every catalog byte and snapshot-test byte from the previous data
head `bb705cc0e53ab07791581f5915a6fd3f3ce75ebf`; this synchronization changes no admitted/withheld
counts, provider roots, pricing, limits or capabilities. Only this ledger adds new data-PR prose.

Prior data CI3458 (run 40327420437) passed all six jobs. Its full macOS log explicitly records
`highlight::tests::unavailable_grammar_keeps_existing_markdown_fallback ... ok`; the earlier repeated
local failures remain observations with no inferred root-cause closure. The new code/base/head
requires fresh unfiltered CI; that prior run is historical evidence for the earlier data head.

Combined-source locked validation: all **4 catalog snapshot cases**, **3 Responses HTTP cases** and
**25 transport/semantic/property cases** passed, zero failed/ignored/filtered. This includes both
completed/failed outcomes with an intentionally unclosed HTTP body. The checkout switched branches
after code validation finished and used that same checkout's own target; no independent worktrees
shared build artifacts. An initial snapshot invocation lost its process/result handle after
`running 4 tests`, so it supplies no verdict. A complete unfiltered rerun of those same four cases
recorded exit 0; both later test invocations retain explicit exit status.

The code candidate separately passed all 347 provider tests and the real binary Responses cases,
plus workspace check/strict Clippy/governance gates; its standard local preflight still stops in
inherited MCP git_prepared at 26 passed / 21 failed under the recorded socket/Git-binding conditions.
This combined-source focused run is not a full local workspace pass. Recorded upstream SSE
provenance, independent API review, distinct-endpoint residual disposition and #749-first merge /
retarget sequencing remain open. Keep Draft; no completion, activation or release is claimed.


## Independent-Review Correction Base Synchronization — 2026-10-27

Code dependency #749 is now `17a3e64a00daaabc163e20e41a2799e9f79bada9`, integrated with main
`33fad1b8db85cd6f319247a2ed25bff6bd495f94` including merged #777/#778. Its complete source tree is
`eb4d581b2a127318ad78cbe3e25c289c90e4fffa`. Independent read-only Agent exact-head technical
review APPROVE closes all three original bb31d012 blockers: actual one-shot tool image continuation,
explicit Compat/Strict tool modes, and representative parallel plus sourced SDK stream replay.
This is shared-account Agent review, not distinct-human approval. Code CI3479 is still running at
this checkpoint; neither old green CI nor this review substitutes for its final machine verdict.

The combined data candidate preserves the catalog blob `cf604fdb9a62c8a93a0bd53476a4ee7cd9e3e486`
and snapshot-test blob `139a39337451b0cf32b81ec208d2d6164f7486e5` exactly. Relative to the code
candidate only the same three files differ; only this ledger adds new data-specific prose. All
6,474 models, 4,016 optional signals, withheld counts, provider roots, pricing, limits and capabilities
remain unchanged from the previously reviewed projection.

Combined-source pinned locked validation passed **41 tests across four groups**: 4 catalog snapshot,
4 actual CLI Agent, 5 HTTP and 28 transport/semantic/property. Zero failed/ignored/filtered; all
processes recorded exit 0. Includes one-shot read_image without --attach, both explicit text modes,
MaxTokens preservation, 503 no-repost, upstream persisted reasoning/error fixtures, parallel calls
and semantic-terminal HTTP release. Formatting and diff checks passed. The same physical checkout
switched branches only after code validation finished and reused its own target; no independent
worktrees shared artifacts.

The code source separately passed 1,799 core/config/provider/CLI/TUI tests, strict checks and both
external runtime SDK modes. Its complete local standard preflight still stops at inherited MCP
26 passed / 21 failed; this focused data run is not a full local workspace pass. Earlier TUI failures,
CI3463 timing failure and cancelled retries remain historical evidence without inferred root-cause
closure. The integrated #778 clock fixture fix passed its own eight-job CI3470 before merge; the
new code/data heads each require fresh unfiltered CI.

Distinct model endpoint routing remains unimplemented and withheld; it is not delivered through
protocol metadata. Keep Draft until current-head validation and remaining owner acceptance/residual
disposition. Merge #749 first, then retarget/reconcile #775 onto main and revalidate. No completion,
default activation through this unmerged PR, or release is claimed.


## Source Recovery And Exhaustive Reprojection — 2026-10-27

Independent exact-head review of data head `3aeee187` approved its structural integrity, fixed-blob
identity, all 6,474 models and non-protocol semantics, but returned REQUEST CHANGES on source
reproducibility: the original 2026-10-26 api.json was missing. Its hash alone could not establish that
all admitted/withheld rows met the stated rules. Earlier source/count statements remain historical;
no independent reconstruction of the missing old input is claimed.

A fresh 2026-10-27 download differs from the old hash, so it is archived under its own date/hash at
`scripts/fixtures/provider008a-models-20261027/api.json.gz` with the upstream MIT notice. It preserves
all original JSON bytes after decompression. `python3 scripts/audit_provider008a_projection.py`
works offline, checks the raw-source SHA256, verifies the original non-protocol semantic digest and
exhaustively compares each packaged protocol key to its source admission. Standard release preflight
runs this audit before Cargo validation; runtime and normal builds remain unchanged and offline.

Reprojection withdraws exactly six Chat hints: aiand/deepseek-ai/deepseek-v4-flash,
aiand/moonshotai/kimi-k2.7-code, aiand/zai-org/glm-5.2,
chutes/Qwen/Qwen3-235B-A22B-Thinking-2507-TEE, chutes/Qwen/Qwen3-32B-TEE, and
kilo/arcee-ai/trinity-large-thinking. They are now absent from upstream; their packaged models stay.
All 119 Responses rows in the signal ledger remain identical. Current counts are **4,010 admitted**
(3,853 Chat / 38 Messages / 119 Responses), **918 absent / 1,349 unsupported transports / 56 distinct
model endpoints / 141 provider-root drift** withheld. Categories are exclusive: absent, unsupported
transport, provider-root drift, distinct model endpoint, then shape admission. No unsupported shape
rows occur. Original old projection counts remain only in dated historical checkpoints.

All non-protocol catalog fields and provider metadata are unchanged. Current catalog SHA256 is
`d339625ab3d55ca47da57f526bd18d9a0e884329d474feefae52ff6fbd8ce1f3`. The frozen count test changes
only the Chat expected count. Source-archive/audit files intentionally expand this PR's former
three-file boundary to make complete input-to-output review possible.

### Validation And Recovery Record

Before environment maintenance, the new-source five-package core/config/provider/CLI/TUI run passed
**1,803 tests across 51 groups**, zero failed/ignored/filtered, exit 0. Independent read-only Agent
review separately recomputed every model from the archived JSON without using the submitted audit
function, found zero differences, verified all 119 Responses ledger rows and the upstream MIT blob,
and returned working-source APPROVE. Source reproduction, gzip/raw hashes, full non-protocol digest,
format and whitespace checks passed.

The first complete local preflight passed static checks, governance, audit, workspace check and strict
workspace Clippy, then failed test compilation with colliding StableCrateId values for talos_config.
The local project build outputs were cleaned and a serial preflight started; its final verdict/log
was lost during automated environment maintenance. It is excluded as acceptance evidence. The prior
inherited MCP 26 passed / 21 failed record remains, and no green local full workspace claim is made.
Published source objects survived: the restored gzip/raw/catalog hashes exactly match the reviewed
new-source bytes above. Restored source requires final exact-head review and fresh CI; CI3480 on
`3aeee187` is successful but predates this source archive stage. Keep Draft, code-first sequencing
and all remaining owner/residual gates; no completion or release is claimed.


### Current Agent Baseline Integration — 2026-10-27

Merge-time inspection found main advanced to `da280c8a77b25798e59df37f3bdc2bf2fafce1c9`, with
I308 invocation authorization/recovery changes. The combined data stage now includes code candidate
`1388874ddff8c70de3ae651f0153cc39003a18d8`, complete code tree
`4143c85625d4507c4c2069055f8aba4cd45eebc3`, equivalent to local `e020107b`. Independent read-only
Agent source review approves the baseline integration: Responses/provider/config and CLI scenario
sources stay identical to previously approved `17a3e64a`, and inherited I308 security code matches
main exactly. Actual current-base runtime checks and fresh CI remain separate required evidence.

The restored archive, audit script, updated snapshot test and catalog exactly match the independently
reviewed source-recovery stage. Data relative to code is limited to the catalog, snapshot tests,
ledger, offline audit, source gzip/README/MIT notice and preflight audit entry. This is an intentional
eight-file provenance/data boundary. No different model endpoint or unsupported transport is admitted.
Final validation and exact published-head review are recorded in the PR; code-first merge, retarget
onto main and merge-time CAS sequencing remain required. Status stays Active/Claimed, with no
completion or release claim.


### Code Merged; Data Retargeted To Main — 2026-10-27

Code PR #749 merged as `60e27eb6653b3a8fef638a2b9a7294a036b78db0` after exact-head CI3498's eight
successful jobs, independent Agent provider/API review, current-main #784 non-overlap review and
Single-maintainer authorization/CAS. Its merge commit preserves `1388874d` ancestry. Model-specific
endpoint routing remains unimplemented/withheld under parent #724; neither code carrier merge nor
this data stage marks #748/I306 Complete or publishes a release.

Data #775 is now retargeted from the code branch to main. The new stable candidate integrates that
already-merged main commit, including #784 Fetch URL Auto, without conflicts. Relative to main the
same eight data/provenance files remain. Every catalog, test, archive, license, README, audit-script
and preflight-entry byte from `0c1abf79` is unchanged; only this ledger appends new integration facts.
The offline audit was repeated on the combined tree: all 6,474 models match the frozen source,
4,010 hints (3,853 Chat / 38 Messages / 119 Responses), unchanged withheld categories and identical
non-protocol semantic digest. No fresh upstream download or new protocol inference is performed.

CI3499 was successful on the prior stacked head, but is historical for the new main integration.
At reconciliation start, environment maintenance had removed the Rust toolchain/build cache. The
first governance invocation could not read SQLite dependency metadata because cargo was missing;
the pinned Rust 1.99.0 toolchain was then restored for that check. No new local Cargo test verdict
is claimed for this reconciliation. Earlier exact 1,803-pass execution
remains attributed to `0c1abf79`. The new published head requires its own full CI against the actual
combined checkout, and independent final head/base/tree binding, before data merge. Retarget alone
and rerunning the old workflow are not substituted for the new integration evidence.


### Final Data Merge — 2026-10-28

Exact data head `76647990c0d8dfa3fad4895024bc6376f62ac6a2` passed CI3509/run40537545803,
all eight jobs including full release preflight, native Windows workspace, Linux/Windows Desktop
and both governance validators. Independent final-object review APPROVE and merge-time CAS passed.
Data #775 merged as `6af8654c6ee6e422d4d3f4e5dad171c78604cafd` with target parent `f0d484b2`
and data parent `76647990`; code-first sequencing is satisfied by #749 / `60e27eb6`.
The seven non-ledger data/test/archive/preflight files remain byte-identical to the reviewed stage.
No upstream re-download, new protocol inference or new local Cargo test verdict is claimed.

The Story/I306 owner now records Review / Claimed, Completion Commit: Pending. Distinct model-endpoint
routing remains unimplemented; 56 records remain withheld. Parent tracking is not implementation or
an acceptance waiver. Owner acceptance requires implementation or an explicit accepted scope decision
before completion; #724/#748 remain open. No sibling implementation, tag or release is claimed.


### Reviewed HTTP Endpoint Projection Candidate — 2026-10-28

Runtime code carrier #792 is exact `f43cd9900f03d2d52afe90fbca6f8a90944a7826`, equivalent to
local `600ef88b`, tree `abc8535011355cc5679a451628bce43a897bb623`, base `e03a2984`.
This data candidate is stacked on that runtime and is not independently active. Code must merge
first. Existing code #749 and protocol-only data #775 remain merged evidence, not this stage's CI.

The same archived 2026-10-27 input is reused without a new models.dev download. Forty-five previously
withheld model records gain both `protocol` and `api_base_url`; the old 4,010 hints remain unchanged.
Total: 3,857 Chat / 65 Messages / 133 Responses = 4,055. All 6,474 model identities and every price,
limit, capability, variant and provider record remain unchanged. Removing only the two model routing
keys gives the original semantic digest `acc90a95ff8f87fbf131788548066575d02cd8e1ccb8f3545a9c62e01712947d`.
Catalog raw SHA256: `cd06869b3e9136ee67f311450be0130cecf5ae495268241329f2b16f9ba03f70`.
The source/gzip/MIT notice are byte-identical to #775. The input README updates audit usage only.

The generator now admits a distinct endpoint only when its already-inferred wire and exact
provider/root pair are in the primary-source review below. It does not infer a family from a
provider name. Unknown pairs retain neither protocol nor endpoint metadata; the runtime cannot
change a credential destination using an unreviewed future source row. The independent Python audit
checks every protocol/endpoint, the frozen source hash and every non-routing field offline.

| Provider | Newly admitted | Model root (adapter completes final path) | Primary contract checked 2026-10-28 |
|---|---:|---|---|
| neon | 14 Responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` | [Neon AI Gateway](https://neon.com/blog/llms-belong-in-your-backend): branch bearer token and separate native Responses dialect path. Explicit config uses `api_key_env = "NEON_AI_GATEWAY_TOKEN"`; the base-URL environment variable is not a credential. |
| ofox | 10 Messages | `https://api.ofox.ai/anthropic/v1` | [Ofox Messages API](https://ofox.ai/docs/api/anthropic/messages): final `/messages`, `x-api-key`, `anthropic-version: 2023-06-01`. |
| zenmux | 17 Messages | `https://zenmux.ai/api/anthropic/v1` | [Zenmux quickstart](https://zenmux.ai/docs/zh/guide/quickstart): final `/messages` and the same native API-key/version headers. |
| freemodel | 4 Chat | `https://api.freemodel.dev/v1` | [Official FreeModel frontend](https://freemodel.dev/): OpenAI SDK example selects this root; one account key covers its model network. This reviewed origin differs from the retained provider root `https://cc.freemodel.dev/v1`. |

FreeModel's public frontend is rendered from [asset index-CDUNPcwU.js](https://freemodel.dev/assets/index-CDUNPcwU.js),
SHA256 `cf29a3b73e749023ce65e7d513bcec5d6b2b3d20102345f01e799669854e354d`.
The review used root/SDK/key-role descriptions, not a live model request or a real API credential.
This artifact is external vendor frontend code, not a runtime dependency or copied repository asset.

Seven Copilot Messages records remain **unreviewed_model_endpoint**. Their SDK/URL metadata alone
has not established compatibility with Talos' native API-key request. The official Microsoft client
uses [CAPI/authentication services](https://github.com/microsoft/vscode/blob/main/extensions/copilot/src/platform/endpoint/node/copilotChatEndpoint.ts)
and [CAPI integration](https://github.com/microsoft/vscode/blob/main/extensions/copilot/src/platform/endpoint/common/capiClient.ts).
The referenced official `@vscode/copilot-api` 0.5.2 package supplies integration headers; its published
[tarball](https://registry.npmjs.org/@vscode/copilot-api/-/copilot-api-0.5.2.tgz) SHA256 is
`7a817c4d468abb7811964af047de00aeee13e70ac97e1f698d3b774d1580d6a4`.
Admission decision/inference: until the required vendor authentication/transport contract is
implemented or independently confirmed, do not newly activate these seven endpoints. This is not
proof that the vendor endpoint itself is unavailable, and no OAuth/token-exchange wrapper is added.
Four Alibaba/Alibaba-CN WSS records are unsupported Realtime transports, an existing explicit non-goal;
they are not reinterpreted as HTTP Chat because an SDK/shape label resembles Chat.
Withheld totals: 918 absent / 1,353 unsupported transports / 7 unreviewed endpoints / 141 root drift.

Runtime acceptance drives the actual compressed catalog, not injected model endpoint overrides:
`neon_packaged_model_endpoint_drives_a_real_agent_round` supplies only the provider branch root and
API-token environment variables, then completes image + native read tool/result over
`/openai/v1/responses`. It asserts the actual bearer token source, model, stateless body and result.
`neon_catalog_responses_rollback_restores_the_matching_provider_root` uses that same catalog/provider
root and proves explicit Chat returns to `/v1/chat/completions`. Actual factory snapshots cover
Freemodel's reviewed origin change and Ofox/Zenmux Messages paths; negative cases prove replaced
roots, custom unique-model protocol inheritance without vendor URL relocation, wire rollback and
Copilot non-activation. A separate regression covers an omitted provider URL: changing the wire
must retain this model's original vendor root, not send its credential to an adapter's hard-coded
OpenAI/Anthropic default. Data activation therefore includes a small runtime safeguard for exact
reviewed endpoint records; explicit URLs, custom gateways and older records without endpoint metadata
retain precedence/behavior. The fallback still expands and validates environment roots. Neon CLI
rollback is tested both with and without an explicitly saved provider URL.
A first snapshot attempt used a nonexistent older Sonnet model ID; that
fixture failure (exit 101) is retained, then corrected to the actual frozen `claude-sonnet-4.5` ID.
No test was ignored or weakened. Final exact candidate results and independent object/source review
are recorded in the data PR; no old code CI is substituted for this data head.

Status remains Review / Claimed, Completion Commit: Pending. The seven unreviewed vendor endpoints
remain an acceptance residual; recording them under #724 is neither implementation nor a scope
waiver. B/C/D and active probing remain outside this claim. No completion or release is claimed.

#### New endpoint rows (the previous 119 Responses ledger rows remain historical baseline)

| Provider | Model | Wire | API root |
|---|---|---|---|
| freemodel | `gpt-5.3-codex` | openai-chat | `https://api.freemodel.dev/v1` |
| freemodel | `gpt-5.4` | openai-chat | `https://api.freemodel.dev/v1` |
| freemodel | `gpt-5.4-mini` | openai-chat | `https://api.freemodel.dev/v1` |
| freemodel | `gpt-5.5` | openai-chat | `https://api.freemodel.dev/v1` |
| neon | `gpt-5` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-1` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-2` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-3-codex` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-4` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-4-mini` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-4-nano` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-5` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-5-pro` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-6-luna` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-6-sol` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-6-terra` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-mini` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| neon | `gpt-5-nano` | openai-responses | `${NEON_AI_GATEWAY_BASE_URL}/openai/v1` |
| ofox | `anthropic/claude-fable-5` | anthropic-messages | `https://api.ofox.ai/anthropic/v1` |
| ofox | `anthropic/claude-haiku-4.5` | anthropic-messages | `https://api.ofox.ai/anthropic/v1` |
| ofox | `anthropic/claude-opus-4.5` | anthropic-messages | `https://api.ofox.ai/anthropic/v1` |
| ofox | `anthropic/claude-opus-4.6` | anthropic-messages | `https://api.ofox.ai/anthropic/v1` |
| ofox | `anthropic/claude-opus-4.7` | anthropic-messages | `https://api.ofox.ai/anthropic/v1` |
| ofox | `anthropic/claude-opus-4.8` | anthropic-messages | `https://api.ofox.ai/anthropic/v1` |
| ofox | `anthropic/claude-opus-5` | anthropic-messages | `https://api.ofox.ai/anthropic/v1` |
| ofox | `anthropic/claude-sonnet-4.5` | anthropic-messages | `https://api.ofox.ai/anthropic/v1` |
| ofox | `anthropic/claude-sonnet-4.6` | anthropic-messages | `https://api.ofox.ai/anthropic/v1` |
| ofox | `anthropic/claude-sonnet-5` | anthropic-messages | `https://api.ofox.ai/anthropic/v1` |
| zenmux | `anthropic/claude-fable-5` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `anthropic/claude-haiku-4.5` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `anthropic/claude-opus-4.1` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `anthropic/claude-opus-4.5` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `anthropic/claude-opus-4.6` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `anthropic/claude-opus-4.7` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `anthropic/claude-opus-4.8` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `anthropic/claude-sonnet-4.5` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `anthropic/claude-sonnet-4.6` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `anthropic/claude-sonnet-5` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `minimax/minimax-m2` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `minimax/minimax-m2.1` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `minimax/minimax-m2.5` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `minimax/minimax-m2.5-lightning` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `minimax/minimax-m2.7` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `minimax/minimax-m2.7-highspeed` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
| zenmux | `minimax/minimax-m3` | anthropic-messages | `https://zenmux.ai/api/anthropic/v1` |
