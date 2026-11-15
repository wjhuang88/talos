# Recover pre-rebase measurement source

The original six local implementation commits were rebased onto main 55617ad7.
Old binary receipts intentionally retain their original source commits. This
bundle preserves those exact commit objects and trees, including baselines
52263c62 / 15ef42c1 and candidate 539adfd2, without publishing extra remote
branches. Its prerequisite d355dd68bb2892ddcd9f042b4d83fd9b71fb65ae is on main.

From a clone containing that prerequisite:

```sh
git bundle verify docs/tasks/evidence/tui-perf-001/source-history/pre-rebase.bundle
git fetch docs/tasks/evidence/tui-perf-001/source-history/pre-rebase.bundle refs/heads/perf-pre-rebase
git show 539adfd2:crates/talos-tui/src/app.rs
```

Fetching writes FETCH_HEAD and preserves recoverable objects; no checkout or
remote branch changes are required. The bundle was verified with Git before
archiving. A temporary bare repository provided only the export ref; the shared
project's refs were not changed. SHA-256 is recorded in manifest.json.
