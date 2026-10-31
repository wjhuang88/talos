//! Manual checkpoint preparation shares ordinary request construction and admission.

use talos_core::message::Message;
use talos_core::session::CompactOutcome;
use talos_plugin::{HookContext, TurnId};

use crate::Agent;
use crate::compaction::{HistorySource, RequestPressure};
use crate::configuration::describe_presented_tools;
use crate::transient_text::PrivateTokens;

pub(crate) enum ManualCheckpointResult {
    Outcome(CompactOutcome),
    Candidate {
        projection: Box<crate::checkpoint_projection::CheckpointProjection>,
        before: u32,
        after: u32,
        deadline: tokio::time::Instant,
    },
}

impl Agent {
    pub(crate) async fn compact_session_history(
        &self,
        history: Vec<Message>,
        generation: u64,
        limit: u32,
    ) -> ManualCheckpointResult {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(120);
        // Dropping this entire future cancels preparation as well as provider I/O.
        tokio::time::timeout_at(deadline, async {
            let source = HistorySource::bind_content_until(
                generation,
                history.len() as u64,
                &history,
                deadline,
            )
            .await
            .ok_or(())?;
            let context = HookContext::new(TurnId::new(), self.workspace_root.clone());
            let (_, mut definitions, _) =
                describe_presented_tools(&self.tools, &self.tool_presentation_policy);
            if !self.image_input_supported {
                definitions.retain(|definition| definition.name != "read_image");
            }
            let protocol = self
                .resolve_tool_protocol_for_definitions(self.tool_protocol, &definitions)
                .await;
            let (mut messages, _) = self
                .build_provider_messages_with_protocol(String::new(), history, &context, protocol)
                .await
                .map_err(|_| ())?;
            // Prompt assembly's synthetic input is never a transcript entry or summary source.
            messages.pop();
            let mut plan = self
                .observe_provider_request_plan(
                    &context,
                    &messages,
                    &definitions,
                    &mut Vec::new(),
                    &PrivateTokens::default(),
                )
                .await
                .map_err(|_| ())?;
            Ok::<_, ()>(
                self.prepare_manual_checkpoint(
                    &mut plan,
                    limit,
                    &PrivateTokens::default(),
                    &source,
                    deadline,
                )
                .await,
            )
        })
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or(ManualCheckpointResult::Outcome(CompactOutcome::Failed))
    }
    /// Prepare against the exact request at a safe boundary. This does not rebuild
    /// prompts, replace raw history, or publish the candidate into shared state.
    pub(crate) async fn prepare_manual_checkpoint(
        &self,
        plan: &mut crate::request_plan::ProviderRequestPlan,
        limit: u32,
        private: &PrivateTokens,
        source: &HistorySource,
        deadline: tokio::time::Instant,
    ) -> ManualCheckpointResult {
        if tokio::time::Instant::now() >= deadline {
            return ManualCheckpointResult::Outcome(CompactOutcome::Failed);
        }
        let before = plan.estimated_tokens;
        if RequestPressure::assess(before, limit) == RequestPressure::Normal {
            return ManualCheckpointResult::Outcome(CompactOutcome::NotEligible { before });
        }
        let mut projection = None;
        let result = tokio::time::timeout_at(
            deadline,
            self.checkpoint_request_once_deferred(
                plan,
                limit,
                private,
                Some(source),
                Some(&mut projection),
            ),
        )
        .await;
        match (result, projection) {
            (Ok(Ok(())), Some(projection)) => ManualCheckpointResult::Candidate {
                projection: Box::new(projection),
                before,
                after: plan.estimated_tokens,
                deadline,
            },
            _ => ManualCheckpointResult::Outcome(CompactOutcome::Failed),
        }
    }
}
