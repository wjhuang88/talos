use super::*;
use crate::permission_pipeline::{PermissionAuthorizationRequest, PermissionPipeline};
use std::sync::atomic::AtomicUsize;
use talos_core::tool::{PreparedPlanIdentity, ToolPermissionFacet};
use talos_permission::{
    PermissionContext, PermissionDecision, PermissionEngine, PermissionRule, PermissionSessionState,
};

struct Assessor {
    behavior: &'static str,
    calls: Arc<AtomicUsize>,
    control: AutoPermissionControl,
    previous: Mutex<Option<String>>,
    state: Arc<PermissionSessionState>,
}

#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(&self, _: AutoPermissionRequest, _: Duration) -> Result<String, String> {
        panic!("effectful command must never use legacy read-only assessment")
    }

    async fn assess_prepared_command(
        &self,
        request: PreparedCommandAssessmentRequest,
        _: Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.command.effects.len(), 6);
        assert!(!request.command.confined);
        assert_eq!(request.command.targets.repository, ".");
        assert_eq!(
            request.command.targets.configured_remote_hosts,
            ["example.test"]
        );
        assert_eq!(
            request.command.targets.configured_program_categories,
            ["credentials"]
        );
        assert!(request.command.targets.dynamic_effects_unknown);
        assert_eq!(
            request.user_intent,
            "update this repository from its configured upstream"
        );
        if self.behavior == "failure" {
            return Err("private provider error".into());
        }
        if self.behavior == "panic" {
            panic!("provider panic");
        }
        if self.behavior == "mode_change" {
            self.control.set_enabled(false);
        }
        if self.behavior == "reset_reenable" {
            self.control.set_enabled(false);
            self.control.set_enabled(true);
        }
        if self.behavior == "revision_change" {
            self.state.rebind_session().expect("session reset");
        }
        let mut response = serde_json::json!({
            "schema": COMMAND_SCHEMA,
            "request_digest": request.request_digest,
            "decision": "allow_once", "confidence": "high",
            "acknowledged_effects": request.command.effects,
            "accepts_unconfined_effects": true,
        });
        match self.behavior {
            "missing_effect" => {
                response["acknowledged_effects"]
                    .as_array_mut()
                    .expect("effects")
                    .pop();
            }
            "wrong_digest" => response["request_digest"] = "old-request".into(),
            "confined" => response["accepts_unconfined_effects"] = false.into(),
            "low" => response["confidence"] = "low".into(),
            "human" => response["decision"] = "human_required".into(),
            "unknown" => response["extra"] = true.into(),
            _ => {}
        }
        let raw = response.to_string();
        let mut previous = self.previous.lock().expect("previous response");
        if self.behavior == "replay"
            && let Some(previous) = previous.as_ref()
        {
            return Ok(previous.clone());
        }
        *previous = Some(raw.clone());
        Ok(raw)
    }
}

struct Human(Arc<AtomicUsize>);
#[async_trait]
impl ApprovalResolver for Human {
    async fn resolve(
        &self,
        _: PermissionApprovalRequest,
        _: Duration,
    ) -> Result<ApprovalChoice, ApprovalResolverError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(ApprovalChoice::Deny)
    }
}

#[tokio::test]
async fn effectful_command_requires_complete_bound_response_and_preserves_policy() {
    for behavior in [
        "allow",
        "missing_effect",
        "wrong_digest",
        "confined",
        "low",
        "human",
        "unknown",
        "failure",
        "panic",
        "mode_change",
        "reset_reenable",
        "revision_change",
        "replay",
        "explicit_ask",
        "deny",
        "missing_intent",
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let mut engine = PermissionEngine::with_workspace_root(root.path().to_owned());
        if matches!(behavior, "explicit_ask" | "deny") {
            engine.add_rule(PermissionRule::new_nature(
                ToolNature::Write,
                Some("unseen-helper-output/**".into()),
                None,
                if behavior == "deny" {
                    PermissionDecision::Deny("private".into())
                } else {
                    PermissionDecision::Ask
                },
            ));
        }
        let state = Arc::new(PermissionSessionState::new(engine));
        let lease = ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
            .expect("lease");
        let control = AutoPermissionControl::new(true);
        let calls = Arc::new(AtomicUsize::new(0));
        let human_calls = Arc::new(AtomicUsize::new(0));
        let resolver = Arc::new(AutoPermissionResolver::new(
            Arc::new(Assessor {
                behavior,
                calls: calls.clone(),
                control: control.clone(),
                previous: Mutex::new(None),
                state: state.clone(),
            }),
            Arc::new(Human(human_calls.clone())),
            lease,
            Duration::from_secs(2),
            control,
        ));
        let pipeline = PermissionPipeline::new(
            state.clone(),
            PermissionContext::new(
                PermissionMode::Interactive,
                InteractionCapability::Available,
            ),
            Some(resolver),
        );
        let facets = [
            ToolPermissionFacet::with_resource(
                ToolNature::Read,
                root.path().to_string_lossy(),
                ToolResourceKind::Path,
            ),
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                root.path().join("git").to_string_lossy(),
                ToolResourceKind::Command,
            ),
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                "git:pull-command",
                ToolResourceKind::Remote,
            ),
        ];
        let input = serde_json::json!({"remote": "origin"});
        for attempt in 0..if behavior == "replay" { 2 } else { 1 } {
            let plan = PreparedPlanIdentity::new();
            let evidence = PreparedCommandEvidence::new(
                plan.clone(),
                PreparedCommandDescription {
                    operation: "git:pull-command/v1".into(),
                    arguments: vec!["pull".into()],
                    effects: vec![
                        PreparedCommandEffect::Execute,
                        PreparedCommandEffect::Network,
                        PreparedCommandEffect::WorktreeMutation,
                        PreparedCommandEffect::MetadataMutation,
                        PreparedCommandEffect::Recursion,
                        PreparedCommandEffect::ConfiguredPrograms,
                    ],
                    confined: false,
                    targets: talos_core::tool::PreparedCommandTargets {
                        repository: ".".into(),
                        metadata: vec![".git".into()],
                        remote_selection: "configured upstream".into(),
                        configured_remote_hosts: vec!["example.test".into()],
                        invocation_remote_host: None,
                        configured_program_categories: vec!["credentials".into()],
                        dynamic_effects_unknown: true,
                    },
                },
            );
            let result = pipeline
                .authorize_prepared_plan_with_evidence(
                    PermissionAuthorizationRequest {
                        tool_name: "git_pull",
                        provenance: ToolProvenance::Native,
                        profile: &facets,
                        input: &input,
                        presentation_input: input.clone(),
                        summary_fields: vec![],
                        deadline: Duration::from_secs(2),
                    },
                    if behavior == "missing_intent" {
                        None
                    } else {
                        Some("update this repository from its configured upstream")
                    },
                    &plan,
                    Some(&evidence),
                )
                .await;
            if behavior == "allow" || (behavior == "replay" && attempt == 0) {
                result
                    .unwrap_or_else(|error| panic!("{behavior}: {error}"))
                    .consume_for_prepared(
                        &talos_permission::PermissionRequest::native("git_pull", &facets, &input),
                        &plan,
                    )
                    .expect("one-use proof");
            } else {
                assert!(result.is_err(), "{behavior} must not approve");
            }
        }
        let expected_calls = match behavior {
            "deny" | "explicit_ask" | "missing_intent" => 0,
            "replay" => 2,
            _ => 1,
        };
        assert_eq!(calls.load(Ordering::SeqCst), expected_calls, "{behavior}");
        assert_eq!(
            human_calls.load(Ordering::SeqCst),
            usize::from(!matches!(behavior, "allow" | "deny")),
            "{behavior}"
        );
        assert_eq!(state.grant_count().expect("grants"), 0, "{behavior}");
    }
}
