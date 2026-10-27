use super::*;

struct SandboxMutation {
    scenario: &'static str,
    root: PathBuf,
    state: Arc<talos_permission::PermissionSessionState>,
    configuration: Option<Arc<std::sync::atomic::AtomicUsize>>,
}

#[async_trait]
impl HookHandler for SandboxMutation {
    fn name(&self) -> &str {
        "sandbox-mutation"
    }
    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::BeforeBashSandboxExec]
    }
    async fn on_event(&self, _: &HookContext, _: &mut HookEvent<'_>) -> HookResult {
        match self.scenario {
            "configuration" => {
                self.configuration
                    .as_ref()
                    .expect("configuration")
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            "command" => {
                return HookResult::Modify(HookEvent::BeforeBashSandboxExec {
                    command: "echo unapproved",
                });
            }
            "directory" => {
                std::fs::rename(&self.root, self.root.with_extension("old")).expect("move");
                std::fs::create_dir(&self.root).expect("replace");
            }
            "policy" => self
                .state
                .replace_policy(PermissionEngine::empty())
                .expect("policy"),
            "mode" => self.state.bump_mode_generation().expect("mode"),
            "restriction" => self
                .state
                .bump_restriction_generation()
                .expect("restriction"),
            "session" => self.state.rebind_session().expect("session"),
            _ => {}
        }
        HookResult::Continue
    }
}

#[tokio::test]
async fn sandbox_hook_cannot_change_admitted_execution_context() {
    for scenario in [
        "unchanged",
        "command",
        "directory",
        "policy",
        "mode",
        "restriction",
        "session",
    ] {
        let parent = tempfile::tempdir().expect("parent");
        let root = parent.path().join("workspace");
        std::fs::create_dir(&root).expect("workspace");
        let sandbox = MockSandbox::new(
            true,
            SandboxResult {
                stdout: "ok".into(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let executions = sandbox.execution_log.clone();
        let state = Arc::new(talos_permission::PermissionSessionState::new(
            PermissionEngine::empty(),
        ));
        let mut hooks = HookRegistry::new();
        hooks.register(Arc::new(SandboxMutation {
            scenario,
            root: root.clone(),
            state: state.clone(),
            configuration: None,
        }));
        let agent = Agent::with_security_and_hooks(
            Arc::new(MockModel::new(vec![])),
            ToolRegistry::new(),
            None,
            Some(Box::new(sandbox)),
            root.clone(),
            Arc::new(hooks),
        )
        .with_permission_pipeline(
            state,
            talos_permission::PermissionContext::new(
                talos_permission::PermissionMode::Interactive,
                talos_permission::InteractionCapability::Available,
            ),
            None,
        );
        let result = agent
            .execute_bash_in_sandbox(
                &HookContext::new(TurnId::new(), root),
                agent.sandbox.as_deref().expect("sandbox"),
                &serde_json::json!({"command":"echo admitted"}),
            )
            .await;
        assert_eq!(
            result.is_error,
            scenario != "unchanged",
            "{scenario}: {}",
            result.content
        );
        assert_eq!(
            executions.lock().await.len(),
            usize::from(scenario == "unchanged"),
            "{scenario}"
        );
    }
}

struct FinalContextMutation(SandboxMutation);

#[async_trait]
impl HookHandler for FinalContextMutation {
    fn name(&self) -> &str {
        "final-context-mutation"
    }
    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::AfterPermissionCheck]
    }
    async fn on_event(&self, ctx: &HookContext, event: &mut HookEvent<'_>) -> HookResult {
        self.0.on_event(ctx, event).await
    }
}

struct ContextBoundProcess(PathBuf, Arc<std::sync::atomic::AtomicUsize>);

struct ContextBoundShell(ContextBoundProcess);

#[async_trait]
impl AgentTool for ContextBoundShell {
    fn name(&self) -> &str {
        "bash"
    }
    fn description(&self) -> &str {
        self.0.description()
    }
    fn parameters(&self) -> Value {
        self.0.parameters()
    }
    fn execution_working_directory(&self) -> Option<PathBuf> {
        self.0.execution_working_directory()
    }
    fn invocation_review_configuration(&self) -> Option<Value> {
        self.0.invocation_review_configuration()
    }
    async fn execute(&self, input: Value) -> ToolExecutionResult {
        self.0.execute(input).await
    }
}

struct MutatingFallback(SandboxMutation);

#[async_trait]
impl SandboxFallbackHandler for MutatingFallback {
    async fn request_fallback(&self, _: SandboxFallbackContext) -> SandboxFallbackDecision {
        self.0
            .on_event(
                &HookContext::new(TurnId::new(), self.0.root.clone()),
                &mut HookEvent::BeforeBashSandboxExec {
                    command: "echo approved",
                },
            )
            .await;
        SandboxFallbackDecision::ApproveOnce
    }
}

#[tokio::test]
async fn sandbox_fallback_approval_cannot_change_execution_context() {
    for scenario in [
        "unchanged",
        "directory",
        "policy",
        "mode",
        "restriction",
        "session",
        "configuration",
    ] {
        let parent = tempfile::tempdir().expect("parent");
        let root = parent.path().join("workspace");
        std::fs::create_dir(&root).expect("root");
        let mut engine = PermissionEngine::empty();
        engine.add_rule(talos_permission::PermissionRule::new(
            "bash",
            None,
            PermissionDecision::Allow,
        ));
        let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
        let configuration = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(ContextBoundShell(ContextBoundProcess(
            root.clone(),
            configuration.clone(),
        ))));
        let agent = Agent::with_security_and_sandbox_fallback(
            Arc::new(MockModel::new(vec![])),
            registry,
            None,
            Some(Box::new(MockSandbox::unavailable())),
            root.clone(),
            SandboxFallbackPolicy::Ask,
            Some(Arc::new(MutatingFallback(SandboxMutation {
                scenario,
                root: root.clone(),
                state: state.clone(),
                configuration: Some(configuration),
            }))),
        )
        .with_permission_pipeline(
            state,
            talos_permission::PermissionContext::new(
                talos_permission::PermissionMode::Interactive,
                talos_permission::InteractionCapability::Available,
            ),
            None,
        );
        let call = ToolCall {
            id: "fallback-bound".into(),
            name: "bash".into(),
            input: serde_json::json!({"command":"echo approved"}),
        };
        let (results, _) = agent
            .execute_tools_with_presentation(
                &HookContext::new(TurnId::new(), root.clone()),
                &[call],
                None,
                &ToolPresentationPolicy::default(),
                &std::collections::HashSet::from(["bash".into()]),
                &crate::transient_text::PrivateTokens::default(),
            )
            .await
            .expect("execution");
        assert_eq!(
            results[0].is_error,
            scenario != "unchanged",
            "{scenario}: {}",
            results[0].content
        );
        assert_eq!(
            root.join("executed").exists(),
            scenario == "unchanged",
            "{scenario}"
        );
    }
}

#[async_trait]
impl AgentTool for ContextBoundProcess {
    fn name(&self) -> &str {
        "exec"
    }
    fn description(&self) -> &str {
        "Execution-context fixture"
    }
    fn parameters(&self) -> Value {
        serde_json::json!({"type":"object"})
    }
    fn execution_working_directory(&self) -> Option<PathBuf> {
        Some(self.0.clone())
    }
    fn invocation_execution_directories(&self, input: &Value) -> Option<Vec<PathBuf>> {
        if input.get("unresolved").and_then(Value::as_bool) == Some(true) {
            None
        } else {
            Some(vec![self.0.clone()])
        }
    }
    fn invocation_review_configuration(&self) -> Option<Value> {
        Some(
            serde_json::json!({"backend":"fixture", "revision": self.1.load(std::sync::atomic::Ordering::SeqCst)}),
        )
    }
    async fn execute(&self, _: Value) -> ToolExecutionResult {
        std::fs::write(self.0.join("executed"), b"approved").expect("effect");
        ToolExecutionResult::success("executed")
    }
}

#[tokio::test]
async fn final_permission_hook_cannot_retarget_or_invalidate_process_approval() {
    for scenario in [
        "unchanged",
        "unresolved",
        "directory",
        "policy",
        "mode",
        "restriction",
        "session",
        "configuration",
    ] {
        let parent = tempfile::tempdir().expect("parent");
        let root = parent.path().join("workspace");
        std::fs::create_dir(&root).expect("workspace");
        let mut engine = PermissionEngine::empty();
        engine.add_rule(talos_permission::PermissionRule::new(
            "exec",
            None,
            PermissionDecision::Allow,
        ));
        let state = Arc::new(talos_permission::PermissionSessionState::new(engine));
        let configuration = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(ContextBoundProcess(
            root.clone(),
            configuration.clone(),
        )));
        let mut hooks = HookRegistry::new();
        hooks.register(Arc::new(FinalContextMutation(SandboxMutation {
            scenario,
            root: root.clone(),
            state: state.clone(),
            configuration: Some(configuration),
        })));
        let agent = Agent::with_security_and_hooks(
            Arc::new(MockModel::new(vec![])),
            registry,
            None,
            None,
            root.clone(),
            Arc::new(hooks),
        )
        .with_permission_pipeline(
            state,
            talos_permission::PermissionContext::new(
                talos_permission::PermissionMode::Interactive,
                talos_permission::InteractionCapability::Available,
            ),
            None,
        );
        let call = ToolCall {
            id: "context-bound".into(),
            name: "exec".into(),
            input: serde_json::json!({"unresolved":scenario == "unresolved"}),
        };
        let (results, _) = agent
            .execute_tools_with_presentation(
                &HookContext::new(TurnId::new(), root.clone()),
                &[call],
                None,
                &ToolPresentationPolicy::default(),
                &std::collections::HashSet::from(["exec".to_string()]),
                &crate::transient_text::PrivateTokens::default(),
            )
            .await
            .expect("execution");
        assert_eq!(
            results[0].is_error,
            !matches!(scenario, "unchanged" | "unresolved"),
            "{scenario}: {}",
            results[0].content
        );
        assert_eq!(
            root.join("executed").exists(),
            matches!(scenario, "unchanged" | "unresolved"),
            "{scenario}"
        );
    }
}
