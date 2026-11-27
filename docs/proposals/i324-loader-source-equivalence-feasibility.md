# I324 D2.3 — selected-writer loader feasibility and decision boundary

**Status: Proposed / non-operative.** Refs #919, effective claim #920.
This is an independent review candidate, not an ADR amendment, a permission
grant, a production implementation or an acceptance of a different writer.
The D1 constraints approved in #1019 remain in force. Until a concrete
implementation proves them or a separately approved decision changes them,
unsupported compatibility queries remain Unsupported. Existing supported
CLI discovery retains its accepted authority.

## 1. Reproducible evidence

- #1017/#1019: upstream archive hashes, source roots and actual rebound
  writer spelling measured in disposable native Windows fixtures. MinGit
  lacks the internal rebound image; PortableGit has it. A bin observation
  is not a substitute writer. A post-spawn module observation is not a
  guarantee about mapped bytes, future loads or another installation.
- #1027: bounded PE32+ AMD64 declared ordinary/delay names. Eight
  checksum-pinned image comparisons against DUMPBIN passed.
- #1032 merged `ec3504ef4036abf76cb44fa3c781cdb57c98c918`: 13
  standalone tests, six reachable images, 24 declared edges, six
  unresolved names (advapi32, kernel32, msvcrt, ntdll, user32, ws2_32),
  native CI #4666 rerun success, independent exact-head APPROVE
  [7209034037](https://github.com/wjhuang88/talos/pull/1032#issuecomment-7209034037).
  Unresolved means missing from the supplied catalog, not a proven trusted
  OS image. No graph node has loader authority.
- Existing ProgramEvidence retains admitted executable bytes and Windows
  sharing guard; SenderProgramBinding retains invocation spelling,
  canonical target and PathIdentity. These are positive file witnesses,
  not negative namespace or mapped-module witnesses.

## 2. The security property that must be preserved

For the five unavailable Checkout/Commit query forms, preparation must
derive all actual relevant configuration/attribute sources and effects of
the *same selected system Git* that will execute the approved mutation.
Any difference that could select an unrepresented config source, include,
attribute/filter, object store, hook or helper, or change a reviewed
effect, must cause refusal before mutation. This includes absence becoming
presence, path/link replacement, relevant environment changes, and replay.
A content hash of the root EXE does not establish this property.

Let W be the retained selected native writer, E the frozen invocation and
environment, S the admitted ordered source/absence evidence, and L the
actual loader state and resolution. A source-equivalence argument must
prove that for every allowed L during the operation, the relevant source
and effect semantics of W(E,L) agree with S and the prepared plan. If
there are two still-possible L with different source/effect semantics, the
proof fails. This is a review obligation, not a new evidence type or API.
Do not assume a library is irrelevant merely because a static import has
a familiar basename. Do not expand trust to arbitrary code behind existing
Command admission. D1 currently additionally requires trusted complete
relevant pre-spawn dependency/search closure; equivalence alone cannot
replace that requirement without explicit acceptance.

## 3. Candidate routes and disposition

| Route | What it can establish | Missing proof / disposition |
|---|---|---|
| Supplied-image graph plus retained file handles | Positive identity and declared names | Windows selects images through other state; negative search candidates and later loads remain unbound. Necessary evidence, not sufficient. |
| Reconstruct loader search in Talos | A modeled list of candidates | API-set host mapping, SxS, redirection, loaded modules, KnownDLLs, exports/forwarders, dynamic loads and namespace races are not covered by the graph. No documented complete pre-spawn resolver identified. Reject as a production attestor until a concrete bounded mechanism is independently proven. |
| Managed App Control/UMCI | Device policy can constrain image execution | Device-wide effective policy and multiple policies, signer/hash semantics, selected path and source equivalence remain independent; cannot require users to install or modify system policy as a generic Scoop repair. Separate managed deployment decision only. |
| Explicitly trusted deployment/source boundary | Potentially narrows the source-equivalence argument if code and namespace enforcement are independently proven | A new trust contract, not an existing capability. No provider or enforceable receipt identified. Requires separate exact-text ADR decision and independent security/API review before implementation. |
| Explicit selection of newer supported Git | Existing supported fixed queries can avoid the old missing query forms | User choice, not a transparent substitution or fulfillment of #919. No automatic fallback. |

Microsoft documents the
[DLL search order](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order),
[process-specific search policy](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-setdefaultdlldirectories),
[file sharing semantics](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilea),
[App Control overview](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/appcontrol-and-applocker-overview),
[multiple policies](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/deploy-multiple-appcontrol-policies)
and [rule/hash semantics](https://learn.microsoft.com/en-us/windows/security/threat-protection/windows-defender-application-control/select-types-of-rules-to-create).
These sources explain individual mechanisms, not a complete attestor.
The feasibility conclusions above are engineering assessments, not a
Microsoft guarantee or a proof that no platform solution can exist.

## 4. Required evidence before a narrower trust contract is considered

A proposed deployment/source boundary must identify the enforcing component,
its actual authority, scope, identity, inputs, validity interval and
failure semantics. A caller assertion, unsigned catalog, pathname hash,
post-spawn module list, or policy-name lookup is not an independently trusted
receipt. No new public permission/resource, filesystem traversal, process
inspection, OS policy alteration, unsafe or stage-budget increase is
proposed here.

| Proof obligation | Required evidence / falsifier |
|---|---|
| Selected writer | Same invocation spelling and canonical admitted identity through spawn; MinGit missing rebound refuses, no implicit bin fallback. |
| Code and namespace | Independently enforced allowed code/loader choices and relevant paths throughout operation; unexpected DLL, changed redirection, API-set/forwarded/dynamic load, higher-priority candidate appearing, or replaced ancestor falsifies the claim. |
| Source semantics | Source-code basis and native oracle for exact compiled roots and global/system precedence; explicit missing/empty roots and Windows known-folder inputs, including ProgramData. Uncertain inputs refuse. |
| Source admission | Nomination is lexical only; exact Read continuation precedes reads, include expansion and tree/object/alternate evidence; exact metadata/absence and identity remain bound, no generic root grants. |
| Mutation boundary | Same retained writer and authorized command, effects, one-use plan and stale checks; no mutation on any missing or stale evidence. Existing supported CLI unchanged. |
| Native acceptance | Original Scoop Git 2.36.1 and newer Git, >16 MiB Checkout/Commit, Unicode/spaces, missing/empty and changed sources, denial, replay, no unapproved children, stage/output bounds, exact-head CI and governance. |

The source equivalence argument must cover the four root queries as well as
target-tree attribute precedence. Target-tree selection alone is not a
way to omit unresolved global/system sources. Push shell remains separate.
Existing source/provenance and graph tests remain reusable evidence but
do not satisfy these rows by themselves.

## 5. Decision requested from independent reviewer

Recommend retaining the strict D1 requirement and explicitly recording
D2.3 as blocked on a concrete trusted loader/source-boundary mechanism.
Do not extend the PE parser merely to accumulate static names. Review whether
a bounded source-equivalence contract could be made defensible with a real
enforcing component and the falsifiers above, or whether the route should
be rejected. Identify a specific missing trust fact or concrete alternative
if possible; do not endorse a hypothetical attestor as implemented.
A change to D1 requires separate exact-text maintainer acceptance.
No production enablement, issue closure or native product success is
claimed. #919 remains Open.
