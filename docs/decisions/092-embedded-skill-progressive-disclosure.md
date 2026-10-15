# ADR-092: Embedded Skill Progressive Disclosure

## Status

Accepted. Proposed in #720 (SKILL-006 / I303 governance claim) and effective at its main merge
`9b7ba281` on 2026-10-15. The maintainer selected the full #719 scope (A+B+C+D) and the
Single-maintainer merge path.

## Context

Issue #719 (OBeiBuddy) reports that embedders using `talos-runtime` only reach Skill Level 0:
`RuntimeBuilder::skill_index` renders name/description, `RuntimeHandle` cannot send
`SessionOp::SetSkillContext`, and the only confined Level 2 loader is private to `talos-cli`.
The public `SkillManager::load_reference` joins `file_path` onto the skill directory without
rejecting `..`, absolute paths, symlink escape, non-regular files or oversize files.

SKILL-002 deliberately excluded automatic activation by fuzzy intent matching for the interactive
CLI. Embedded expert products have no `/skills activate` step and currently duplicate a private
`skill_read` tool with their own confinement and chunking.

## Decision

### 1. Shared confined disclosure in `talos-skill` (Requirement A/C)

`talos-skill` owns one public, Rust-native disclosure surface used by CLI, runtime and embedders:

- Confined file read for a parsed `Skill`: reject empty, absolute, root/prefix and `..`
  components; canonicalize both the skill directory and the candidate and require the candidate
  to remain under the directory (symlink escape is rejected); require a regular file; enforce a
  byte ceiling before reading (default 2 MiB); require UTF-8.
- Bounded file listing of the skill directory: symlinks are not followed, entries are relative,
  sorted and capped (default 256 entries, depth 8) with an explicit truncation flag.
- UTF-8-boundary text chunking with offset continuation (default chunk 48 KiB).
- Activation-context rendering equivalent to the current CLI format (`## Skill Body`, appended
  `## Reference: <path>` blocks) with the existing 24 KiB body and 16 KiB reference budgets.

`SkillManager::load_reference` delegates to the confined read. Rejections reuse the existing
`SkillError::IoError` variant (`InvalidInput`/`PermissionDenied` kinds) so the exhaustive public
`SkillError` enum is not broken. Rejecting traversal is an intentional security behavior change.
`talos-cli::skill_runtime` must use the shared functions instead of its private copy; CLI output
format and budgets are unchanged.

### 2. Runtime handle activation (Requirement A)

`RuntimeHandle` gains additive async methods that set and clear the activated skill context by
enqueuing `SessionOp::SetSkillContext`, using the same admission gate as `submit`. Commands are
FIFO, so a context set before `submit` applies to that turn's provider request. While a turn is
active or submissions are pending, the session keeps its existing behavior: the change is rejected
and reported as `SessionEvent::Error` ("cannot change active skill while a turn is active"); the
method's `Ok` means accepted for ordering, not applied. The context is Agent-local, is not written
to durable session history and must be re-applied after a runtime rebuild. It is rendered in the
cacheable system-prompt section, so changing it invalidates the provider prompt-cache prefix.
`SessionOp`/`talos-core` are not changed.

### 3. Opt-in model-initiated `skill_read` tool (Requirement B)

`RuntimeBuilder` gains an explicit opt-in that binds a set of parsed `Skill` values and registers a
built-in read-only `skill_read` tool. It is never registered by `RuntimeBuilder::new()`, a preset,
or an empty binding, and an explicitly registered caller tool with the same name wins.

- Input: `skill` (must be one of the bound names), optional `reference` (skill-relative path) and
  optional `offset`. The parameter is deliberately not named `path`/`file` so the permission
  resource extractor does not reinterpret a skill-relative path as a workspace path.
- Output: without `reference`, the Level 1 `SKILL.md` body plus the bounded file listing; with
  `reference`, the Level 2 file. Both use the confined read and chunking from section 1.
- Nature `Read`, native provenance, executed through the normal Agent permission pipeline (default
  `Read` is Allow; host rules may deny it). It cannot read outside the bound skill directories.
- Multiple bound skills per session are supported; this does not change `ActivatedSkillContext`.
- Results are ordinary tool results: they enter conversation history, are persisted under the
  configured durable-session policy, are lower authority than runtime/user instructions per
  ADR-074, and do not modify the system prompt or its cache prefix.

This is not the fuzzy auto-activation excluded by SKILL-002: the host opts in and chooses the bound
set; the model requests specific content through an auditable tool call; no prompt is mutated.

### 4. Documentation (Requirement D)

`RuntimeBuilder::skill_index` rustdoc, the Runtime SDK contract and the README Skills section state
that the index injects Level 0 only and point to sections 2/3, including migration from a
downstream custom `skill_read`.

## Compatibility And Migration

All runtime/skill API changes are additive except the deliberate `load_reference` rejection of
unsafe paths. Re-exports at the `talos_runtime` root follow RUNTIME-006 single-dependency rules.
No dependency, sandbox, `talos-permission` or `unsafe` change. No release is authorized.

## Residual Risks

Canonicalize-then-open leaves a check/use window against a concurrently replaced symlink or
special file inside a host-owned skill directory (a FIFO swapped in after the check could block the
blocking reader); the opened handle's metadata and a bounded read cap memory. Hardlinks to outside
inodes are not detectable by path confinement. Skill roots are host-trusted and require write
access to exploit; these windows are documented, not closed.
Skill content remains untrusted model input.
