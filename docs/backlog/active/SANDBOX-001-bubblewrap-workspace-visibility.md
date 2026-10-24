# SANDBOX-001: Bubblewrap Workspace Visibility

**Status**: Ready / Unclaimed
**Type**: Defect
**Source Issue**: #757
**Parent**: None; prerequisite for I300 Linux validation, not Git scope expansion
**Selected Iteration**: [I307](../../iterations/I307-bubblewrap-workspace-visibility.md)

## Requirement

An authorized Linux workspace below /tmp must stay visible and be the actual
working directory. Host temporary siblings and writes outside the workspace
retain isolation. Extra read paths remain read-only; a missing workspace must
not cause execution in HOME.

I307 owns full scope, acceptance, validation, documentation and claim.
No implementation is authorized until that governance record reaches main.
User documentation target: crates/talos-sandbox/README.md.
No completion evidence exists. This does not close T7 or I161 residuals.
