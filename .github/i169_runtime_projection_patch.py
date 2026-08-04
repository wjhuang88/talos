from pathlib import Path

REVIEWED_HEAD = "bb835b0525da68273739198bb4cc5afc546adab4"


def replace_once(path: str, old: str, new: str, label: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match in {path}, found {count}")
    file.write_text(text.replace(old, new, 1))


# 1. Centralize provider/model/variant -> effective runtime options materialization.
replace_once(
    "crates/talos-cli/src/model_lifecycle.rs",
    '''    VariantResolution {
        reasoning_effort: variant
            .reasoning_effort
            .clone()
            .filter(|_| model_capabilities.reasoning),
        diagnostic: None,
    }
}

/// Applies a variant change to a Config in-memory.
''',
    '''    VariantResolution {
        reasoning_effort: variant
            .reasoning_effort
            .clone()
            .filter(|_| model_capabilities.reasoning),
        diagnostic: None,
    }
}

/// Materializes the declarative provider/model/variant identity into the exact
/// effective runtime configuration consumed by Provider construction.
///
/// Persisted configuration keeps the stable variant identity separate from
/// derived request options. Every runtime reconstruction must call this helper
/// so live switching, startup, resume, new/fork and headless modes cannot
/// disagree about the first Provider request.
pub(crate) fn materialize_runtime_model_config(
    config: &Config,
) -> (Config, VariantResolution) {
    let mut runtime_config = config.clone();
    let all_models = config.all_models();
    let metadata = talos_config::model::find_model_by_provider(
        &all_models,
        &config.provider,
        &config.model,
    );
    let resolution = metadata.map_or_else(
        || resolve_variant(config.variant.as_deref(), &[], &ModelCapabilities::default()),
        |model| {
            resolve_variant(
                config.variant.as_deref(),
                &model.variants,
                &model.capabilities,
            )
        },
    );

    if let Some(reasoning_effort) = resolution.reasoning_effort.clone() {
        let provider = runtime_config
            .providers
            .entry(runtime_config.provider.clone())
            .or_default();
        let reasoning = provider
            .models
            .entry(runtime_config.model.clone())
            .or_default()
            .reasoning
            .get_or_insert_with(ReasoningOptions::default);
        reasoning.effort = Some(reasoning_effort);
    }

    (runtime_config, resolution)
}

/// Applies a variant change to a Config in-memory.
''',
    "insert shared runtime materialization boundary",
)

replace_once(
    "crates/talos-cli/src/model_lifecycle.rs",
    '''    let mut runtime_model_config = model_config.clone();
    let all_models = model_config.all_models();
    let metadata = talos_config::model::find_model_by_provider(
        &all_models,
        &model_config.provider,
        &model_config.model,
    );
    let resolution = metadata.map_or_else(
        || resolve_variant(variant.as_deref(), &[], &ModelCapabilities::default()),
        |model| resolve_variant(variant.as_deref(), &model.variants, &model.capabilities),
    );
    if let Some(diagnostic) = resolution.diagnostic.as_deref() {
        tracing::warn!(
            provider = %model_config.provider,
            model = %model_config.model,
            "{diagnostic}"
        );
    }
    if let Some(reasoning_effort) = resolution.reasoning_effort {
        let provider = runtime_model_config
            .providers
            .entry(runtime_model_config.provider.clone())
            .or_default();
        let reasoning = provider
            .models
            .entry(runtime_model_config.model.clone())
            .or_default()
            .reasoning
            .get_or_insert_with(ReasoningOptions::default);
        reasoning.effort = Some(reasoning_effort);
    }

''',
    '''    let (runtime_model_config, _) = materialize_runtime_model_config(model_config);

''',
    "replace live-switch-only variant projection",
)

# 2. Make build_provider the mandatory materialization boundary for every root.
replace_once(
    "crates/talos-cli/src/provider_setup.rs",
    '''pub(crate) fn build_provider(
    config: &Config,
    api_key: &str,
    mock: bool,
) -> Arc<dyn talos_core::provider::LanguageModel> {
    if mock {
''',
    '''pub(crate) fn build_provider(
    config: &Config,
    api_key: &str,
    mock: bool,
) -> Arc<dyn talos_core::provider::LanguageModel> {
    let (runtime_config, resolution) =
        crate::model_lifecycle::materialize_runtime_model_config(config);
    if let Some(diagnostic) = resolution.diagnostic.as_deref() {
        tracing::warn!(
            provider = %config.provider,
            model = %config.model,
            variant = ?config.variant,
            "{diagnostic}"
        );
    }
    let config = &runtime_config;

    if mock {
''',
    "materialize every provider construction",
)

provider_setup = Path("crates/talos-cli/src/provider_setup.rs")
provider_text = provider_setup.read_text().rstrip()
provider_tests = r'''

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::Message;

    fn openai_o3_config(variant: Option<&str>) -> Config {
        let mut config = Config::default();
        config
            .set_active_model("openai/o3")
            .expect("builtin openai/o3 model");
        crate::model_lifecycle::apply_variant_change(&mut config, variant);
        config
    }

    fn reasoning_effort_from_real_request(config: &Config) -> Option<String> {
        let provider = build_provider(config, "sk-test-materialization", false);
        let preview = provider
            .request_preview(&[Message::User {
                content: "runtime materialization probe".to_string(),
            }])
            .expect("real provider request preview");
        preview
            .pointer("/body/reasoning_effort")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    }

    #[test]
    fn provider_build_materializes_variant_reasoning_into_real_request() {
        let config = openai_o3_config(Some("high-reasoning"));
        assert_eq!(
            reasoning_effort_from_real_request(&config).as_deref(),
            Some("high")
        );
    }

    #[test]
    fn provider_build_normalizes_default_to_baseline_request_options() {
        let baseline = openai_o3_config(None);
        let explicit_default = openai_o3_config(Some("DEFAULT"));
        assert_eq!(explicit_default.variant, None);
        assert_eq!(
            reasoning_effort_from_real_request(&explicit_default),
            reasoning_effort_from_real_request(&baseline)
        );
    }

    #[test]
    fn provider_build_uses_safe_fallback_for_unknown_variant() {
        let baseline = openai_o3_config(None);
        let unknown = openai_o3_config(Some("deleted-variant"));
        let (_, resolution) =
            crate::model_lifecycle::materialize_runtime_model_config(&unknown);
        assert!(resolution.diagnostic.is_some());
        assert_eq!(
            reasoning_effort_from_real_request(&unknown),
            reasoning_effort_from_real_request(&baseline)
        );
    }
}
'''
if "provider_build_materializes_variant_reasoning_into_real_request" in provider_text:
    raise SystemExit("provider materialization tests already present")
provider_setup.write_text(provider_text + provider_tests + "\n")

# 3. Upgrade Session reconstruction tests from resolver-only assertions to the
#    actual production Provider request preview.
replace_once(
    "crates/talos-cli/src/mode_runtime.rs",
    '''    fn append_activation(session: &talos_session::Session, activation: &SessionModelActivation) {
        session
            .append_with_metadata(
                &talos_core::message::Message::System {
                    content: format!("[System] activation {}", activation.activation_id),
                    cache_markers: Vec::new(),
                },
                session_model_activation_metadata(activation).unwrap(),
            )
            .unwrap();
    }

''',
    '''    fn append_activation(session: &talos_session::Session, activation: &SessionModelActivation) {
        session
            .append_with_metadata(
                &talos_core::message::Message::System {
                    content: format!("[System] activation {}", activation.activation_id),
                    cache_markers: Vec::new(),
                },
                session_model_activation_metadata(activation).unwrap(),
            )
            .unwrap();
    }

    fn provider_reasoning_effort(config: &Config) -> Option<String> {
        let provider = crate::provider_setup::build_provider(config, "sk-test-session", false);
        let preview = provider
            .request_preview(&[talos_core::message::Message::User {
                content: "session reconstruction probe".to_string(),
            }])
            .expect("real provider request preview");
        preview
            .pointer("/body/reasoning_effort")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    }

''',
    "insert production provider preview helper",
)

replace_once(
    "crates/talos-cli/src/mode_runtime.rs",
    '''        let catalog = config.all_models();
        let metadata =
            talos_config::model::find_model_by_provider(&catalog, &config.provider, &config.model)
                .expect("openai/o3 catalog metadata");
        let resolution = crate::model_lifecycle::resolve_variant(
            config.variant.as_deref(),
            &metadata.variants,
            &metadata.capabilities,
        );
        assert_eq!(
            resolution.reasoning_effort,
            Some(talos_core::model::ReasoningEffort::High)
        );
        assert_eq!(resolution.diagnostic, None);
''',
    '''        assert_eq!(provider_reasoning_effort(&config).as_deref(), Some("high"));
''',
    "replace manual high variant resolver assertion",
)

replace_once(
    "crates/talos-cli/src/mode_runtime.rs",
    '''        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.variant, None);
''',
    '''        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.variant, None);
        assert_eq!(provider_reasoning_effort(&config), None);
''',
    "assert baseline request after default normalization",
)

replace_once(
    "crates/talos-cli/src/mode_runtime.rs",
    '''        let catalog = config.all_models();
        let metadata =
            talos_config::model::find_model_by_provider(&catalog, &config.provider, &config.model)
                .expect("openai/o3 catalog metadata");
        let resolution = crate::model_lifecycle::resolve_variant(
            config.variant.as_deref(),
            &metadata.variants,
            &metadata.capabilities,
        );
        assert_eq!(resolution.reasoning_effort, None);
        assert!(resolution.diagnostic.is_some());
''',
    '''        let (_, resolution) =
            crate::model_lifecycle::materialize_runtime_model_config(&config);
        assert_eq!(resolution.reasoning_effort, None);
        assert!(resolution.diagnostic.is_some());
        assert_eq!(provider_reasoning_effort(&config), None);
''',
    "assert unknown variant request fallback",
)

# 4. Compare exact normalized identities for /model no-op detection.
replace_once(
    "crates/talos-cli/src/session_handlers.rs",
    '''#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_model(
''',
    '''fn same_model_activation_identity(current: &Config, requested: &Config) -> bool {
    crate::mode_runtime::SessionModelIdentity::new(
        &current.provider,
        &current.model,
        current.variant.as_deref(),
    ) == crate::mode_runtime::SessionModelIdentity::new(
        &requested.provider,
        &requested.model,
        requested.variant.as_deref(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_session_model(
''',
    "insert normalized model no-op comparator",
)

replace_once(
    "crates/talos-cli/src/session_handlers.rs",
    '''    if config.model == parsed_model_id
        && config.provider == provider_name
        && config.variant == variant
    {
        return None;
    }
''',
    '''    if same_model_activation_identity(config, &model_config) {
        return None;
    }
''',
    "replace raw variant no-op guard",
)

replace_once(
    "crates/talos-cli/src/session_handlers.rs",
    '''        assert!(
            ui_rx.try_recv().is_err(),
            "boundary helper emits only two outputs"
        );
    }
}
''',
    '''        assert!(
            ui_rx.try_recv().is_err(),
            "boundary helper emits only two outputs"
        );
    }

    #[tokio::test]
    async fn normalized_default_model_selection_is_a_true_noop() {
        let dir = tempfile::tempdir().unwrap();
        let session_manager =
            talos_session::SessionManager::with_dir(dir.path().join("sessions"));
        let session = session_manager.create_session("project", "").unwrap();
        let (raw_sq_tx, _raw_sq_rx) = mpsc::channel(4);
        let transition = Arc::new(Mutex::new(
            SessionTransition::new(raw_sq_tx.clone(), session.clone()).unwrap(),
        ));
        let generation_before = transition.lock().await.active_generation();

        let (ui_tx, mut ui_rx) = mpsc::unbounded_channel();
        let (session_watch_tx, session_watch_rx) = watch::channel(session.clone());
        let (sq_tx_watch_tx, _sq_tx_watch_rx) = watch::channel(raw_sq_tx);
        let (bridge_rx_update_tx, _bridge_rx_update_rx) = mpsc::unbounded_channel();
        let hooks = build_hook_registry(true);
        let mcp_config = talos_config::McpConfig::default();

        let mut config = Config::default();
        config
            .set_active_model("openai/o3")
            .expect("builtin openai/o3 model");
        crate::model_lifecycle::apply_variant_change(&mut config, None);

        let result = handle_session_model(
            &transition,
            &ui_tx,
            &config,
            &hooks,
            dir.path(),
            &mcp_config,
            &session_watch_tx,
            &sq_tx_watch_tx,
            &bridge_rx_update_tx,
            &session_watch_rx,
            &session_manager,
            "o3@DEFAULT".to_string(),
            Some("openai".to_string()),
            true,
        )
        .await;

        assert!(result.is_none());
        assert_eq!(
            transition.lock().await.active_generation(),
            generation_before,
            "equivalent baseline selection must not fence or replace the Actor"
        );
        assert!(
            session
                .read_entries()
                .unwrap()
                .iter()
                .all(|entry| crate::mode_runtime::session_model_activation_from_metadata(
                    &entry.metadata
                )
                .is_none()),
            "equivalent baseline selection must not append an activation record"
        );
        assert!(
            ui_rx.try_recv().is_err(),
            "a true no-op must not publish status or error output"
        );
    }
}
''',
    "add production-path normalized default no-op test",
)

# 5. Add a source-layout guard that prevents future Provider roots from
#    bypassing the shared variant materialization boundary.
source_test = Path("crates/talos-cli/tests/i169_source_layout.rs")
source_text = source_test.read_text().rstrip()
source_append = r'''

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
'''
if "provider_construction_cannot_bypass_variant_materialization" in source_text:
    raise SystemExit("provider construction source guard already present")
source_test.write_text(source_text + source_append + "\n")

# 6. Synchronize owner evidence without advancing lifecycle state.
doc_sections = {
    "docs/backlog/active/TUI-044-transactional-batched-steering-turn.md": '''## 2026-08-04 runtime variant projection remediation

The current review cycle centralizes provider/model/variant runtime materialization before every
Provider construction. Durable Session identity remains declarative, while one shared boundary
resolves the normalized variant and projects its effective request options for live switching,
startup, resume, new/fork and headless construction paths. Equivalent baseline spellings such as
`None`, empty and `default` are true no-ops and cannot advance generation or append an activation.

Production request-preview tests verify the actual OpenAI request `reasoning_effort`, rather than
only the persisted label or resolver result. This remains implementation evidence under review;
TUI-044 and I169 remain Active, ADR-056 remains Proposed, and Issue #119 remains Open.''',
    "docs/iterations/I169-batched-steering-turn.md": '''## 2026-08-04 runtime variant projection remediation

The latest remediation removes the split between durable variant identity and effective Provider
request options. `build_provider` now materializes the normalized active variant through the same
shared projection used by live model replacement, covering initial TUI construction, resume,
new/fork and other CLI roots. Real request previews assert High reasoning restoration and bounded
unknown/default fallback. The `/model` no-op guard compares normalized activation identities, so an
equivalent baseline selection performs no fence, replacement or durable activation append.

This evidence is a fresh review handoff only. I169/TUI-044 remain Active, ADR-056 remains Proposed,
and Issue #119 remains Open pending exact-head CI and independent review.''',
    "docs/decisions/056-transactional-steering-submission-boundary.md": '''## Runtime projection clarification (2026-08-04)

A durable provider/model/variant identity is not sufficient unless every runtime reconstruction
deterministically materializes the same effective Provider request options. The implementation under
review therefore uses one shared materialization boundary before Provider construction across live
switch, startup, resume, new/fork and headless roots. Persisted identity remains declarative; derived
reasoning/options are reconstructed from the selected catalog variant on every build. Equivalent
baseline spellings are normalized before no-op decisions and cannot create a new activation.

This clarification records the implementation contract while ADR-056 remains Proposed. It does not
complete I169/TUI-044, close Issue #119, approve PR #131 or authorize merge.''',
}
for name, section in doc_sections.items():
    path = Path(name)
    text = path.read_text().rstrip()
    if section.splitlines()[0] in text:
        raise SystemExit(f"document section already present: {name}")
    path.write_text(text + "\n\n" + section + "\n")
