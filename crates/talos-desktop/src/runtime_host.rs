//! Thin Desktop adapter for the existing Talos Runtime.
//!
//! The adapter owns one Tokio runtime task and forwards bounded commands and
//! authoritative session events to the presentation layer. It deliberately
//! does not create tools, permissions, storage, or a second execution engine.

use std::path::PathBuf;
use std::sync::Arc;

use talos_runtime::{AgentEvent, LanguageModel, RuntimeBuilder, RuntimeHandle, SessionEvent};
use tokio::sync::mpsc;

const COMMAND_CAPACITY: usize = 16;
const EVENT_CAPACITY: usize = 128;
const MAX_PENDING_BYTES: usize = 1024 * 1024;

#[derive(Default)]
struct PendingOutput {
    queue: std::collections::VecDeque<RuntimeOutput>,
    bytes: usize,
}

impl PendingOutput {
    fn push(&mut self, output: RuntimeOutput) -> Result<(), ()> {
        let bytes = output_bytes(&output);
        if self.queue.len() >= EVENT_CAPACITY
            || bytes > MAX_PENDING_BYTES.saturating_sub(self.bytes)
        {
            return Err(());
        }
        self.bytes += bytes;
        self.queue.push_back(output);
        Ok(())
    }

    fn pop(&mut self) -> Option<RuntimeOutput> {
        let output = self.queue.pop_front()?;
        self.bytes -= output_bytes(&output);
        Some(output)
    }
}

fn output_bytes(output: &RuntimeOutput) -> usize {
    match output {
        RuntimeOutput::Text(text)
        | RuntimeOutput::Error(text)
        | RuntimeOutput::ToolUnavailable(text) => text.len(),
        RuntimeOutput::Started { turn_id } => turn_id.len(),
        RuntimeOutput::Completed {
            status: TerminalStatus::Error(error),
        } => error.len(),
        _ => 0,
    }
}

/// Commands accepted by a live Desktop runtime host.
#[derive(Debug)]
pub(crate) enum RuntimeCommand {
    /// Submit one user message to the existing Runtime.
    Submit(String),
    /// Interrupt the current Runtime turn.
    Interrupt,
    /// Request bounded Runtime shutdown.
    Shutdown,
}

/// Presentation-safe projection of an authoritative Runtime event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeOutput {
    /// A tool was requested but this host has no executable tools registered.
    ToolUnavailable(String),
    /// A model text fragment.
    Text(String),
    /// A turn began.
    Started { turn_id: String },
    /// A turn reached a terminal status.
    Completed { status: TerminalStatus },
    /// The Runtime reported an error.
    Error(String),
    /// The host has stopped accepting work.
    Stopped,
}

/// Small terminal projection; never exposes Runtime message-history internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TerminalStatus {
    Success,
    Cancelled,
    Error(String),
}

/// Handle used by the Desktop presentation to communicate with a live host.
pub(crate) struct RuntimeHost {
    commands: mpsc::Sender<RuntimeCommand>,
    outputs: mpsc::Receiver<RuntimeOutput>,
    terminal: Option<tokio::sync::oneshot::Receiver<RuntimeOutput>>,
    exit: Option<HostExit>,
}

/// Application-owned completion receipt, independent of GPUI task lifetime.
pub(crate) struct HostExit {
    commands: mpsc::Sender<RuntimeCommand>,
    result: std::sync::mpsc::Receiver<RuntimeOutput>,
}

impl HostExit {
    /// Called only after the GUI event loop has ended.
    pub(crate) fn finish(self, timeout: std::time::Duration) -> Result<(), String> {
        let _ = self.commands.try_send(RuntimeCommand::Shutdown);
        match self.result.recv_timeout(timeout) {
            Ok(RuntimeOutput::Stopped) => Ok(()),
            Ok(RuntimeOutput::Error(error)) => Err(error),
            Ok(_) => Err("host returned an invalid shutdown result".into()),
            Err(error) => Err(format!("host shutdown was not confirmed: {error}")),
        }
    }
}

impl RuntimeHost {
    /// Start one host task around an already constructed provider.
    #[cfg(test)]
    pub(crate) fn start(
        provider: Arc<dyn LanguageModel>,
        workspace_root: impl Into<PathBuf>,
    ) -> Result<Self, String> {
        Self::start_with(move || Ok((provider, 128_000)), workspace_root)
    }

    /// Load configuration and construct networking resources on the host thread.
    pub(crate) fn configured(workspace_root: impl Into<PathBuf>) -> Result<Self, String> {
        Self::start_with(
            || {
                let config = talos_config::Config::load().map_err(|error| error.to_string())?;
                let provider = crate::provider::configured_provider(&config)?;
                Ok((provider, config.resolve_model_limits().0))
            },
            workspace_root,
        )
    }

    fn start_with(
        provider: impl FnOnce() -> Result<(Arc<dyn LanguageModel>, u32), String> + Send + 'static,
        workspace_root: impl Into<PathBuf>,
    ) -> Result<Self, String> {
        let workspace_root = workspace_root.into();
        let (commands, command_rx) = mpsc::channel(COMMAND_CAPACITY);
        let (outputs, output_rx) = mpsc::channel(EVENT_CAPACITY);
        let (terminal_tx, terminal_rx) = tokio::sync::oneshot::channel();
        let (exit_tx, exit_rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("talos-desktop-runtime".into())
            .spawn(move || {
                let executor = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                let terminal = match executor {
                    Ok(executor) => executor.block_on(async move {
                        let (provider, context_limit) = match provider() {
                            Ok(provider) => provider,
                            Err(error) => {
                                return RuntimeOutput::Error(error);
                            }
                        };
                        match RuntimeBuilder::new()
                            .provider(provider)
                            .model_context_limit(context_limit)
                            .workspace_root(workspace_root)
                            .build()
                        {
                            Ok(handle) => run_host(command_rx, outputs, handle).await,
                            Err(error) => RuntimeOutput::Error(error.to_string()),
                        }
                    }),
                    Err(error) => {
                        RuntimeOutput::Error(format!("runtime host unavailable: {error}"))
                    }
                };
                let _ = terminal_tx.send(terminal.clone());
                let _ = exit_tx.send(terminal);
            })
            .map_err(|error| error.to_string())?;
        Ok(Self {
            exit: Some(HostExit {
                commands: commands.clone(),
                result: exit_rx,
            }),
            commands,
            outputs: output_rx,
            terminal: Some(terminal_rx),
        })
    }

    /// Queue a command without blocking the UI thread.
    #[cfg(test)]
    pub(crate) fn try_send(&self, command: RuntimeCommand) -> Result<(), RuntimeCommand> {
        self.commands
            .try_send(command)
            .map_err(|error| error.into_inner())
    }

    /// Separate command submission from event observation without sharing the receiver.
    pub(crate) fn command_sender(&self) -> mpsc::Sender<RuntimeCommand> {
        self.commands.clone()
    }

    pub(crate) fn take_exit(&mut self) -> Option<HostExit> {
        self.exit.take()
    }

    /// Receive the next bounded presentation event.
    pub(crate) async fn recv(&mut self) -> Option<RuntimeOutput> {
        if let Some(output) = self.outputs.recv().await {
            return Some(output);
        }
        match self.terminal.take() {
            Some(terminal) => Some(terminal.await.unwrap_or_else(|_| {
                RuntimeOutput::Error("runtime host exited without a shutdown result".into())
            })),
            None => None,
        }
    }
}

async fn run_host(
    mut commands: mpsc::Receiver<RuntimeCommand>,
    outputs: mpsc::Sender<RuntimeOutput>,
    mut handle: RuntimeHandle,
) -> RuntimeOutput {
    // Continue draining authoritative events while the presentation is slow.
    // Overflow stops the runtime explicitly rather than silently dropping text.
    let mut pending = PendingOutput::default();
    loop {
        tokio::select! {
            _ = outputs.closed() => return stop_host(handle).await,
            permit = outputs.reserve(), if !pending.queue.is_empty() => {
                match permit {
                    Ok(permit) => {
                        if let Some(output) = pending.pop() {
                            permit.send(output);
                        }
                    }
                    Err(_) => {
                        return stop_host(handle).await;
                    }
                }
            }
            command = commands.recv() => {
                match command {
                    Some(RuntimeCommand::Submit(message)) => {
                        if let Err(error) = handle.submit(message).await
                            && pending.push(RuntimeOutput::Error(error.to_string())).is_err()
                        {
                            return overflow_host(handle).await;
                        }
                    }
                    Some(RuntimeCommand::Interrupt) => {
                        if let Err(error) = handle.interrupt().await
                            && pending.push(RuntimeOutput::Error(error.to_string())).is_err()
                        {
                            return overflow_host(handle).await;
                        }
                    }
                    Some(RuntimeCommand::Shutdown) | None => {
                        return stop_host(handle).await;
                    }
                }
            }
            event = handle.next_event() => {
                let Some(event) = event else {
                    return stop_host(handle).await;
                };
                if let Some(output) = project_event(event)
                    && pending.push(output).is_err()
                {
                    return overflow_host(handle).await;
                }
            }
        }
    }
}

async fn stop_host(handle: RuntimeHandle) -> RuntimeOutput {
    match handle.shutdown().await {
        Ok(()) => RuntimeOutput::Stopped,
        Err(error) => RuntimeOutput::Error(format!("runtime shutdown failed: {error}")),
    }
}

async fn overflow_host(handle: RuntimeHandle) -> RuntimeOutput {
    let shutdown = stop_host(handle).await;
    let detail = match shutdown {
        RuntimeOutput::Error(error) => format!("; {error}"),
        _ => String::new(),
    };
    RuntimeOutput::Error(format!(
        "Presentation buffer exceeded; runtime stopped; displayed output is incomplete{detail}"
    ))
}

fn project_event(event: SessionEvent) -> Option<RuntimeOutput> {
    match event {
        SessionEvent::SubmissionStarted { turn_id, .. } => Some(RuntimeOutput::Started { turn_id }),
        SessionEvent::TurnEvent { payload, .. } => match payload {
            talos_runtime::TurnEventPayload::Progress {
                event: AgentEvent::ToolCall { call, .. },
            } => Some(RuntimeOutput::ToolUnavailable(call.name)),
            talos_runtime::TurnEventPayload::Progress {
                event: AgentEvent::TextDelta { delta },
            } => Some(RuntimeOutput::Text(delta)),
            talos_runtime::TurnEventPayload::Completed { status } => {
                let status = match status {
                    talos_runtime::TurnCompletionStatus::Success { .. } => TerminalStatus::Success,
                    talos_runtime::TurnCompletionStatus::Cancelled => TerminalStatus::Cancelled,
                    talos_runtime::TurnCompletionStatus::Error { message } => {
                        TerminalStatus::Error(message)
                    }
                };
                Some(RuntimeOutput::Completed { status })
            }
            _ => None,
        },
        SessionEvent::Error { message } => Some(RuntimeOutput::Error(message)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_provider::mock::MockProvider;

    #[tokio::test(flavor = "current_thread")]
    async fn requested_tools_are_explicitly_unavailable_in_no_tools_host() {
        let provider = MockProvider::new()
            .with_tool_call("bash", Default::default())
            .with_response("No tool ran.");
        let mut host = RuntimeHost::start(Arc::new(provider), ".").expect("host starts");
        host.try_send(RuntimeCommand::Submit("request a tool".into()))
            .expect("submit queues");
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let mut unavailable = false;
            loop {
                match host.recv().await.expect("turn event") {
                    RuntimeOutput::ToolUnavailable(name) => {
                        assert_eq!(name, "bash");
                        unavailable = true;
                    }
                    RuntimeOutput::Completed { .. } => {
                        assert!(unavailable);
                        break;
                    }
                    RuntimeOutput::Error(error) => panic!("unexpected host error: {error}"),
                    _ => {}
                }
            }
            host.try_send(RuntimeCommand::Shutdown)
                .expect("shutdown queues");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
        })
        .await
        .expect("bounded tool-unavailable turn");
    }

    struct PausedProvider {
        entered: tokio::sync::Notify,
        disconnected: Arc<tokio::sync::Notify>,
    }

    struct PendingConnection {
        entered: tokio::sync::Notify,
    }

    #[tokio::test(flavor = "current_thread")]
    async fn interrupt_during_history_compaction_reaches_cancelled_terminal() {
        let provider = Arc::new(PendingConnection {
            entered: tokio::sync::Notify::new(),
        });
        let mut handle = RuntimeBuilder::new()
            .provider(provider.clone())
            .model_context_limit(1024)
            .initial_history(
                (0..40)
                    .map(|index| talos_runtime::Message::User {
                        content: format!("turn {index}: {}", "previous history ".repeat(128)),
                    })
                    .collect(),
            )
            .build()
            .expect("runtime starts");
        handle.submit("next").await.expect("submission accepted");
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            provider.entered.notified(),
        )
        .await
        .expect("compaction provider reached");
        handle.interrupt().await.expect("interrupt accepted");
        let result = tokio::time::timeout(std::time::Duration::from_secs(1), async {
            loop {
                let event = handle.next_event().await.expect("terminal event");
                if let Some(RuntimeOutput::Completed { status }) = project_event(event) {
                    return status;
                }
            }
        })
        .await;
        handle.shutdown().await.expect("cleanup");
        assert_eq!(
            result.expect("compaction cancellation must not wait for provider"),
            TerminalStatus::Cancelled
        );
    }

    #[async_trait::async_trait]
    impl LanguageModel for PendingConnection {
        async fn stream(
            &self,
            _: &[talos_runtime::Message],
        ) -> talos_runtime::ProviderResult<talos_runtime::Receiver<AgentEvent>> {
            self.entered.notify_one();
            std::future::pending().await
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn interrupt_cancels_before_provider_connection_returns() {
        let provider = Arc::new(PendingConnection {
            entered: tokio::sync::Notify::new(),
        });
        let mut host = RuntimeHost::start(provider.clone(), ".").expect("host starts");
        host.try_send(RuntimeCommand::Submit("hello".into()))
            .expect("submit queues");
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            provider.entered.notified().await;
            host.try_send(RuntimeCommand::Interrupt)
                .expect("interrupt queues");
            loop {
                match host.recv().await.expect("cancel terminal") {
                    RuntimeOutput::Completed { status } => {
                        assert_eq!(status, TerminalStatus::Cancelled);
                        break;
                    }
                    RuntimeOutput::Error(error) => panic!("unexpected error: {error}"),
                    _ => {}
                }
            }
            host.try_send(RuntimeCommand::Shutdown)
                .expect("shutdown queues");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
        })
        .await
        .expect("bounded connection cancellation");
    }

    #[async_trait::async_trait]
    impl LanguageModel for PausedProvider {
        async fn stream(
            &self,
            _: &[talos_runtime::Message],
        ) -> talos_runtime::ProviderResult<talos_runtime::Receiver<AgentEvent>> {
            let (sender, receiver) = mpsc::channel(1);
            let disconnected = self.disconnected.clone();
            tokio::spawn(async move {
                sender.closed().await;
                disconnected.notify_one();
            });
            self.entered.notify_one();
            Ok(receiver)
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn interrupt_cancels_a_paused_provider_and_closes_its_stream() {
        let provider = Arc::new(PausedProvider {
            entered: tokio::sync::Notify::new(),
            disconnected: Arc::new(tokio::sync::Notify::new()),
        });
        let mut host = RuntimeHost::start(provider.clone(), ".").expect("host starts");
        host.try_send(RuntimeCommand::Submit("hello".into()))
            .expect("submit queues");
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            provider.entered.notified().await;
            host.try_send(RuntimeCommand::Interrupt)
                .expect("interrupt queues");
            loop {
                match host.recv().await.expect("cancel terminal") {
                    RuntimeOutput::Completed { status } => {
                        assert_eq!(status, TerminalStatus::Cancelled);
                        break;
                    }
                    RuntimeOutput::Error(error) => panic!("unexpected error: {error}"),
                    _ => {}
                }
            }
            provider.disconnected.notified().await;
            host.try_send(RuntimeCommand::Shutdown)
                .expect("shutdown queues");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
        })
        .await
        .expect("bounded cancellation");
    }

    #[test]
    fn application_exit_observes_cleanup_after_view_disposal() {
        let mut host = RuntimeHost::start(Arc::new(MockProvider::new()), ".").expect("host starts");
        let exit = host.take_exit().expect("application receipt");
        drop(host);
        exit.finish(std::time::Duration::from_secs(5))
            .expect("confirmed shutdown");
    }

    #[test]
    fn application_exit_timeout_is_reported_without_claiming_success() {
        let (commands, _commands_rx) = mpsc::channel(1);
        let (_result_tx, result) = std::sync::mpsc::channel();
        let exit = HostExit { commands, result };
        assert!(
            exit.finish(std::time::Duration::ZERO)
                .expect_err("no completion")
                .contains("not confirmed")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn oversized_output_stops_with_an_explicit_incomplete_output_error() {
        let mut host = RuntimeHost::start(
            Arc::new(MockProvider::new().with_response("x".repeat(MAX_PENDING_BYTES + 1))),
            ".",
        )
        .expect("host starts");
        host.try_send(RuntimeCommand::Submit("hello".into()))
            .expect("submit queues");
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                match host.recv().await.expect("overflow must be reported") {
                    RuntimeOutput::Error(message) => {
                        assert!(message.contains("displayed output is incomplete"));
                        break;
                    }
                    RuntimeOutput::Completed { .. } => panic!("overflow must not report success"),
                    _ => {}
                }
            }
            assert_eq!(host.recv().await, None);
        })
        .await
        .expect("bounded overflow shutdown");
    }

    #[test]
    fn pending_output_enforces_bytes_and_count_without_overwriting() {
        let mut pending = PendingOutput::default();
        assert!(
            pending
                .push(RuntimeOutput::Text("x".repeat(MAX_PENDING_BYTES)))
                .is_ok()
        );
        assert!(
            pending
                .push(RuntimeOutput::Text("overflow".into()))
                .is_err()
        );
        assert!(
            matches!(pending.pop(), Some(RuntimeOutput::Text(text)) if text.len() == MAX_PENDING_BYTES)
        );
        assert_eq!(pending.bytes, 0);
        for _ in 0..EVENT_CAPACITY {
            pending
                .push(RuntimeOutput::Stopped)
                .expect("within event limit");
        }
        assert!(pending.push(RuntimeOutput::Stopped).is_err());
        assert_eq!(pending.queue.len(), EVENT_CAPACITY);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_finishes_with_full_undrained_presentation_queue() {
        let handle = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new()))
            .build()
            .expect("runtime builds");
        let (commands, command_rx) = mpsc::channel(1);
        let (outputs, mut output_rx) = mpsc::channel(1);
        outputs
            .try_send(RuntimeOutput::Text("queued".into()))
            .expect("queue filled");
        commands
            .try_send(RuntimeCommand::Shutdown)
            .expect("shutdown queued");
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            run_host(command_rx, outputs, handle),
        )
        .await
        .expect("shutdown must not await presentation");
        assert_eq!(result, RuntimeOutput::Stopped);
        assert_eq!(
            output_rx.try_recv(),
            Ok(RuntimeOutput::Text("queued".into()))
        );
        assert!(output_rx.is_closed());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn receiver_disconnection_is_detected_without_pending_output() {
        let handle = RuntimeBuilder::new()
            .provider(Arc::new(MockProvider::new()))
            .build()
            .expect("runtime builds");
        let (_commands, command_rx) = mpsc::channel(1);
        let (outputs, output_rx) = mpsc::channel(1);
        drop(output_rx);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            run_host(command_rx, outputs, handle),
        )
        .await
        .expect("receiver closure must be observed");
        assert_eq!(result, RuntimeOutput::Stopped);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn provider_failure_is_not_projected_as_success() {
        let mut host = RuntimeHost::start(Arc::new(MockProvider::new().with_error(401)), ".")
            .expect("host starts");
        host.try_send(RuntimeCommand::Submit("hello".into()))
            .expect("submit queues");
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                match host.recv().await.expect("terminal event") {
                    RuntimeOutput::Completed {
                        status: TerminalStatus::Error(message),
                    } => {
                        assert!(message.contains("authentication failed"));
                        break;
                    }
                    RuntimeOutput::Completed { status } => {
                        panic!("unexpected terminal: {status:?}")
                    }
                    _ => {}
                }
            }
            host.try_send(RuntimeCommand::Shutdown)
                .expect("shutdown queues");
            assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
        })
        .await
        .expect("bounded provider failure");
    }

    #[test]
    fn full_command_queue_returns_the_unsent_message() {
        let (commands, _receiver) = mpsc::channel(1);
        let (_sender, outputs) = mpsc::channel(1);
        let host = RuntimeHost {
            commands,
            outputs,
            terminal: None,
            exit: None,
        };
        host.try_send(RuntimeCommand::Submit("first".into()))
            .expect("first queues");
        assert!(
            matches!(host.try_send(RuntimeCommand::Submit("second".into())),
            Err(RuntimeCommand::Submit(message)) if message == "second")
        );
    }

    #[test]
    fn host_starts_without_an_ambient_tokio_runtime() {
        let provider = Arc::new(MockProvider::new().with_response("hello"));
        let mut host = RuntimeHost::start(provider, ".").expect("host starts");
        host.try_send(RuntimeCommand::Submit("hello".into()))
            .expect("submit queues");
        let executor = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test executor");
        executor.block_on(async {
            tokio::time::timeout(std::time::Duration::from_secs(5), async {
                loop {
                    match host.recv().await.expect("host event") {
                        RuntimeOutput::Completed { .. } => break,
                        RuntimeOutput::Error(error) => panic!("host failed: {error}"),
                        _ => {}
                    }
                }
                host.try_send(RuntimeCommand::Shutdown)
                    .expect("shutdown queues");
                assert_eq!(host.recv().await, Some(RuntimeOutput::Stopped));
            })
            .await
            .expect("bounded host completion");
        });
    }

    #[tokio::test(flavor = "current_thread")]
    async fn host_forwards_mock_text_and_terminal_event() {
        let provider = Arc::new(MockProvider::new().with_response("hello"));
        let mut host = RuntimeHost::start(provider, ".").expect("host starts");
        host.try_send(RuntimeCommand::Submit("say hello".into()))
            .expect("submit queues");
        let mut text = String::new();
        let mut completed = false;
        for _ in 0..16 {
            match host.recv().await.expect("output") {
                RuntimeOutput::Text(value) => text.push_str(&value),
                RuntimeOutput::Completed { .. } => {
                    completed = true;
                    break;
                }
                RuntimeOutput::Error(error) => panic!("unexpected runtime error: {error}"),
                _ => {}
            }
        }
        assert_eq!(text, "hello");
        assert!(completed);
        host.try_send(RuntimeCommand::Shutdown)
            .expect("shutdown queues");
    }
}
