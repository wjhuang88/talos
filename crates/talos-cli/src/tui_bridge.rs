// Temporary I169 compile probe: the implementation stays in a sibling include
// while receipt-state compilation is stabilized. This wrapper will be removed
// before review readiness and the implementation will be restored as a normal
// rustfmt-managed module.
include!("tui_bridge_impl.rs");
