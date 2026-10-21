# SEARCH-001-C: Dormant Prepared Compatibility Invocation

Private I293/#712/ADR-090 prerequisite based on #743/main181d7584. The direct-only owned plan
now has an implementation of the existing core PreparedNetworkToolInvocation trait. This is a
library-only bridge; WebSearchTool's registered preparation/legacy execution and official roots
are not switched. Full C/EP, parity and D/privacy activation gates remain effective.

## Retained Data And Dispatch Clock

PreparedCompatibilityInvocation prepares exactly one existing plan and retains bounded original
query, include_snippets and validated TransportLimits. Manifest/profile accessors borrow the
same plan. The invocation is consuming/non-Clone and uses fixed redacted Debug. It accepts no
replacement input/manifest/transport/DNS/headers at dispatch and mints no permission authority.

TransportLimits validates the existing duration>0..60s, requests1..8 and response body bytes
1..1048576 bounds without reading a clock. These explicit private limits are not default-activation
policy. TransportBudget::new still validates the same bounds and starts immediately, preserving
its existing callers. The new trait execution starts one budget only at actual dispatch after
receiving external live authority. Preparation/approval waiting therefore consumes no execution
time. Every source/hop uses that same budget and retained plan; no per-source reset or reprepare.

Successful admitted rows use the shared existing WebSearch formatter. The original method now
delegates to the same pure function; source labels, numbering, original query/display text,
snippet flag and current UTF-8 truncation behavior are preserved byte-for-byte. The formatter
does not admit URLs, authorize result fetching/rendering or qualify a backend. The bridge only
passes existing admitted compatibility rows and fixed DuckDuckGo family identity to it.
Success deliberately contains the established query/result display; errors contain only closed
existing codes, without query/URL/body or permission/dependency diagnostics. A final live budget
check after conversion/formatting prevents a late success from escaping. It does not make bounded
synchronous formatting interruptible or establish a hard CPU/allocation cap. No continuation
artifact/provider-specific content is introduced.

## Evidence And Limits

Real PermissionSessionState proposal/Once/Session/admit_network fixtures exercise the boxed core
trait and actual controlled HTTP. They cover complete manifest previews, a preparation wait longer
than the execution duration (Tokio time resumed before socket setup), captured query/snippet output,
revoked and wrong-invocation authority before DNS/send, Session scope reuse with fresh authority,
whole-plan Deny and rejected facet stripping, one shared request budget and caller-abort peer EOF
with joined fixture cleanup. Separate golden formatter evidence preserves the existing UTF-8
boundary even where one character ends beyond byte300. There is no live official-host query/probe.

These fixtures use explicit cfg(test) relocation/DNS, not official AgentTool registration or Agent/
SDK/CLI/TUI/MCP/standalone dispatch acceptance. Direct-only configuration still cannot silently
replace an unsupported configured proxy. Proxy/redirect/probe/retry/source/charset/entropy parity,
D/privacy/result-use, official roots, router/health/hedging/cancellation/rollback/default activation
and release retain their gates. No public API/model schema/dependency/permission-policy change.
New bridge/budget/formatter plumbing is Talos-authored; existing MIT provenance remains unchanged.
I293 Active/Claimed, Completion Pending; #643/#624 open, #705 open/unmerged. Exact validation,
protected reviews and native terminal receipts are indexed in I293/#643.
