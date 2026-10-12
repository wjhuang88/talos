# ADR-089: Structured Git Effect Authorization

## Status

Accepted by maintainer on 2026-10-11: “接受”. I300 still requires an effective
target-branch claim before implementation. The accompanying all-built-in Auto
request is separately tracked as PERM-007-H, not silently folded into this contract.

## Context And Evidence

The session handoff T7 reports `permission facet 0 has no safe typed resource`
for a two-file `git_add`. Current GitAddTool, GitCommitTool and GitCheckoutTool
inherit an untyped Write profile; grant compilation rejects that profile when
grant scope is required. This is source evidence matching the error, not yet
proof of the originating executable or production admission reproduction.

Git add also interprets caller paths from the discovered repository root and
passes them as Git arguments/pathspecs. A configured subdirectory workspace,
option-looking filename or pathspec can therefore disagree with intended scope.
Adding a workspace-wide Write facet alone would conceal this mismatch.

ADR-010 already requires bounded host bridges and a Rust-native read-only
primary path. ADR-026 already requires all risk facets. This decision defines
their concrete Git effect and compatibility contract, without weakening either.

## Proposed Decision

1. **Literal paths.** `git_add.paths` denotes literal workspace-relative file or
   directory paths, not options or Git pathspec expressions. A directory includes
   its bounded subtree. Option-looking and wildcard-containing real filenames
   remain usable. Reject empty, outside-workspace and escaping ancestor paths
   before mutation. Correct subworkspace resolution without staging sibling paths.
   Workspace-contained absolute paths remain supported after the same checks.
   Subtree bounds describe authorization scope; any resource-limit rejection is
   explicit and whole-request, never silent partial staging.
2. **Actual effects.** Bind invocation approval to canonical repository/worktree
   identity, selected paths and actual index/metadata locations. Source content
   is read; staging writes Git metadata. Preserve index entries outside the
   selected scope. Linked worktrees use their actual index, not an assumed
   `<workspace>/.git/index`. External metadata never becomes an implicit grant.
3. **Existing facets first.** Use existing Path/Command/Remote facets and existing
   explicit external-path authorization where sufficient. Do not add a Git enum,
   persistent grant format or public API merely for convenience. If existing
   carriers cannot faithfully bind these effects, return with a concrete API
   amendment and migration plan before implementing a broader authority surface.
   Existing facets express admission policy; they do not by themselves provide an
   immutable execution plan. Today `permission_profile` is separate from execution
   and `ToolExecutionAuthorization` carries Path authorization, not a complete
   Git command/configuration identity. Prove the binding through the production
   chain before claiming that existing carriers suffice.
4. **Repository-wide operations.** Commit, checkout and pull declare their actual
   repository effects, not a subworkspace-only write. `commit all=true` includes
   tracked worktree staging; ordinary commit consumes the existing index. Push
   and pull retain network facets and safe structured argument boundaries.
   Repair same-family scope defects found by T7 without adding new Git operations.
5. **Configured execution.** Host Git and repository-configured hooks, filters,
   signing and transport helpers require execution authorization; file approval
   alone cannot authorize arbitrary configured programs. Preserve their normal
   semantics by first reusing existing admission, process and authorization
   mechanisms, after verifying they can bind the reviewed execution/configuration
   plan. If they cannot, follow item 3's amendment gate. If a configuration
   cannot be safely represented/enforced, refuse before mutation with a clear
   reason and explicit separately authorized fallback. Never silently disable
   hooks or filters, silently stage unfiltered bytes, or automatically invoke shell.
   Execution admission is not OS containment or a guarantee that a hook has only
   filesystem effects. Review the actual executor capability and any network or
   external-path effects; do not infer confinement from a Command facet.
6. **Environment and lifetime.** Bind Git environment/configuration affecting
   repository/index selection; inherited redirection cannot substitute a different
   target after approval. Host calls have bounded time/output and owned cancellation
   using existing process boundaries. Missing host Git and errors degrade safely.
   Cancellation bounds process lifetime, not a transaction rollback. Report known
   partial effects truthfully and never automatically retry a Git mutation after
   an ambiguous timeout/cancellation.
7. **Filesystem caveats.** Distinguish staging a final symlink entry from following
   an ancestor symlink outside the workspace. Cover deleted and broken-link entries.
   Revalidate identity before mutation and fail closed on relevant changes; do not
   claim that path rechecks alone provide race-free or OS-enforced confinement.
   Ordinary file canonicalization must not turn authorization of a symlink entry
   into authorization of its target. Bind the entry/parent and metadata effects
   consistently; use item 3's amendment gate if the carrier cannot represent this.
8. **Diff fidelity.** Staged means HEAD→index; unstaged means index→worktree.
   Untracked files are excluded from ordinary unstaged diff. Preserve Rust-native
   gix as primary and existing bounded ref-to-ref bridge semantics. Represent new,
   deleted, binary, mode and symlink changes distinctly; errors are not empty diffs.
   Preserve valid ref comparisons while rejecting option/argument injection;
   compatibility does not preserve an unsafe argument boundary.

## Compatibility And Boundaries

Literal paths deliberately remove implicit option/pathspec interpretation from
`git_add`; document migration to explicit paths/directories. More complete facets
may request approval where prior profiles understated effects. No automatic grant,
force push, hook bypass, credential probe, network test or release is authorized.
No new dependencies, native exception or unsafe code is authorized by this proposal.

If public Rust APIs must change, record exact inventory and migration before
implementation; source-breaking changes are next pre-1.0 minor only. This contract
does not permit dropping ordinary Git functionality simply to make tests pass.
Unsupported configurations must have an explicit explanation and tracked residual.

## Verification

Production Agent admission/Runtime composition must reproduce the reported failure
on an identified build and then demonstrate the exact two-file request succeeds
after approval, preserves unrelated staged entries, and cannot mutate on denial.
Cover invalid resources, literal special filenames, directories, deletions,
subworkspaces, linked worktrees, symlinks, environment redirection, configured
programs, timeout/cancellation and same-family permission profiles.

Diff tests independently cover staged-only, unstaged-only, three versions of one
file, additions/deletions, unborn HEAD, binary/mode/symlink changes, literal path
component boundaries, truncation and untracked exclusion. Existing tests that add
an unstaged edit to make a staged file visible do not prove staged-only behavior.
Independent permission/security/API review, standard preflight and exact-head CI
remain required. Revert the bounded repair if approval/effect binding regresses.

## Related

- [I300](../iterations/I300-structured-git-repair.md)
- [ADR-010](010-git-search-tool-dependency-boundary.md)
- [ADR-026](026-multi-resource-tool-permissions.md)
- [ADR-047](047-external-path-tool-authorization.md)
- [Handoff T7](../tasks/2026-10-10-session-todo-handoff.md)
