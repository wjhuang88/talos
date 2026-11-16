# Cache retention source audit

Source f725b74b (candidate cef024a1 has identical code).
LayoutCache in app_layout.rs owns one Option of size/metrics/layout, replaced on miss.
ComposerHeightCache in app/frame.rs owns one key containing one input String, replaced
on miss. It is bounded by current input size, not a constant byte budget.
HistoryProjectionCache owns one current-width all_rows/logical_lines pair; full reflow
replaces Arcs instead of appending width-indexed snapshots. Its size is proportional
to retained transcript/projected rows, not independent of history size. The displayed
last_history_projection retains another Arc view until replaced by draw_frame.
Existing repeated_reflow_releases_obsolete_snapshots_after_view_is_dropped verifies
weak references expire after old consumer release over128 edits/appends/resizes.
This test passed in the42 focused tests and final5319 workspace tests.

These are static ownership and test evidence, not heap byte attribution. Earlier
five-minute profile precedes candidate; its835bytes/message growth is not yet fully
assigned to retained transcript versus projection allocations. Do not label general
leak freedom or all long-run acceptance complete from these facts.
