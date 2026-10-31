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
            let before = plan.estimated_tokens;
            if RequestPressure::assess(before, limit) == RequestPressure::Normal {
                return Ok(ManualCheckpointResult::Outcome(
                    CompactOutcome::NotEligible { before },
                ));
            }
            let mut projection = None;
            self.checkpoint_request_once_deferred(
                &mut plan,
                limit,
                &PrivateTokens::default(),
                Some(&source),
                Some(&mut projection),
            )
            .await
            .map_err(|_| ())?;
            Ok::<_, ()>(ManualCheckpointResult::Candidate {
                projection: Box::new(projection.ok_or(())?),
                before,
                after: plan.estimated_tokens,
                deadline,
            })
        })
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or(ManualCheckpointResult::Outcome(CompactOutcome::Failed))
    }
}
