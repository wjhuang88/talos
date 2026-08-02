use std::fs;
use std::path::Path;

#[test]
fn transactional_bridge_sources_are_normal_rust_modules() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    for removed in ["build.rs", "src/tui_bridge_impl.rs", "src/tests_impl.rs"] {
        assert!(
            !crate_root.join(removed).exists(),
            "temporary I169 source generator artifact must remain removed: {removed}"
        );
    }

    for canonical in ["src/tui_bridge.rs", "src/tests.rs"] {
        let source = fs::read_to_string(crate_root.join(canonical))
            .unwrap_or_else(|error| panic!("read canonical source {canonical}: {error}"));
        assert!(
            !source.contains("OUT_DIR") && !source.contains("include!("),
            "canonical source must not depend on generated include output: {canonical}"
        );
    }
}
