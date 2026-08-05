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

#[test]
fn model_switch_activation_durability_precedes_replacement_publication() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(crate_root.join("src/model_lifecycle.rs"))
        .expect("read model lifecycle source")
        .replace("\r\n", "\n");

    let barrier = source
        .find("establish_model_activation_and_read_final_history(")
        .expect("durable model activation barrier");
    let prepare = source
        .find("transition_guard.prepare(")
        .expect("replacement prepare boundary");
    let commit = source
        .find("transition_guard.commit(")
        .expect("replacement commit boundary");
    let publish = source
        .find(".publish_commit(")
        .expect("acknowledged replacement publication");
    let success = publish
        + source[publish..]
            .find("success_message,")
            .expect("model-switch success publication after acknowledged route install");

    assert!(barrier < prepare);
    assert!(prepare < commit);
    assert!(commit < publish);
    assert!(publish < success);
    assert!(source.contains("previous_variant"));
    assert!(source.contains("SessionModelIdentity::new("));
    assert!(source.contains("No replacement route was published"));

    let helper_start = source
        .find("async fn persist_model_activation_and_read_final_history(")
        .expect("activation barrier helper definition");
    let helper_end = source[helper_start..]
        .find("\n/// Test-only compatibility harness")
        .map(|offset| helper_start + offset)
        .expect("activation barrier helper end");
    let helper = &source[helper_start..helper_end];

    let fence = helper
        .find(".quiesce_same_session_for_activation(session, &activation)")
        .expect("atomic activation stage and old-runtime retirement");
    let activation = helper
        .find("SessionModelActivation::new(")
        .expect("exact generation + variant-aware activation identity");
    let tail_check = helper
        .find("model_activation_tail(session)")
        .expect("machine-readable activation tail check");
    let marker_commit = helper
        .find(".append_with_metadata(&marker, marker_metadata)")
        .expect("durable activation commit");
    let replay = helper
        .find("verified_activation_history(session, &activation, &marker)")
        .expect("canonical replay after activation commit");

    assert!(activation < fence);
    assert!(fence < tail_check);
    assert!(tail_check < marker_commit);
    assert!(marker_commit < replay);
    assert!(!helper.contains("left_content == right_content"));
    assert!(!helper.contains("transition_guard.prepare("));
    assert!(!helper.contains("transition_guard.commit("));
    assert!(!helper.contains(".publish_commit("));
}

#[test]
fn provider_construction_cannot_bypass_variant_materialization() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let provider_setup = fs::read_to_string(crate_root.join("src/provider_setup.rs"))
        .expect("read provider setup source")
        .replace("\r\n", "\n");
    let materialize = provider_setup
        .find("materialize_runtime_model_config(config)")
        .expect("shared runtime materialization call");
    let mock_branch = provider_setup
        .find("if mock {")
        .expect("provider mock branch");
    assert!(materialize < mock_branch);

    let lifecycle = fs::read_to_string(crate_root.join("src/model_lifecycle.rs"))
        .expect("read model lifecycle source");
    assert!(lifecycle.contains("pub(crate) fn materialize_runtime_model_config("));

    let handlers = fs::read_to_string(crate_root.join("src/session_handlers.rs"))
        .expect("read session handlers source");
    assert!(handlers.contains("same_model_activation_identity(config, &model_config)"));
    assert!(!handlers.contains("config.variant == variant"));

    fn visit(directory: &Path, provider_setup: &Path) {
        for entry in fs::read_dir(directory).expect("read source directory") {
            let path = entry.expect("source entry").path();
            if path.is_dir() {
                visit(&path, provider_setup);
                continue;
            }
            if path.extension().and_then(|value| value.to_str()) != Some("rs")
                || path == provider_setup
            {
                continue;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            assert!(
                !source.contains("OpenAIProvider::new(")
                    && !source.contains("AnthropicProvider::new("),
                "Provider construction must remain centralized through build_provider: {}",
                path.display()
            );
        }
    }

    visit(
        &crate_root.join("src"),
        &crate_root.join("src/provider_setup.rs"),
    );
}

#[test]
fn tui_runtime_construction_is_centralized_in_the_shared_builder() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let builder = fs::read_to_string(crate_root.join("src/tui_runtime_builder.rs"))
        .expect("read TUI runtime builder source");
    assert!(builder.contains("AppServerSession::new("));
    assert!(builder.contains("McpSessionRuntime::start("));
    assert!(builder.contains("set_request_budget_spec("));

    for (root, required_builder_boundary) in [
        (
            "src/model_lifecycle.rs",
            "runtime_builder: &'a TuiRuntimeBuilder",
        ),
        (
            "src/session_handlers.rs",
            "runtime_builder: &TuiRuntimeBuilder",
        ),
    ] {
        let source = fs::read_to_string(crate_root.join(root))
            .unwrap_or_else(|error| panic!("read {root}: {error}"));
        for forbidden in [
            "AppServerSession::new(",
            "McpSessionRuntime::start(",
            "build_tui_tool_registry(",
        ] {
            assert!(
                !source.contains(forbidden),
                "ad-hoc production TUI runtime construction escaped the shared builder in {root}: {forbidden}"
            );
        }
        assert!(
            source.contains(required_builder_boundary),
            "production runtime transition must accept the shared TuiRuntimeBuilder in {root}"
        );
    }
}

#[test]
fn every_production_provider_root_installs_the_shared_request_budget() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for root in [
        "src/tui_runtime_builder.rs",
        "src/mode_print.rs",
        "src/mode_inline.rs",
        "src/mode_interactive.rs",
        "src/mode_runners.rs",
    ] {
        let source = fs::read_to_string(crate_root.join(root))
            .unwrap_or_else(|error| panic!("read {root}: {error}"));
        assert!(
            source.contains("set_request_budget_spec("),
            "Provider dispatch root must install the shared output/image budget: {root}"
        );
    }
}
