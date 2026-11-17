//! Turn-local evidence for background terminal results seen by a provider.
//! Recording a tool response alone is deliberately not acknowledgement.

use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use talos_core::background_job::BackgroundJobId;
use talos_core::message::{Message, MessageToolResult};

#[derive(Clone, Default)]
pub(crate) struct Consumption {
    state: Arc<Mutex<State>>,
}

#[derive(Default)]
struct State {
    observed: HashMap<String, BackgroundJobId>,
    confirmed: HashSet<BackgroundJobId>,
    projected: HashMap<String, [u8; 32]>,
    pending: HashSet<String>,
}

/// Keep one invocation identity occupied through the result-binding boundary.
/// Failed/cancelled execution releases its reservation without consumption.
pub(crate) struct CallGuard {
    consumption: Consumption,
    call_id: String,
    completed: bool,
}
impl CallGuard {
    pub(crate) fn complete(&mut self) {
        self.completed = true;
    }
}
impl Drop for CallGuard {
    fn drop(&mut self) {
        if !self.completed
            && let Ok(mut state) = self.consumption.state.lock()
        {
            state.pending.remove(&self.call_id);
            state.observed.remove(&self.call_id);
            state.projected.remove(&self.call_id);
        }
    }
}

/// Scoped around one model-originated tool invocation, never a host-only read.
#[derive(Clone)]
pub(crate) struct ToolObservation {
    pub(crate) consumption: Consumption,
    pub(crate) call_id: String,
}

tokio::task_local! {
    pub(crate) static OBSERVATION: ToolObservation;
    pub(crate) static TURN_CONSUMPTION: Consumption;
}

pub(crate) fn observe_terminal(id: &BackgroundJobId) -> Result<(), String> {
    OBSERVATION.try_with(|observation| {
        let mut state = observation.consumption.state.lock()
            .map_err(|_| "background consumption evidence unavailable".to_owned())?;
        if state.confirmed.contains(id) {
            return Ok(());
        }
        if !state.observed.contains_key(&observation.call_id)
            && state.observed.len().saturating_add(state.confirmed.len()) >= 128
        {
            return Err("background consumption evidence capacity reached; terminal result was not delivered".into());
        }
        state.observed.insert(observation.call_id.clone(), id.clone());
        Ok(())
    }).unwrap_or(Ok(()))
}

impl Consumption {
    /// A later invocation with the same provider ID cannot inherit observation
    /// evidence from an older result, including when the later tool is unrelated.
    pub(crate) fn begin_call(&self, call_id: &str) -> Result<CallGuard, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "background consumption evidence unavailable".to_owned())?;
        if state.pending.contains(call_id) {
            return Err("duplicate tool call ID before result binding".into());
        }
        if state.pending.len() >= 128 {
            return Err("tool result binding capacity reached".into());
        }
        state.pending.insert(call_id.into());
        state.observed.remove(call_id);
        state.projected.remove(call_id);
        Ok(CallGuard {
            consumption: self.clone(),
            call_id: call_id.into(),
            completed: false,
        })
    }

    /// Bind terminal observation to the exact permitted model result projection.
    /// The caller must reject hook rewrites of the original call or tool result.
    pub(crate) fn bind_result(&self, call_id: &str, result: Option<&MessageToolResult>) {
        if let Ok(mut state) = self.state.lock() {
            state.pending.remove(call_id);
            state.projected.remove(call_id);
            if let Some(result) =
                result.filter(|result| !result.is_error && result.tool_use_id == call_id)
                && state.observed.contains_key(call_id)
            {
                state.projected.insert(
                    call_id.into(),
                    Sha256::digest(result.content.as_bytes()).into(),
                );
            } else {
                state.observed.remove(call_id);
            }
        }
    }

    /// Confirm only one unmodified result in the exact successful provider request.
    /// Repeated call IDs are ambiguous even when one matching payload is present.
    pub(crate) fn confirm_messages(&self, messages: &[Message]) {
        if let Ok(mut state) = self.state.lock() {
            let consumed: Vec<_> = state
                .observed
                .iter()
                .filter_map(|(call_id, id)| {
                    let expected = state.projected.get(call_id)?;
                    let mut matches = messages.iter().filter_map(|message| match message {
                        Message::Tool { result } if result.tool_use_id == *call_id => Some(result),
                        _ => None,
                    });
                    let result = matches.next()?;
                    if result.is_error || matches.next().is_some() {
                        return None;
                    }
                    let digest: [u8; 32] = Sha256::digest(result.content.as_bytes()).into();
                    (digest == *expected).then(|| (call_id.clone(), id.clone()))
                })
                .collect();
            for (call_id, id) in consumed {
                state.observed.remove(&call_id);
                state.projected.remove(&call_id);
                state.confirmed.insert(id);
            }
        }
    }

    // Low-level supervisor tests synthesize the projection/dispatch boundary;
    // production never confirms an observation from an ID alone.
    #[cfg(test)]
    pub(crate) fn confirm_request(&self, result_ids: &[String]) {
        let messages: Vec<_> = result_ids
            .iter()
            .map(|id| {
                let result = MessageToolResult {
                    tool_use_id: id.clone(),
                    content: "terminal".into(),
                    is_error: false,
                };
                self.bind_result(id, Some(&result));
                Message::Tool { result }
            })
            .collect();
        self.confirm_messages(&messages);
    }

    /// Transfer evidence to the existing turn commit boundary.
    pub(crate) fn confirmed(&self) -> Vec<BackgroundJobId> {
        self.state
            .lock()
            .map(|state| state.confirmed.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invocation_reservation_survives_execution_until_binding_and_cleans_up_cancellation() {
        let evidence = Consumption::default();
        let first = evidence.begin_call("call").expect("first");
        assert!(evidence.begin_call("call").is_err());
        drop(first);
        let mut second = evidence
            .begin_call("call")
            .expect("cancel released reservation");
        second.complete();
        drop(second);
        assert!(
            evidence.begin_call("call").is_err(),
            "result still needs binding"
        );
        evidence.bind_result("call", None);
        assert!(
            evidence.begin_call("call").is_ok(),
            "binding releases identity"
        );
    }

    #[tokio::test]
    async fn rewritten_duplicate_and_reused_result_ids_cannot_confirm_consumption() {
        for mode in ["rewritten", "duplicate", "reused", "unbound", "valid"] {
            let evidence = Consumption::default();
            let id = BackgroundJobId::new("job_original");
            OBSERVATION
                .scope(
                    ToolObservation {
                        consumption: evidence.clone(),
                        call_id: "same-call".into(),
                    },
                    async {
                        observe_terminal(&id).expect("observation");
                    },
                )
                .await;
            let original = MessageToolResult {
                tool_use_id: "same-call".into(),
                content: "original terminal output".into(),
                is_error: false,
            };
            if mode != "unbound" {
                evidence.bind_result("same-call", Some(&original));
            }
            let mut dispatched = original.clone();
            if mode == "rewritten" {
                dispatched.content = "replacement".into();
            }
            if mode == "reused" {
                let _call = evidence.begin_call("same-call").expect("new invocation");
            }
            let mut messages = vec![Message::Tool { result: dispatched }];
            if mode == "duplicate" {
                messages.push(Message::Tool { result: original });
            }
            evidence.confirm_messages(&messages);
            assert_eq!(
                evidence.confirmed(),
                if mode == "valid" { vec![id] } else { vec![] },
                "{mode}"
            );
        }
    }

    #[tokio::test]
    async fn concurrent_observations_keep_call_identity_isolated() {
        let evidence = Consumption::default();
        let first = OBSERVATION.scope(
            ToolObservation {
                consumption: evidence.clone(),
                call_id: "first".into(),
            },
            async {
                tokio::task::yield_now().await;
                observe_terminal(&BackgroundJobId::new("job_first")).expect("observation");
            },
        );
        let second = OBSERVATION.scope(
            ToolObservation {
                consumption: evidence.clone(),
                call_id: "second".into(),
            },
            async {
                observe_terminal(&BackgroundJobId::new("job_second")).expect("observation");
                tokio::task::yield_now().await;
            },
        );
        tokio::join!(first, second);
        evidence.confirm_request(&["first".into()]);
        assert_eq!(
            evidence.confirmed(),
            vec![BackgroundJobId::new("job_first")]
        );
        evidence.confirm_request(&["second".into()]);
        let confirmed = evidence.confirmed();
        assert_eq!(confirmed.len(), 2);
        assert!(confirmed.contains(&BackgroundJobId::new("job_second")));
    }
    #[tokio::test]
    async fn observation_and_confirmation_share_one_capacity_bound() {
        let evidence = Consumption::default();
        for index in 0..256 {
            let call_id = format!("call-{index}");
            OBSERVATION
                .scope(
                    ToolObservation {
                        consumption: evidence.clone(),
                        call_id: call_id.clone(),
                    },
                    async {
                        assert_eq!(
                            observe_terminal(&BackgroundJobId::new(format!("job-{index}"))).is_ok(),
                            index < 128
                        );
                    },
                )
                .await;
            if index % 2 == 0 {
                evidence.confirm_request(&[call_id]);
            }
            let state = evidence.state.lock().expect("state");
            assert!(state.observed.len() + state.confirmed.len() <= 128);
        }
        let state = evidence.state.lock().expect("state");
        assert_eq!(state.observed.len() + state.confirmed.len(), 128);
    }

    #[tokio::test]
    async fn only_successfully_dispatched_result_identity_confirms_consumption() {
        let evidence = Consumption::default();
        let job = BackgroundJobId::new("job_model");
        OBSERVATION
            .scope(
                ToolObservation {
                    consumption: evidence.clone(),
                    call_id: "call-1".into(),
                },
                async {
                    observe_terminal(&job).expect("observation");
                },
            )
            .await;
        assert!(
            evidence.confirmed().is_empty(),
            "tool execution is not consumption"
        );
        evidence.confirm_request(&["unrelated-call".into()]);
        assert!(evidence.confirmed().is_empty());
        evidence.confirm_request(&["call-1".into()]);
        evidence.confirm_request(&["call-1".into()]);
        assert_eq!(evidence.confirmed(), vec![job]);
    }

    #[tokio::test]
    async fn missing_or_error_result_does_not_confirm_observation() {
        use talos_core::message::{Message, MessageToolResult};
        let evidence = Consumption::default();
        let id = BackgroundJobId::new("job_result");
        OBSERVATION
            .scope(
                ToolObservation {
                    consumption: evidence.clone(),
                    call_id: "call".into(),
                },
                async {
                    observe_terminal(&id).expect("observation");
                },
            )
            .await;
        evidence.confirm_messages(&[]);
        let mut result = MessageToolResult {
            tool_use_id: "call".into(),
            content: "terminal".into(),
            is_error: true,
        };
        evidence.confirm_messages(&[Message::Tool {
            result: result.clone(),
        }]);
        assert!(evidence.confirmed().is_empty());
        result.is_error = false;
        evidence.bind_result("call", Some(&result));
        evidence.confirm_messages(&[Message::Tool { result }]);
        assert_eq!(evidence.confirmed(), vec![id]);
    }

    #[tokio::test]
    async fn host_read_and_other_turn_cannot_confirm_this_turn() {
        let evidence = Consumption::default();
        observe_terminal(&BackgroundJobId::new("host-only")).expect("observation");
        evidence.confirm_request(&["call-1".into()]);
        assert!(evidence.confirmed().is_empty());
        let other = Consumption::default();
        OBSERVATION
            .scope(
                ToolObservation {
                    consumption: other.clone(),
                    call_id: "call-1".into(),
                },
                async {
                    observe_terminal(&BackgroundJobId::new("other-turn")).expect("observation");
                },
            )
            .await;
        evidence.confirm_request(&["call-1".into()]);
        assert!(evidence.confirmed().is_empty());
        assert!(
            other.confirmed().is_empty(),
            "unconfirmed observation remains unconsumed"
        );
    }
}
