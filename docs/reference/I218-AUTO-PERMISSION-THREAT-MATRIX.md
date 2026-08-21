# I218 Auto Permission Current-Path And Threat Matrix

**Status**: Decision evidence for PERM-007-A / I218
**Code baseline**: `main@ca30081a883cdf130784ceb04551465b71adf505`
**Behavior claim**: none; this matrix describes current code and the boundary proposed by ADR-064.

## Current Path

| Surface | Current authority and Ask resolution | Evidence | Gap before `auto` |
|---|---|---|---|
| Permission engine | `PermissionEngine::evaluate_profile` evaluates every facet; any Deny wins, otherwise any Ask wins. Read/Internal default Allow; Write/Execute/Network default Ask. External concrete paths return Ask unless an exact scoped rule allows them. | `crates/talos-permission/src/lib.rs` (`evaluate_profile`, `evaluate_facet`) | No structured request/report, decision provenance, operation mode or model-assistance seam yet. |
| TUI | `TuiApprovalHandler` re-evaluates the profile, emits `ToolApprovalRequest` for Ask, and fails to Deny if the channel/response closes. A human may approve once or install session rules. | `crates/talos-cli/src/registry.rs` | Approval UI is also the decision resolver; there is no shared pre-human resolver. |
| Interactive CLI | `PermissionAwareTool` prompts on Ask. Prompt errors become Deny. | `crates/talos-cli/src/registry.rs`, `approval.rs` | Logic is duplicated from TUI and cannot share a cross-surface model result. |
| Print/headless CLI | Ask becomes Deny because interactive approval is unavailable. Session `RuntimePolicy` is `HeadlessDeny`. | `crates/talos-cli/src/registry.rs`, `mode_print.rs`, `mode_inline.rs`, `mode_runners.rs` | No bounded headless auto policy; default denial must remain the fallback. |
| Runtime SDK | `RuntimePermissionAwareTool` invokes an embedder `ApprovalHandler` only after Ask; without one it denies. Approved calls receive exact execution authorizations. | `crates/talos-runtime/src/lib.rs` | The SDK must not read CLI config implicitly; an embedder must opt into a resolver explicitly. |
| Agent core | `ToolExecutor` can evaluate a `PermissionEngine`; Ask leaves `permission_allowed=false`. Sandbox fallback is a separate policy/handler and cannot be conflated with normal approval. | `crates/talos-agent/src/tool_execution.rs` | Multiple composition roots exist; PERM-006-C must establish the single authoritative evaluate-to-execute seam first. |
| Configuration | `Config` has no permission-auto field. It is a public Rust struct and serde/JSON-Schema type. | `crates/talos-config/src/types.rs` | Adding a public field affects exhaustive struct literals and requires a versioned migration note. |
| Slash commands | The static registry has no `/auto`; built-ins are parsed by the Conversation engine and projected by the TUI bridge. | `crates/talos-conversation/src/command_registry.rs`, `engine/commands.rs` | The command needs typed session state and must not mutate persistent policy implicitly. |

## Threat Matrix

| Threat / failure | Attack or failure path | Required invariant / mitigation | Required test seam | Failure result |
|---|---|---|---|---|
| Policy bypass | Model is called before policy evaluation or converts Deny to Allow. | Only an authoritative, structured `Ask` may enter the resolver; Deny and hard-boundary reason codes are terminal. | Deny/Ask/Allow matrix at the shared PERM-006-C seam. | Deny. |
| Scope enlargement | Model changes tool, operation, resource, lifetime or facets. | Resolver output carries no replacement fields. Allow is `Once` for the exact request digest and exact execution authorization only. | Mutate every request field after assessment and prove authorization rejection. | Human or Deny. |
| Persistent privilege | Model selects “always”, writes policy or creates a reusable grant. | Model output has only `AllowOnce` or `HumanRequired`; it cannot call grant APIs. | Schema rejection and grant-store non-mutation tests. | Human or Deny. |
| Ineligible mutation | Delete, rename, chmod, binary write, credential/config mutation, user-owned dirty-file overwrite or operation on `main` is classified low risk. | Initial allowlist admits only declared structured text create/modify under a typed managed-workspace lease, on a non-`main` worktree and without pre-existing user changes. Talos CLI derives the lease from its effective claimed Work Slice; SDK hosts must provide equivalent scoped evidence. | Table-driven operation-subtype, branch/worktree, lease and dirty-resource fixtures. | Human; headless Deny. |
| Execute/network/sandbox escape | Shell, process, network, plugin/MCP or unsandboxed fallback is treated as ordinary low-risk work. | Execute, Network, sandbox fallback, plugin/MCP-originated calls and mixed profiles containing them are ineligible in the first implementation. | One negative fixture per provenance/facet combination. | Human; headless Deny. |
| External-path/data access | A read or write outside the workspace is auto-approved. | Any external path, unresolved/canonicalization-failed path, symlink drift or secret-bearing resource is ineligible. Existing exact authorization re-check remains mandatory. | External path, symlink swap, traversal and missing-ancestor fixtures. | Human; headless Deny. |
| Prompt injection | Tool arguments, file content, model output or prior conversation instructs the evaluator to ignore policy. | Policy/eligibility are deterministic code. Context is a bounded untrusted-data field; no tool use, nested model call or free-form output controls authorization. | Adversarial strings in every redacted input field and malformed output fixtures. | Human; headless Deny. |
| Secret disclosure | Raw arguments, environment, credentials, file contents or reasoning enter evaluator input or logs. | Send only schema-versioned metadata, workspace-relative resource labels, bounded redacted intent/preview and digests. Never store raw prompt, arguments, content, secrets or reasoning in audit. | Canary secrets across input, provider capture and tracing sinks. | Human or Deny; circuit failure counted. |
| Ambiguous model answer | Low confidence, contradictory reason, unknown enum, extra fields or invalid schema is accepted. | Closed output fields are `schema_version`, `request_digest`, `decision`, `reason_code` and `confidence`. `AllowOnce` requires `confidence=high`, reason `bounded_workspace_text_change`, known policy/model version and every deterministic validator predicate. | Property/fuzz tests for malformed/extra/conflicting fields. | Human; headless Deny. |
| Timeout/provider failure | Evaluation hangs, retries multiply cost, or provider is unavailable. | One request, no automatic retry, default 8-second deadline, configurable only up to 30 seconds. | Paused time and provider-error fixtures. | Human; headless Deny. |
| Repeated unsafe advice | Evaluator repeatedly emits invalid or human-required results while auto remains active. | Session circuit opens after two consecutive technical/validation failures or three consecutive `HumanRequired` outcomes. Only explicit `/auto on` resets it. | Counter/reset/session-transition tests. | Human; headless Deny. |
| Self-approval recursion | Evaluator can use tools or its own request triggers permission evaluation. | Evaluator request is isolated, tool-free and non-recursive; provider transport permission is composition-owned, not evaluated as the target call. | Resolver recursion guard fixture. | Human; headless Deny. |
| Stale/replayed result | A prior AllowOnce is reused for a later call or after policy/mode/resource changes. | Bind result to request digest, policy revision, mode generation and session; consume once. | Replay, config change, `/auto off`, session fork/new/resume and resource-change fixtures. | Human or Deny. |
| Cross-surface divergence | TUI, Goal, print, Runtime or MCP applies different eligibility for equivalent context. | One resolver contract after PERM-006-C. Composition roots supply typed mode and fallback capability; they do not reimplement eligibility. | Shared conformance corpus for every surface. | Fail closed on unsupported surface. |
| Headless surprise | Default-on config silently turns a Runtime embedder into an autonomous writer. | CLI config is not an SDK global. Runtime/MCP hosts must inject an explicit resolver/policy; absent handler remains `HeadlessDeny`. | Builder-default and no-handler compatibility fixtures. | Deny. |
| Race/TOCTOU | Worktree, policy, mode or path changes between assessment and execution. | Final authorization construction/revalidation occurs after assessment and binds normalized resources; no model result bypasses execution-time checks. | Symlink/policy/mode generation race fixtures. | Deny. |
| Audit leakage or ambiguity | Logs cannot distinguish policy Allow, model AllowOnce and human approval, or contain sensitive content. | Construct the redacted structured report before authorization. Record safe IDs/digests, tool/provenance/risk class, effective mode source, evaluator/policy versions, outcome/reason, latency and circuit state only; optional sink delivery is host-owned. | Captured report/tracing/audit canary tests and report-construction failure. | Human/headless Deny if the report cannot be constructed; optional sink failure cannot add authority. |

## First-Implementation Eligibility

An Ask is eligible for model assistance only when every predicate below is already established by
trusted code, not inferred by the evaluator:

1. the request is the exact output of the authoritative permission pipeline and contains no Deny;
2. every facet is a workspace-local `Write` for a structured text create/modify operation;
3. the tool/provenance pair is in a code-owned allowlist (native Talos tools only);
4. a typed managed-workspace lease proves an isolated non-`main` worktree and the target has no
   pre-existing user/parallel-session change; Talos CLI derives this from the effective claimed
   slice, while SDK hosts must provide an equivalently scoped lease;
5. normalized resources are inside that workspace and exclude credentials, secret stores,
   executable metadata, VCS internals, deletion, rename, chmod, binary data and sandbox fallback;
6. the assessment can be represented by bounded redacted metadata and a request digest; and
7. effective `auto` mode is enabled and its circuit is closed.

All other Ask outcomes go to a human when an interactive resolver exists and become Deny when it
does not. Existing policy Allow never calls the evaluator. This narrow eligibility is deliberate:
it supports governed unattended local convergence without turning general command/network access
or arbitrary user workspaces into model-owned authority.

## Planned Child Boundaries

- **PERM-007-B**: versioned config and typed `/auto` session control only.
- **PERM-007-C**: resolver, eligibility validator, redacted request/output schema, audit and circuit
  breaker at the completed PERM-006-C seam.
- **PERM-007-D**: CLI/TUI/Goal/print/Runtime/MCP conformance, adversarial corpus and rollout/rollback
  evidence.

No child is authorized by this matrix. Each requires a runnable iteration, effective claim and the
PERM-006 prerequisites recorded by PERM-007.
