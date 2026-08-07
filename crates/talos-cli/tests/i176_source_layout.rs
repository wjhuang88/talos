use std::fs;
use std::path::Path;

#[test]
fn session_workflows_stay_private_behind_the_existing_facade() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let facade = fs::read_to_string(crate_root.join("src/session_handlers.rs"))
        .expect("read session handler facade");
    let provider_model =
        fs::read_to_string(crate_root.join("src/session_handlers/provider_model.rs"))
            .expect("read provider/model workflow module");
    let lifecycle = fs::read_to_string(crate_root.join("src/session_handlers/lifecycle.rs"))
        .expect("read Session lifecycle workflow module");

    for module in ["lifecycle", "provider_model"] {
        assert!(facade.contains(&format!("mod {module};")));
        assert!(!facade.contains(&format!("pub mod {module};")));
        assert!(!facade.contains(&format!("pub(crate) mod {module};")));
    }

    for handler in [
        "handle_connect",
        "handle_connect_with_credential",
        "handle_provider_setup",
        "handle_register_custom_provider",
        "handle_session_model",
        "handle_session_model_with_credential",
    ] {
        assert!(provider_model.contains(&format!("fn {handler}(")));
        assert!(!facade.contains(&format!("fn {handler}(")));
        assert!(!lifecycle.contains(&format!("fn {handler}(")));
    }

    for handler in [
        "handle_session_delete",
        "handle_session_new",
        "handle_session_resume",
        "handle_session_fork",
    ] {
        assert!(lifecycle.contains(&format!("fn {handler}(")));
        assert!(!facade.contains(&format!("fn {handler}(")));
        assert!(!provider_model.contains(&format!("fn {handler}(")));
    }

    assert!(facade.contains("pub(crate) use lifecycle::"));
    assert!(facade.contains("pub(crate) use provider_model::"));
    assert!(provider_model.contains("transition: &Arc<Mutex<SessionTransition>>"));
    assert!(lifecycle.contains("transition: &Arc<Mutex<SessionTransition>>"));
    assert!(lifecycle.contains(".publish_commit("));
    assert!(lifecycle.contains("emit_session_identity_after_queue_clear("));
    assert!(lifecycle.contains("talos storage maintenance --reconcile"));
    assert!(provider_model.contains("same_model_activation_identity(config, &model_config)"));

    for responsibility in [&provider_model, &lifecycle] {
        assert!(!responsibility.contains("pub mod lifecycle"));
        assert!(!responsibility.contains("pub mod provider_model"));
    }
}
