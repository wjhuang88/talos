from pathlib import Path


def read(path):
    return Path(path).read_text()


def write(path, text):
    Path(path).write_text(text)


def replace_once(text, old, new, label):
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


# mode_runtime.rs: persist and restore exact Session-owned activation identity.
path = "crates/talos-cli/src/mode_runtime.rs"
text = read(path)
text = replace_once(
    text,
    "use anyhow::{Result, anyhow};\n",
    "use anyhow::{Result, anyhow};\nuse serde::{Deserialize, Serialize};\nuse sha2::{Digest, Sha256};\n",
    "mode_runtime imports",
)
text = replace_once(
    text,
    'const TODO_PROMPT_MAX_CHARS: usize = 2400;\n',
    '''const TODO_PROMPT_MAX_CHARS: usize = 2400;
const SESSION_MODEL_ACTIVATION_PREFIX: &str = "talos:model-activation:v1:";

/// Exact model runtime identity owned by a durable Session.
///
/// `None`, an empty string, and the legacy `default` spelling all normalize to
/// the same baseline variant so live activation and restart reconstruction
/// cannot disagree about the effective Provider request options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SessionModelIdentity {
    pub provider: String,
    pub model: String,
    pub variant: Option<String>,
}

impl SessionModelIdentity {
    #[must_use]
    pub(crate) fn new(provider: &str, model: &str, variant: Option<&str>) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            variant: crate::model_lifecycle::normalize_variant_id(variant).map(str::to_string),
        }
    }

    #[must_use]
    pub(crate) fn display_name(&self) -> String {
        match self.variant.as_deref() {
            Some(variant) => format!("{}/{}@{variant}", self.provider, self.model),
            None => format!("{}/{}", self.provider, self.model),
        }
    }
}

/// Machine-readable, append-only activation record.
///
/// The generation and exact previous/target identities form the immutable
/// logical-operation identity. `activation_id` is a deterministic digest of
/// that tuple, so a retry of the same interrupted activation is stable while a
/// new transition — including a variant-only transition — is distinct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SessionModelActivation {
    pub version: u8,
    pub activation_id: String,
    pub generation: u64,
    pub previous: SessionModelIdentity,
    pub target: SessionModelIdentity,
}

impl SessionModelActivation {
    #[must_use]
    pub(crate) fn new(
        generation: u64,
        previous: SessionModelIdentity,
        target: SessionModelIdentity,
    ) -> Self {
        let canonical = serde_json::to_vec(&(generation, &previous, &target))
            .expect("model activation identity contains only serializable values");
        let digest = Sha256::digest(canonical);
        let suffix: String = digest
            .iter()
            .take(16)
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Self {
            version: 1,
            activation_id: format!("model-activation-g{generation}-{suffix}"),
            generation,
            previous,
            target,
        }
    }
}

pub(crate) fn session_model_activation_metadata(
    activation: &SessionModelActivation,
) -> Result<SessionMetadata, serde_json::Error> {
    let mut metadata =
        session_metadata_for_model(&activation.target.model, &activation.target.provider);
    metadata.raw_content = Some(format!(
        "{SESSION_MODEL_ACTIVATION_PREFIX}{}",
        serde_json::to_string(activation)?
    ));
    Ok(metadata)
}

pub(crate) fn session_model_activation_from_metadata(
    metadata: &SessionMetadata,
) -> Option<SessionModelActivation> {
    let payload = metadata
        .raw_content
        .as_deref()?
        .strip_prefix(SESSION_MODEL_ACTIVATION_PREFIX)?;
    let activation: SessionModelActivation = serde_json::from_str(payload).ok()?;
    if activation.version != 1 {
        return None;
    }
    let expected = SessionModelActivation::new(
        activation.generation,
        activation.previous.clone(),
        activation.target.clone(),
    );
    if activation.activation_id != expected.activation_id
        || metadata.provider.as_deref() != Some(activation.target.provider.as_str())
        || metadata.model.as_deref() != Some(activation.target.model.as_str())
    {
        return None;
    }
    Some(activation)
}
''',
    "mode_runtime activation types",
)
old_block = '''fn latest_session_model_info(session: &talos_session::Session) -> Option<(String, String)> {
    session
        .read_entries()
        .ok()?
        .into_iter()
        .rev()
        .find_map(
            |entry| match (entry.metadata.model, entry.metadata.provider) {
                (Some(model), Some(provider)) => Some((model, provider)),
                (Some(model), None) => Some((model, String::new())),
                _ => None,
            },
        )
}

pub(crate) fn apply_session_model_to_config(config: &mut Config, session: &talos_session::Session) {
    let Some((model, provider)) = latest_session_model_info(session) else {
        return;
    };
    let model_ref = if provider.is_empty() || model.starts_with(&format!("{provider}/")) {
        model
    } else {
        format!("{provider}/{model}")
    };
    if let Err(e) = config.set_active_model(&model_ref) {
        tracing::warn!(
            session_id = %session.id,
            model = %model_ref,
            "failed to restore session model metadata: {e}"
        );
    }
}
'''
new_block = '''enum LatestSessionModelInfo {
    Activation(SessionModelIdentity),
    Legacy { model: String, provider: String },
}

fn latest_session_model_info(session: &talos_session::Session) -> Option<LatestSessionModelInfo> {
    let mut legacy = None;
    for entry in session.read_entries().ok()?.into_iter().rev() {
        if entry.role == "system"
            && let Some(activation) = session_model_activation_from_metadata(&entry.metadata)
        {
            return Some(LatestSessionModelInfo::Activation(activation.target));
        }
        if legacy.is_none() {
            legacy = match (entry.metadata.model, entry.metadata.provider) {
                (Some(model), Some(provider)) => {
                    Some(LatestSessionModelInfo::Legacy { model, provider })
                }
                (Some(model), None) => Some(LatestSessionModelInfo::Legacy {
                    model,
                    provider: String::new(),
                }),
                _ => None,
            };
        }
    }
    legacy
}

pub(crate) fn apply_session_model_to_config(config: &mut Config, session: &talos_session::Session) {
    let Some(model_info) = latest_session_model_info(session) else {
        return;
    };
    let (model, provider, exact_variant) = match model_info {
        LatestSessionModelInfo::Activation(identity) => {
            (identity.model, identity.provider, Some(identity.variant))
        }
        LatestSessionModelInfo::Legacy { model, provider } => (model, provider, None),
    };
    let model_ref = if provider.is_empty() || model.starts_with(&format!("{provider}/")) {
        model
    } else {
        format!("{provider}/{model}")
    };
    if let Err(e) = config.set_active_model(&model_ref) {
        tracing::warn!(
            session_id = %session.id,
            model = %model_ref,
            "failed to restore session model metadata: {e}"
        );
        return;
    }
    if let Some(variant) = exact_variant {
        crate::model_lifecycle::apply_variant_change(config, variant.as_deref());
    }
}
'''
text = replace_once(text, old_block, new_block, "mode_runtime restore block")

test_insert = r'''
    fn activation_test_session(name: &str) -> talos_session::Session {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.keep();
        let path = root.join(format!("{name}.jsonl"));
        std::fs::write(&path, b"").unwrap();
        talos_session::Session::new(Uuid::new_v4(), "test".into(), String::new(), path)
    }

    fn append_activation(
        session: &talos_session::Session,
        activation: &SessionModelActivation,
    ) {
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

    #[test]
    fn session_activation_restores_exact_variant_and_reasoning_options() {
        let session = activation_test_session("restore-variant");
        let activation = SessionModelActivation::new(
            7,
            SessionModelIdentity::new("openai", "o3", Some("low-reasoning")),
            SessionModelIdentity::new("openai", "o3", Some("high-reasoning")),
        );
        append_activation(&session, &activation);

        let mut config = Config::default();
        config.variant = Some("low-reasoning".into());
        apply_session_model_to_config(&mut config, &session);

        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "o3");
        assert_eq!(config.variant.as_deref(), Some("high-reasoning"));

        let catalog = config.all_models();
        let metadata = talos_config::model::find_model_by_provider(
            &catalog,
            &config.provider,
            &config.model,
        )
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
    }

    #[test]
    fn session_activation_normalizes_default_and_clears_stale_variant() {
        let session = activation_test_session("clear-variant");
        let activation = SessionModelActivation::new(
            8,
            SessionModelIdentity::new("openai", "o3", Some("high-reasoning")),
            SessionModelIdentity::new("openai", "gpt-4o", Some("default")),
        );
        assert_eq!(activation.target.variant, None);
        append_activation(&session, &activation);

        let mut config = Config::default();
        config.variant = Some("high-reasoning".into());
        apply_session_model_to_config(&mut config, &session);

        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.variant, None);
    }

    #[test]
    fn unknown_persisted_variant_restores_identity_but_uses_safe_fallback() {
        let session = activation_test_session("unknown-variant");
        let activation = SessionModelActivation::new(
            9,
            SessionModelIdentity::new("openai", "o3", None),
            SessionModelIdentity::new("openai", "o3", Some("deleted-variant")),
        );
        append_activation(&session, &activation);

        let mut config = Config::default();
        apply_session_model_to_config(&mut config, &session);
        assert_eq!(config.variant.as_deref(), Some("deleted-variant"));

        let catalog = config.all_models();
        let metadata = talos_config::model::find_model_by_provider(
            &catalog,
            &config.provider,
            &config.model,
        )
        .expect("openai/o3 catalog metadata");
        let resolution = crate::model_lifecycle::resolve_variant(
            config.variant.as_deref(),
            &metadata.variants,
            &metadata.capabilities,
        );
        assert_eq!(resolution.reasoning_effort, None);
        assert!(resolution.diagnostic.is_some());
    }
'''
idx = text.rfind("\n}")
if idx == -1:
    raise SystemExit("mode_runtime tests closing brace not found")
text = text[:idx] + "\n" + test_insert + text[idx:]
write(path, text)

# model_lifecycle.rs: establish exact activation record before publication.
path = "crates/talos-cli/src/model_lifecycle.rs"
text = read(path)
text = replace_once(
    text,
    "use crate::mcp_runtime::McpSessionRuntime;\n",
    "use crate::mcp_runtime::McpSessionRuntime;\nuse crate::mode_runtime::{SessionModelActivation, SessionModelIdentity};\n",
    "model_lifecycle imports",
)
old_apply = '''pub(crate) fn apply_variant_change(config: &mut Config, new_variant: Option<&str>) -> bool {
    let changed = config.variant.as_deref() != new_variant;
    if changed {
        config.variant = new_variant.map(str::to_string);
    }
    changed
}
'''
new_apply = '''#[must_use]
pub(crate) fn normalize_variant_id(variant: Option<&str>) -> Option<&str> {
    variant
        .map(str::trim)
        .filter(|id| !id.is_empty() && !id.eq_ignore_ascii_case("default"))
}

pub(crate) fn apply_variant_change(config: &mut Config, new_variant: Option<&str>) -> bool {
    let normalized = normalize_variant_id(new_variant);
    let changed = config.variant.as_deref() != normalized;
    if changed {
        config.variant = normalized.map(str::to_string);
    }
    changed
}
'''
text = replace_once(text, old_apply, new_apply, "normalize apply_variant_change")
text = replace_once(
    text,
    "    pub previous_provider: String,\n    pub model_id: String,\n",
    "    pub previous_provider: String,\n    pub previous_variant: Option<String>,\n    pub model_id: String,\n",
    "RebuildSessionParams previous_variant field",
)
text = replace_once(
    text,
    "        previous_provider,\n        model_id,\n",
    "        previous_provider,\n        previous_variant,\n        model_id,\n",
    "RebuildSessionParams destructure previous_variant",
)
old_marker = '''    let switch_marker = model_switch_marker(
        &previous_provider,
        &previous_model,
        &model_config.provider,
        &model_config.model,
    );
    let marker_metadata = crate::mode_runtime::session_metadata_for_model(
        &runtime_model_config.model,
        &runtime_model_config.provider,
    );
'''
new_marker = '''    let previous_identity = SessionModelIdentity::new(
        &previous_provider,
        &previous_model,
        previous_variant.as_deref(),
    );
    let target_identity = SessionModelIdentity::new(
        &model_config.provider,
        &model_config.model,
        variant.as_deref(),
    );
'''
text = replace_once(text, old_marker, new_marker, "model activation identities")
old_call = '''    let history = match persist_switch_marker_and_read_final_history(
        &mut transition_guard,
        &current_session,
        &switch_marker,
        marker_metadata,
    )
'''
new_call = '''    let history = match establish_model_activation_and_read_final_history(
        &mut transition_guard,
        &current_session,
        &previous_identity,
        &target_identity,
    )
'''
text = replace_once(text, old_call, new_call, "production activation helper call")

helper_anchor = '''/// Retires the old same-Session runtime, durably establishes the model-switch
/// marker, and returns the exact persisted history used to construct the
/// replacement Actor. No replacement command or event route is reachable
/// while this activation barrier is running.
async fn persist_switch_marker_and_read_final_history(
'''
new_helpers = r'''fn model_activation_tail(
    session: &Session,
) -> Result<Option<SessionModelActivation>, FinalHistoryError> {
    Ok(session
        .read_entries()
        .map_err(FinalHistoryError::Persist)?
        .last()
        .and_then(|entry| {
            crate::mode_runtime::session_model_activation_from_metadata(&entry.metadata)
        }))
}

fn verified_activation_history(
    session: &Session,
    activation: &SessionModelActivation,
    marker: &Message,
) -> Result<Vec<Message>, FinalHistoryError> {
    let tail = model_activation_tail(session)?;
    if tail.as_ref() != Some(activation) {
        return Err(FinalHistoryError::Persist(SessionError::ParseError(
            "durable model activation record is not the exact final Session entry".to_string(),
        )));
    }
    let history = session
        .read_messages()
        .map_err(FinalHistoryError::Persist)?;
    if !history
        .last()
        .is_some_and(|message| model_switch_markers_match(message, marker))
    {
        return Err(FinalHistoryError::Persist(SessionError::ParseError(
            "durable model activation marker is not the final replacement history entry"
                .to_string(),
        )));
    }
    Ok(history)
}

/// Reuses an already committed exact activation after an interrupted
/// post-commit/pre-publication cut point. The full machine record, including
/// generation and variant-aware previous/target identities, must match; visible
/// marker text alone is never an idempotency key.
async fn establish_model_activation_and_read_final_history(
    transition: &mut SessionTransition,
    session: &Session,
    previous: &SessionModelIdentity,
    target: &SessionModelIdentity,
) -> Result<Vec<Message>, FinalHistoryError> {
    let recovered = SessionModelActivation::new(
        transition.active_generation(),
        previous.clone(),
        target.clone(),
    );
    if model_activation_tail(session)?.as_ref() == Some(&recovered) {
        let marker = model_switch_marker_for_activation(&recovered);
        return verified_activation_history(session, &recovered, &marker);
    }

    persist_model_activation_and_read_final_history(transition, session, previous, target).await
}

/// Retires the old same-Session runtime, durably establishes the exact
/// variant-aware activation identity, and returns the persisted history used to
/// construct the replacement Actor. No replacement command or event route is
/// reachable while this activation barrier is running.
async fn persist_model_activation_and_read_final_history(
    transition: &mut SessionTransition,
    session: &Session,
    previous: &SessionModelIdentity,
    target: &SessionModelIdentity,
) -> Result<Vec<Message>, FinalHistoryError> {
    let generation = transition
        .quiesce_same_session(session)
        .await
        .map_err(FinalHistoryError::Fence)?;
    let activation =
        SessionModelActivation::new(generation, previous.clone(), target.clone());
    let marker = model_switch_marker_for_activation(&activation);
    let marker_metadata = crate::mode_runtime::session_model_activation_metadata(&activation)
        .map_err(|error| {
            FinalHistoryError::Persist(SessionError::ParseError(format!(
                "failed to encode model activation record: {error}"
            )))
        })?;

    if model_activation_tail(session)?.as_ref() != Some(&activation) {
        session
            .append_with_metadata(&marker, marker_metadata)
            .map_err(FinalHistoryError::Persist)?;
    }

    verified_activation_history(session, &activation, &marker)
}

/// Test-only compatibility harness for the earlier content marker regression
/// cases. Production activation uses the machine-readable helper above.
#[cfg(test)]
async fn persist_switch_marker_and_read_final_history(
'''
text = replace_once(text, helper_anchor, new_helpers, "insert exact activation helpers")
marker_anchor = '''fn model_switch_marker(
    previous_provider: &str,
'''
new_marker_fn = '''fn model_switch_marker_for_activation(activation: &SessionModelActivation) -> Message {
    Message::System {
        content: format!(
            "[System] Model switch activation {}: {} -> {}.\\n[System] Active model for subsequent requests: {}.",
            activation.activation_id,
            activation.previous.display_name(),
            activation.target.display_name(),
            activation.target.display_name(),
        ),
        cache_markers: Vec::new(),
    }
}

#[cfg(test)]
fn model_switch_marker(
    previous_provider: &str,
'''
text = replace_once(text, marker_anchor, new_marker_fn, "variant-aware marker function")

test_anchor = '''    #[test]
    fn duplicate_model_ids_keep_provider_side_ids_for_structured_switching() {
'''
activation_tests = r'''    #[tokio::test]
    async fn model_switch_activation_distinguishes_sequential_variant_changes() {
        let temp = tempfile::tempdir().unwrap();
        let manager = SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-variant-activation-sequence")
            .unwrap();
        let session = durable.session().clone();

        let (first_tx, first_rx) = mpsc::channel(1);
        drop(first_rx);
        let mut first_transition = SessionTransition::new(first_tx, session.clone()).unwrap();
        establish_model_activation_and_read_final_history(
            &mut first_transition,
            &session,
            &SessionModelIdentity::new("openai", "o3", Some("low-reasoning")),
            &SessionModelIdentity::new("openai", "o3", Some("high-reasoning")),
        )
        .await
        .unwrap();
        drop(first_transition);

        let (second_tx, second_rx) = mpsc::channel(1);
        drop(second_rx);
        let mut second_transition = SessionTransition::new(second_tx, session.clone()).unwrap();
        establish_model_activation_and_read_final_history(
            &mut second_transition,
            &session,
            &SessionModelIdentity::new("openai", "o3", Some("high-reasoning")),
            &SessionModelIdentity::new("openai", "o3", Some("low-reasoning")),
        )
        .await
        .unwrap();

        let entries = session.read_entries().unwrap();
        let activations: Vec<_> = entries
            .iter()
            .filter_map(|entry| {
                crate::mode_runtime::session_model_activation_from_metadata(&entry.metadata)
            })
            .collect();
        assert_eq!(activations.len(), 2);
        assert_eq!(activations[0].generation, 1);
        assert_eq!(activations[1].generation, 2);
        assert_eq!(
            activations[0].target.variant.as_deref(),
            Some("high-reasoning")
        );
        assert_eq!(
            activations[1].target.variant.as_deref(),
            Some("low-reasoning")
        );
        assert_ne!(activations[0].activation_id, activations[1].activation_id);
    }

    #[tokio::test]
    async fn model_switch_activation_retry_matches_full_logical_identity_once() {
        let temp = tempfile::tempdir().unwrap();
        let manager = SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-variant-activation-retry")
            .unwrap();
        let session = durable.session().clone();
        let previous = SessionModelIdentity::new("openai", "o3", Some("low-reasoning"));
        let target = SessionModelIdentity::new("openai", "o3", Some("high-reasoning"));

        let (first_tx, first_rx) = mpsc::channel(1);
        drop(first_rx);
        let mut first_transition = SessionTransition::new(first_tx, session.clone()).unwrap();
        let first_history = establish_model_activation_and_read_final_history(
            &mut first_transition,
            &session,
            &previous,
            &target,
        )
        .await
        .unwrap();
        drop(first_transition);

        let (restart_tx, restart_rx) = mpsc::channel(1);
        drop(restart_rx);
        let mut restarted_transition =
            SessionTransition::new(restart_tx, session.clone()).unwrap();
        let retried_history = establish_model_activation_and_read_final_history(
            &mut restarted_transition,
            &session,
            &previous,
            &target,
        )
        .await
        .unwrap();

        let entries = session.read_entries().unwrap();
        let activations: Vec<_> = entries
            .iter()
            .filter_map(|entry| {
                crate::mode_runtime::session_model_activation_from_metadata(&entry.metadata)
            })
            .collect();
        assert_eq!(activations.len(), 1);
        assert_eq!(activations[0].generation, 1);
        assert_eq!(activations[0].target, target);
        assert_eq!(format!("{first_history:?}"), format!("{retried_history:?}"));
    }

    #[test]
    fn model_switch_activation_marker_includes_exact_variants() {
        let activation = SessionModelActivation::new(
            3,
            SessionModelIdentity::new("openai", "o3", Some("low-reasoning")),
            SessionModelIdentity::new("openai", "o3", Some("high-reasoning")),
        );
        let Message::System { content, .. } =
            model_switch_marker_for_activation(&activation)
        else {
            unreachable!();
        };

        assert!(content.contains("openai/o3@low-reasoning"));
        assert!(content.contains("openai/o3@high-reasoning"));
        assert!(content.contains(&activation.activation_id));
    }

    #[test]
    fn default_variant_is_one_normalized_baseline_identity() {
        assert_eq!(normalize_variant_id(None), None);
        assert_eq!(normalize_variant_id(Some("")), None);
        assert_eq!(normalize_variant_id(Some("default")), None);
        assert_eq!(normalize_variant_id(Some("DEFAULT")), None);
        assert_eq!(
            SessionModelIdentity::new("openai", "gpt-4o", Some("default")).variant,
            None
        );
    }

'''
text = replace_once(text, test_anchor, activation_tests + test_anchor, "activation identity tests")
write(path, text)

# session_handlers.rs: pass previous exact variant into the activation record.
path = "crates/talos-cli/src/session_handlers.rs"
text = read(path)
text = replace_once(
    text,
    '''    let previous_model = config.model.clone();
    let previous_provider = config.provider.clone();
    let mut model_config = config.clone();
''',
    '''    let previous_model = config.model.clone();
    let previous_provider = config.provider.clone();
    let previous_variant = config.variant.clone();
    let mut model_config = config.clone();
''',
    "normal model previous variant",
)
text = replace_once(
    text,
    '''        previous_model,
        previous_provider,
        model_id: parsed_model_id.clone(),
''',
    '''        previous_model,
        previous_provider,
        previous_variant,
        model_id: parsed_model_id.clone(),
''',
    "normal model pass previous variant",
)
text = replace_once(
    text,
    '''    let previous_model = config.model.clone();
    let previous_provider = config.provider.clone();
    let credential_provider = cred.provider.clone();
''',
    '''    let previous_model = config.model.clone();
    let previous_provider = config.provider.clone();
    let previous_variant = config.variant.clone();
    let credential_provider = cred.provider.clone();
''',
    "credential model previous variant",
)
text = replace_once(
    text,
    '''        previous_model,
        previous_provider,
        model_id: parsed_model_id.clone(),
''',
    '''        previous_model,
        previous_provider,
        previous_variant,
        model_id: parsed_model_id.clone(),
''',
    "credential model pass previous variant",
)
write(path, text)

# Static source-order guard: exact machine identity must precede publication.
path = "crates/talos-cli/tests/i169_source_layout.rs"
text = read(path)
old_test = r'''#[test]
fn model_switch_marker_durability_precedes_replacement_publication() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(crate_root.join("src/model_lifecycle.rs"))
        .expect("read model lifecycle source")
        .replace("\r\n", "\n");

    let barrier = source
        .find("persist_switch_marker_and_read_final_history(")
        .expect("durable model-switch activation barrier");
    let prepare = source
        .find("transition_guard.prepare(")
        .expect("replacement prepare boundary");
    let commit = source
        .find("transition_guard.commit(")
        .expect("replacement commit boundary");
    let publish = source
        .find("sq_tx_watch_tx.send(")
        .expect("replacement SQ publication");
    let success = source
        .find("MessageSource::System,\n                success_message")
        .expect("model-switch success publication");

    assert!(barrier < prepare);
    assert!(prepare < commit);
    assert!(commit < publish);
    assert!(publish < success);
    assert!(!source.contains("failed to persist model switch marker"));
    assert!(source.contains("No replacement route was published"));

    let helper_start = source
        .find("async fn persist_switch_marker_and_read_final_history(")
        .expect("activation barrier helper definition");
    let helper_end = source[helper_start..]
        .find("\nfn model_switch_markers_match(")
        .map(|offset| helper_start + offset)
        .expect("activation barrier helper end");
    let helper = &source[helper_start..helper_end];

    let fence = helper
        .find(".quiesce_same_session(session)")
        .expect("old runtime retirement");
    let tail_check = helper
        .find(".read_entries()")
        .expect("durable marker tail check");
    let marker_commit = helper
        .find(".append_with_metadata(switch_marker, marker_metadata)")
        .expect("durable marker commit");
    let replay = helper
        .find(".read_messages()")
        .expect("canonical replay after marker commit");

    assert!(fence < tail_check);
    assert!(tail_check < marker_commit);
    assert!(marker_commit < replay);
    assert!(!helper.contains("transition_guard.prepare("));
    assert!(!helper.contains("transition_guard.commit("));
    assert!(!helper.contains("sq_tx_watch_tx.send("));
}
'''
new_test = r'''#[test]
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
        .find("sq_tx_watch_tx.send(")
        .expect("replacement SQ publication");
    let success = source
        .find("MessageSource::System,\n                success_message")
        .expect("model-switch success publication");

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
        .find(".quiesce_same_session(session)")
        .expect("old runtime retirement");
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

    assert!(fence < activation);
    assert!(activation < tail_check);
    assert!(tail_check < marker_commit);
    assert!(marker_commit < replay);
    assert!(!helper.contains("left_content == right_content"));
    assert!(!helper.contains("transition_guard.prepare("));
    assert!(!helper.contains("transition_guard.commit("));
    assert!(!helper.contains("sq_tx_watch_tx.send("));
}
'''
text = replace_once(text, old_test, new_test, "source layout activation guard")
write(path, text)

governance_note = r'''

## 2026-08-04 exact activation identity remediation

The current review cycle requires same-Session model activation durability to carry the complete
ADR-048 identity: provider, model, and normalized variant. PR #131 now records a machine-readable
activation object containing the durable target generation, deterministic activation ID, exact
previous identity, and exact target identity. `None`, empty, and `default` variants normalize to the
same baseline identity.

Visible marker text is not the idempotency key. Only an exact activation object may be reused after
an interrupted commit/publication cut point; a new intentional switch, including a variant-only
switch on the same provider/model, creates a distinct activation. Session startup restores the
variant from this Session-owned record before Provider construction, so a later global config write
failure cannot silently restore different request semantics.

This is implementation evidence under review. TUI-044 and I169 remain Active, ADR-056 remains
Proposed, and Issue #119 remains Open.
'''
for path in [
    "docs/backlog/active/TUI-044-transactional-batched-steering-turn.md",
    "docs/iterations/I169-batched-steering-turn.md",
    "docs/decisions/056-transactional-steering-submission-boundary.md",
]:
    text = read(path)
    if "## 2026-08-04 exact activation identity remediation" not in text:
        text = text.rstrip() + governance_note + "\n"
        write(path, text)
