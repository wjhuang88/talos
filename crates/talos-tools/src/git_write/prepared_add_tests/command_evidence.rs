#[tokio::test]
async fn git_command_evidence_is_final_bound_and_redacted() {
    for operation in ["add", "commit", "checkout", "push"] {
        let root = fixture();
        integration_git(root.path(), &["add", "file"]);
        integration_git(root.path(), &["commit", "-m", "initial"]);
        integration_git(root.path(), &["branch", "private-branch-marker"]);
        let input = match operation {
            "add" => serde_json::json!({"paths":["file"]}),
            "commit" => serde_json::json!({"message":"private-message-marker","all":true}),
            "checkout" => serde_json::json!({"branch":"private-branch-marker"}),
            "push" => serde_json::json!({"remote":"git://127.0.0.1:9/private-path-marker","branch":"HEAD:refs/heads/private-branch-marker","force":true}),
            _ => unreachable!(),
        };
        let plan = match operation {
            "add" => AddPlan::prepare(root.path(), &input, ToolProvenance::Native),
            "commit" => AddPlan::prepare_commit(root.path(), &input, ToolProvenance::Native),
            "checkout" => AddPlan::prepare_checkout(root.path(), &input, ToolProvenance::Native),
            "push" => AddPlan::prepare_push(root.path(), &input, ToolProvenance::Native),
            _ => unreachable!(),
        }.expect("prepare");
        assert!(plan.prepared_command_evidence().is_none(), "{operation}: premature evidence");
        let plan = Box::new(plan).discover().await.expect("discovery");
        let plan = if operation == "push" { finish_push_program_approved(plan).await } else { plan };
        assert!(plan.phase == Phase::Mutation, "{operation}: final mutation frontier");
        let evidence = plan.prepared_command_evidence().expect("final evidence");
        assert!(evidence.belongs_to(&plan.identity));
        assert!(!evidence.belongs_to(&PreparedPlanIdentity::new()));
        assert_eq!(evidence.description().operation, format!("git:{operation}-command/v1"));
        assert!(plan.facets.iter().any(|facet| facet.resource.as_deref() == Some(format!("git:{operation}-command").as_str())));
        let encoded = serde_json::to_string(evidence.description()).expect("redacted evidence");
        for private in ["private-message-marker", "private-branch-marker", "private-path-marker"] {
            assert!(!encoded.contains(private), "{operation}: private input leaked");
        }
        assert_eq!(integration_git(root.path(), &["status", "--porcelain"]), "");
    }
}
