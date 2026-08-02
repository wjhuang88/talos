// Temporary I169 compile probe: build.rs applies only the exact known compiler
// fixes to the receipt-state implementation and writes the result to OUT_DIR.
// This wrapper and build.rs will be removed before review readiness.
include!(concat!(env!("OUT_DIR"), "/tui_bridge_impl.rs"));
