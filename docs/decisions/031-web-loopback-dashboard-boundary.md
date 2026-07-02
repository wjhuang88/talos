# 031: WEB-001 Loopback Dashboard Boundary

## Status

Accepted; amended 2026-07-02 (a) to make the loopback dashboard enabled by default with a visible
startup URL and an explicit config opt-out, and (b) to make the per-process bearer token opt-in
via `[dashboard] loopback_only = false` so the loopback bind is the only access control by default.

## Context

WEB-001 has a completed MVP design for a loopback-only dashboard, but T28 remained blocked because
starting an embedded HTTP server is a new runtime capability. The design gate must decide whether a
small read-only local dashboard can proceed without opening a remote-control surface, permission
bypass, or secret-display path.

This ADR evaluates only the first dashboard slice. It does not approve remote access, browser
automation, web-based approvals, config writes, or a full management UI.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
| --- | --- | --- | --- |
| No remote-control transport without an owning ADR/spike gate. | Hard | Four-month plan operating constraints / REMOTE-001 boundary | No |
| No secret echo in display surfaces. | Hard | AGENTS.md Hard Constraint #3 / ADR-023 | No |
| All write-capable tools remain permission-gated. | Hard | AGENTS.md Hard Constraint #4 | No |
| Web control surfaces must not bypass TUI/session/config ownership boundaries. | Hard | WEB-001 / GOV-003 / CONF-001 | No |
| Dashboard should improve product observability and governance visibility. | Soft | WEB-001 product differentiation goal | Yes |
| MVP should avoid Node.js or browser automation runtime dependencies. | Soft | Rust-first project posture / four-month plan | Yes with ADR |

## Reasoning

The blocked risk is not static HTML. The risk is accidentally creating an unauthenticated local
control plane that can read secrets, execute actions, or grow into remote access. A narrow
loopback-only, read-only dashboard satisfies the product goal while preserving the runtime
boundaries:

- bind only to `127.0.0.1`;
- start by default in TUI mode with an explicit config opt-out;
- print the access URL at startup;
- expose GET-only status/history/governance/config views;
- mask credentials using the same secret-display boundary as CLI/config surfaces;
- HTML-escape user-controlled content;
- provide no tool execution, approval, config-write, file-write, or session-mutating endpoint.

The per-process bearer token is now opt-in via `[dashboard] loopback_only = false`. By default the
loopback bind is the only access control: the dashboard does not print a token, and the auth
middleware is skipped. This trades the prior defense-in-depth against other local users, local
processes with network access, and browser-based attacks on shared machines in exchange for
zero-friction browser access on the common single-user developer machine. Users on shared or
multi-user machines should set `[dashboard] loopback_only = false` to require the token.

The 2026-07-02 amendments (a) make the dashboard default-on so it is a visible product surface and
(b) make the token opt-in for the common single-user case. The bind, route, write/action, and
remote-access boundaries are unchanged. Remote access, tunnels, browser connectors, and write
actions still require separate decisions.

## Decision

1. **WEB-001 read-only loopback MVP is unblocked.**
   - T42 may implement a read-only status/history/governance subset.
   - T28 remains a historical blocked checkpoint; new implementation should proceed through the
     current planned T42/T58 path.

2. **Dashboard startup is default-on in TUI mode, with config opt-out.**
   - TUI startup starts the loopback dashboard unless `[dashboard] enabled = false` is configured.
   - Startup prints the local access URL.
   - No remote or background daemon mode is approved.

3. **The MVP binds only to loopback.**
   - Bind address is hardcoded to `127.0.0.1`.
   - No `0.0.0.0`, LAN, tunnel, or remote mode is approved.
   - Port is OS-assigned or otherwise non-predictable by default.

4. **Authentication is opt-in via `[dashboard] loopback_only`.**
   - Default: `loopback_only = true`; no token is generated and the auth middleware is skipped.
   - When `loopback_only = false`: a per-process token is generated at startup, stored only in
     memory, and required as `Authorization: Bearer <token>` on every request. Requests without it
     fail with `401`. The token is not written to config, logs, session files, or history.

5. **MVP routes are read-only.**
   - Approved initial routes: `/status`, `/history`, `/governance`, `/config`.
   - All routes are GET-only.
   - `/config` must mask `api_key` and any credential-like values per ADR-023.
   - `/history` must avoid raw secret-bearing tool arguments by default.

6. **No runtime privilege expansion.**
   - No tool calls, approvals, config writes, file writes, shell execution, browser automation, or
     session mutation through the dashboard MVP.
   - Any later write/action route requires a new security review and permission-pipeline design.

7. **Implementation validation is mandatory.**
   - Browser/local smoke test for loopback bind and, when `loopback_only = false`, token rejection.
   - Regression proving secret masking on config/history surfaces.
   - Regression proving no write routes are registered.
   - Governance validation and targeted crate tests before closing T42.

## Rejected Alternatives

- **Keep WEB-001 blocked until a complete web app is designed.** Rejected because a read-only,
  local-only slice can produce useful evidence without approving risky actions.
- **Expose dashboard without auth because it is loopback-only.** Previously rejected; the
  2026-07-02 (b) amendment now allows this as the default for the common single-user case. Token
  auth remains the recommended setting for shared or multi-user machines.
- **Use web dashboard for approvals in the MVP.** Rejected; approval actions are write-capable
  runtime control and require a separate permission/UI review.
- **Enable remote/LAN access.** Rejected; that belongs to REMOTE-001 and needs a separate ADR.

## Reversal Trigger

Revisit if loopback bind proves insufficient for the common single-user case, if route content
cannot be made secret-safe without excessive redaction, or if implementation pressure requires
write/action endpoints in the first slice.

## Related

- [WEB-001](../backlog/active/WEB-001-embedded-web-control-surface.md)
- [WEB-001 loopback dashboard design](../proposals/web-001-loopback-dashboard-design.md)
- [ADR-023 Inline API Key Boundary](023-inline-api-key-boundary.md)
- [ADR-006 Event Architecture Boundary](006-event-architecture-boundary.md)
- [GOV-003](../backlog/active/GOV-003-builtin-project-governance.md)
