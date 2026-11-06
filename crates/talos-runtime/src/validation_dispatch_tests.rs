//! Production admission/dispatch with an isolated harmless Cargo executable fixture.
use super::*;
use async_trait::async_trait;
use std::{
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
    sync::Mutex,
    time::{Duration, Instant},
};
use talos_core::{
    message::{AgentEvent, Message},
    provider::{LanguageModel, ProviderResult},
};
use talos_provider::mock::MockProvider;

struct ObservedProvider {
    inner: MockProvider,
    requests: Arc<Mutex<Vec<Vec<Message>>>>,
}

#[async_trait]
impl LanguageModel for ObservedProvider {
    async fn stream(
        &self,
        messages: &[Message],
    ) -> ProviderResult<tokio::sync::mpsc::Receiver<AgentEvent>> {
        self.requests
            .lock()
            .expect("requests")
            .push(messages.to_vec());
        self.inner.stream(messages).await
    }
}

struct Child(std::process::Child);
impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn approved_validation_runs_owned_checks_and_returns_provider_evidence() {
    if let Some(workspace) = std::env::var_os("TALOS_VALIDATION_DISPATCH_FIXTURE") {
        let failed = std::env::var_os("TALOS_VALIDATION_FIXTURE_FAIL").is_some();
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
            .block_on(exercise(PathBuf::from(workspace), failed));
        return;
    }
    for failed in [false, true] {
        let root = tempfile::tempdir().expect("workspace");
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("manifest");
        std::fs::create_dir(root.path().join(".agent-governance")).expect("governance");
        std::fs::write(
            root.path().join(".agent-governance/manifest.yaml"),
            "profile: minimal\nstatus: adopting\n",
        )
        .expect("governance manifest");
        let bin = root.path().join("bin");
        std::fs::create_dir(&bin).expect("bin");
        let cargo = bin.join("cargo");
        std::fs::write(&cargo, "#!/bin/sh\nprintf '%s\\n' \"$*\" >> calls\nprintf 'owned-validation-output'\nif [ -n \"$TALOS_VALIDATION_FIXTURE_FAIL\" ]; then exit 7; fi\n").expect("fixture");
        std::fs::set_permissions(&cargo, std::fs::Permissions::from_mode(0o755)).expect("mode");
        let mut paths = vec![bin];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        let log = root.path().join("child.log");
        let output = std::fs::File::create(&log).expect("log");
        let mut command = Command::new(std::env::current_exe().expect("test executable"));
        command.args(["--exact", "validation_dispatch_tests::approved_validation_runs_owned_checks_and_returns_provider_evidence", "--nocapture"])
            .env("TALOS_VALIDATION_DISPATCH_FIXTURE", root.path())
            .env("PATH", std::env::join_paths(paths).expect("PATH"))
            .env_remove("TALOS_VALIDATION_FIXTURE_FAIL")
            .stdout(Stdio::from(output.try_clone().expect("log clone"))).stderr(Stdio::from(output));
        if failed {
            command.env("TALOS_VALIDATION_FIXTURE_FAIL", "1");
        }
        let mut child = Child(command.spawn().expect("isolated test"));
        let deadline = Instant::now() + Duration::from_secs(30);
        let status = loop {
            if let Some(status) = child.0.try_wait().expect("status") {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "isolated validation exceeded deadline"
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(
            status.success(),
            "{}",
            std::fs::read_to_string(log).expect("log")
        );
        assert_eq!(
            std::fs::read_to_string(root.path().join("calls")).expect("real command marker"),
            "fmt --all -- --check\ncheck --locked --workspace\ntest --quiet --locked --workspace\n"
        );
    }
}

async fn exercise(workspace: PathBuf, failed: bool) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let provider = Arc::new(ObservedProvider {
        inner: MockProvider::new()
            .with_tool_call(
                "validate_project",
                serde_json::json!({"mode":"run","profile":"workspace"}),
            )
            .with_response("validation observed"),
        requests: requests.clone(),
    });
    let mut runtime = RuntimeBuilder::new()
        .provider(provider)
        .workspace_root(workspace)
        .shared_tools()
        .permission_rule(PermissionRule::new_nature(
            talos_core::tool::ToolNature::Execute,
            None,
            None,
            PermissionDecision::Allow,
        ))
        .sandbox_fallback_policy(SandboxFallbackPolicy::AllowUnsandboxed)
        .build()
        .expect("runtime");
    runtime
        .submit("run fixed workspace validation")
        .await
        .expect("submit");
    let result = tokio::time::timeout(Duration::from_secs(20), async {
        let mut result = None;
        while let Some(event) = runtime.next_event().await {
            match event {
                SessionEvent::TurnEvent {
                    payload:
                        talos_core::session::TurnEventPayload::Progress {
                            event:
                                AgentEvent::ToolResult {
                                    result: observed, ..
                                },
                        },
                    ..
                } => result = Some(observed),
                SessionEvent::TurnEvent {
                    payload: talos_core::session::TurnEventPayload::Completed { .. },
                    ..
                } => break,
                _ => {}
            }
        }
        result.expect("tool result")
    })
    .await
    .expect("bounded turn");
    let evidence: serde_json::Value =
        serde_json::from_str(&result.content).expect("structured evidence");
    let host_records: Vec<_> = evidence["records"]
        .as_array()
        .expect("records")
        .iter()
        .filter(|record| record["execution_mode"] == "host_tool")
        .collect();
    assert_eq!(host_records.len(), 3);
    for record in host_records {
        assert_eq!(record["status"], if failed { "failed" } else { "passed" });
        assert_eq!(record["exit_status"], if failed { 7 } else { 0 });
        assert!(
            record["permission_decision"]
                .as_str()
                .expect("decision")
                .contains("exact prepared invocation admitted and consumed")
        );
    }
    assert_eq!(
        result.is_error, failed,
        "full validation result: {}",
        result.content
    );
    let serialized =
        serde_json::to_string(&*requests.lock().expect("requests")).expect("requests JSON");
    assert!(
        serialized.contains("owned-validation-output"),
        "provider continuation must receive evidence"
    );
    runtime.shutdown().await.expect("owned cleanup");
}
