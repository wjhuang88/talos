//! Embedded Skill Level 1/2 disclosure through the public runtime API (ADR-092).
use super::*;
use talos_core::provider::ToolDefinition;

type Requests = Arc<StdMutex<Vec<(String, Vec<String>)>>>;

struct RecordingModel {
    inner: MockProvider,
    requests: Requests,
}

#[async_trait]
impl LanguageModel for RecordingModel {
    async fn stream(&self, messages: &[Message]) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.stream_with_tools(messages, &[]).await
    }

    async fn stream_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> ProviderResult<mpsc::Receiver<AgentEvent>> {
        self.requests.lock().expect("requests").push((
            format!("{messages:?}"),
            tools.iter().map(|tool| tool.name.clone()).collect(),
        ));
        self.inner.stream_with_tools(messages, tools).await
    }
}

fn recording(inner: MockProvider) -> (Arc<RecordingModel>, Requests) {
    let requests = Requests::default();
    (
        Arc::new(RecordingModel {
            inner,
            requests: requests.clone(),
        }),
        requests,
    )
}

fn write_skill(root: &std::path::Path, name: &str, body: &str) -> RuntimeSkill {
    let dir = root.join(name);
    std::fs::create_dir_all(&dir).expect("skill dir");
    std::fs::write(
        dir.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: {name} skill\n---\n{body}\n"),
    )
    .expect("write SKILL.md");
    let mut loader = RuntimeSkillLoader {
        search_paths: vec![root.to_path_buf()],
        ..RuntimeSkillLoader::default()
    };
    loader.discover().expect("discover");
    loader
        .skills
        .into_iter()
        .find(|skill| skill.name == name)
        .expect("skill discovered")
}

async fn run_turn(runtime: &mut RuntimeHandle, message: &str) -> Vec<SessionEvent> {
    runtime.submit(message).await.expect("submit");
    let mut events = Vec::new();
    while let Some(event) = runtime.next_event().await {
        let done = matches!(
            event,
            SessionEvent::TurnEvent {
                payload: TurnEventPayload::Completed { .. },
                ..
            }
        );
        events.push(event);
        if done {
            break;
        }
    }
    events
}

#[tokio::test]
async fn set_skill_context_reaches_next_provider_request_and_clear_removes_it() {
    let skills = tempfile::tempdir().expect("skills");
    let skill = write_skill(skills.path(), "recipe", "FOLLOW-RECIPE-MARKER");
    let (model, requests) = recording(
        MockProvider::new()
            .with_response("one")
            .with_response("two"),
    );
    let mut runtime = RuntimeBuilder::new()
        .provider(model)
        .skill_index(vec![RuntimeSkillIndex {
            name: skill.name.clone(),
            description: skill.description.clone(),
            triggers: Vec::new(),
            estimated_tokens: 1,
            source: skill.source,
        }])
        .build()
        .expect("runtime");

    let activation = RuntimeSkillActivation::new(&skill, 24 * 1024);
    runtime
        .set_skill_context(activation.name(), activation.content())
        .await
        .expect("set");
    let events = run_turn(&mut runtime, "first").await;
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SessionEvent::Error { .. })),
        "{events:?}"
    );

    runtime.clear_skill_context().await.expect("clear");
    run_turn(&mut runtime, "second").await;

    let requests = requests.lock().expect("requests").clone();
    assert_eq!(requests.len(), 2);
    assert!(requests[0].0.contains("# Activated Skill: recipe"));
    assert!(requests[0].0.contains("FOLLOW-RECIPE-MARKER"));
    assert!(!requests[1].0.contains("FOLLOW-RECIPE-MARKER"));
    assert!(requests[1].0.contains("recipe skill"));
    runtime.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn set_skill_context_during_active_turn_is_rejected_with_error_event() {
    let (mut runtime, entered, release) = gated_runtime().await;
    runtime.submit("hold").await.expect("submit");
    entered.notified().await;
    runtime
        .set_skill_context("late", "LATE-MARKER")
        .await
        .expect("queued");
    release.notify_one();
    let mut saw_rejection = false;
    while let Some(event) = runtime.next_event().await {
        if let SessionEvent::Error { message } = &event {
            saw_rejection |= message.contains("cannot change active skill while a turn is active");
        }
        if matches!(
            event,
            SessionEvent::TurnEvent {
                payload: TurnEventPayload::Completed { .. },
                ..
            }
        ) {
            break;
        }
    }
    if !saw_rejection {
        let event = tokio::time::timeout(Duration::from_secs(5), runtime.next_event())
            .await
            .expect("rejection event");
        assert!(
            matches!(&event, Some(SessionEvent::Error { message }) if message.contains("while a turn is active")),
            "{event:?}"
        );
    }
    runtime.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn skill_read_tool_is_opt_in_and_reads_bound_skills() {
    let skills = tempfile::tempdir().expect("skills");
    let alpha = write_skill(skills.path(), "alpha", "ALPHA-BODY");
    let beta = write_skill(skills.path(), "beta", "BETA-BODY");
    std::fs::write(skills.path().join("beta/contract.md"), "BETA-CONTRACT").expect("write");

    let (model, requests) = recording(MockProvider::new().with_response("plain"));
    let mut runtime = RuntimeBuilder::new()
        .provider(model)
        .skill_read_tool(Vec::new())
        .build()
        .expect("runtime");
    run_turn(&mut runtime, "hi").await;
    assert!(
        !requests.lock().expect("requests")[0]
            .1
            .iter()
            .any(|name| name == "skill_read")
    );
    runtime.shutdown().await.expect("shutdown");

    let (model, requests) = recording(
        MockProvider::new()
            .with_tool_call("skill_read", serde_json::json!({"skill": "alpha"}))
            .with_response("one")
            .with_tool_call(
                "skill_read",
                serde_json::json!({"skill": "beta", "reference": "contract.md"}),
            )
            .with_response("two")
            .with_tool_call(
                "skill_read",
                serde_json::json!({"skill": "beta", "reference": "../alpha/SKILL.md"}),
            )
            .with_response("three"),
    );
    let mut runtime = RuntimeBuilder::new()
        .provider(model)
        .skill_read_tool(vec![alpha, beta])
        .build()
        .expect("runtime");
    for message in ["body", "reference", "escape"] {
        run_turn(&mut runtime, message).await;
    }
    let requests = requests.lock().expect("requests").clone();
    assert_eq!(requests.len(), 6, "tool round plus final per turn");
    assert!(requests[0].1.iter().any(|name| name == "skill_read"));
    assert!(requests[1].0.contains("ALPHA-BODY"), "{}", requests[1].0);
    assert!(requests[1].0.contains("- SKILL.md"), "{}", requests[1].0);
    assert!(requests[3].0.contains("BETA-CONTRACT"), "{}", requests[3].0);
    assert!(
        requests[5]
            .0
            .contains("must stay inside the skill directory"),
        "{}",
        requests[5].0
    );
    runtime.shutdown().await.expect("shutdown");
}

struct CallerSkillRead;

#[tokio::test]
async fn skill_read_respects_read_policy_for_body_and_reference() {
    for denied in [false, true] {
        for reference in [false, true] {
            let skills = tempfile::tempdir().expect("skills");
            let skill = write_skill(skills.path(), "policy", "POLICY-BODY-MARKER");
            std::fs::write(
                skills.path().join("policy/reference.md"),
                "POLICY-REFERENCE-MARKER",
            )
            .expect("reference");
            let mut input = serde_json::json!({"skill": "policy"});
            if reference {
                input["reference"] = serde_json::json!("reference.md");
            }
            let (model, requests) = recording(
                MockProvider::new()
                    .with_tool_call("skill_read", input)
                    .with_response("done"),
            );
            let mut builder = RuntimeBuilder::new()
                .provider(model)
                .workspace_root(skills.path())
                .skill_read_tool(vec![skill]);
            if denied {
                builder = builder.permission_rule(PermissionRule::new_nature(
                    ToolNature::Read,
                    None,
                    None,
                    PermissionDecision::Deny("skill reading disabled by user".into()),
                ));
            }
            let mut runtime = builder.build().expect("runtime");
            tokio::time::timeout(Duration::from_secs(5), run_turn(&mut runtime, "read skill"))
                .await
                .expect("bounded turn");
            let captured = requests.lock().expect("requests").clone();
            assert_eq!(captured.len(), 2, "tool result must reach next request");
            assert!(!captured[0].0.contains("POLICY-BODY-MARKER"));
            assert!(!captured[0].0.contains("POLICY-REFERENCE-MARKER"));
            let result = &captured[1].0;
            if denied {
                assert!(!result.contains("POLICY-BODY-MARKER"), "{result}");
                assert!(!result.contains("POLICY-REFERENCE-MARKER"), "{result}");
                assert!(result.to_lowercase().contains("denied"), "{result}");
            } else {
                let expected = if reference {
                    "POLICY-REFERENCE-MARKER"
                } else {
                    "POLICY-BODY-MARKER"
                };
                assert!(result.contains(expected), "{result}");
                if reference {
                    assert!(!result.contains("POLICY-BODY-MARKER"), "{result}");
                }
            }
            runtime.shutdown().await.expect("shutdown");
        }
    }
}

#[async_trait]
impl AgentTool for CallerSkillRead {
    fn name(&self) -> &str {
        "skill_read"
    }
    fn description(&self) -> &str {
        "caller-owned reader"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({"type": "object"})
    }
    fn is_read_only(&self) -> bool {
        true
    }
    async fn execute(&self, _input: Value) -> ToolResult {
        ToolResult::success("CALLER-READER")
    }
}

#[tokio::test]
async fn caller_skill_read_tool_takes_precedence() {
    let skills = tempfile::tempdir().expect("skills");
    let alpha = write_skill(skills.path(), "alpha", "ALPHA-BODY");
    let (model, requests) = recording(
        MockProvider::new()
            .with_tool_call("skill_read", serde_json::json!({"skill": "alpha"}))
            .with_response("done"),
    );
    let mut runtime = RuntimeBuilder::new()
        .provider(model)
        .tool(Arc::new(CallerSkillRead))
        .skill_read_tool(vec![alpha])
        .build()
        .expect("runtime");
    run_turn(&mut runtime, "read").await;
    let requests = requests.lock().expect("requests").clone();
    let last = &requests.last().expect("request").0;
    assert!(last.contains("CALLER-READER"), "{last}");
    assert!(!last.contains("ALPHA-BODY"), "{last}");
    runtime.shutdown().await.expect("shutdown");
}
