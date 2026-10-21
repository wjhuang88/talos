# SEARCH-001-C: Dormant Direct Compatibility Plan

Private I293/#712/ADR-090 prerequisite based on #742/main9f998392. One consuming plan binds
the manifest exposed for future approval to the exact frozen fixed-source chain later executed.
This runnable library-only prerequisite does not activate an official tool/root or default.

## Configuration And Preparation

Store DirectCompatibilityConfiguration once per future direct-only adapter. Preparation clones
its immutable scope, preserving configuration identity across queries, while every invocation
receives a fresh ID. PreparedCompatibilityPlan retains manifest, frozen chain and matching
permission profile. It is not Clone and exposes only borrowed manifest/profile accessors. Both
configuration and plan have fixed redacted Debug. Invalid preparation cannot revise configuration
or return an executable plan.

Scope contains exactly the existing Lite POST, Instant Answer GET, HTML GET and Bing GET
destinations. Canonical hosts derive deduplicated Network/Domain facets, without wildcard,
query or description. The complete manifest separately retains port/method/path authority;
host facets cannot substitute for it. Four protocols remain one compound compatibility family.

The constructor is explicitly direct-only: no proxy, private-address exception or redirect,
and no ambient proxy/user configuration read. It must never automatically replace an unsupported
configured proxy. Supported proxy/self-hosted/redirect snapshot migration remains an activation gate.

## Execution And Evidence

Execution receives external live ToolNetworkAuthority and an already existing Arc TransportBudget.
It creates controlled transport from the retained manifest, then consumes the original chain using
production SystemSearchDns. Callers cannot replace manifest/transport/query/headers/destinations/DNS
at dispatch. Relocation and DNS injection exist only under cfg(test). The plan mints no authority,
reprepares no requests, rerolls no headers and creates/resets no budget. A previously spent or
expired budget stays spent or expired; equal configuration scopes cannot share invocation authority.

Only the existing admitted display-result wrapper can succeed. Chain continuation/terminal rules
remain unchanged. Errors contain closed transport/chain enums and fixed Display codes, without
query/URL/body/dependency diagnostics. Manifest cloning alone grants no authority. Revocation cannot
undo bytes already exposed at an authorized logical start.

Local tests cover canonical scope/facets/redaction, configuration reuse/fresh invocation identity,
invalid preparation, actual owned-query HTTP execution after configuration/input disposal, denied/
mismatched authority, later revocation, preconsumed/expired budget and caller abort with observed
peer EOF and joined fixture cleanup. No live upstream query/probe. Exact validation/review/native
receipts are indexed in I293/#643. This plan code is Talos-authored; existing MIT provenance remains.

Official PreparedNetworkToolInvocation/formatter/permission-lifecycle/Agent/SDK/CLI/MCP/standalone
integration, proxy/redirect/probe/retry/source/charset/entropy parity, D/privacy/result use,
router/health/hedging/cancellation/rollback/default activation and release retain their gates.
No dependency, public API/model schema or permission policy changes. I293 Active/Claimed,
Completion Pending; #643/#624 stay open, #705 stays open/unmerged.
