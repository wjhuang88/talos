//! Rebuildable checkpoint projection. This retains bindings, never raw messages.

use talos_core::message::Message;

use crate::Agent;
use crate::compaction::HistorySource;
use crate::request_plan::ProviderRequestPlan;
use crate::transient_text::PrivateTokens;

#[derive(Clone)]
pub(crate) struct CheckpointProjection {
    /// Unique within this Agent; invalidation never recycles a sequence.
    pub(crate) sequence: u64,
    /// Predecessor whose entire covered source set is retained by this checkpoint.
    pub(crate) parent_sequence: Option<u64>,
    pub(crate) source: HistorySource,
    pub(crate) covered: Vec<usize>,
    pub(crate) content: String,
    pub(crate) context_limit: u32,
    pub(crate) private: PrivateTokens,
}

impl Agent {
    /// Called by automatic request sealing or the Actor's manual commit gate.
    pub(crate) fn install_checkpoint_projection(
        &self,
        mut projection: CheckpointProjection,
        deadline: tokio::time::Instant,
    ) -> crate::compaction::CompactionResult<()> {
        use crate::compaction::CompactionError;
        let mut cached = self.checkpoint_projection_state.lock().map_err(|_| {
            CompactionError::CompactionFailed("checkpoint state unavailable".into())
        })?;
        projection.parent_sequence = cached
            .as_ref()
            .filter(|previous| {
                previous.source.revision() <= projection.source.revision()
                    && previous.covered.iter().all(|index| {
                        projection.covered.binary_search(index).is_ok()
                            && previous.source.same_entry(&projection.source, *index)
                    })
            })
            .map(|previous| previous.sequence);
        projection.sequence = self
            .checkpoint_sequence
            .try_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |current| current.checked_add(1),
            )
            .map_err(|_| {
                CompactionError::CompactionFailed("checkpoint sequence exhausted".into())
            })?
            + 1;
        if tokio::time::Instant::now() >= deadline {
            return Err(CompactionError::CompactionFailed(
                "checkpoint deadline exceeded".into(),
            ));
        }
        if let Some(session) = self
            .checkpoint_session
            .read()
            .map_err(|_| {
                CompactionError::CompactionFailed("checkpoint session unavailable".into())
            })?
            .as_ref()
        {
            let prior = cached
                .as_ref()
                .filter(|p| Some(p.sequence) == projection.parent_sequence)
                .map(|p| (p.content.as_str(), p.covered.len()));
            let persisted = session
                .archive_checkpoint(
                    &projection.content,
                    prior,
                    projection.covered.len(),
                    |position, id, message| {
                        projection
                            .covered
                            .get(position)
                            .and_then(|index| {
                                projection.source.resolve(
                                    projection.source.generation(),
                                    projection.source.revision(),
                                    *index,
                                    message,
                                )
                            })
                            .is_some_and(|reference| {
                                !reference.starts_with("tlog:")
                                    || reference == format!("tlog:{}:{id}", session.id)
                            })
                    },
                )
                .map_err(|error| {
                    tracing::warn!(%error, "checkpoint archive publication failed");
                    CompactionError::CompactionFailed(
                        "checkpoint archive publication failed".into(),
                    )
                })?;
            if persisted.is_some_and(|content| content != projection.content) {
                return Err(CompactionError::CompactionFailed(
                    "existing checkpoint already covers these sources".into(),
                ));
            }
        }
        *cached = Some(projection);
        Ok(())
    }

    pub(crate) fn invalidate_checkpoint_projection(&mut self) {
        let slot = self
            .checkpoint_projection_state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *slot = None;
    }

    /// Materialize the effective request and report whether a replacement can cover new history.
    pub(super) async fn verified_checkpoint_view(
        &self,
        plan: &ProviderRequestPlan,
        source: &HistorySource,
        context_limit: u32,
        private: &PrivateTokens,
    ) -> Option<(ProviderRequestPlan, bool)> {
        let projection = self
            .checkpoint_projection_state
            .lock()
            .ok()
            .and_then(|cached| cached.clone())?;
        let deadline = source.deadline().unwrap_or_else(|| {
            tokio::time::Instant::now() + crate::checkpoint_request::CHECKPOINT_TIMEOUT
        });
        for &index in &projection.covered {
            tokio::task::yield_now().await;
            if tokio::time::Instant::now() >= deadline {
                return None;
            }
            let message = plan.messages.get(index)?;
            let old = projection.source.resolve(
                projection.source.generation(),
                projection.source.revision(),
                index,
                message,
            );
            if old.is_none()
                || old != source.resolve(source.generation(), source.revision(), index, message)
            {
                if let Ok(mut cached) = self.checkpoint_projection_state.lock() {
                    *cached = None;
                }
                return None;
            }
        }
        let valid = projection.sequence != 0
            && projection
                .parent_sequence
                .is_none_or(|parent| parent < projection.sequence)
            && projection.source.generation() == source.generation()
            && projection.source.revision() <= source.revision()
            && projection.context_limit == context_limit
            && projection.private.same_tokens(private)
            && crate::checkpoint_request::eligible_prefix_end(&plan.messages)
                .is_some_and(|end| projection.covered.last().is_some_and(|last| *last < end));
        if !valid {
            if let Ok(mut cached) = self.checkpoint_projection_state.lock() {
                *cached = None;
            }
            return None;
        }
        let mut candidate = Vec::with_capacity(plan.messages.len() - projection.covered.len() + 1);
        for (index, message) in plan.messages.iter().enumerate() {
            tokio::task::yield_now().await;
            if tokio::time::Instant::now() >= deadline {
                return None;
            }
            if index == projection.covered[0] {
                candidate.push(Message::Context {
                    content: projection.content.clone(),
                });
            }
            if projection.covered.binary_search(&index).is_err() {
                candidate.push(message.clone());
            }
        }
        let estimated = self
            .estimate_provider_request_tokens(&candidate, &plan.tool_definitions)
            .saturating_add(plan.ephemeral_image_tokens);
        if tokio::time::Instant::now() >= deadline {
            return None;
        }
        let end = crate::checkpoint_request::eligible_prefix_end(&plan.messages)?;
        let new_prefix = (projection.covered.last()? + 1..end).any(|index| {
            let message = &plan.messages[index];
            !matches!(message, Message::System { .. })
                && (!matches!(message, Message::Context { .. })
                    || source
                        .resolve(source.generation(), source.revision(), index, message)
                        .is_some())
        });
        Some((
            ProviderRequestPlan {
                messages: candidate,
                estimated_tokens: estimated,
                tool_definitions: plan.tool_definitions.clone(),
                tool_protocol: plan.tool_protocol,
                ephemeral_image_tokens: plan.ephemeral_image_tokens,
                omitted_tool_exchanges: plan.omitted_tool_exchanges,
            },
            new_prefix,
        ))
    }
}
