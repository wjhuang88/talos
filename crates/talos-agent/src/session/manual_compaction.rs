//! One Actor-owned manual operation; raw history stays with the Actor.

use super::*;
use crate::checkpoint_manual::ManualCheckpointResult;
use crate::compaction::{CompactAdmission, CompactOperations};
use talos_core::session::{CompactOutcome, CompactState};

pub(super) struct ManualCompaction {
    ledger: CompactOperations,
    queued: Option<u64>,
    boundary_resume: Option<(
        tokio::sync::oneshot::Sender<Option<crate::SteeringBoundaryBatch>>,
        Option<crate::SteeringBoundaryBatch>,
    )>,
    running: Option<(u64, CancellationToken, JoinHandle<ManualCheckpointResult>)>,
}

impl ManualCompaction {
    pub(super) fn new(generation: u64) -> Self {
        Self {
            ledger: CompactOperations::new(generation),
            queued: None,
            boundary_resume: None,
            running: None,
        }
    }

    fn emit(actor: &AppServerSession, id: u64, state: CompactState) {
        let _ = actor.eq_tx.send(SessionEvent::CompactStatus {
            session_id: actor.session_id.clone(),
            session_generation: actor.session_generation,
            operation_id: id,
            state,
        });
    }

    pub(super) fn admit(&mut self, actor: &AppServerSession, generation: u64, id: u64) {
        let state = match self.ledger.admit(generation, id) {
            CompactAdmission::Accepted => {
                self.queued = Some(id);
                CompactState::Queued
            }
            CompactAdmission::Pending => {
                if self.running.is_some() {
                    CompactState::Started
                } else {
                    CompactState::Queued
                }
            }
            CompactAdmission::Busy => CompactState::Busy,
            CompactAdmission::Completed(outcome) => CompactState::Completed {
                outcome,
                replayed: true,
            },
            CompactAdmission::Expired => CompactState::Expired,
            CompactAdmission::StaleGeneration => CompactState::StaleGeneration,
        };
        Self::emit(actor, id, state);
    }

    pub(super) fn is_running(&self) -> bool {
        self.running.is_some()
    }

    pub(super) fn start_boundary(
        &mut self,
        actor: &AppServerSession,
        request: crate::SteeringBoundaryRequest,
        injected: Option<crate::SteeringBoundaryBatch>,
        turn_token: &CancellationToken,
    ) {
        let Some(id) = self.queued.take() else {
            let _ = request.compact.send(None);
            let _ = request.response.send(injected);
            return;
        };
        let (result, received) = tokio::sync::oneshot::channel();
        if request
            .compact
            .send(Some(crate::BoundaryCompactJob { result }))
            .is_err()
        {
            self.finish(actor, id, CompactOutcome::Cancelled);
            let _ = request.response.send(injected);
            return;
        }
        let token = turn_token.child_token();
        let waiting_token = token.clone();
        let task = tokio::spawn(async move {
            tokio::select! { biased;
                () = waiting_token.cancelled() => ManualCheckpointResult::Outcome(CompactOutcome::Cancelled),
                result = received => result.unwrap_or(ManualCheckpointResult::Outcome(CompactOutcome::Failed)),
            }
        });
        self.boundary_resume = Some((request.response, injected));
        self.running = Some((id, token, task));
        Self::emit(actor, id, CompactState::Started);
    }

    pub(super) fn start(&mut self, actor: &AppServerSession) {
        if self.running.is_some() {
            return;
        }
        let Some(id) = self.queued.take() else {
            return;
        };
        let token = CancellationToken::new();
        if actor
            .runtime_admission
            .as_ref()
            .is_some_and(|control| !control.commit_start(token.clone()))
        {
            self.finish(actor, id, CompactOutcome::Cancelled);
            return;
        }
        Self::emit(actor, id, CompactState::Started);
        let agent = actor.agent.clone();
        let history = actor.history.clone();
        let generation = actor.session_generation;
        let recovered = actor.recovered_source.clone();
        let limit = actor.model_context_limit;
        let task_token = token.clone();
        let task = tokio::spawn(async move {
            tokio::select! { biased;
                () = task_token.cancelled() => ManualCheckpointResult::Outcome(CompactOutcome::Cancelled),
                outcome = agent.compact_session_history(history, generation, limit, recovered.as_ref()) => outcome,
            }
        });
        self.running = Some((id, token, task));
    }

    pub(super) async fn wait(&mut self) -> ManualCheckpointResult {
        match self.running.as_mut() {
            Some((_, _, task)) => task
                .await
                .unwrap_or(ManualCheckpointResult::Outcome(CompactOutcome::Failed)),
            None => std::future::pending().await,
        }
    }

    pub(super) fn complete(&mut self, actor: &AppServerSession, result: ManualCheckpointResult) {
        let Some((id, token, _)) = self.running.take() else {
            return;
        };
        let commit = || {
            if token.is_cancelled() {
                return CompactOutcome::Cancelled;
            }
            match result {
                ManualCheckpointResult::Outcome(outcome) => outcome,
                ManualCheckpointResult::Candidate {
                    projection,
                    before,
                    after,
                    deadline,
                } => {
                    if projection.source.generation() != actor.session_generation
                        || projection.source.revision() != actor.history.len() as u64
                    {
                        return CompactOutcome::Cancelled;
                    }
                    match actor
                        .agent
                        .install_checkpoint_projection(*projection, deadline)
                    {
                        Ok(()) => CompactOutcome::Applied { before, after },
                        Err(_) => CompactOutcome::Failed,
                    }
                }
            }
        };
        let outcome = if let Some(control) = &actor.runtime_admission {
            control
                .commit_checkpoint(commit)
                .unwrap_or(CompactOutcome::Cancelled)
        } else {
            commit()
        };
        if self.boundary_resume.is_none()
            && let Some(control) = &actor.runtime_admission
        {
            let status = match outcome {
                CompactOutcome::Applied { .. } | CompactOutcome::NotEligible { .. } => {
                    TurnCompletionStatus::Success {
                        final_text: String::new(),
                        new_messages: vec![],
                    }
                }
                CompactOutcome::Cancelled => TurnCompletionStatus::Cancelled,
                CompactOutcome::Failed => TurnCompletionStatus::Error {
                    message: "checkpoint failed".into(),
                },
            };
            control.finish_active(Some(&status));
        }
        self.finish(actor, id, outcome);
        if let Some((response, injected)) = self.boundary_resume.take() {
            let _ = response.send(injected);
        }
    }

    fn finish(&mut self, actor: &AppServerSession, id: u64, outcome: CompactOutcome) {
        if self.ledger.finish(actor.session_generation, id, outcome) {
            Self::emit(
                actor,
                id,
                CompactState::Completed {
                    outcome,
                    replayed: false,
                },
            );
        }
    }

    pub(super) fn cancel_operation(&mut self, actor: &AppServerSession, generation: u64, id: u64) {
        match self.ledger.observe(generation, id) {
            CompactAdmission::Pending => self.cancel(actor),
            CompactAdmission::Completed(outcome) => Self::emit(
                actor,
                id,
                CompactState::Completed {
                    outcome,
                    replayed: true,
                },
            ),
            CompactAdmission::StaleGeneration => {
                Self::emit(actor, id, CompactState::StaleGeneration)
            }
            _ => Self::emit(actor, id, CompactState::Expired),
        }
    }

    pub(super) fn cancel(&mut self, actor: &AppServerSession) {
        if let Some((_, token, _)) = &self.running {
            token.cancel();
        }
        if let Some(id) = self.queued.take() {
            self.finish(actor, id, CompactOutcome::Cancelled);
        }
    }
}

impl Drop for ManualCompaction {
    fn drop(&mut self) {
        if let Some((_, token, task)) = &self.running {
            token.cancel();
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use talos_core::message::{AgentEvent, Message, StopReason, Usage};
    use talos_core::provider::{LanguageModel, ProviderResult};
    use talos_core::session::{RuntimePolicy, SessionConfig};
    use talos_core::tool::ToolRegistry;

    struct SummaryModel;
    #[async_trait]
    impl LanguageModel for SummaryModel {
        async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
            let Message::User { content } = &messages[0] else {
                panic!("summary request")
            };
            let sources: Vec<serde_json::Value> =
                serde_json::from_str(content.split("\nSources: ").last().expect("sources"))
                    .expect("JSON");
            let item = serde_json::json!({"text":"retain original task", "sources":[sources[0]["source"]]});
            let payload =
                serde_json::json!({"schema_version":1,"summary":item,"goal":item,"next_step":item,
                "constraints":[],"completed":[],"in_progress":[],"pending":[],"decisions":[],
                "facts":[],"assumptions":[],"resources":[],"open_loops":[]})
                .to_string();
            let (tx, rx) = mpsc::channel(4);
            tx.send(AgentEvent::TextDelta { delta: payload })
                .await
                .expect("text");
            tx.send(AgentEvent::TurnEnd {
                stop_reason: StopReason::EndTurn,
                usage: Usage::default(),
            })
            .await
            .expect("end");
            Ok(rx)
        }
    }

    #[tokio::test]
    async fn completed_candidate_waits_for_actor_cancel_or_apply_decision() {
        let raw: Vec<_> = (0..30)
            .flat_map(|_| {
                [
                    Message::User {
                        content: "constraint ".repeat(400),
                    },
                    Message::Assistant {
                        content: "recorded".into(),
                        tool_calls: vec![],
                        reasoning: None,
                    },
                ]
            })
            .collect();
        let agent = Agent::with_security(
            Arc::new(SummaryModel),
            ToolRegistry::new(),
            None,
            None,
            std::env::temp_dir(),
        );
        let (mut handle, mut actor) = AppServerSession::new(
            agent,
            SessionConfig {
                runtime_policy: RuntimePolicy::interactive(),
                workspace_root: std::env::temp_dir(),
                initial_history: raw.clone(),
                model_context_limit: 30_000,
            },
        );
        let control = RuntimeAdmissionControl::new();
        actor.set_runtime_admission(control.clone());
        let mut manual = ManualCompaction::new(0);
        for id in 1..=3 {
            let candidate = actor
                .agent
                .compact_session_history(raw.clone(), 0, 30_000, None)
                .await;
            assert!(matches!(
                &candidate,
                ManualCheckpointResult::Candidate { .. }
            ));
            assert!(
                actor
                    .agent
                    .checkpoint_projection_state
                    .lock()
                    .expect("cache")
                    .is_none()
            );
            assert_eq!(manual.ledger.admit(0, id), CompactAdmission::Accepted);
            let token = CancellationToken::new();
            assert!(control.commit_start(token.clone()));
            manual.running = Some((
                id,
                token,
                tokio::spawn(async { ManualCheckpointResult::Outcome(CompactOutcome::Failed) }),
            ));
            if id == 1 {
                control.interrupt_active();
            } else if id == 2 {
                manual.cancel_operation(&actor, 1, id);
                assert!(matches!(
                    handle.eq_rx.recv().await,
                    Some(SessionEvent::CompactStatus {
                        state: CompactState::StaleGeneration,
                        ..
                    })
                ));
                manual.cancel_operation(&actor, 0, id + 99);
                assert!(matches!(
                    handle.eq_rx.recv().await,
                    Some(SessionEvent::CompactStatus {
                        state: CompactState::Expired,
                        ..
                    })
                ));
                assert!(!manual.running.as_ref().expect("running").1.is_cancelled());
                manual.cancel_operation(&actor, 0, id);
            }
            manual.complete(&actor, candidate);
            let event = handle.eq_rx.recv().await.expect("terminal receipt");
            if id <= 2 {
                assert!(matches!(
                    event,
                    SessionEvent::CompactStatus {
                        state: CompactState::Completed {
                            outcome: CompactOutcome::Cancelled,
                            replayed: false
                        },
                        ..
                    }
                ));
                assert!(
                    actor
                        .agent
                        .checkpoint_projection_state
                        .lock()
                        .expect("cache")
                        .is_none()
                );
            } else {
                assert!(
                    matches!(event, SessionEvent::CompactStatus { state: CompactState::Completed { outcome: CompactOutcome::Applied { before, after }, replayed: false }, .. } if after < before)
                );
                assert!(
                    actor
                        .agent
                        .checkpoint_projection_state
                        .lock()
                        .expect("cache")
                        .is_some()
                );
                manual.cancel_operation(&actor, 0, id);
                assert!(
                    matches!(
                        handle.eq_rx.recv().await,
                        Some(SessionEvent::CompactStatus {
                            state: CompactState::Completed {
                                outcome: CompactOutcome::Applied { .. },
                                replayed: true
                            },
                            ..
                        })
                    ),
                    "completed cancellation must return prior receipt without undoing application"
                );
            }
            assert_eq!(actor.history, raw);
        }
    }
}
