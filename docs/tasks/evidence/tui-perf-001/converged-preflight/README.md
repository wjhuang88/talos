# Frozen candidate validation

Implementation commits:211ad3b0 and e7ccb938. Validation ran on b7923cf7 plus
the archived source.patch (SHA256 a094f6b20c45158d08fd23326a9307c486c60426bd7892ea41b19b5700fc3539).
Runtime/test sources did not change during validation. Commits afterward only
record those already-tested changes and evidence.

`./scripts/release_preflight.sh` exited0 with its passed marker:148 Rust
summaries,5317 passed,0 failed,0 ignored; workspace check/Clippy, formatting,
governance/claims and external SDK fixture passed. Wall23m9.774s,
CPU10m44.402s,peak2GiB,swap0. Linking was slow under memory reclaim.

`cargo xwin check --locked -p talos-tui --tests --examples --target x86_64-pc-windows-msvc`
exited0:wall1.575s,CPU1.248s,peak706.5MiB,swap0.
Both used Rust1.99.0, jobs1/testthreads1, CPU100%/affinity0,
memory high2G/max3G,swap0,Tasks128,Nice10,LC_ALL=C/LANG=C and the shared
disk-backed target. Raw logs retained here. Five Python summary tests passed;
an isolated comparator check confirmed Debug exact-count routing, unchanged
Release thresholds and conservative legacy fallback. Independent incremental
source reviews found no blocker in either runtime patch.

This closes local validation for the frozen candidate, not whole-task acceptance.
Resize latency, bounded heap, native terminals and final remote gates remain open.
