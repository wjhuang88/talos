# I243 Shell Auto Classifier Threat Matrix

Status: Proposed decision evidence; no implementation authority.

| Threat / scenario | Deterministic gate before model | Classifier context / expected result | Required implementation evidence |
|---|---|---|---|
| Explicit permission Deny | Deny immediately | Classifier not called | zero assessor calls |
| Explicit content-scoped Ask | Human checkpoint | Classifier not called | prompt retained in auto mode |
| Existing valid human grant | Existing policy decides | Classifier not required | grant semantics unchanged |
| Routine local inspection (`ls -la`) | No deny/ask match; workspace bounded | Exact command as untrusted data + cwd/trust context; may `AllowOnce` | no per-command approval branch; real TUI pass |
| Previously unseen executable | Secret/path hard guards | Model judges semantics; uncertainty is `HumanRequired` | corpus proves unknown does not default allow |
| File deletion/overwrite or destructive Git | Hard/soft risk rules | Never automatic without an accepted explicit-intent exception permitted by ADR-070 | destructive fixtures blocked/escalated |
| Network/external destination | Target/trust classification | External or unknown target is `HumanRequired`; configured trusted target remains model-assessed | domain/remote/bucket fixtures |
| Secret in argument/environment | Credential detector and environment-value exclusion | Classifier not sent raw secret; deny or human path | sentinel never appears in model/audit bytes |
| `PATH`, wrapper, loader or toolchain override | Environment-identity gate | Non-empty high-impact override is human-required | execution cannot differ from assessment |
| Privilege/system/protected environment | Protected-target/system rule | Hard deny or human-required | sudo/systemctl/prod fixtures |
| Pipeline/redirection/substitution/subshell | Structural evidence, aggregate request identity | Model sees exact composition; any ambiguous/mixed effect is human-required | quoting/composition adversarial suite |
| Prompt injection embedded in command/path | Serialized as untrusted action data | Classifier system rules remain authoritative | injection corpus cannot produce allow for hard risk |
| Repository-controlled trust config | Config provenance gate | Repo may tighten, never add trusted target/allow | precedence/provenance tests |
| User explicitly names exact risky action | Permission Deny/Ask still wins | May clear only accepted soft deny, never hard deny | exact-vs-general intent fixtures |
| Context compaction or unrelated history | Only current bounded intent is supplied | Missing intent is not inferred | compaction/restart remains fail closed |
| Stale policy/mode/session/cwd/action | Revision and digest CAS | Reject response | mutation fixtures |
| Timeout/cancel/provider/schema error | Deadline/cancellation/schema guard | `HumanRequired` or Deny | no execution, no permanent grant |
| Classifier tries tool call | No tools exposed | Malformed/fail closed | tool-call response rejected |
| Cross-provider fallback | Provider trust-boundary gate | No fallback | configured provider failure stays closed |
| CLI/TUI/Runtime/MCP equivalent request | Shared Agent-owned pipeline | Same classifier identity/result semantics | conformance matrix |
| Auto disabled/circuit open | Kill switch | Existing human/headless behavior | zero classifier calls, no state migration |

## Adjacent Authority

- PERM-006-D / Issue #56 owns any new authoritative public typed-effect/resource API, permission
  compiler/store schema, and audited command-descriptor contract. I243/I244 must not create a second
  public effect authority; the initial classifier may consume existing `AccessEvidence` only as
  advisory input.
- PERM-006-E / Issue #57 owns the broader permission-pipeline cross-surface security gate. I244 must
  pass its own classifier conformance but cannot close or absorb Issue #57.
- ADR-012 and ADR-040 remain correct that parsing/access evidence alone is not authorization.
