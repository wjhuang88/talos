# I324 option B — source-specific compatibility contract

**Proposed / non-operative; requires independent security/API review and separate
exact-text maintainer acceptance.** Refs #919, effective #920, D1 #1019,
static evidence #1027/#1032 and feasibility #1037. The maintainer selected B
for redesign and review ([decision receipt](https://github.com/wjhuang88/talos/issues/919#issuecomment-7213718098)),
not permission to activate a weaker runtime policy. Strict D1 remains effective.

## Decision candidate

Separate the ordinary host execution trust boundary from the additional proof
of source equivalence required by a missing query. Do not build a general
Windows loader in Talos. Investigate accepting a **narrow, conditional**
equivalence claim under the same explicitly reviewed trusted host execution
premise used for supported fixed Git CLI calls, plus exact source-specific
evidence. A successful proof may replace only the selected unavailable query.
If that premise is not already defensible, or a new trusted deployment fact
cannot be established, refuse. Do not treat this proposed premise as an
existing permission or infer it from Command Execute, a version string, a
pathname, archive provenance or a root EXE hash.

### What changes relative to D1

D1 demands a complete relevant pre-spawn loader/search closure. B would
remove that universal prerequisite **only if** the independent reviewer
accepts an explicit shared host-execution assumption for these substitutions.
Under that assumption, the platform and selected Git installation execute
the reviewed Git semantics, including dependencies and startup path logic.
Talos must still bind the selected native image, frozen invocation, exact
source paths and all admitted effects. This is a conditional security claim,
not proof against arbitrary native code execution or DLL hijacking.

A malicious DLL that changes config source selection after a correct
static profile match is a concrete counterexample outside that premise.
In particular, a DLL could make fixed discovery queries report expected
values but cause the final mutation to use a different source; the same
possibility exists for a malicious host writer under the existing CLI
boundary, but this observation **does not** prove the new static nomination
is equally safe against an installation/namespace attacker. Review must
explicitly establish whether excluding such an attacker is acceptable,
which component enforces the trusted installation premise, and whether
the substitution introduces an additional differential risk for an attacker
that remains in scope. If the existing contract actually protects against
that attacker, B must preserve that protection or remain Unsupported.
No unconditional OS-loader trust, no blanket assumption based on approval.

## Split by query instead of one universal attestor

| Query | Candidate source basis | Independent negative gate |
|---|---|---|
| GIT_ATTR_GLOBAL | Effective admitted configuration, exact core.attributesFile semantics or verified XDG/HOME fallback; same repository context | Unknown expansion, precedence, absent/empty source, ambient/default home uncertainty, conditional include effects or unsupported context refuse. |
| GIT_CONFIG_GLOBAL | Verified Git-version semantics for ordered XDG and HOME entrypoints and frozen environment | Preserve both potential roots, even empty/missing, explicit overrides only where already authorized; no invented HOME. |
| GIT_ATTR_SYSTEM | Selected writer profile with reviewed exact compiled root and runtime-prefix behavior | Unknown installation or spelling, source profile mismatch, unproven prefix or system attribute semantics refuse. |
| GIT_CONFIG_SYSTEM | Selected writer profile plus all actual platform startup/global configuration roots in order | ProgramData/known-folder behavior and startup sources must be proved, not inferred from an environment marker; unknown source refuses. |
| check-attr --source | Existing #928 supplied-byte selector, exact admitted target tree/objects, ordered system/global/tree/info attribute evidence | No --cached substitution, no implicit object store or omitted roots, ambiguous macro/filter semantics refuse. |
| GIT_SHELL_PATH | Separate Push shell effect contract | Out of this candidate; existing refusal remains. |

These are independent proof obligations, **not independent activation
permissions**: Checkout/Commit still refuse when any required source or
effect is unresolved. The global roots are the smallest first research
slice because they may be derived from already admitted configuration and
frozen inputs instead of the compiled system prefix. No claim that this
alone makes a full operation succeed.

Use exact source revision
[Git for Windows v2.36.1.windows.1 attr.c](https://github.com/git-for-windows/git/blob/v2.36.1.windows.1/attr.c),
[config.c](https://github.com/git-for-windows/git/blob/v2.36.1.windows.1/config.c),
[path.c](https://github.com/git-for-windows/git/blob/v2.36.1.windows.1/path.c)
and [Git 2.36 configuration docs](https://git-scm.com/docs/git-config/2.36.3).
The table is a hypothesis to test, not a completed semantic proof.

## Proposed evidence pipeline (no production implementation)

1. Reuse existing selected-writer capture/rebound and frozen environment;
   bind to actual invocation, not outer shim. Retain current guards and
   revalidation. Check already captured admitted bytes against a reviewed,
   exact-image source profile. A match identifies known source semantics
   only under the explicitly accepted host-execution premise; it is not an
   executable trust attestor. Unknown image, missing MinGit rebound, or
   unproven installation premise refuses. No fallback to bin/git.exe.
2. Retain already authorized fixed config --show-origin --null --list
   --includes and supported ordinary/cached attributes and exec-path.
   Reconcile observed ordered records with candidate sources; origin
   records cannot prove absent/empty roots. Never discard an unexpected
   origin, and never introduce a new command, retry on arbitrary exit
   failures, or force environment variables to suppress sources.
3. Nominate finite lexical paths from exact reviewed semantics, without
   stat/open/readlink/process or ambient environment. Existing admission
   mechanisms only for their accepted semantics; any new use needs review.
   Exact Read continuation precedes bytes, metadata, missing-source and
   ancestor evidence, include and target object expansion. Reuse current
   bounds, one-use proof and stale checks; do not silently widen
   git:configuration-chain or grant recursive filesystem access.
4. Reconcile native fixed-query output and byte-derived source/effect
   semantics in the same bound invocation. Verify negative paths and
   identity just as required by the existing prepared plan; distinguish
   prevention from sampling/revalidation and do not claim a stronger
   race guarantee than the actual admitted API supplies. If any source
   requires a guarantee not provided, refuse.
5. Use the unchanged selected system Git final mutation and all current
   effect, Command/Path/Remote, output/stage budget, hooks/filter and stale
   checks. A candidate proof cannot authorize any changed operation.
   Never retry denial, timeout or unrelated failure through the new path.

## Concrete validation and decision gates

First a test-only source-semantics matrix over pinned 2.36.1 and a pinned
newer Git, with synthetic explicit HOME/XDG, core.attributesFile values
(absolute, relative, tilde and unsupported forms), include order,
missing/empty roots, Unicode/spaces, and platform startup source cases.
Use isolated fixtures and independently reviewed oracle invocations, not
new production CLI rights. Compare known newer var queries and actual
old-Git source observations; demonstrate false-negative detection rather
than infer absent roots from show-origin. Treat unobservable negative cases
as unresolved. Record actual archive/image provenance and fixture limits.
Use #1027/#1032 only as diagnostic evidence, not a general runtime prerequisite.

Before production work, independent security/API reviewer must answer:
- Is the shared host-execution premise consistent with the *actual* accepted
  ADR-089 and existing process/security boundary? What attacker is in scope?
- What concrete evidence or enforcing component establishes the premise for
  a real Scoop installation, including relevant namespace changes? If none,
  can any narrower per-query route satisfy the existing attacker model
  without this assumption? Name the counterexample or missing proof.
- Does the proposed nomination add an effect or a read/metadata permission?
  Can existing stages and bounds actually carry the exact evidence?
- Which explicit D1 text would need replacement, and which security
  guarantees are unchanged, conditional, or unsupported?

If the shared premise is rejected, do not implement it. If it is accepted
in principle but no enforceable mechanism exists, restrict work to source
matrix research; no runtime activation. If a concrete candidate is viable,
submit a separate exact-text ADR redline and maintainer decision before
implementation. Afterwards require independent protected-scope review,
locked exact-head CI/governance and real original Scoop/newer Git >16 MiB
Checkout/Commit, denial/stale/negative cases and unapproved-child sentinel.
No production enablement or #919 completion in this proposal.
