//! Session factory dispatch through the public Runtime API, including default features.
use super::*;

#[tokio::test]
async fn session_todo_dispatch_respects_policy_and_session_binding() {
    let directory = tempfile::tempdir().expect("sessions");
    let database = directory.path().join("todos.sqlite");
    let repository = talos_session::todo::TodoRepository::new(&database).expect("repository");
    repository.init_schema().expect("schema");
    for denied in [false, true] {
        let session = uuid::Uuid::new_v4();
        let other_session = uuid::Uuid::new_v4();
        let provider = MockProvider::new()
            .with_tool_call(
                "todo_create",
                serde_json::json!({"title":"runtime-bound-item"}),
            )
            .with_response("done");
        let mut runtime = RuntimeBuilder::new()
            .provider(Arc::new(provider))
            .workspace_root(directory.path())
            .session_todo_tools(directory.path(), session)
            .permission_rule(PermissionRule::new_nature(
                ToolNature::Internal,
                None,
                None,
                if denied {
                    PermissionDecision::Deny("todo disabled".into())
                } else {
                    PermissionDecision::Allow
                },
            ))
            .build()
            .expect("runtime");
        tokio::time::timeout(Duration::from_secs(10), async {
            runtime
                .submit("create a planning item")
                .await
                .expect("submit");
            let mut saw_result = false;
            while let Some(event) = runtime.next_event().await {
                match event {
                    SessionEvent::TurnEvent {
                        payload:
                            TurnEventPayload::Progress {
                                event: AgentEvent::ToolResult { .. },
                            },
                        ..
                    } => saw_result = true,
                    SessionEvent::TurnEvent {
                        payload: TurnEventPayload::Completed { .. },
                        ..
                    } => break,
                    _ => {}
                }
            }
            assert!(saw_result, "must dispatch through the tool loop");
        })
        .await
        .expect("bounded turn");
        runtime.shutdown().await.expect("shutdown");
        let items = repository
            .list(session, Default::default())
            .expect("stored items");
        assert_eq!(items.len(), usize::from(!denied));
        if !denied {
            assert_eq!(items[0].title, "runtime-bound-item");
        }
        assert!(
            repository
                .list(other_session, Default::default())
                .expect("other session")
                .is_empty()
        );
    }
}

#[test]
fn every_session_todo_factory_tool_preserves_only_internal_identity() {
    let directory = tempfile::tempdir().expect("sessions");
    let session = uuid::Uuid::new_v4();
    let contributions =
        talos_session::todo_tool_contributions_for_sessions_dir(directory.path(), session);
    assert_eq!(contributions.len(), 9);
    for contribution in contributions {
        let caller = contribution.tool().clone();
        let name = caller.name().to_owned();
        for caller_last in [false, true] {
            let builder = if caller_last {
                RuntimeBuilder::new()
                    .session_todo_tools(directory.path(), session)
                    .tool(caller.clone())
            } else {
                RuntimeBuilder::new()
                    .tool(caller.clone())
                    .session_todo_tools(directory.path(), session)
            };
            let (registry, _) = register_runtime_tools(builder.tools).expect("registration");
            assert_eq!(registry.is_builtin_instance(&name), !caller_last, "{name}");
        }
    }
}
