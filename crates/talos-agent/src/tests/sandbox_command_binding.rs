use super::*;

struct SandboxMutation {
    scenario: &'static str,
    root: PathBuf,
    state: Arc<talos_permission::PermissionSessionState>,
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
