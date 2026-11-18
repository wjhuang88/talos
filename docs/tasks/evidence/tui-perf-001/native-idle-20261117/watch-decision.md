# Ignored-file notification investigation

Current watcher source from 6d5651df was compiled directly, without edits, against preserved dependencies selected by the original Cargo fingerprint. Receipt and commands: watch-receipt.json. This is a diagnostic build, not a full-workspace validation.

Five operations per isolated fixture: reads 0 notifications; untracked ignored writes 5; tracked-but-ignored writes 7; ordinary writes 6. Kernel event count is not one-to-one with writes. Results: watch-controls.json. No causal attribution to live-session CPU2.67% or the historical479ms tail.

Design constraint: gix0.90 Repository::excludes covers configured global excludes, common-dir info/exclude and nested ignore files. Its cached stack performs filesystem reads and borrows the repository. It must not run in the native notify callback. A callback-level ignore matcher assembled only from root .gitignore would be incorrect for tracked files, nested negations, changed index/config, and deleted/renamed paths.

Safe implementation direction, if live-event evidence warrants this optimization: keep bounded callback handoff; evaluate events on the existing worker with tracked-index precedence and repository-aware exclusion rules. Preserve metadata and ignore-rule events, overflow/failure fallback, unknown-path conservative invalidation, and both rename endpoints. Bound retained event data; do not replace constant flags with an unbounded path queue. Rebuild cached rules on index/config/ignore changes. Tests must cover tracked ignored paths, nested negations, global/info excludes, deletion/rename, index replacement, and overflow.

Next decision gate: establish whether ignored-file events materially contribute to the live idle cost. The isolated result alone does not justify this extra state and I/O. Existing worker coalescing and native fallback remain intact; no production change made.
