# I300 native Pull command authorization alternative

Status: Accepted by the maintainer on 2026-10-23; implementation and activation gates pending.
Owner: docs/iterations/I300-structured-git-repair.md (T7, including the minimal shared Auto/Pull prerequisite under #755); remaining all-built-in Auto coverage stays T8.

## Problem

ADR-089's accepted flow is bounded discovery, one complete-effects approval,
then one native Pull. It still requires independent authority for external
paths, hooks, filters, helpers and recursive roots/destinations. The current
implementation therefore cannot infer that approving the Git executable also
approves all configured programs or their effects. Native recursion remains
gated. This is a contract difference from BashTool, whose foreground profile
is Execute/Command for the concrete command, not an exhaustive file/network
manifest (crates/talos-tools/src/bash_tool.rs, permission_profile).

Real runner tests demonstrate recursive post-checkout execution and protected
global uploadpack.packObjectsHook execution. A hook or filter can itself access
arbitrary paths or destinations. Enumerating its executable does not prove its
transitive effects. Retaining complete-effect discovery is a legitimate choice,
but cannot be presented as equivalent to Bash's authorization granularity.

## Accepted approach

Keep bounded read-only discovery for a useful preview. Make the mutation grant
an invocation-local native-command execution grant, with the same granularity
as approving an explicit Git command through Bash. It authorizes that frozen
command and its ordinary transitive Git/configured-helper effects. It is not
an enforceable per-path/per-destination confinement grant.

Bind the executable, exact argv, cwd, retained environment/configuration and
repository identity to the immutable one-use prepared proof. Show the command,
known roots/destinations, configured helpers and the explicit transitive-effect
boundary to the human or Auto assessor. Known effects are review evidence,
not a claim that every dynamically accessed path/endpoint was enumerated.
Do not expose credential values. Keep the existing supervised native runner;
no shell string interpolation, hidden retry, reset or automatic conflict repair.
Preserve foreground maintenance, Trace2 suppression, deadlines, cancellation,
output bounds, identity revalidation and honest partial-failure results.

Use the shared assessor for Auto. Unknown risk or assessor failure asks the
human. This grant never overrides explicit Deny or human-only policy. Where
active finer-grained Deny rules cannot be enforced/evaluated for the command's
transitive effects, refuse this route; human approval cannot override Deny.
Do not claim OS confinement or that denying an undiscovered resource has been
successfully enforced. The implementation/review must prove this policy
compatibility check before activation.

Retire Pull-only effect prediction/recursive scheduling after the replacement
passes acceptance. Reuse discovery/evidence code still needed by other Git
operations. Do not change Add/Commit/Checkout/Push authority in this decision.
Do not expand generic Path grants or grant persistent authority to Git/helpers.

## Alternative

Retain ADR-089's complete-effects boundary and finish the existing discovery
model, refusing effects that cannot be represented. This preserves stronger
pre-execution scope promises, but may require additional frontiers and excludes
some ordinary configured native behavior until separately modeled.

## Acceptance before activation

The maintainer accepted this proposal explicitly on 2026-10-23. ADR-089 records
the superseding authorization boundary. Implement typed command-scoped
proof and compatible-policy checks; test human/Auto allow, explicit Deny,
human-only, assessor failure, stale/replay, changed command/config/identity,
recursive helpers, cancellation, conflict and partial failure across
Tools/Agent/MCP/Runtime. Obtain independent security review and current-head CI;
validate Windows using cargo xwin plus native runtime evidence. Passing a
focused test or accepting this proposal does not complete T7/T8.
