# 044: Defer A2A-001 Multi-Instance Discovery And Communication

> Status: Accepted (Defer)
> Date: 2026-07-16
> Iteration: I133 / P140

## Context

A2A-001 asks whether Talos should support any bounded communication or discovery between
independent instances before a network protocol is considered. The spike must produce a threat
model for identity, authentication, authorization, discovery, capability advertisement, data
retention, revocation, credentials, and transcript exposure — or explicitly defer/reject.

## Constraint Decomposition

| Constraint | Type | Source | Can Change? |
|---|---|---|---|
| No global pub/sub bus | Hard | ADR-006 | No |
| No speculative features | Hard | AGENTS.md #7 | No |
| No secrets in transit without owning ADR | Hard | AGENTS.md #3 | No |
| All write-capable tools gated by permissions | Hard | AGENTS.md #4 | No |
| No automatic discovery, network service, protocol, credential transfer, multi-agent | Hard | P140 non-goals | No |
| REMOTE-001 (prerequisite) is P4 Research, not implemented | Fact | Backlog | Yes, through separate work |

## Threat Model

### Identity and Authentication

Talos instances have no identity infrastructure. There is no PKI, certificate authority, shared
secret store, or mutual authentication mechanism. Any inter-instance communication would require
a new identity layer — a significant security-sensitive design that cannot be solved within P140's
non-goal constraints.

**Risk**: Unauthorized instance impersonation if identity is not cryptographically verified.

### Authorization

Talos's permission model is per-process and per-session. There is no cross-instance authorization
framework. An instance receiving a request from another instance has no way to evaluate whether
the requesting instance's user has permission to read/write the target session's data.

**Risk**: Privilege escalation if a remote instance can access sessions it should not see.

### Discovery

Automatic discovery (mDNS, UDP broadcast, etc.) would expose Talos instance existence and metadata
to the local network. This violates the loopback-only boundary established in ADR-031 for the
dashboard and contradicts the principle of explicit, user-initiated connections.

**Risk**: Network-exposed attack surface; information leakage about user's development environment.

### Credentials and Transcript Exposure

Session transcripts contain:
- API keys and provider tokens (redacted in display, but present in some storage forms)
- User source code, prompts, and tool results
- Provider raw responses and reasoning content

Cross-instance sharing of any of this data without an end-to-end encrypted, authenticated transport
creates a credential exposure risk. No such transport exists or is authorized.

**Risk**: Credential theft, source code exfiltration, provider response leakage.

### Retention and Revocation

No mechanism exists for an instance to revoke another instance's access, expire shared data, or
enforce retention policies across instance boundaries.

**Risk**: Persistent unauthorized access after a relationship should have ended.

## Decision: Defer

A2A-001 multi-instance discovery and communication is **deferred**. No implementation, protocol,
discovery mechanism, or credential transfer is authorized. The defer is based on:

1. **No product need.** Talos is a single-process local developer tool. No user has requested
   multi-instance communication. The prerequisite (REMOTE-001) is P4 Research with no
   implementation.

2. **All implementation paths violate P140 non-goals.** Automatic discovery, network services,
   protocol implementation, credential transfer, and multi-agent orchestration are all explicitly
   excluded.

3. **Severe unresolved security risks.** The threat model above identifies five risk categories
   (identity, authorization, discovery exposure, credential/transcript leakage, retention/revocation)
   that have no mitigation within Talos's current architecture.

4. **AGENTS.md #7** (no speculative features) prohibits building inter-instance infrastructure
   "in case" a need appears.

## What This Decision Does NOT Approve

- No instance identity, discovery protocol, or network listener.
- No credential transfer or cross-instance session sharing.
- No multi-agent orchestration or implicit authority.
- No global event bus spanning instances (ADR-006).
- No claim that the capability is "delivered" or "substantially delivered."

## Reversal Trigger

Revisit this defer if **all** of the following hold:

1. REMOTE-001 (remote session protocol) has been designed, implemented, and accepted through its
   own ADR; **and**
2. A concrete product need requires **bounded, explicit, user-initiated** inter-instance
   communication (not automatic discovery); **and**
3. The proposed design addresses all five threat-model risk categories with specific mitigations;
   **and**
4. The design preserves ADR-006 (no global bus), AGENTS.md #3 (no secrets in transit without ADR),
   #4 (permission gating), and #7 (no speculative features).

## Comparison: Explicit Host-Managed vs Automatic Discovery

| Dimension | Explicit host-managed | Automatic discovery |
|---|---|---|
| Security | User explicitly configures connections; no surprise exposure | Instances broadcast existence; network attack surface |
| Complexity | Simple: host passes connection info | Complex: discovery protocol, identity negotiation, conflict resolution |
| Permission | Host controls which instances connect | Any discoverable instance can initiate |
| ADR-006 compliance | Compatible (point-to-point channels) | Likely requires broadcast (violates ADR-006) |
| P140 non-goal compliance | Partially compatible (no auto-discovery) | Directly violates "no automatic discovery" |

**Conclusion**: If multi-instance communication ever becomes needed, the explicit host-managed
approach is the only one compatible with Talos's constraints. Automatic discovery is rejected.

## Related

- [ADR-006: Event Architecture Boundary](006-event-architecture-boundary.md)
- [ADR-031: WEB-001 Loopback Dashboard Boundary](031-web-loopback-dashboard-boundary.md)
- [ADR-043: Defer Persistent Task Runtime](043-defer-persistent-task-runtime.md)
- [A2A-001 owner doc](../backlog/active/A2A-001-multi-instance-discovery-spike.md)
- [REMOTE-001 owner doc](../backlog/active/REMOTE-001-remote-session-protocol.md)
- [RUNTIME-001 owner doc](../backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md)
