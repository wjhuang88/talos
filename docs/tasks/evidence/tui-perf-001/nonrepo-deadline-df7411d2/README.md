# Non-repository Debug animation with specific deadline observations

Candidate df7411d2; source-bound build receipt included. Baseline 52263c62 uses
its original Debug binary. Main was rebased to 55617ad7 before this candidate;
dependency changes and host drift prevent attributing CPU deltas solely to the
instrumentation refinement. Three baseline-first pairs, history 10000, 35 seconds
with 5 seconds warmup, input probes and process-tree sampling. No concurrent
builds, tests or other experiments were launched by this task.

All evaluated checks pass. CPU medians 1.747760% baseline / 1.517662% candidate;
peak candidate PSS 46312 KiB. Visible animation observations by round: 201, 200,
201; all zero over 50 ms. Maximum animation delay 4.399247 ms. Stable layout and
history rebuild counts are zero. Mixed deadline observations remain available.

This is independent new-version evidence. It does not explain or erase the
pre-rebase failure in ../git-nonrepo-debug/. That report still fails the legacy
conservative check. Neither the prior mixed histogram nor this later pass proves
host scheduling caused that failure. Native pixel timing and other workload,
long-run, allocation and remote gates remain outstanding.
