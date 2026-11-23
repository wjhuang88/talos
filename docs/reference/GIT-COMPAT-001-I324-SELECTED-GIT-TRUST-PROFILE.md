# GIT-COMPAT-001 / I324 — selected-system-Git trust-profile contract (design review only)

> **Status: Historical broad-preflight trust design, Proposed / NOT ACCEPTED; NOT a universal I324 delivery gate after the 2026-11-19 correction.**
> Source #919, I324; independent security/API/authorization review must precede any ADR-089 amendment or runtime use.
> This document deliberately excludes native Git 2.36 validation and never represents a profile as available.
>
> **2026-11-19 scope correction:** ADR-approved Git CLI preflight may continue for commands supported by older Git. This broad all-defaults profile was proposed under a superseded all-gix preflight goal. Trustworthy selected-writer *attribute-source provenance* remains relevant only for a replacement of proven-unsupported `git var GIT_ATTR_SYSTEM/GLOBAL` (and any separately documented additional replacement), not a universal prerequisite to use existing bounded CLI. Treat the state machine below as historical research, not currently approved implementation.

## Problem and non-negotiable boundary

A Git writer is selected for one immutable invocation, yet the process's compiled installation defaults and potentially executable maintenance/transport paths need not be derivable from HOME, XDG_CONFIG_HOME, Git version or its installation directory. gix config-path helpers may run Git or consult lazy process-global state. A **new native substitution** may not launch unapproved Git, implicitly open these paths or declare missing roots absent without admissible evidence. Previously accepted, supported bounded Git CLI preflight is unaffected. Ordinary approved mutation must still use the **same selected system Git writer**.

A trusted executable hash identifies bytes; **it cannot establish which Git configuration, attribute and command defaults those bytes were built with**. A signed manifest is meaningful only if its independent authority is accepted to attest those exact facts about this binary/build. The filename, origin URL, signing key and trust chain are not self-validating. No source satisfying this requirement has yet been accepted.

## Proposed typed evidence tuple — NOT an authority grant

| Required field | Review meaning and failure disposition |
|---|---|
| Frozen invocation identity | Selected executable spelling, retained canonical path, content digest and platform file identity, with invocation cwd, repository common/gitdir, frozen environment and explicit command/facet context |
| Independent source provenance | A separate trusted provisioning/build channel, package/source revision, signed and verifiable build declaration or equivalent enterprise installation inventory, plus trusted issuer policy and verification version; reject user/repository-controlled untrusted assertions |
| Exact compiled-default semantics | Closed enumeration of system and installation configuration paths, system/global attributes, executable/maintenance prefix and sender, shell/transport defaults, including explicitly documented environment-dependent choices |
| Source evaluation model | All allowed selectors, path expressions, branch-specific overrides, multiplicity/order, empty roots and missing-file semantics; evaluation uses only the frozen invocation context and trusted profile fields |
| Coverage declaration | Explicit positive support for every default the planner needs for a particular Checkout/Commit/Push operation. A partially covered profile is Unsupported rather than silently omitting a class |
| Revocation/freshness | Trusted issuer and profile version, signature/trust-anchor validity, executable identity recheck and evidence invalidation if installation changes or a previously missing source appears |
| Operation-specific effects | Exact Read/Write/Execute/Remote/metadata admission for the selected mutation; possible hooks, filters, remote helpers, signing, maintenance and shell children are not authorized by the profile alone |

**A profile cannot be self-supplied by a Git repository, project settings, a model, a directory guess or an unsigned JSON file.** Signature validity alone is also insufficient unless the signer and attested compiled defaults are separately trusted by policy. The proposal does not assert that Git for Windows, Scoop or an OS package currently provides such a manifest.

## Proposed preparation and fail-closed state machine

1. **Select** the existing writer with the already accepted executable admission and freeze the invocation command/environment/cwd/repository context. Do not run Git, probe PATH, or use gix::config_path / gix repository-open convenience APIs as an unreviewed discovery channel.
2. **Verify** an independently provisioned per-installation profile whose issuer, scoped authority, exact selected binary identity and completeness are policy-approved. Unknown signer, stale/different binary, missing required compiled defaults or unsupported dynamic semantics return UnsupportedInstallationProfile before any newly named source content read.
3. **Name only** candidate system/global/config/attribute/exec roots lexically from this profile and frozen context. Naming is metadata planning, never a grant for open/stat/readlink/execute. A missing root is a named and subsequently witnessed absence, not a path simply omitted by the planner.
4. **Admit** each named path, ancestor/link metadata, byte content, missing-source state and included conditional frontier under separately exact Path Read/metadata authorizations. Existing denial precedence, bounded size, cumulative budgets and explicit parent identities remain in force.
5. **Interpret** only the admitted bytes and immutable refs/index/tree evidence using selected pure gix/Talos APIs. Unknown filter semantics, implicit gix I/O, unresolved symbolic/ref state and any unrepresented transport/hook/maintenance effects stop without mutation.
6. **Present and approve** the exact operations and effects under the existing one-use permission gate. No profile can mint an authorization, turn path-only authority into general Read, or reduce a hook/filter/child executable effect.
7. **Revalidate** before mutation: binary spelling/canonical identity/digest, approved profile issuer and binding, frozen environment, missing sources, all file/source identities and exact downstream effects. Any drift invalidates the plan; the user must authorize a new one. Only after that may the selected system Git execute the exact frozen native mutation.

Unsupported is a stable **explicit typed outcome**, not a retry with git var, git config, git --exec-path, PATH probing, a guessed location, or generic filesystem scanning. No **unapproved** command fallback is allowed for an unsupported query. The unchanged ADR-089-authorized CLI preflight for supported commands may continue.

## Concrete independent review decisions still required

| Decision | Status and who must resolve |
|---|---|
| Identify an **actual** trusted per-executable defaults attestor/provisioner for deployed Git distributions; verify whether it covers Git for Windows 2.36.1 and newer | **OPEN**. Architecture/security/maintainer decision; do not fabricate from a binary hash |
| Define the authority to name candidate paths separately from exact Read/metadata facets and from profile trust-anchor input | **OPEN**. Independent Git permission review and explicit accepted ADR-089 amendment |
| Prove gix 0.90.0 and gix-path 0.14.1 selected call graph uses no hidden Git or global discovery in the chosen pure subset | **OPEN**. Pinned source audit + process sentinel; test-only byte parsing is narrower than the full preflight |
| Define typed Unsupported data, safe diagnostics and effect-state reporting without leaking raw profiles, paths or secrets | **OPEN**. Public API/error and security review |
| Establish build and configuration invariance of GIT_INSTALL_ROOT if a narrow inherited-environment exception is requested | **OPEN**. Separate semantics test, not inferred from the name or version |
| Verify genuine native Git 2.36.1 Windows prepared Checkout/Commit, injected-path denials and external/default sources | **OPEN / explicitly excluded from the present non-native work slice** |

## Evidence claim and non-goals

This is a **decision-ready candidate contract**, not an implementation or an Accepted ADR. The I324 test-only gix candidate and isolated PATH sentinel exercise pure byte/lexical APIs but do not validate an installation attestation or guarantee absence of hidden ambient I/O in the entire runtime. **If no independently trusted source exists, the truthful completion state is UnsupportedInstallationProfile, not Windows Git compatibility delivered.**

Do not activate this profile in production, merge #928 or #931 on the strength of this proposal, or close #919 without the independently reviewed authorization contract and remaining non-native/native acceptance evidence.
