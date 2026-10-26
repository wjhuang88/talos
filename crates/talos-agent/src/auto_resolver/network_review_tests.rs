use super::*;
use crate::permission_pipeline::{PermissionAuthorizationRequest, PermissionPipeline};
use std::sync::atomic::AtomicUsize;
use talos_core::tool::{ToolNetworkEndpoint, ToolNetworkManifest, ToolNetworkScope};
use talos_permission::{
    PermissionContext, PermissionDecision, PermissionEngine, PermissionRule, PermissionSessionState,
};

struct Assessor {
    behavior: &'static str,
    calls: Arc<AtomicUsize>,
    state: Arc<PermissionSessionState>,
    control: AutoPermissionControl,
    previous: Mutex<Option<String>>,
}
#[async_trait]
impl AutoPermissionAssessor for Assessor {
    async fn assess(&self, _: AutoPermissionRequest, _: Duration) -> Result<String, String> {
        panic!("network invocation must not use the read-only classifier")
    }
    async fn assess_builtin_network(
        &self,
        request: BuiltinNetworkAssessmentRequest,
        _: Duration,
    ) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.input["query"], "rust documentation");
        assert_eq!(request.facets.len(), 1);
        assert_eq!(request.endpoints[0].host, "example.com");
        assert_eq!(request.endpoints[0].method, "GET");
        assert_eq!(request.user_intent, "search rust documentation");
        match self.behavior {
            "error" => return Err("private provider error".into()),
            "panic" => panic!("provider panic"),
            "cancel" => {
                self.control.set_enabled(false);
            }
            "stale" => {
                self.state.rebind_session().expect("rebind");
            }
            _ => {}
        }
        let mut response = serde_json::json!({"schema": SCHEMA, "request_digest": request.request_digest,
            "decision": "allow_once", "confidence": "high"});
        match self.behavior {
            "uncertain" => response["decision"] = "human_required".into(),
            "low" => response["confidence"] = "low".into(),
            "digest" => response["request_digest"] = "old".into(),
            "unknown" => response["extra"] = true.into(),
            _ => {}
        }
        let mut previous = self.previous.lock().expect("previous");
        if self.behavior == "replay"
            && let Some(raw) = previous.as_ref()
        {
            return Ok(raw.clone());
        }
        let raw = response.to_string();
        *previous = Some(raw.clone());
        Ok(raw)
    }
}
struct Human(Arc<AtomicUsize>);

#[test]
fn review_serialization_enforces_encoded_byte_limit() {
    let exact = "a".repeat(MAX_REVIEW_BYTES - 2);
    assert_eq!(bounded_json(&exact).expect("fits").len(), MAX_REVIEW_BYTES);
    assert!(bounded_json(&(exact + "a")).is_none());
    // JSON escaping counts toward the limit, not merely source string length.
    assert!(bounded_json(&"\n".repeat(MAX_REVIEW_BYTES / 2)).is_none());
}

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
async fn network_auto_admission_binds_complete_review_and_preserves_policy() {
    for behavior in [
        "allow",
        "error",
        "panic",
        "uncertain",
        "low",
        "digest",
        "unknown",
        "cancel",
        "stale",
        "replay",
        "deny",
        "human",
        "external",
        "mixed",
        "no_intent",
        "secret_key",
        "secret_nested",
        "secret_array",
        "secret_prefix",
        "secret_intent",
        "secret_path",
    ] {
        let root = tempfile::tempdir().expect("workspace");
        let mut engine = PermissionEngine::with_workspace_root(root.path().to_owned());
        if matches!(behavior, "deny" | "human") {
            engine = PermissionEngine::empty();
            engine.set_workspace_root(root.path().to_owned());
            engine.add_rule(PermissionRule::new_nature(
                ToolNature::Network,
                Some("example.com".into()),
                None,
                if behavior == "deny" {
                    PermissionDecision::Deny("policy".into())
                } else {
                    PermissionDecision::Ask
                },
            ));
        }
        let state = Arc::new(PermissionSessionState::new(engine));
        let control = AutoPermissionControl::new(true);
        let calls = Arc::new(AtomicUsize::new(0));
        let humans = Arc::new(AtomicUsize::new(0));
        let resolver = Arc::new(AutoPermissionResolver::new(
            Arc::new(Assessor {
                behavior,
                calls: calls.clone(),
                state: state.clone(),
                control: control.clone(),
                previous: Mutex::new(None),
            }),
            Arc::new(Human(humans.clone())),
            ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                .expect("lease"),
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
        let path = if behavior == "secret_path" {
            "/ghp_example-private-value"
        } else {
            "/search"
        };
        let endpoint =
            ToolNetworkEndpoint::new("https", "example.com", 443, "GET", path).expect("endpoint");
        let manifest = ToolNetworkManifest::new(
            ToolNetworkScope::new(
                uuid::Uuid::new_v4(),
                0,
                vec![endpoint.clone()],
                None,
                vec![],
                0,
            )
            .expect("scope"),
        );
        let mut input = serde_json::json!({"query": "rust documentation"});
        match behavior {
            "secret_key" => input["api_key"] = "example-private-value".into(),
            "secret_nested" => {
                input["nested"] = serde_json::json!({"Authorization": "private-value"})
            }
            "secret_array" => {
                input["items"] = serde_json::json!([{"access_token": "private-value"}])
            }
            "secret_prefix" => input["items"] = serde_json::json!(["(ghp_example-private-value)"]),
            _ => {}
        }
        let mut facets = vec![ToolPermissionFacet::with_resource(
            ToolNature::Network,
            "example.com",
            ToolResourceKind::Domain,
        )];
        if behavior == "mixed" {
            facets.push(ToolPermissionFacet::with_resource(
                ToolNature::Write,
                root.path().join("output.txt").display().to_string(),
                ToolResourceKind::Path,
            ));
        }
        for attempt in 0..if behavior == "replay" { 2 } else { 1 } {
            let result = pipeline
                .authorize_registered_network(
                    PermissionAuthorizationRequest {
                        tool_name: "web_search",
                        provenance: ToolProvenance::Native,
                        profile: &facets,
                        input: &input,
                        presentation_input: serde_json::json!({"query": "<redacted>"}),
                        summary_fields: vec![],
                        deadline: Duration::from_secs(2),
                    },
                    &manifest,
                    if behavior == "no_intent" {
                        None
                    } else if behavior == "secret_intent" {
                        Some("search (ghp_example-private-value)")
                    } else {
                        Some("search rust documentation")
                    },
                    behavior != "external",
                )
                .await;
            if behavior == "allow" || (behavior == "replay" && attempt == 0) {
                let authority = result.unwrap_or_else(|error| panic!("{behavior}: {error}"));
                authority
                    .check(manifest.invocation_id(), &endpoint)
                    .expect("actual request start authorized");
                assert!(authority.check(uuid::Uuid::new_v4(), &endpoint).is_err());
                let other =
                    ToolNetworkEndpoint::new("https", "other.example", 443, "GET", "/search")
                        .expect("other endpoint");
                assert!(authority.check(manifest.invocation_id(), &other).is_err());
                state.rebind_session().expect("revoke authority");
                assert!(
                    authority
                        .check(manifest.invocation_id(), &endpoint)
                        .is_err()
                );
            } else {
                assert!(result.is_err(), "{behavior} cannot execute");
            }
        }
        assert_eq!(
            calls.load(Ordering::SeqCst),
            match behavior {
                "deny" | "human" | "external" | "mixed" | "no_intent" | "secret_key"
                | "secret_nested" | "secret_array" | "secret_prefix" | "secret_intent"
                | "secret_path" => 0,
                "replay" => 2,
                _ => 1,
            },
            "{behavior}"
        );
        assert_eq!(state.grant_count().expect("grants"), 0, "{behavior}");
    }
}
