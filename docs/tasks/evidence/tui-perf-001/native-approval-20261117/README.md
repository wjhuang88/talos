# Native production approval acceptance — 2026-11-17

The maintainer launched the installed Talos without `--mock`, in the temporary
non-repository workspace `/tmp/talos-native-quiet-zG2fZw`, and used `/auto off`.
The supplied screenshots show auto assistance disabled. This uses the production
permission/tool path, unlike the previous display-only replay.

Maintainer observations: the outside-workspace file-write request displayed an
approval panel with complete text and a normal cursor. Selecting deny closed the
panel and restored input. A subsequent `stat` request for the same path displayed
another approval panel; selecting 1 (approve once) executed it and returned
`file not found`. The screenshot shows the denial acknowledgement, approved stat,
missing-file result, and restored composer. A later read-only host check also
found `/tmp/talos-approval-uat-20261117.txt` absent. No actual file write was approved.

Receipt: PID879502, installed executable SHA256
5347488615ef9be965fa5a156dde541dca9eea098084698577d10941ed39c49c, matching the
previous installation receipt for source b2a7d471. Terminal geometry was 92x47
when sampled after the operations, not measured during the approval itself.
Environment remains the previously recorded Ghostty/niri Linux setup. Screenshot
contents were inspected in this conversation; screenshots are not copied into the
repository to avoid archiving unrelated UI/account metadata.

Pass scope: production approval presentation, denial/resumption, approve-once
keyboard response for stat, and absence of the denied write. This does not prove
always-approve semantics, every permission path, pixel latency, or Windows behavior.
Native Windows remains unavailable and unwaived. No performance tests were rerun.
