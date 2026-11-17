use super::*;
use std::sync::Mutex;
use std::time::Duration;
use talos_agent::permission_pipeline::{PermissionAuthorizationRequest, PermissionPipeline};
use talos_core::tool::{
    PreparedFileMutationDescription, PreparedFileMutationEvidence, PreparedPlanIdentity,
    ToolNature, ToolResourceKind,
};
use talos_provider::mock::MockProvider;
use tokio::sync::oneshot;

struct ReviewProvider(Arc<Mutex<Vec<Value>>>);
#[async_trait]
impl LanguageModel for ReviewProvider {
    async fn stream(&self, _: &[Message]) -> ProviderResult<Receiver<AgentEvent>> {
        panic!("review-only fixture");
    }
    async fn stream_auto_review(
        &self,
        messages: &[Message],
        _: DecisionRequestLimits,
    ) -> ProviderResult<Receiver<AgentEvent>> {
        let Message::User { content } = &messages[1] else {
            panic!("request");
        };
        let payload: Value = serde_json::from_str(content).expect("payload");
        self.0.lock().expect("payloads").push(payload.clone());
        MockProvider::new()
            .with_response(
                serde_json::json!({
                    "schema":"file-mutation/v1", "request_digest":payload["request_digest"],
                    "decision":"human_required", "confidence":"low"
                })
                .to_string(),
            )
            .stream(&[])
            .await
    }
}

struct ObservedHost(mpsc::UnboundedSender<oneshot::Sender<ApprovalObservation>>);
#[async_trait]
impl ApprovalHandler for ObservedHost {
    async fn request_approval(&self, _: &str, _: &Value, _: &[String]) -> ApprovalChoice {
        panic!("typed host must not be downgraded");
    }
    async fn request_scoped_approval_observed(
        &self,
        _: &str,
        _: &Value,
        _: &[String],
        _: &GrantPreview,
        _: Option<&str>,
    ) -> ApprovalObservation {
        let (tx, rx) = oneshot::channel();
        if self.0.send(tx).is_err() {
            return ApprovalObservation::unavailable();
        }
        rx.await
            .unwrap_or_else(|_| ApprovalObservation::unavailable())
    }
}

struct LegacyDeny;
#[async_trait]
impl ApprovalHandler for LegacyDeny {
    async fn request_approval(&self, _: &str, _: &Value, _: &[String]) -> ApprovalChoice {
        ApprovalChoice::Deny
    }
}

#[tokio::test]
async fn real_sdk_bridge_records_host_results_in_next_provider_request() {
    for scenario in ["human", "closed", "cancel", "legacy"] {
        let root = tempfile::tempdir().expect("workspace");
        let state = Arc::new(PermissionSessionState::new(
            PermissionEngine::with_workspace_root(root.path().into()),
        ));
        let captured = Arc::new(Mutex::new(Vec::new()));
        let (tx, mut rx) = mpsc::unbounded_channel();
        let host: Arc<dyn ApprovalHandler> = if scenario == "legacy" {
            Arc::new(LegacyDeny)
        } else {
            Arc::new(ObservedHost(tx))
        };
        let resolver = AutoPermissionResolver::new(
            Arc::new(ProviderAutoPermissionAssessor::new(Arc::new(
                ReviewProvider(captured.clone()),
            ))),
            Arc::new(RuntimeApprovalResolver { inner: host }),
            ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                .expect("lease"),
            Duration::from_secs(2),
            AutoPermissionControl::new(true),
        );
        let pipeline = PermissionPipeline::new(
            state.clone(),
            PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            ),
            Some(Arc::new(resolver)),
        );
        let operations = async {
            for _ in 0..2 {
                let plan = PreparedPlanIdentity::new();
                let path = root.path().join("note.txt");
                let evidence = PreparedFileMutationEvidence::new(
                    plan.clone(),
                    PreparedFileMutationDescription {
                        path: path.clone(),
                        before: "before".into(),
                        after: Some("after".into()),
                    },
                )
                .expect("evidence");
                let input = serde_json::json!({"path":path, "retain_backup":true});
                let facets = [ToolPermissionFacet::with_resource(
                    ToolNature::Write,
                    path.to_string_lossy(),
                    ToolResourceKind::Path,
                )];
                let result = pipeline
                    .authorize_prepared_file_mutation(
                        PermissionAuthorizationRequest {
                            tool_name: "edit",
                            provenance: ToolProvenance::Native,
                            profile: &facets,
                            input: &input,
                            presentation_input: input.clone(),
                            summary_fields: vec![],
                            deadline: Duration::from_secs(2),
                        },
                        Some("update note"),
                        &plan,
                        &evidence,
                    )
                    .await;
                assert!(result.is_err(), "{scenario} must not authorize");
            }
        };
        let driver = async {
            if scenario == "legacy" {
                return;
            }
            for _ in 0..2 {
                let reply = rx.recv().await.expect("real SDK host callback");
                match scenario {
                    "closed" => drop(reply),
                    "cancel" => {
                        reply
                            .send(ApprovalObservation::user_cancelled())
                            .expect("cancel");
                    }
                    _ => {
                        reply
                            .send(ApprovalObservation::observed_human(ApprovalChoice::Deny))
                            .expect("human");
                    }
                }
            }
        };
        tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(operations, driver);
        })
        .await
        .expect("bounded host chain");
        let payloads = captured.lock().expect("payloads");
        assert_eq!(payloads.len(), 2);
        assert!(
            payloads[0]["approval_history"]["entries"]
                .as_array()
                .expect("first")
                .is_empty()
        );
        let entries = payloads[1]["approval_history"]["entries"]
            .as_array()
            .expect("next");
        assert_eq!(entries.len(), 1);
        let expected = match scenario {
            "human" => serde_json::json!(["denied", "observed_human"]),
            "closed" => serde_json::json!(["unavailable", "request_lifecycle"]),
            "cancel" => serde_json::json!(["explicit_user_cancellation", "observed_human"]),
            _ => serde_json::json!(["denied", "legacy_host_unknown"]),
        };
        assert_eq!(entries[0]["fallback"], expected, "{scenario}");
        assert_ne!(payloads[0]["request_digest"], payloads[1]["request_digest"]);
        assert_eq!(state.grant_count().expect("grants"), 0);
    }
}
