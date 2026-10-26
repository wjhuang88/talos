//! Session Todo operations retain deterministic policy admission in Auto mode.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use talos_core::tool::{ToolContribution, ToolContributionSource};
use talos_session::todo::*;

struct UnexpectedAssessor(Arc<AtomicUsize>);
#[async_trait]
impl AutoPermissionAssessor for UnexpectedAssessor {
    async fn assess(
        &self,
        _: AutoPermissionRequest,
        _: std::time::Duration,
    ) -> Result<String, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err("internal policy should resolve before assessment".into())
    }
}

#[tokio::test]
async fn real_todo_operations_preserve_internal_policy_and_session_scope() {
    for denied in [false, true] {
        for (operation, foreign_target) in (0..9)
            .map(|operation| (operation, false))
            .chain((2..8).map(|operation| (operation, true)))
        {
            let root = tempfile::tempdir().expect("workspace");
            let db = root.path().join("todos.sqlite");
            let session = uuid::Uuid::new_v4();
            let other_session = uuid::Uuid::new_v4();
            let repo = TodoRepository::new(&db).expect("repository");
            repo.init_schema().expect("schema");
            let seed = |session_id, title: &str| {
                repo.create(CreateTodo {
                    session_id,
                    title: title.into(),
                    description: None,
                    priority: TodoPriority::Medium,
                    assigned_to_turn: None,
                    tags: vec![],
                })
                .expect("seed")
            };
            let first = seed(session, "first");
            let second = seed(session, "second");
            let other = seed(other_session, "other-session-item");
            if operation == 7 {
                repo.add_dependency(session, first.id, second.id)
                    .expect("dependency");
            }
            let before =
                serde_json::to_value(repo.list(session, TodoQuery::default()).expect("list"))
                    .expect("serialize");
            let dependencies_before =
                serde_json::to_value(repo.list_dependencies(session).expect("dependencies"))
                    .expect("serialize");
            let (tool, input): (Arc<dyn AgentTool>, Value) = match operation {
                0 => (
                    Arc::new(TodoCreateTool::new(db.clone(), session)),
                    serde_json::json!({"title":"created"}),
                ),
                1 => (
                    Arc::new(TodoCreateBatchTool::new(db.clone(), session)),
                    serde_json::json!({"items":[{"title":"created"},{"title":"batch-created"}]}),
                ),
                2 => (
                    Arc::new(TodoUpdateBatchTool::new(db.clone(), session)),
                    serde_json::json!({"items":[{"id":first.id,"title":"changed"}]}),
                ),
                3 => (
                    Arc::new(TodoUpdateStatusTool::new(db.clone(), session)),
                    serde_json::json!({"id":first.id,"status":"in_progress"}),
                ),
                4 => (
                    Arc::new(TodoUpdateTool::new(db.clone(), session)),
                    serde_json::json!({"id":first.id,"title":"changed"}),
                ),
                5 => (
                    Arc::new(TodoDeleteTool::new(db.clone(), session)),
                    serde_json::json!({"id":first.id}),
                ),
                6 => (
                    Arc::new(TodoAddDependencyTool::new(db.clone(), session)),
                    serde_json::json!({"parent_id":first.id,"child_id":second.id}),
                ),
                7 => (
                    Arc::new(TodoRemoveDependencyTool::new(db.clone(), session)),
                    serde_json::json!({"parent_id":first.id,"child_id":second.id}),
                ),
                _ => (
                    Arc::new(TodoQueryTool::new(db.clone(), session)),
                    serde_json::json!({}),
                ),
            };
            let mut input = input;
            if foreign_target {
                if operation == 2 {
                    input["items"][0]["id"] = serde_json::json!(other.id);
                } else if operation >= 6 {
                    input["child_id"] = serde_json::json!(other.id);
                } else {
                    input["id"] = serde_json::json!(other.id);
                }
            }
            let name = tool.name().to_owned();
            let facets = tool.permission_profile(&input);
            assert_eq!(facets.len(), 1, "{name}");
            assert_eq!(facets[0].nature, ToolNature::Internal, "{name}");
            let mut registry = ToolRegistry::new();
            registry
                .register_builtin_contributions([ToolContribution::new(
                    ToolContributionSource::new("talos-session:todo"),
                    tool,
                )])
                .expect("registration");
            let mut engine = PermissionEngine::with_workspace_root(root.path().into());
            if denied {
                engine = PermissionEngine::empty();
                engine.set_workspace_root(root.path().into());
                engine.add_rule(talos_permission::PermissionRule::new_nature(
                    ToolNature::Internal,
                    None,
                    None,
                    PermissionDecision::Deny("explicit Todo denial".into()),
                ));
            }
            let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
            let calls = Arc::new(AtomicUsize::new(0));
            let resolver = AutoPermissionResolver::new(
                Arc::new(UnexpectedAssessor(calls.clone())),
                Arc::new(AutoDenyFallback),
                ManagedWorkspaceLease::for_permission_session(root.path(), state.clone())
                    .expect("lease"),
                std::time::Duration::from_secs(2),
                AutoPermissionControl::new(true),
            );
            let model = MockModel::new(vec![
                vec![
                    AgentEvent::ToolCall {
                        call: ToolCall {
                            id: "todo-proof".into(),
                            name: name.clone(),
                            input,
                        },
                        provenance: Default::default(),
                        summary_fields: vec![],
                    },
                    AgentEvent::TurnEnd {
                        stop_reason: StopReason::ToolUse,
                        usage: Usage::default(),
                    },
                ],
                text_done_events("done"),
            ]);
            let agent =
                Agent::with_security(Arc::new(model), registry, None, None, root.path().into())
                    .with_permission_pipeline(
                        state.clone(),
                        talos_permission::PermissionContext::new(
                            talos_permission::PermissionMode::Interactive,
                            talos_permission::InteractionCapability::Available,
                        ),
                        Some(Arc::new(resolver)),
                    );
            let (tx, mut rx) = mpsc::unbounded_channel();
            agent
                .run_streaming("update this session's plan".into(), vec![], tx)
                .await
                .expect("run");
            let mut results = 0;
            while let Ok(event) = rx.try_recv() {
                if let AgentEvent::ToolResult { result, .. } = event {
                    results += 1;
                    assert_eq!(
                        result.is_error,
                        denied || (foreign_target && operation != 5 && operation != 7),
                        "{name} foreign={foreign_target}: {}",
                        result.content
                    );
                    assert!(!result.content.contains("other-session-item"), "{name}");
                    if !denied && foreign_target && (operation == 5 || operation == 7) {
                        assert!(result.content.contains("not found"), "{name}");
                    }
                }
            }
            assert_eq!(results, 1, "{name}");
            assert_eq!(calls.load(Ordering::SeqCst), 0, "{name}");
            assert_eq!(state.grant_count().expect("grants"), 0, "{name}");
            let after = repo.list(session, TodoQuery::default()).expect("list");
            let dependencies = repo.list_dependencies(session).expect("dependencies");
            if denied || foreign_target || operation == 8 {
                assert_eq!(
                    serde_json::to_value(&after).expect("serialize"),
                    before,
                    "{name}"
                );
                assert_eq!(
                    serde_json::to_value(&dependencies).expect("serialize"),
                    dependencies_before,
                    "{name}"
                );
            } else {
                match operation {
                    0 => assert_eq!(after.len(), 3),
                    1 => assert_eq!(after.len(), 4),
                    2 | 4 => assert_eq!(
                        repo.get(session, first.id)
                            .expect("get")
                            .expect("item")
                            .title,
                        "changed"
                    ),
                    3 => assert_eq!(
                        repo.get(session, first.id)
                            .expect("get")
                            .expect("item")
                            .status,
                        TodoStatus::InProgress
                    ),
                    5 => assert!(repo.get(session, first.id).expect("get").is_none()),
                    6 => assert_eq!(dependencies.len(), 1),
                    7 => assert!(dependencies.is_empty()),
                    _ => unreachable!(),
                }
            }
            assert_eq!(
                serde_json::to_value(
                    repo.get(other_session, other.id)
                        .expect("get")
                        .expect("other item")
                )
                .expect("serialize"),
                serde_json::to_value(other).expect("serialize"),
                "{name}"
            );
        }
    }
}
