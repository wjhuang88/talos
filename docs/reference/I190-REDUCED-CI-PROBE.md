# I190 Reduced CI Probe

This documentation-only change probes the change-aware CI routing introduced by implementation
merge `a69ffa30afed16271885d4ef3d11931ab3189673`.

The pull request containing this file must classify as reduced. The documentation, governance,
remote Issue-owner, and Windows installer gates must still run, while the Unix release preflight
steps and the Windows Rust workspace job must not execute. Actual run identifiers, job conclusions,
and merge evidence belong in the later I190 closeout; this probe file does not certify itself.
