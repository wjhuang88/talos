# Branch cleanup checkpoint

## Evidence and actions

Baseline main/origin/main: `0626aa25d353caf6ce0810a8b01547d29cc92e1f`. Only main was checked out in a worktree. Fetched all remotes with prune before classification. Deleted 10 local branches with `git branch -d`, and 35 origin branches whose tips were ancestors of origin/main. No force deletion, force push, or integration of unreviewed unique patches was performed.

The initial full ref/SHA inventory is in `/tmp/talos-branch-inventory.txt` on the execution host; the unmerged remote inventory is `/tmp/talos-unmerged-remote-branches.txt`. These are temporary local evidence, not durable repository attachments. Deleted commits remain reachable from the baseline main.

Local deleted branches: backup/ui-before-approval-integration-20261117, backup/ui-before-main-integration-20261117, feat/bg-default-continuation, fix/approval-history-20261116, fix/background-custody-foundation-20261115, fix/checkpoint-post-hook-authority-20261118, fix/edit-line-locations-20261116, fix/recovery-query-followup-20261117, fix/session-recovery-integration-20261111, fix/session-ui-continuity-20261116.

Remote deleted groups: five session repair branches in the first batch; thirty ancestor-only claim/closeout/docs/feat/fix/gov/governance/integration/test branches in the second batch. Archive, backup, checkpoint, and the I337/copilot branches were retained pending purpose review.

## Remaining review (not complete)

Five non-main local branches retained:
- backup/background-custody-before-recovery-20261116 (`8f9dcc29`)
- backup/recovery-keep0-validation-6483241c (`6483241c`)
- fix/session-recovery-storage-20261111 (`1f5602eb`)
- recovery-pr917-backup-20261115 (`0b8ccbe4`)
- test/session-handoff-integration-20261117 (`ff068667`)

`git cherry main` reports five non-equivalent patches on the handoff integration branch and one on the storage branch. The latter changes 11 files with 1,300 additions and 48 removals, including persistent session state. This is not evidence that the changes are needed: compare with the later recovery implementations and owner acceptance before merging. Keep the branches until this semantic review is complete.

The initial unmerged remote inventory contains 202 entries, predominantly archives/backups plus development branches; individual semantic and task/PR ownership review remains pending. Do not batch merge or delete these based on naming or patch-id alone.

A final fetch observed another actor force-updating `origin/test/i334-e3a-memory-first-query` from `06670fbc` to `a38f9c09`. This branch was not touched. Refresh and revalidate exact remote tips before further deletion; the repository is concurrently active.

## Resume

Review remaining branches against owner documents and live PR status. For squash/rebase histories, compare actual behavior and required acceptance, not ancestry alone. Merge only reviewed, needed changes with applicable governance/security checks. Preserve archives with recovery purpose. This checkpoint records partial cleanup, not completion of the overall branch audit.

## Local disposition at 1c80425b

Removed the local storage and PR #917 intermediate branch names after verifying
the ancestor chain 1f5602eb → 0b8ccbe4 → 6483241c. The last commit is retained
by backup/recovery-keep0-validation-6483241c. The earlier custody backup was
retired after patch comparison with #953. The #1045 branch was retired locally
and on origin after merge.

Retired Windows diagnostics stash 622a573e. Its CLI fixture, session dependency
and durable implementation match the #966 merge. The index had no staged delta
and the stash had no untracked parent. The recovery owner records the later
preflight and the #962 fixture integration.

Retain integrated candidate ff068667 for comparison with the recovery and UI
owners. They link combined candidates 0e0ee5d3, ceb86be4 and ff068667 to their
standalone validations. Keep the remaining recovery assertions with the recovery
owner. The November 11 handoff lists the approval, diagnostics and edit-preview
acceptance items that can be exercised on main.

The active local stashes are 12d2c1a0 (combined handoff), 851b86f0 (recovery)
and cf940fc8 (performance evidence for #923). Of the 66 untracked performance
files, 54 match main at the same path, eight paths are absent and four differ.
Eleven older rebase snapshots remain under refs/archive/stash-review on this
host for their distinct file versions. These local refs preserve original data
while the existing task owners track outstanding acceptance.

## Mainline reconciliation at 1c80425b

The seven recovery patches with differing patch IDs have corresponding mainline
implementations, in candidate order: 8f163abc, 23c8a118, 715d0198, a1a0a210,
29e8d58b, 63d195e2, and 0f8cff08. Comparing the touched Rust files at each
pair leaves only the independent storage_contracts exports and workspace path
display changes. Replaying the recovery series would carry older versions of
these files into the current implementation.

The integrated handoff's approval fixture was reconciled by #962. Main carries
the query-bound implementation in 56ceeef1, source access and retry handling
in 15bdc03f, user history spacing in 405ce9da and subsequent tests, and watcher
fixture isolation in 8bcb7b47. The watcher file matches the integrated tip
byte for byte. Main includes all three named user-separator regressions from
6aa91f46.

All 54 same-path identical untracked performance files include the original
sample archives. The eight missing paths are generated Python bytecode. The
four differing files are the tracker, native acceptance note, comparison script
and its tests. Main retains the native note in history and updates the tracker
through #960. The current comparison script and tests add mixed-deadline
handling. Windows terminal acceptance continues under #923.
