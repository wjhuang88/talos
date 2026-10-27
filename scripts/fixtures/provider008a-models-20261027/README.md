# Frozen I306 protocol projection input

`api.json.gz` losslessly archives https://models.dev/api.json acquired 2026-10-27 UTC.
Gzip uses mtime 0. This catalog contains public model metadata, not live endpoint responses.

- Uncompressed: 5,955,477 bytes; 232 providers.
- Uncompressed SHA256: `4e97268dc7621205179df996f68a2c17a32284997d291cb8c6a29c43ffc33c74`.
- Gzip SHA256: `2237ad226802d24d2659764e058eb0bf85d857a834df41a799cceec084c4b0ff`.
- Upstream repository: https://github.com/anomalyco/models.dev .
- Exact MIT notice is included in `LICENSE`, retrieved from the upstream dev branch on the same day.

Run `python3 scripts/audit_provider008a_projection.py` (Python 3.11+) from an offline checkout.
It verifies the frozen hash, independently computes every packaged model's admission and compares
all protocol keys and the preserved non-protocol semantic digest. Standard release preflight
runs this audit; normal Rust builds do not decompress, fetch or execute this maintenance input.
The production generator/runtime share `protocol_signal.rs`; this audit explicitly reproduces the
accepted SDK/shape and endpoint rules for an exhaustive cross-check of this fixed dataset.

The 2026-10-26 input hash recorded in earlier ledger checkpoints could not be recovered. A current
download had different bytes and is deliberately archived as a new input. Reprojection withdraws
six Chat hints for models now absent upstream; model records themselves and all non-protocol data
are preserved. Historical old snapshot/hash/counts are retained as history, not newly verified facts.
