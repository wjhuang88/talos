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