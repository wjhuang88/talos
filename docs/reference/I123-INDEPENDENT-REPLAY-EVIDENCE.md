# I123 Independent Replay Evidence

Use this record only for the remaining I123 F132 acceptance evidence. The replay must be run by
an operator other than the person who produced the prior same-host records, on a different host or
CI environment. Do not include credentials, local home paths, or unrelated command output.

## Operator Procedure

1. Start from the reviewed I123 branch or its merged commit.
2. Build the binary using the repository-pinned toolchain:

   ```bash
   cargo build --locked -p talos-cli
   ```

3. Run the packet:

   ```bash
   bash scripts/replay_trial.sh target/debug/talos
   ```

4. Preserve the emitted JSON file under `target/trial-replay/` and record its path, commit, host
   operating system/architecture, and whether `pwsh` was available.
5. Compare the `steps[].exit_code` and `steps[].summary` values with the prior packet. Ignore
   `generated_utc` and absolute `binary` paths. Explain any platform, architecture, or PowerShell
   availability differences.
6. Attach the redacted JSON record and the completed template below to the I123 review or PR.

## Evidence Template

| Field | Record |
|---|---|
| Operator | |
| Date/time (UTC) | |
| Commit | |
| Host OS / architecture | |
| Rust version | |
| PowerShell availability/version | |
| Replay JSON artifact | |
| Overall exit | |
| Step result comparison | |
| Expected variance | |
| Unexpected variance / follow-up | |

Acceptance is met only when this independent record reports exit 0 and any variance is explained.
It does not authorize a tag, publish, deployment, or a REL-002 claim.
