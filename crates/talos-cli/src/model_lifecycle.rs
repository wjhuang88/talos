//! Model lifecycle helpers for the Talos CLI.
//!
//! Contains the model picker data construction and the shared session rebuild
//! logic used when switching models at runtime.

use std::sync::Arc;

use talos_config::{Config, ReasoningOptions};
use talos_conversation::{ModelPickerData, ModelPickerItem, ModelPickerVariantItem};
use talos_core::message::Message;
use talos_core::model::{ModelCapabilities, ReasoningEffort, VariantDef};
use talos_core::session::{SessionEvent, SessionOp};
#[cfg(test)]
use talos_session::SessionMetadata;
use talos_session::{Session, SessionError};
use tokio::sync::{Mutex, mpsc, watch};

use crate::mode_runtime::{SessionModelActivation, SessionModelIdentity};
use crate::session_transition::SessionTransition;
use crate::tui_runtime_builder::TuiRuntimeBuilder;

fn model_picker_label(
    config: &Config,
    model: &talos_config::model::ModelMetadata,
    is_builtin_provider: bool,
) -> String {
    let context = model
        .context_limit
        .map(|limit| {
            let suffix = if !is_builtin_provider
                && config
                    .providers
                    .get(&model.provider)
                    .and_then(|provider| provider.models.get(&model.id))
                    .is_some_and(|configured| configured.context_limit.is_none())
            {
                " (catalog)"
            } else {
                ""
            };
            format!("{}K{suffix}", limit / 1000)
        })
        .unwrap_or_else(|| "?".to_string());
    format!("{}   {}   {}", model.id, model.provider, context)
}

/// Constructs [`ModelPickerData`] from the given [`Config`].
///
/// Iterates the model catalog, checks provider authentication, and formats display strings. Models from
/// authenticated providers appear in `ready_models`; unauthenticated providers
/// are intentionally omitted from `/model` and handled by `/connect`.
pub(crate) fn build_model_picker_data(config: &Config) -> ModelPickerData {
    let catalog = config.all_models();
    let builtin_providers: std::collections::HashSet<_> = talos_config::model::builtin_providers()
        .into_iter()
        .map(|provider| provider.id)
        .collect();

    let mut ready_models: Vec<ModelPickerItem> = Vec::new();
    for m in &catalog {
        let provider_authed = config.provider_authenticated(&m.provider);
        let pricing_str = m.pricing.as_ref().map(|p| {
            let input = p.input_per_1m.map(|v| format!("${v}")).unwrap_or_default();
            let output = p.output_per_1m.map(|v| format!("${v}")).unwrap_or_default();
            if input.is_empty() && output.is_empty() {
                String::new()
            } else {
                format!("{input}/{output}")
            }
        });
        if provider_authed {
            ready_models.push(ModelPickerItem {
                command: "/model".to_string(),
                // Provider identity travels in the separate `provider` field.
                // Keeping this provider-side ID opaque avoids double-prefixing
                // duplicate model IDs in the structured switch lifecycle.
                model_id: m.id.clone(),
                provider: m.provider.clone(),
                label: model_picker_label(config, m, builtin_providers.contains(&m.provider)),
                context_limit: m.context_limit,
                pricing: pricing_str,
                authenticated: true,
                is_current: m.id == config.model && m.provider == config.provider,
                variants: m
                    .variants
                    .iter()
                    .map(|variant| ModelPickerVariantItem {
                        variant_id: variant.id.clone(),
                        label: variant.label.clone(),
                        provider: m.provider.clone(),
                        model_id: m.id.clone(),
                    })
                    .collect(),
                variant: None,
            });
        }
    }

    let mut recent_items: Vec<ModelPickerItem> = Vec::new();
    let recent_list = crate::recent_models::load_recent_models(None);

    for entry in recent_list.entries {
        if !config.provider_authenticated(&entry.provider) {
            continue;
        }

        let m = catalog
            .iter()
            .find(|c| c.provider == entry.provider && c.id == entry.model_id);
        if let Some(m) = m {
            let is_current = entry.provider == config.provider
                && entry.model_id == config.model
                && entry.variant == config.variant;
            if is_current {
                continue;
            }

            let pricing_str = m.pricing.as_ref().map(|p| {
                let input = p.input_per_1m.map(|v| format!("${v}")).unwrap_or_default();
                let output = p.output_per_1m.map(|v| format!("${v}")).unwrap_or_default();
                if input.is_empty() && output.is_empty() {
                    String::new()
                } else {
                    format!("{input}/{output}")
                }
            });
            recent_items.push(ModelPickerItem {
                command: "/model".to_string(),
                model_id: m.id.clone(),
                provider: m.provider.clone(),
                label: model_picker_label(config, m, builtin_providers.contains(&m.provider)),
                context_limit: m.context_limit,
                pricing: pricing_str,
                authenticated: true,
                is_current: false,
                variants: m
                    .variants
                    .iter()
                    .map(|variant| ModelPickerVariantItem {
                        variant_id: variant.id.clone(),
                        label: variant.label.clone(),
                        provider: m.provider.clone(),
                        model_id: m.id.clone(),
                    })
                    .collect(),
                variant: entry.variant.clone(),
            });
        }
    }

    ModelPickerData {
        recent: recent_items,
        ready_models,
        setup_providers: Vec::new(),
    }
}

/// The safe runtime projection of a selected catalog variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VariantResolution {
    /// Reasoning effort to apply to the provider request, when supported.
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Bounded note for a selected variant absent from the active model catalog.
    pub diagnostic: Option<String>,
}

/// Resolves a selected variant against the active model's catalog metadata.
///
/// The legacy `"default"` identity preserves baseline behavior without a
/// diagnostic. Reasoning overrides are silently omitted when unsupported.
pub(crate) fn resolve_variant(
    variant_id: Option<&str>,
    model_variants: &[VariantDef],
    model_capabilities: &ModelCapabilities,
) -> VariantResolution {
    let Some(variant_id) = variant_id.filter(|id| *id != "default") else {
        return VariantResolution {
            reasoning_effort: None,
            diagnostic: None,
        };
    };

    let Some(variant) = model_variants
        .iter()
        .find(|variant| variant.id == variant_id)
    else {
        return VariantResolution {
            reasoning_effort: None,
            diagnostic: Some(format!(
                "Variant '{variant_id}' not found; using no variant"
            )),
        };
    };

    VariantResolution {
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
pub(crate) fn materialize_runtime_model_config(config: &Config) -> (Config, VariantResolution) {
    let mut runtime_config = config.clone();
    let all_models = config.all_models();
    let metadata =
        talos_config::model::find_model_by_provider(&all_models, &config.provider, &config.model);
    let resolution = metadata.map_or_else(
        || {
            resolve_variant(
                config.variant.as_deref(),
                &[],
                &ModelCapabilities::default(),
            )
        },
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
///
/// Returns `true` when the value actually changed (so the caller can persist).
/// Always assigns — including `None` — so switching to a variant-less model
/// correctly clears any prior variant. This is the single source of truth for
/// variant-clearing semantics across all model-switch entry points.
#[must_use]
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

/// Resolves the model to activate after provider-level credential setup.
///
/// Provider setup rows represent a provider rather than a specific model. After
/// credentials are saved, prefer the current configured model when it belongs
/// to that provider; otherwise use the first catalog model for the provider.
/// Duplicate model IDs are provider-qualified so `Config::set_active_model`
/// resolves the intended provider.
pub(crate) fn provider_setup_target_model(config: &Config, provider: &str) -> Option<String> {
    let catalog = config.all_models();
    let mut id_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for m in &catalog {
        *id_counts.entry(m.id.as_str()).or_default() += 1;
    }

    let target = catalog
        .iter()
        .find(|m| m.provider == provider && m.id == config.model)
        .or_else(|| catalog.iter().find(|m| m.provider == provider))?;

    Some(
        if id_counts.get(target.id.as_str()).copied().unwrap_or(0) > 1 {
            format!("{}/{}", target.provider, target.id)
        } else {
            target.id.clone()
        },
    )
}

/// Parameters for the shared session rebuild logic when switching models.
///
/// This struct bundles all the context needed to rebuild a session for a new
/// model, parameterized by the already-resolved `api_key` so that both the
/// normal model switch path and the credential-first path can share the same
/// implementation.
pub(crate) struct RebuildSessionParams<'a> {
    pub transition: &'a Arc<Mutex<SessionTransition>>,
    pub ui_tx: &'a mpsc::UnboundedSender<talos_conversation::UiOutput>,
    pub model_config: &'a Config,
    pub runtime_builder: &'a TuiRuntimeBuilder,
    pub session_watch_tx: &'a watch::Sender<Session>,
    pub sq_tx_watch_tx: &'a watch::Sender<mpsc::Sender<SessionOp>>,
    pub bridge_rx_update_tx:
        &'a mpsc::UnboundedSender<(Session, mpsc::UnboundedReceiver<SessionEvent>)>,
    pub session_watch_rx: &'a watch::Receiver<Session>,
    pub previous_model: String,
    pub previous_provider: String,
    pub previous_variant: Option<String>,
    pub model_id: String,
    pub variant: Option<String>,
    pub provider_for_status: String,
    pub success_message: String,
}

/// Shared session rebuild logic for model switching.
///
/// Encapsulates the common agent-build + transition logic: resolves model
/// limits, ensures session persistence, reads history, builds SessionConfig,
/// constructs provider+registry+agent, prepares+commits the SessionTransition,
/// and updates watch channels.
///
/// The caller is responsible for resolving the `api_key` and constructing the
/// `success_message` and `provider_for_status` strings, which differ between
/// the normal model switch path and the credential-first path.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn rebuild_session_for_model(params: RebuildSessionParams<'_>) -> bool {
    let RebuildSessionParams {
        transition,
        ui_tx,
        model_config,
        runtime_builder,
        session_watch_tx,
        sq_tx_watch_tx,
        bridge_rx_update_tx,
        session_watch_rx,
        previous_model,
        previous_provider,
        previous_variant,
        model_id,
        variant,
        provider_for_status,
        success_message,
    } = params;

    let (runtime_model_config, _) = materialize_runtime_model_config(model_config);

    let mut current_session = session_watch_rx.borrow().clone();
    if let Err(e) = current_session.ensure_persisted() {
        let text = format!("[Error] Failed to create session file: {e}\n");
        send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
        return false;
    }
    let previous_identity = SessionModelIdentity::new(
        &previous_provider,
        &previous_model,
        previous_variant.as_deref(),
    );
    let target_identity = SessionModelIdentity::new(
        &model_config.provider,
        &model_config.model,
        variant.as_deref(),
    );

    let prepared_runtime = match runtime_builder
        .prepare(&runtime_model_config, &current_session)
        .await
    {
        Ok(runtime) => runtime,
        Err(error) => {
            let text = format!("[Error] Failed to prepare replacement runtime: {error}\n");
            send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
            return false;
        }
    };

    let mut transition_guard = transition.lock().await;
    let history = match establish_model_activation_and_read_final_history(
        &mut transition_guard,
        &current_session,
        &previous_identity,
        &target_identity,
    )
    .await
    {
        Ok(history) => history,
        Err(FinalHistoryError::Fence(error)) => {
            let text = format!(
                "[Error] Failed to fence model switch: {error}. Previous model remains active.
"
            );
            send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
            return false;
        }
        Err(FinalHistoryError::Persist(error)) => {
            let text = format!(
                "[Error] Model switch fenced the old runtime but failed to durably commit the switch marker and final Session history: {error}. No replacement route was published. The Session runtime is stopped; retry the switch, start a new Session, or resume before continuing.
"
            );
            send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
            return false;
        }
    };
    let built_runtime = prepared_runtime.finish(history);
    let handle = built_runtime.handle;
    let actor = built_runtime.actor;
    let sched_pending = built_runtime.pending_scheduler;
    if let Err(error) = transition_guard.prepare_mcp_runtime(built_runtime.mcp_runtime) {
        let text = format!(
            "[Error] Failed to retain replacement MCP runtime after fencing: {error}. The Session runtime is stopped; retry the switch, start a new Session, or resume before continuing.\n"
        );
        send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
        return false;
    }
    let session_for_prepare = current_session.clone();
    if let Err(e) = transition_guard.prepare(handle, session_for_prepare) {
        transition_guard.rollback();
        let text = format!(
            "[Error] Failed to prepare model switch after fencing: {e}. The Session runtime is stopped; retry the switch, start a new Session, or resume before continuing.\n"
        );
        send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
        return false;
    }

    match transition_guard.commit(actor, sched_pending).await {
        Ok(result) => match transition_guard
            .publish_commit(
                result,
                current_session.clone(),
                session_watch_tx,
                sq_tx_watch_tx,
                bridge_rx_update_tx,
            )
            .await
        {
            Ok(_) => {
                send_stream(
                    ui_tx,
                    talos_conversation::MessageSource::System,
                    success_message,
                );
                let (ctx_limit, _) = runtime_model_config.resolve_model_limits();
                let all_models = runtime_model_config.all_models();
                let meta = talos_config::model::find_model_by_provider(
                    &all_models,
                    &runtime_model_config.provider,
                    &runtime_model_config.model,
                );
                let pricing = meta.and_then(|m| m.pricing.as_ref());
                let _ = ui_tx.send(talos_conversation::UiOutput::Status(
                    talos_conversation::StatusSnapshot {
                        model_name: model_id.clone(),
                        provider: provider_for_status,
                        context_limit: Some(ctx_limit),
                        input_price_per_million: pricing.and_then(|p| p.input_per_1m),
                        output_price_per_million: pricing.and_then(|p| p.output_per_1m),
                        variant: variant.clone(),
                        ..Default::default()
                    },
                ));

                let mut recent = crate::recent_models::load_recent_models(None);
                recent.record(crate::recent_models::RecentModelEntry {
                    provider: runtime_model_config.provider.clone(),
                    model_id: runtime_model_config.model.clone(),
                    variant,
                });
                if let Err(e) = crate::recent_models::save_recent_models(&recent, None) {
                    tracing::warn!("Failed to persist recent models: {e}");
                }

                true
            }
            Err(e) => {
                let text = format!("[Error] {e}\n");
                send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
                false
            }
        },
        Err(e) => {
            transition_guard.rollback();
            let text = format!(
                "[Error] Failed to publish model switch after fencing: {e}. The Session runtime is stopped; retry the switch, start a new Session, or resume before continuing.\n"
            );
            send_stream(ui_tx, talos_conversation::MessageSource::Error, text);
            false
        }
    }
}

#[derive(Debug)]
enum FinalHistoryError {
    Fence(String),
    Persist(SessionError),
}

fn model_activation_tail(
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
        let store = talos_session::PendingSubmissionStore::for_session(session);
        let state = store.runtime_state().map_err(|error| {
            FinalHistoryError::Persist(SessionError::ParseError(format!(
                "failed to read Session runtime activation state: {error}"
            )))
        })?;
        if state
            .as_ref()
            .is_some_and(|state| state.activation == recovered)
        {
            store
                .commit_runtime_activation(&recovered.activation_id)
                .map_err(|error| {
                    FinalHistoryError::Persist(SessionError::ParseError(format!(
                        "failed to finalize recovered Session runtime activation: {error}"
                    )))
                })?;
        }
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
    let activation = SessionModelActivation::new(
        transition
            .active_generation()
            .checked_add(1)
            .ok_or_else(|| FinalHistoryError::Fence("runtime generation exhausted".to_string()))?,
        previous.clone(),
        target.clone(),
    );
    transition
        .quiesce_same_session_for_activation(session, &activation)
        .await
        .map_err(FinalHistoryError::Fence)?;
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
    talos_session::PendingSubmissionStore::for_session(session)
        .commit_runtime_activation(&activation.activation_id)
        .map_err(|error| {
            FinalHistoryError::Persist(SessionError::ParseError(format!(
                "failed to commit Session runtime activation: {error}"
            )))
        })?;

    verified_activation_history(session, &activation, &marker)
}

/// Test-only compatibility harness for the earlier content marker regression
/// cases. Production activation uses the machine-readable helper above.
#[cfg(test)]
async fn persist_switch_marker_and_read_final_history(
    transition: &mut SessionTransition,
    session: &Session,
    switch_marker: &Message,
    marker_metadata: SessionMetadata,
) -> Result<Vec<Message>, FinalHistoryError> {
    transition
        .quiesce_same_session(session)
        .await
        .map_err(FinalHistoryError::Fence)?;

    let Message::System {
        content: marker_content,
        ..
    } = switch_marker
    else {
        return Err(FinalHistoryError::Persist(SessionError::ParseError(
            "model-switch marker must be a system message".to_string(),
        )));
    };
    let encoded_marker = format!("__SYSTEM__:{marker_content}");
    let marker_is_durable_tail = session
        .read_entries()
        .map_err(FinalHistoryError::Persist)?
        .last()
        .is_some_and(|entry| entry.role == "system" && entry.content == encoded_marker);

    if !marker_is_durable_tail {
        session
            .append_with_metadata(switch_marker, marker_metadata)
            .map_err(FinalHistoryError::Persist)?;
    }

    let history = session
        .read_messages()
        .map_err(FinalHistoryError::Persist)?;
    if !history
        .last()
        .is_some_and(|message| model_switch_markers_match(message, switch_marker))
    {
        return Err(FinalHistoryError::Persist(SessionError::ParseError(
            "durable model-switch marker is not the final replacement history entry".to_string(),
        )));
    }
    Ok(history)
}

fn model_switch_markers_match(left: &Message, right: &Message) -> bool {
    matches!(
        (left, right),
        (
            Message::System {
                content: left_content,
                ..
            },
            Message::System {
                content: right_content,
                ..
            }
        ) if left_content == right_content
    )
}

fn model_switch_marker_for_activation(activation: &SessionModelActivation) -> Message {
    Message::System {
        content: format!(
            "[System] Model switch activation {}: {} -> {}.\n[System] Active model for subsequent requests: {}.",
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
    previous_model: &str,
    new_provider: &str,
    new_model: &str,
) -> Message {
    Message::System {
        content: format!(
            "[System] Model switch: {previous_provider}/{previous_model} -> {new_provider}/{new_model}.\n[System] Active model for subsequent requests: {new_provider}/{new_model}."
        ),
        cache_markers: Vec::new(),
    }
}

fn send_stream(
    ui_tx: &mpsc::UnboundedSender<talos_conversation::UiOutput>,
    source: talos_conversation::MessageSource,
    text: String,
) {
    use talos_conversation::{ContentOutput, UiOutput};

    let _ = ui_tx.send(UiOutput::Content(ContentOutput::Block { source, text }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::Path;
    use talos_agent::Agent;
    use talos_config::{ModelConfig, ProviderConfig};
    use talos_core::model::{ModelCapabilities, ReasoningEffort, VariantDef};
    use talos_core::tool::ToolRegistry;
    use talos_provider::mock::MockProvider;
    use talos_session::{
        JsonlSessionStore, SessionEntry, SessionInfo, SessionManager, SessionStore,
    };
    use uuid::Uuid;

    #[derive(Debug)]
    struct FailingAppendStore;

    impl SessionStore for FailingAppendStore {
        fn read_entries(&self, file_path: &Path) -> Result<Vec<SessionEntry>, SessionError> {
            SessionStore::read_entries(&JsonlSessionStore, file_path)
        }

        fn append_entry(
            &self,
            _file_path: &Path,
            _entry: &SessionEntry,
        ) -> Result<(), SessionError> {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "injected model-switch marker failure",
            )
            .into())
        }

        fn replace_entries_atomically(
            &self,
            file_path: &Path,
            entries: &[SessionEntry],
        ) -> Result<(), SessionError> {
            SessionStore::replace_entries_atomically(&JsonlSessionStore, file_path, entries)
        }

        fn read_last_entry_id(&self, file_path: &Path) -> Option<String> {
            SessionStore::read_last_entry_id(&JsonlSessionStore, file_path)
        }

        fn scan_file(&self, file_path: &Path) -> Result<SessionInfo, SessionError> {
            SessionStore::scan_file(&JsonlSessionStore, file_path)
        }

        fn read_bytes(&self, file_path: &Path) -> Result<Vec<u8>, SessionError> {
            SessionStore::read_bytes(&JsonlSessionStore, file_path)
        }

        fn file_extension(&self) -> &'static str {
            "jsonl"
        }
    }

    #[test]
    fn ready_models_have_correct_provider_and_context_limit() {
        let mut config = Config {
            model: "claude-sonnet-4-5".to_string(),
            provider: "anthropic".to_string(),
            ..Default::default()
        };
        config.providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                api_key: Some("sk-test-key".to_string()),
                ..Default::default()
            },
        );

        let data = build_model_picker_data(&config);

        let anthropic_models: Vec<_> = data
            .ready_models
            .iter()
            .filter(|m| m.provider == "anthropic")
            .collect();
        assert!(
            !anthropic_models.is_empty(),
            "Expected at least one anthropic model in ready_models"
        );

        for m in &data.ready_models {
            assert!(
                m.authenticated,
                "Model {} should be authenticated",
                m.model_id
            );
        }
    }

    #[test]
    fn custom_catalog_context_is_labeled_without_relabeling_builtin_models() {
        let catalog = talos_config::model::builtin_models();
        let source = catalog
            .iter()
            .find(|candidate| {
                candidate.context_limit.is_some()
                    && catalog
                        .iter()
                        .filter(|model| model.id == candidate.id)
                        .count()
                        == 1
            })
            .expect("catalog should contain one unique model with context metadata");
        let model_id = source.id.clone();
        let builtin_provider = source.provider.clone();
        let mut config = Config {
            model: model_id.clone(),
            provider: "my-private-gateway".to_string(),
            ..Default::default()
        };
        config.providers.insert(
            "my-private-gateway".to_string(),
            ProviderConfig {
                api_key: Some("test-key".to_string()),
                models: HashMap::from([(model_id.clone(), ModelConfig::default())]),
                ..Default::default()
            },
        );
        config.providers.insert(
            builtin_provider.clone(),
            ProviderConfig {
                api_key: Some("test-key".to_string()),
                ..Default::default()
            },
        );

        let data = build_model_picker_data(&config);
        let custom = data
            .ready_models
            .iter()
            .find(|model| model.provider == "my-private-gateway" && model.model_id == model_id)
            .expect("custom model should be visible");
        let builtin = data
            .ready_models
            .iter()
            .find(|model| model.provider == builtin_provider && model.model_id == model_id)
            .expect("built-in model should be visible");

        assert!(custom.label.contains("(catalog)"));
        assert!(!builtin.label.contains("(catalog)"));
    }

    #[tokio::test]
    async fn model_rebuild_history_is_read_only_after_old_runtime_quiesces() {
        let temp = tempfile::tempdir().expect("operation should succeed");
        let manager = SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-model-final-history")
            .expect("operation should succeed");
        let session = durable.session().clone();
        session
            .append(&Message::User {
                content: "history-before-switch".into(),
            })
            .expect("operation should succeed");

        let (raw_tx, mut raw_rx) = mpsc::channel(4);
        let command_tx = raw_tx.clone();
        let mut transition =
            SessionTransition::new(raw_tx, session.clone()).expect("operation should succeed");
        let actor_session = session.clone();
        let actor_join = tokio::spawn(async move {
            while let Some(operation) = raw_rx.recv().await {
                match operation {
                    SessionOp::Interrupt => actor_session
                        .append(&Message::User {
                            content: "committed-during-handoff".into(),
                        })
                        .expect("operation should succeed"),
                    SessionOp::Shutdown => break,
                    _ => {}
                }
            }
        });
        let scheduler_cancel = tokio_util::sync::CancellationToken::new();
        let scheduler_token = scheduler_cancel.clone();
        let scheduler_join = tokio::spawn(async move {
            scheduler_token.cancelled().await;
        });
        transition
            .attach_active_runtime(actor_join, scheduler_cancel, scheduler_join)
            .expect("operation should succeed");

        command_tx
            .send(SessionOp::Interrupt)
            .await
            .expect("operation should succeed");
        let marker = model_switch_marker("old-provider", "old-model", "new-provider", "new-model");
        let history = persist_switch_marker_and_read_final_history(
            &mut transition,
            &session,
            &marker,
            SessionMetadata {
                provider: Some("new-provider".into()),
                model: Some("new-model".into()),
                ..Default::default()
            },
        )
        .await
        .expect("operation should succeed");

        let user_contents: Vec<_> = history
            .iter()
            .filter_map(|message| match message {
                Message::User { content } => Some(content.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            user_contents,
            vec!["history-before-switch", "committed-during-handoff"]
        );
        assert!(matches!(history.last(), Some(Message::System { .. })));
    }

    #[tokio::test]
    async fn model_switch_marker_write_failure_stops_before_publication() {
        let temp = tempfile::tempdir().expect("operation should succeed");
        let file_path = temp.path().join("session.jsonl");
        std::fs::write(&file_path, b"").expect("operation should succeed");
        let session = Session::with_store(
            Uuid::new_v4(),
            "test".into(),
            String::new(),
            file_path,
            Arc::new(FailingAppendStore),
        );
        let (raw_tx, raw_rx) = mpsc::channel(1);
        drop(raw_rx);
        let mut transition =
            SessionTransition::new(raw_tx, session.clone()).expect("operation should succeed");
        let marker = model_switch_marker("old-provider", "old-model", "new-provider", "new-model");

        let error = persist_switch_marker_and_read_final_history(
            &mut transition,
            &session,
            &marker,
            SessionMetadata {
                provider: Some("new-provider".into()),
                model: Some("new-model".into()),
                ..Default::default()
            },
        )
        .await
        .expect_err("operation should fail");

        assert!(matches!(
            error,
            FinalHistoryError::Persist(SessionError::IoError(_))
        ));
        assert_eq!(
            talos_session::PendingSubmissionStore::for_session(&session)
                .runtime_generation()
                .expect("operation should succeed"),
            1
        );
        assert!(
            session
                .read_entries()
                .expect("operation should succeed")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn model_switch_marker_retry_after_restart_is_idempotent_and_replay_equivalent() {
        let temp = tempfile::tempdir().expect("operation should succeed");
        let manager = SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-model-marker-retry")
            .expect("operation should succeed");
        let session = durable.session().clone();
        session
            .append(&Message::User {
                content: "history-before-switch".into(),
            })
            .expect("operation should succeed");
        let marker = model_switch_marker("old-provider", "old-model", "new-provider", "new-model");
        let metadata = SessionMetadata {
            provider: Some("new-provider".into()),
            model: Some("new-model".into()),
            ..Default::default()
        };

        let (first_tx, first_rx) = mpsc::channel(1);
        drop(first_rx);
        let mut first_transition =
            SessionTransition::new(first_tx, session.clone()).expect("operation should succeed");
        let first_history = persist_switch_marker_and_read_final_history(
            &mut first_transition,
            &session,
            &marker,
            metadata.clone(),
        )
        .await
        .expect("operation should succeed");
        drop(first_transition);

        let (restart_tx, restart_rx) = mpsc::channel(1);
        drop(restart_rx);
        let mut restarted_transition =
            SessionTransition::new(restart_tx, session.clone()).expect("operation should succeed");
        let retried_history = persist_switch_marker_and_read_final_history(
            &mut restarted_transition,
            &session,
            &marker,
            metadata,
        )
        .await
        .expect("operation should succeed");

        let Message::System { content, .. } = &marker else {
            unreachable!();
        };
        let encoded_marker = format!("__SYSTEM__:{content}");
        assert_eq!(
            session
                .read_entries()
                .expect("operation should succeed")
                .iter()
                .filter(|entry| entry.role == "system" && entry.content == encoded_marker)
                .count(),
            1
        );
        assert_eq!(format!("{first_history:?}"), format!("{retried_history:?}"));

        let reopened = Session::new(
            session.id,
            session.project.clone(),
            session.workspace_root.clone(),
            session.file_path.clone(),
        );
        assert_eq!(
            format!("{retried_history:?}"),
            format!(
                "{:?}",
                reopened.read_messages().expect("operation should succeed")
            )
        );
    }

    #[test]
    fn model_switch_marker_includes_previous_and_new_identity() {
        let marker = model_switch_marker("anthropic", "claude-old", "openai", "gpt-new");
        let Message::System { content, .. } = marker else {
            panic!("model switch marker must be a system message");
        };

        assert!(content.contains("anthropic/claude-old"));
        assert!(content.contains("openai/gpt-new"));
        assert!(content.contains("Active model for subsequent requests"));
    }

    #[test]
    fn model_switch_marker_survives_session_jsonl_round_trip() {
        let dir = tempfile::tempdir().expect("operation should succeed");
        let session = talos_session::Session::new(
            Uuid::new_v4(),
            "test".into(),
            String::new(),
            dir.path().join("session.jsonl"),
        );
        let marker = model_switch_marker("anthropic", "claude-old", "openai", "gpt-new");

        session
            .append_with_metadata(
                &marker,
                talos_session::SessionMetadata {
                    provider: Some("openai".into()),
                    model: Some("gpt-new".into()),
                    ..Default::default()
                },
            )
            .expect("operation should succeed");

        let messages = session.read_messages().expect("operation should succeed");
        assert_eq!(messages.len(), 1);
        let Message::System { content, .. } = &messages[0] else {
            panic!("round-tripped marker must remain a system message");
        };
        assert!(content.contains("anthropic/claude-old"));
        assert!(content.contains("openai/gpt-new"));

        let entries = session.read_entries().expect("operation should succeed");
        assert_eq!(entries[0].metadata.provider, Some("openai".into()));
        assert_eq!(entries[0].metadata.model, Some("gpt-new".into()));
    }

    #[tokio::test]
    async fn model_switch_marker_is_visible_in_request_preview() {
        let marker = model_switch_marker("anthropic", "claude-old", "openai", "gpt-new");
        let provider = MockProvider::new().with_request_debug_builder(|messages| {
            let system_messages: Vec<_> = messages
                .iter()
                .filter_map(|message| match message {
                    Message::System { content, .. } => Some(content.clone()),
                    _ => None,
                })
                .collect();
            serde_json::json!({ "systems": system_messages }).to_string()
        });
        let agent = Agent::with_security(
            Arc::new(provider),
            ToolRegistry::new(),
            Some(Arc::new(talos_permission::PermissionEngine::new())),
            None,
            std::path::PathBuf::from("/tmp"),
        );

        let preview = agent
            .preview_request("continue".to_string(), vec![marker])
            .await
            .expect("operation should succeed")
            .expect("operation should succeed");

        assert!(preview.contains("Model switch"));
        assert!(preview.contains("openai/gpt-new"));
    }

    #[tokio::test]
    async fn model_switch_activation_distinguishes_sequential_variant_changes() {
        let temp = tempfile::tempdir().expect("operation should succeed");
        let manager = SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-variant-activation-sequence")
            .expect("operation should succeed");
        let session = durable.session().clone();

        let (first_tx, first_rx) = mpsc::channel(1);
        drop(first_rx);
        let mut first_transition =
            SessionTransition::new(first_tx, session.clone()).expect("operation should succeed");
        establish_model_activation_and_read_final_history(
            &mut first_transition,
            &session,
            &SessionModelIdentity::new("openai", "o3", Some("low-reasoning")),
            &SessionModelIdentity::new("openai", "o3", Some("high-reasoning")),
        )
        .await
        .expect("operation should succeed");
        drop(first_transition);

        let (second_tx, second_rx) = mpsc::channel(1);
        drop(second_rx);
        let mut second_transition =
            SessionTransition::new(second_tx, session.clone()).expect("operation should succeed");
        establish_model_activation_and_read_final_history(
            &mut second_transition,
            &session,
            &SessionModelIdentity::new("openai", "o3", Some("high-reasoning")),
            &SessionModelIdentity::new("openai", "o3", Some("low-reasoning")),
        )
        .await
        .expect("operation should succeed");

        let entries = session.read_entries().expect("operation should succeed");
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
        let temp = tempfile::tempdir().expect("operation should succeed");
        let manager = SessionManager::with_dir(temp.path().join("sessions"));
        let durable = manager
            .create_or_open_session("i169-variant-activation-retry")
            .expect("operation should succeed");
        let session = durable.session().clone();
        let previous = SessionModelIdentity::new("openai", "o3", Some("low-reasoning"));
        let target = SessionModelIdentity::new("openai", "o3", Some("high-reasoning"));

        let (first_tx, first_rx) = mpsc::channel(1);
        drop(first_rx);
        let mut first_transition =
            SessionTransition::new(first_tx, session.clone()).expect("operation should succeed");
        let first_history = establish_model_activation_and_read_final_history(
            &mut first_transition,
            &session,
            &previous,
            &target,
        )
        .await
        .expect("operation should succeed");
        drop(first_transition);

        let (restart_tx, restart_rx) = mpsc::channel(1);
        drop(restart_rx);
        let mut restarted_transition =
            SessionTransition::new(restart_tx, session.clone()).expect("operation should succeed");
        let retried_history = establish_model_activation_and_read_final_history(
            &mut restarted_transition,
            &session,
            &previous,
            &target,
        )
        .await
        .expect("operation should succeed");

        let entries = session.read_entries().expect("operation should succeed");
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
        let Message::System { content, .. } = model_switch_marker_for_activation(&activation)
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

    #[test]
    fn duplicate_model_ids_keep_provider_side_ids_for_structured_switching() {
        let mut config = Config {
            model: "glm-5.2".to_string(),
            provider: "zai".to_string(),
            ..Default::default()
        };
        config.providers.insert(
            "zai".to_string(),
            ProviderConfig {
                api_key: Some("sk-zai-key".to_string()),
                ..Default::default()
            },
        );
        config.providers.insert(
            "zhipuai".to_string(),
            ProviderConfig {
                api_key: Some("sk-zhipu-key".to_string()),
                ..Default::default()
            },
        );

        let data = build_model_picker_data(&config);

        let glm_entries: Vec<_> = data
            .ready_models
            .iter()
            .filter(|m| m.model_id.contains("glm-5.2"))
            .collect();

        assert!(
            glm_entries.len() > 1,
            "fixture must contain duplicate glm-5.2 IDs"
        );
        assert!(glm_entries.iter().all(|entry| entry.model_id == "glm-5.2"));
        assert!(glm_entries.iter().any(|entry| entry.provider == "zai"));
        assert!(glm_entries.iter().any(|entry| entry.provider == "zhipuai"));

        let selected = glm_entries
            .iter()
            .find(|entry| entry.provider == "zai")
            .expect("zai duplicate must be selectable");
        let mut selected_config = config.clone();
        selected_config
            .set_active_model(&format!("{}/{}", selected.provider, selected.model_id))
            .expect("structured provider + raw model ID must resolve a duplicate");
        assert_eq!(selected_config.provider, "zai");
        assert_eq!(selected_config.model, "glm-5.2");
    }

    #[test]
    fn unauthenticated_providers_are_omitted_from_model_picker() {
        let mut config = Config {
            model: "claude-sonnet-4-5".to_string(),
            provider: "anthropic".to_string(),
            ..Default::default()
        };
        config.providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                api_key: Some("sk-test-key".to_string()),
                ..Default::default()
            },
        );
        // openai has no api_key and no env var set — unauthenticated.
        config.providers.insert(
            "openai".to_string(),
            ProviderConfig {
                ..Default::default()
            },
        );

        let data = build_model_picker_data(&config);

        assert!(
            data.setup_providers.is_empty(),
            "unauthenticated providers belong in /connect, not /model"
        );
        assert!(
            data.ready_models.iter().all(|m| m.authenticated),
            "/model picker must contain only authenticated providers"
        );
        assert!(
            data.ready_models.iter().all(|m| m.provider != "openai"),
            "unauthenticated openai models must be omitted from /model"
        );
    }

    #[test]
    fn is_current_flags_active_model_and_provider() {
        let mut config = Config {
            model: "claude-sonnet-4-5".to_string(),
            provider: "anthropic".to_string(),
            ..Default::default()
        };
        config.providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                api_key: Some("sk-test-key".to_string()),
                ..Default::default()
            },
        );

        let data = build_model_picker_data(&config);

        let current_models: Vec<_> = data.ready_models.iter().filter(|m| m.is_current).collect();
        assert_eq!(
            current_models.len(),
            1,
            "Expected exactly one current model, found {}",
            current_models.len()
        );
        assert_eq!(
            current_models[0].model_id, "claude-sonnet-4-5",
            "Structured picker model IDs must remain provider-side IDs"
        );
        assert_eq!(
            current_models[0].provider, "anthropic",
            "Current model provider should be anthropic"
        );

        for m in &data.ready_models {
            if m.model_id != "claude-sonnet-4-5" || m.provider != "anthropic" {
                assert!(
                    !m.is_current,
                    "Model {} ({}) should not be current",
                    m.model_id, m.provider
                );
            }
        }
    }

    #[test]
    fn model_picker_includes_only_declared_variants() {
        let mut config = Config::default();
        config.providers.insert(
            "openai".to_string(),
            ProviderConfig {
                api_key: Some("sk-test-key".to_string()),
                ..Default::default()
            },
        );

        let data = build_model_picker_data(&config);
        let o3 = data
            .ready_models
            .iter()
            .find(|model| model.provider == "openai" && model.model_id == "o3")
            .expect("openai/o3 is in the picker");
        assert_eq!(o3.variants.len(), 2);
        assert_eq!(o3.variants[0].variant_id, "high-reasoning");
        assert_eq!(o3.variants[1].variant_id, "low-reasoning");

        let gpt_4o = data
            .ready_models
            .iter()
            .find(|model| model.provider == "openai" && model.model_id == "gpt-4o")
            .expect("openai/gpt-4o is in the picker");
        assert!(gpt_4o.variants.is_empty());
    }

    fn reasoning_variant() -> VariantDef {
        VariantDef {
            id: "high-reasoning".to_string(),
            label: "High Reasoning".to_string(),
            reasoning_effort: Some(ReasoningEffort::High),
        }
    }

    #[test]
    fn resolve_variant_without_selection_uses_baseline() {
        let resolution =
            resolve_variant(None, &[reasoning_variant()], &ModelCapabilities::default());

        assert_eq!(resolution.reasoning_effort, None);
        assert_eq!(resolution.diagnostic, None);
    }

    #[test]
    fn resolve_variant_unknown_selection_reports_bounded_diagnostic() {
        let resolution = resolve_variant(
            Some("removed-variant"),
            &[reasoning_variant()],
            &ModelCapabilities::default(),
        );

        assert_eq!(resolution.reasoning_effort, None);
        assert_eq!(
            resolution.diagnostic.as_deref(),
            Some("Variant 'removed-variant' not found; using no variant")
        );
    }

    #[test]
    fn resolve_variant_applies_reasoning_effort_when_supported() {
        let resolution = resolve_variant(
            Some("high-reasoning"),
            &[reasoning_variant()],
            &ModelCapabilities {
                reasoning: true,
                ..Default::default()
            },
        );

        assert_eq!(resolution.reasoning_effort, Some(ReasoningEffort::High));
        assert_eq!(resolution.diagnostic, None);
    }

    #[test]
    fn resolve_variant_omits_reasoning_effort_when_unsupported() {
        let resolution = resolve_variant(
            Some("high-reasoning"),
            &[reasoning_variant()],
            &ModelCapabilities::default(),
        );

        assert_eq!(resolution.reasoning_effort, None);
        assert_eq!(resolution.diagnostic, None);
    }

    #[test]
    fn resolve_variant_without_reasoning_override_is_valid() {
        let variant = VariantDef {
            id: "preset".to_string(),
            label: "Preset".to_string(),
            reasoning_effort: None,
        };
        let resolution = resolve_variant(
            Some("preset"),
            &[variant],
            &ModelCapabilities {
                reasoning: true,
                ..Default::default()
            },
        );

        assert_eq!(resolution.reasoning_effort, None);
        assert_eq!(resolution.diagnostic, None);
    }

    #[test]
    fn provider_setup_target_prefers_current_model_for_provider() {
        let config = Config {
            model: "glm-5.2".to_string(),
            provider: "zai".to_string(),
            ..Default::default()
        };

        let target = provider_setup_target_model(&config, "zai").expect("target model");

        assert_eq!(target, "zai/glm-5.2");
    }

    #[test]
    fn provider_setup_target_falls_back_to_first_provider_model() {
        let config = Config {
            model: "claude-sonnet-4-5".to_string(),
            provider: "anthropic".to_string(),
            ..Default::default()
        };

        let target = provider_setup_target_model(&config, "anthropic").expect("target model");

        assert!(!target.is_empty());
        let found = config
            .all_models()
            .into_iter()
            .find(|m| m.id == target || format!("{}/{}", m.provider, m.id) == target)
            .expect("target exists in catalog");
        assert_eq!(found.provider, "anthropic");
        // provider_setup_target_model for current provider returns the exact current model
        assert_eq!(found.id, "claude-sonnet-4-5");
    }

    // Regression coverage for the Oracle-identified variant-clearing bug:
    // switching from `Some(variant)` to `None` (variant-less model) must clear
    // `Config.variant` and report a change so the caller persists it.
    #[test]
    fn apply_variant_change_clears_when_switching_to_none() {
        let mut config = Config {
            variant: Some("high-reasoning".to_string()),
            ..Default::default()
        };

        let changed = apply_variant_change(&mut config, None);
        assert!(changed, "switching Some → None must report a change");
        assert!(config.variant.is_none(), "variant must be cleared");
    }

    #[test]
    fn apply_variant_change_sets_when_switching_to_some() {
        let mut config = Config {
            variant: None,
            ..Default::default()
        };

        let changed = apply_variant_change(&mut config, Some("low-reasoning"));
        assert!(changed, "switching None → Some must report a change");
        assert_eq!(config.variant.as_deref(), Some("low-reasoning"));
    }

    #[test]
    fn apply_variant_change_updates_when_switching_between_variants() {
        let mut config = Config {
            variant: Some("high-reasoning".to_string()),
            ..Default::default()
        };

        let changed = apply_variant_change(&mut config, Some("low-reasoning"));
        assert!(
            changed,
            "switching Some → Some(different) must report a change"
        );
        assert_eq!(config.variant.as_deref(), Some("low-reasoning"));
    }

    #[test]
    fn apply_variant_change_noop_when_value_matches() {
        let mut config = Config {
            variant: Some("high-reasoning".to_string()),
            ..Default::default()
        };

        let changed = apply_variant_change(&mut config, Some("high-reasoning"));
        assert!(!changed, "identical value must not report a change");
        assert_eq!(config.variant.as_deref(), Some("high-reasoning"));
    }

    #[test]
    fn apply_variant_change_noop_when_both_none() {
        let mut config = Config {
            variant: None,
            ..Default::default()
        };

        let changed = apply_variant_change(&mut config, None);
        assert!(!changed, "None → None must not report a change");
        assert!(config.variant.is_none());
    }
}
