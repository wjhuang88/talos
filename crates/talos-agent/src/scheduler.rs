//! Session-scoped scheduled follow-up scheduler.
//!
//! Scheduled fires enter the same Actor-owned structured submission protocol as
//! interactive steering (ADR-056 / TUI-044). A timer firing is not considered
//! delivered when it merely enters the bounded SQ: the scheduler retains the
//! exact immutable fire identity until the Session Actor returns a durable
//! `AcceptedPending`/`AlreadyAccepted` receipt. Lost receipts are reconciled
//! with the same identity, and recurring schedules do not advance while a fire
//! remains unresolved.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use talos_core::session::{
    MAX_SUBMISSION_ITEM_BYTES, SessionOp, StructuredSubmission, SubmissionItem, SubmissionKind,
    SubmissionReceipt, SubmissionReceiptDisposition, SubmissionSource,
};
use talos_core::tool::{AgentTool, ToolFamily, ToolNature, ToolResult};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

/// Minimum allowed one-shot delay.
pub(crate) const MIN_DELAY_SECS: u64 = 1;
/// Maximum allowed one-shot delay (24 hours).
pub(crate) const MAX_DELAY_SECS: u64 = 86_400;
/// Minimum allowed recurring interval.
pub(crate) const MIN_INTERVAL_SECS: u64 = 5;
/// Maximum allowed recurring interval (1 hour).
pub(crate) const MAX_INTERVAL_SECS: u64 = 3_600;

const SCHEDULER_COMMAND_CAPACITY: usize = 64;
const DELIVERY_SEND_TIMEOUT: Duration = Duration::from_millis(250);
const DELIVERY_RECEIPT_TIMEOUT: Duration = Duration::from_secs(1);
const DELIVERY_RETRY_DELAY: Duration = Duration::from_secs(1);

/// Visible prefix retained for transcript and provider readability.
pub(crate) const SCHEDULED_FOLLOWUP_LABEL: &str = "[scheduled-followup]";

static NEXT_TASK_SEQ: AtomicU64 = AtomicU64::new(1);
static NEXT_DELIVERY_SEQ: AtomicU64 = AtomicU64::new(1);

pub(crate) fn validate_delay_secs(delay_secs: u64) -> Result<(), String> {
    if delay_secs < MIN_DELAY_SECS {
        return Err(format!(
            "delay_secs must be at least {MIN_DELAY_SECS}; got {delay_secs}"
        ));
    }
    if delay_secs > MAX_DELAY_SECS {
        return Err(format!(
            "delay_secs must be at most {MAX_DELAY_SECS}; got {delay_secs}"
        ));
    }
    Ok(())
}

pub(crate) fn validate_interval_secs(interval_secs: u64) -> Result<(), String> {
    if interval_secs < MIN_INTERVAL_SECS {
        return Err(format!(
            "interval_secs must be at least {MIN_INTERVAL_SECS}; got {interval_secs}"
        ));
    }
    if interval_secs > MAX_INTERVAL_SECS {
        return Err(format!(
            "interval_secs must be at most {MAX_INTERVAL_SECS}; got {interval_secs}"
        ));
    }
    Ok(())
}

pub(crate) fn label_scheduled_message(message: &str) -> String {
    format!("{SCHEDULED_FOLLOWUP_LABEL} {message}")
}

fn validate_scheduled_message(message: &str) -> Result<String, String> {
    if message.is_empty() {
        return Err("message must not be empty".into());
    }
    let labeled = label_scheduled_message(message);
    if labeled.len() > MAX_SUBMISSION_ITEM_BYTES {
        return Err(format!(
            "message exceeds the structured submission item limit of {MAX_SUBMISSION_ITEM_BYTES} UTF-8 bytes"
        ));
    }
    Ok(labeled)
}

pub(crate) fn next_task_id() -> String {
    format!("sched_{}", NEXT_TASK_SEQ.fetch_add(1, Ordering::Relaxed))
}

fn next_delivery_identity(task_id: &str) -> String {
    let registration = NEXT_DELIVERY_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("scheduler:{task_id}:registration:{registration}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScheduleKind {
    OneShot,
    Recurring { interval: Duration },
}

impl fmt::Display for ScheduleKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OneShot => write!(f, "one-shot"),
            Self::Recurring { interval } => write!(f, "recurring ({}s)", interval.as_secs()),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ScheduledTaskInfo {
    pub id: String,
    pub message: String,
    pub kind: ScheduleKind,
    pub created_at: Instant,
    pub fire_at: Instant,
}

impl ScheduledTaskInfo {
    #[must_use]
    pub fn remaining(&self) -> Duration {
        self.fire_at.saturating_duration_since(Instant::now())
    }

    #[must_use]
    fn delivery_state(&self) -> &'static str {
        if self.remaining().is_zero() {
            "delivery-blocked"
        } else {
            "scheduled"
        }
    }
}

#[derive(Debug)]
pub(crate) enum ScheduleRegistrationResult {
    Registered { task_id: String },
    InvalidDuration { reason: String },
    InvalidMessage { reason: String },
}

#[derive(Debug)]
pub(crate) enum CancelResult {
    Cancelled,
    NotFound,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum ScheduleCommand {
    RegisterOneShot {
        id: Option<String>,
        message: String,
        delay: Duration,
        response_tx: oneshot::Sender<ScheduleRegistrationResult>,
    },
    RegisterRecurring {
        id: Option<String>,
        message: String,
        interval: Duration,
        response_tx: oneshot::Sender<ScheduleRegistrationResult>,
    },
    Cancel {
        id: String,
        response_tx: oneshot::Sender<CancelResult>,
    },
    List {
        response_tx: oneshot::Sender<Vec<ScheduledTaskInfo>>,
    },
    Shutdown,
}

#[derive(Clone)]
pub(crate) struct SchedulerHandle {
    cmd_tx: mpsc::Sender<ScheduleCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchedulerSendError {
    Full,
    Closed,
}

impl SchedulerHandle {
    pub(crate) fn new(cmd_tx: mpsc::Sender<ScheduleCommand>) -> Self {
        Self { cmd_tx }
    }

    pub(crate) async fn send(&self, command: ScheduleCommand) -> Result<(), SchedulerSendError> {
        match self.cmd_tx.try_send(command) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => Err(SchedulerSendError::Full),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(SchedulerSendError::Closed),
        }
    }
}

impl fmt::Debug for SchedulerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SchedulerHandle")
            .field("cmd_tx", &"mpsc::Sender<ScheduleCommand>")
            .finish()
    }
}

struct ActiveTask {
    info: ScheduledTaskInfo,
    handle: JoinHandle<()>,
}

struct AcceptedFire {
    task_id: String,
    next_fire_at: Option<Instant>,
}

pub(crate) struct SchedulerActor {
    cmd_rx: mpsc::Receiver<ScheduleCommand>,
    sq_tx: mpsc::Sender<SessionOp>,
    session_generation: u64,
    cancel_token: CancellationToken,
    tasks: HashMap<String, ActiveTask>,
    accepted_tx: mpsc::UnboundedSender<AcceptedFire>,
    accepted_rx: mpsc::UnboundedReceiver<AcceptedFire>,
}

impl SchedulerActor {
    pub(crate) fn new(
        cmd_rx: mpsc::Receiver<ScheduleCommand>,
        sq_tx: mpsc::Sender<SessionOp>,
        session_generation: u64,
        cancel_token: CancellationToken,
    ) -> Self {
        let (accepted_tx, accepted_rx) = mpsc::unbounded_channel();
        Self {
            cmd_rx,
            sq_tx,
            session_generation,
            cancel_token,
            tasks: HashMap::new(),
            accepted_tx,
            accepted_rx,
        }
    }

    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                biased;

                _ = self.cancel_token.cancelled() => break,

                Some(accepted) = self.accepted_rx.recv() => {
                    match accepted.next_fire_at {
                        None => {
                            self.tasks.remove(&accepted.task_id);
                        }
                        Some(next_fire_at) => {
                            if let Some(task) = self.tasks.get_mut(&accepted.task_id) {
                                task.info.fire_at = next_fire_at;
                            }
                        }
                    }
                }

                command = self.cmd_rx.recv() => {
                    match command {
                        Some(ScheduleCommand::RegisterOneShot {
                            id,
                            message,
                            delay,
                            response_tx,
                        }) => self.handle_register_one_shot(id, message, delay, response_tx),
                        Some(ScheduleCommand::RegisterRecurring {
                            id,
                            message,
                            interval,
                            response_tx,
                        }) => self.handle_register_recurring(
                            id,
                            message,
                            interval,
                            response_tx,
                        ),
                        Some(ScheduleCommand::Cancel { id, response_tx }) => {
                            self.handle_cancel(id, response_tx);
                        }
                        Some(ScheduleCommand::List { response_tx }) => {
                            self.handle_list(response_tx);
                        }
                        Some(ScheduleCommand::Shutdown) | None => break,
                    }
                }
            }
        }

        for (_, task) in self.tasks.drain() {
            task.handle.abort();
        }
    }

    fn handle_register_one_shot(
        &mut self,
        id: Option<String>,
        message: String,
        delay: Duration,
        response_tx: oneshot::Sender<ScheduleRegistrationResult>,
    ) {
        if let Err(reason) = validate_delay_secs(delay.as_secs()) {
            let _ = response_tx.send(ScheduleRegistrationResult::InvalidDuration { reason });
            return;
        }
        let labeled_message = match validate_scheduled_message(&message) {
            Ok(message) => message,
            Err(reason) => {
                let _ = response_tx.send(ScheduleRegistrationResult::InvalidMessage { reason });
                return;
            }
        };

        let task_id = id.unwrap_or_else(next_task_id);
        if self.tasks.contains_key(&task_id) {
            let _ = response_tx.send(ScheduleRegistrationResult::InvalidMessage {
                reason: format!("task ID {task_id} is already active"),
            });
            return;
        }
        let delivery_identity = next_delivery_identity(&task_id);
        let now = Instant::now();
        let sq_tx = self.sq_tx.clone();
        let session_generation = self.session_generation;
        let accepted_tx = self.accepted_tx.clone();
        let task_id_for_fire = task_id.clone();
        let message_for_fire = labeled_message.clone();

        let handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let submission =
                scheduled_submission(&delivery_identity, 1, session_generation, message_for_fire);
            deliver_until_accepted(&sq_tx, submission).await;
            let _ = accepted_tx.send(AcceptedFire {
                task_id: task_id_for_fire,
                next_fire_at: None,
            });
        });

        self.tasks.insert(
            task_id.clone(),
            ActiveTask {
                info: ScheduledTaskInfo {
                    id: task_id.clone(),
                    message: labeled_message,
                    kind: ScheduleKind::OneShot,
                    created_at: now,
                    fire_at: now + delay,
                },
                handle,
            },
        );
        let _ = response_tx.send(ScheduleRegistrationResult::Registered { task_id });
    }

    fn handle_register_recurring(
        &mut self,
        id: Option<String>,
        message: String,
        interval: Duration,
        response_tx: oneshot::Sender<ScheduleRegistrationResult>,
    ) {
        if let Err(reason) = validate_interval_secs(interval.as_secs()) {
            let _ = response_tx.send(ScheduleRegistrationResult::InvalidDuration { reason });
            return;
        }
        let labeled_message = match validate_scheduled_message(&message) {
            Ok(message) => message,
            Err(reason) => {
                let _ = response_tx.send(ScheduleRegistrationResult::InvalidMessage { reason });
                return;
            }
        };

        let task_id = id.unwrap_or_else(next_task_id);
        if self.tasks.contains_key(&task_id) {
            let _ = response_tx.send(ScheduleRegistrationResult::InvalidMessage {
                reason: format!("task ID {task_id} is already active"),
            });
            return;
        }
        let delivery_identity = next_delivery_identity(&task_id);
        let now = Instant::now();
        let sq_tx = self.sq_tx.clone();
        let session_generation = self.session_generation;
        let accepted_tx = self.accepted_tx.clone();
        let task_id_for_fire = task_id.clone();
        let message_for_fire = labeled_message.clone();

        let handle = tokio::spawn(async move {
            let mut fire_sequence = 1_u64;
            tokio::time::sleep(interval).await;
            loop {
                let submission = scheduled_submission(
                    &delivery_identity,
                    fire_sequence,
                    session_generation,
                    message_for_fire.clone(),
                );
                deliver_until_accepted(&sq_tx, submission).await;

                let next_fire_at = Instant::now() + interval;
                if accepted_tx
                    .send(AcceptedFire {
                        task_id: task_id_for_fire.clone(),
                        next_fire_at: Some(next_fire_at),
                    })
                    .is_err()
                {
                    break;
                }
                let Some(next_sequence) = fire_sequence.checked_add(1) else {
                    tracing::warn!(
                        task_id = %task_id_for_fire,
                        "recurring scheduler exhausted its fire sequence"
                    );
                    break;
                };
                fire_sequence = next_sequence;
                tokio::time::sleep(interval).await;
            }
        });

        self.tasks.insert(
            task_id.clone(),
            ActiveTask {
                info: ScheduledTaskInfo {
                    id: task_id.clone(),
                    message: labeled_message,
                    kind: ScheduleKind::Recurring { interval },
                    created_at: now,
                    fire_at: now + interval,
                },
                handle,
            },
        );
        let _ = response_tx.send(ScheduleRegistrationResult::Registered { task_id });
    }

    fn handle_cancel(&mut self, id: String, response_tx: oneshot::Sender<CancelResult>) {
        if let Some(task) = self.tasks.remove(&id) {
            task.handle.abort();
            let _ = response_tx.send(CancelResult::Cancelled);
        } else {
            let _ = response_tx.send(CancelResult::NotFound);
        }
    }

    fn handle_list(&self, response_tx: oneshot::Sender<Vec<ScheduledTaskInfo>>) {
        let snapshot = self.tasks.values().map(|task| task.info.clone()).collect();
        let _ = response_tx.send(snapshot);
    }
}

fn scheduled_submission(
    delivery_identity: &str,
    fire_sequence: u64,
    session_generation: u64,
    text: String,
) -> StructuredSubmission {
    let submission_id = format!("{delivery_identity}:fire:{fire_sequence}");
    StructuredSubmission {
        id: submission_id.clone(),
        source: SubmissionSource::Scheduler,
        sender_generation: session_generation,
        items: vec![SubmissionItem {
            id: format!("{submission_id}:item:1"),
            enqueue_sequence: fire_sequence,
            kind: SubmissionKind::UserTurn,
            text,
            attachments: Vec::new(),
        }],
    }
}

async fn deliver_until_accepted(sq_tx: &mpsc::Sender<SessionOp>, submission: StructuredSubmission) {
    let (receipt_tx, mut receipt_rx) = mpsc::unbounded_channel();
    let mut submit_required = true;
    let mut attempts = 0_u64;

    loop {
        attempts = attempts.saturating_add(1);
        let operation = if submit_required {
            SessionOp::SubmitStructuredTracked {
                submission: submission.clone(),
                receipt_tx: Some(receipt_tx.clone()),
            }
        } else {
            SessionOp::ReconcileStructuredTracked {
                submission: submission.clone(),
                receipt_tx: Some(receipt_tx.clone()),
            }
        };

        match tokio::time::timeout(DELIVERY_SEND_TIMEOUT, sq_tx.send(operation)).await {
            Ok(Ok(())) => {}
            Ok(Err(_)) | Err(_) => {
                log_blocked_delivery(&submission.id, attempts, "session queue unavailable");
                tokio::time::sleep(DELIVERY_RETRY_DELAY).await;
                continue;
            }
        }

        let receipt = tokio::time::timeout(
            DELIVERY_RECEIPT_TIMEOUT,
            next_matching_receipt(&mut receipt_rx, &submission.id),
        )
        .await;
        match receipt {
            Ok(Some(receipt)) if receipt.disposition.has_durable_custody() => return,
            Ok(Some(receipt)) => {
                submit_required = match receipt.disposition {
                    SubmissionReceiptDisposition::NotAccepted => true,
                    SubmissionReceiptDisposition::Rejected { reason } => {
                        if attempts == 1 || attempts.is_power_of_two() {
                            tracing::warn!(
                                submission_id = %submission.id,
                                ?reason,
                                attempts,
                                "scheduled fire remains blocked before durable custody"
                            );
                        }
                        true
                    }
                    SubmissionReceiptDisposition::AcceptedPending
                    | SubmissionReceiptDisposition::AlreadyAccepted { .. } => false,
                };
            }
            Ok(None) | Err(_) => {
                // The submit may have committed while its receipt was lost. Ask
                // the same Actor authority before considering a resend.
                submit_required = false;
                log_blocked_delivery(&submission.id, attempts, "durable receipt unavailable");
            }
        }
        tokio::time::sleep(DELIVERY_RETRY_DELAY).await;
    }
}

async fn next_matching_receipt(
    receipt_rx: &mut mpsc::UnboundedReceiver<SubmissionReceipt>,
    submission_id: &str,
) -> Option<SubmissionReceipt> {
    while let Some(receipt) = receipt_rx.recv().await {
        if receipt.submission_id == submission_id && receipt.source == SubmissionSource::Scheduler {
            return Some(receipt);
        }
    }
    None
}

fn log_blocked_delivery(submission_id: &str, attempts: u64, reason: &str) {
    if attempts == 1 || attempts.is_power_of_two() {
        tracing::warn!(
            submission_id,
            attempts,
            reason,
            "scheduled fire retained for bounded retry"
        );
    }
}

#[allow(dead_code)]
pub(crate) fn spawn_scheduler_actor(
    sq_tx: mpsc::Sender<SessionOp>,
    session_generation: u64,
    cancel_token: CancellationToken,
) -> (SchedulerHandle, JoinHandle<()>) {
    let (cmd_tx, cmd_rx) = mpsc::channel(SCHEDULER_COMMAND_CAPACITY);
    let handle = SchedulerHandle::new(cmd_tx);
    let actor = SchedulerActor::new(cmd_rx, sq_tx, session_generation, cancel_token);
    let join = tokio::spawn(async move { actor.run().await });
    (handle, join)
}

pub fn create_delay_tool_and_scheduler() -> (Arc<dyn AgentTool>, PendingSchedulerActor) {
    let (cmd_tx, cmd_rx) = mpsc::channel(SCHEDULER_COMMAND_CAPACITY);
    let handle = SchedulerHandle::new(cmd_tx);
    let tool: Arc<dyn AgentTool> = Arc::new(DelayTool::new(handle));
    (tool, PendingSchedulerActor { cmd_rx })
}

pub fn create_scheduler_tools() -> (Vec<Arc<dyn AgentTool>>, PendingSchedulerActor) {
    let (cmd_tx, cmd_rx) = mpsc::channel(SCHEDULER_COMMAND_CAPACITY);
    let handle = SchedulerHandle::new(cmd_tx);
    let tools: Vec<Arc<dyn AgentTool>> = vec![
        Arc::new(DelayTool::new(handle.clone())),
        Arc::new(ScheduleTool::new(handle.clone())),
        Arc::new(ListScheduledTasksTool::new(handle.clone())),
        Arc::new(CancelScheduledTaskTool::new(handle)),
    ];
    (tools, PendingSchedulerActor { cmd_rx })
}

pub struct PendingSchedulerActor {
    cmd_rx: mpsc::Receiver<ScheduleCommand>,
}

impl PendingSchedulerActor {
    pub fn spawn(
        self,
        sq_tx: mpsc::Sender<SessionOp>,
        session_generation: u64,
        cancel_token: CancellationToken,
    ) -> JoinHandle<()> {
        let actor = SchedulerActor::new(self.cmd_rx, sq_tx, session_generation, cancel_token);
        tokio::spawn(async move { actor.run().await })
    }
}

impl fmt::Debug for PendingSchedulerActor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingSchedulerActor")
            .field("cmd_rx", &"mpsc::Receiver<ScheduleCommand>")
            .finish()
    }
}

pub(crate) struct DelayTool {
    handle: SchedulerHandle,
}

impl DelayTool {
    pub(crate) fn new(handle: SchedulerHandle) -> Self {
        Self { handle }
    }
}

#[async_trait]
impl AgentTool for DelayTool {
    fn name(&self) -> &str {
        "delay"
    }

    fn description(&self) -> &str {
        "Schedule a one-shot delayed follow-up message. Delivery transfers only after the Session Actor durably accepts the exact scheduled fire. Session-scoped; minimum 1 second, maximum 86400 seconds."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {"type": "string", "description": "Follow-up message."},
                "delay_secs": {
                    "type": "integer",
                    "minimum": MIN_DELAY_SECS,
                    "maximum": MAX_DELAY_SECS
                }
            },
            "required": ["message", "delay_secs"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> ToolResult {
        let message = match input.get("message").and_then(|value| value.as_str()) {
            Some(message) if !message.is_empty() => message.to_owned(),
            _ => return ToolResult::error("missing or empty 'message' field"),
        };
        let delay_secs = match input.get("delay_secs").and_then(|value| value.as_u64()) {
            Some(delay_secs) => delay_secs,
            None => {
                return ToolResult::error(
                    "missing or invalid 'delay_secs' field (expected a positive integer)",
                );
            }
        };
        if let Err(reason) = validate_delay_secs(delay_secs) {
            return ToolResult::error(reason);
        }

        let (response_tx, response_rx) = oneshot::channel();
        if let Err(error) = self
            .handle
            .send(ScheduleCommand::RegisterOneShot {
                id: None,
                message,
                delay: Duration::from_secs(delay_secs),
                response_tx,
            })
            .await
        {
            return scheduler_send_error(error);
        }
        registration_result(response_rx.await, format!("Delay: {delay_secs} second(s)"))
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Execute
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }
}

pub(crate) struct ScheduleTool {
    handle: SchedulerHandle,
}

impl ScheduleTool {
    pub(crate) fn new(handle: SchedulerHandle) -> Self {
        Self { handle }
    }
}

#[async_trait]
impl AgentTool for ScheduleTool {
    fn name(&self) -> &str {
        "schedule"
    }

    fn description(&self) -> &str {
        "Schedule a recurring follow-up. Each fire uses a stable structured identity and the next interval does not begin until Actor durable acceptance. Session-scoped; interval 5 to 3600 seconds."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {"type": "string", "description": "Recurring follow-up message."},
                "interval_secs": {
                    "type": "integer",
                    "minimum": MIN_INTERVAL_SECS,
                    "maximum": MAX_INTERVAL_SECS
                }
            },
            "required": ["message", "interval_secs"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> ToolResult {
        let message = match input.get("message").and_then(|value| value.as_str()) {
            Some(message) if !message.is_empty() => message.to_owned(),
            _ => return ToolResult::error("missing or empty 'message' field"),
        };
        let interval_secs = match input.get("interval_secs").and_then(|value| value.as_u64()) {
            Some(interval_secs) => interval_secs,
            None => {
                return ToolResult::error(
                    "missing or invalid 'interval_secs' field (expected a positive integer)",
                );
            }
        };
        if let Err(reason) = validate_interval_secs(interval_secs) {
            return ToolResult::error(reason);
        }

        let (response_tx, response_rx) = oneshot::channel();
        if let Err(error) = self
            .handle
            .send(ScheduleCommand::RegisterRecurring {
                id: None,
                message,
                interval: Duration::from_secs(interval_secs),
                response_tx,
            })
            .await
        {
            return scheduler_send_error(error);
        }
        registration_result(
            response_rx.await,
            format!("Interval: {interval_secs} second(s)"),
        )
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Execute
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }
}

pub(crate) struct ListScheduledTasksTool {
    handle: SchedulerHandle,
}

impl ListScheduledTasksTool {
    pub(crate) fn new(handle: SchedulerHandle) -> Self {
        Self { handle }
    }
}

#[async_trait]
impl AgentTool for ListScheduledTasksTool {
    fn name(&self) -> &str {
        "list_scheduled_tasks"
    }

    fn description(&self) -> &str {
        "List active scheduled tasks, including whether a due fire is blocked awaiting durable Actor custody."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }

    async fn execute(&self, _input: serde_json::Value) -> ToolResult {
        let (response_tx, response_rx) = oneshot::channel();
        if let Err(error) = self
            .handle
            .send(ScheduleCommand::List { response_tx })
            .await
        {
            return scheduler_send_error(error);
        }
        match response_rx.await {
            Ok(tasks) if tasks.is_empty() => ToolResult::success("No active scheduled tasks."),
            Ok(tasks) => {
                const MAX_DISPLAY: usize = 20;
                let total = tasks.len();
                let mut text = format!("{total} active task(s):\n");
                for info in tasks.iter().take(MAX_DISPLAY) {
                    text.push_str(&format!(
                        "  {} | {} | next: {}s | {}\n",
                        info.id,
                        info.kind,
                        info.remaining().as_secs(),
                        info.delivery_state(),
                    ));
                }
                let omitted = total.saturating_sub(MAX_DISPLAY);
                if omitted > 0 {
                    text.push_str(&format!("... and {omitted} more task(s) not shown\n"));
                }
                ToolResult::success(text.trim_end().to_owned())
            }
            Err(_) => ToolResult::error("scheduler dropped the request"),
        }
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Read
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }
}

pub(crate) struct CancelScheduledTaskTool {
    handle: SchedulerHandle,
}

impl CancelScheduledTaskTool {
    pub(crate) fn new(handle: SchedulerHandle) -> Self {
        Self { handle }
    }
}

#[async_trait]
impl AgentTool for CancelScheduledTaskTool {
    fn name(&self) -> &str {
        "cancel_scheduled_task"
    }

    fn description(&self) -> &str {
        "Cancel an active scheduled follow-up by task ID."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {"task_id": {"type": "string"}},
            "required": ["task_id"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> ToolResult {
        let task_id = match input.get("task_id").and_then(|value| value.as_str()) {
            Some(task_id) if !task_id.is_empty() => task_id.to_owned(),
            _ => return ToolResult::error("missing or empty 'task_id' field"),
        };
        let (response_tx, response_rx) = oneshot::channel();
        if let Err(error) = self
            .handle
            .send(ScheduleCommand::Cancel {
                id: task_id.clone(),
                response_tx,
            })
            .await
        {
            return scheduler_send_error(error);
        }
        match response_rx.await {
            Ok(CancelResult::Cancelled) => {
                ToolResult::success(format!("Task {task_id} cancelled."))
            }
            Ok(CancelResult::NotFound) => {
                ToolResult::success(format!("Task {task_id} not found or already completed."))
            }
            Err(_) => ToolResult::error("scheduler dropped the request"),
        }
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Execute
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::Extension
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["task_id"]
    }
}

fn scheduler_send_error(error: SchedulerSendError) -> ToolResult {
    match error {
        SchedulerSendError::Full => ToolResult::error("scheduler is busy; try again"),
        SchedulerSendError::Closed => ToolResult::error("scheduler is not available"),
    }
}

fn registration_result(
    result: Result<ScheduleRegistrationResult, oneshot::error::RecvError>,
    timing: String,
) -> ToolResult {
    match result {
        Ok(ScheduleRegistrationResult::Registered { task_id }) => ToolResult::success(format!(
            "Scheduled follow-up registered.\nTask ID: {task_id}\n{timing}"
        )),
        Ok(ScheduleRegistrationResult::InvalidDuration { reason })
        | Ok(ScheduleRegistrationResult::InvalidMessage { reason }) => ToolResult::error(reason),
        Err(_) => ToolResult::error("scheduler dropped the request"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::session::{PendingSubmissionState, SubmissionRejectionReason};

    async fn yield_times(times: usize) {
        for _ in 0..times {
            tokio::task::yield_now().await;
        }
    }

    async fn advance_delivery_retry() {
        // `advance` only moves timers that already exist. Give the delivery
        // future time to arm its receipt timeout, move that timeout, then let
        // it arm the separate retry sleep before moving the clock again.
        yield_times(10).await;
        tokio::time::advance(DELIVERY_RECEIPT_TIMEOUT).await;
        yield_times(10).await;
        tokio::time::advance(DELIVERY_RETRY_DELAY).await;
        yield_times(20).await;
    }

    async fn register_one_shot(handle: &SchedulerHandle, id: &str, delay: Duration) {
        let (response_tx, response_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterOneShot {
                id: Some(id.to_owned()),
                message: "check the build".into(),
                delay,
                response_tx,
            })
            .await
            .expect("operation should succeed");
        assert!(matches!(
            response_rx.await.expect("operation should succeed"),
            ScheduleRegistrationResult::Registered { .. }
        ));
    }

    fn split_tracked_operation(
        operation: SessionOp,
    ) -> (
        bool,
        StructuredSubmission,
        mpsc::UnboundedSender<SubmissionReceipt>,
    ) {
        match operation {
            SessionOp::SubmitStructuredTracked {
                submission,
                receipt_tx: Some(receipt_tx),
            } => (true, submission, receipt_tx),
            SessionOp::ReconcileStructuredTracked {
                submission,
                receipt_tx: Some(receipt_tx),
            } => (false, submission, receipt_tx),
            other => panic!("expected tracked structured operation, got {other:?}"),
        }
    }

    fn accept(
        submission: &StructuredSubmission,
        receipt_tx: &mpsc::UnboundedSender<SubmissionReceipt>,
        disposition: SubmissionReceiptDisposition,
    ) {
        receipt_tx
            .send(SubmissionReceipt {
                session_id: "session-test".into(),
                session_generation: submission.sender_generation,
                submission_id: submission.id.clone(),
                reservation_id: submission.id.clone(),
                receipt_id: "receipt-test".into(),
                source: submission.source,
                item_count: submission.items.len(),
                total_text_bytes: submission.total_text_bytes(),
                disposition,
            })
            .expect("operation should succeed");
    }

    async fn list(handle: &SchedulerHandle) -> Vec<ScheduledTaskInfo> {
        let (response_tx, response_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::List { response_tx })
            .await
            .expect("operation should succeed");
        response_rx.await.expect("operation should succeed")
    }

    #[test]
    fn validates_bounds_and_label() {
        assert!(validate_delay_secs(MIN_DELAY_SECS).is_ok());
        assert!(validate_delay_secs(0).is_err());
        assert!(validate_interval_secs(MIN_INTERVAL_SECS).is_ok());
        assert!(validate_interval_secs(MIN_INTERVAL_SECS - 1).is_err());
        assert_eq!(
            label_scheduled_message("check"),
            "[scheduled-followup] check"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn scheduled_fire_uses_the_bound_actor_generation() {
        let (sq_tx, mut sq_rx) = mpsc::channel(8);
        let (handle, _join) = spawn_scheduler_actor(sq_tx, 7, CancellationToken::new());
        register_one_shot(&handle, "generation-bound", Duration::from_secs(1)).await;

        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;
        let (is_submit, submission, receipt_tx) =
            split_tracked_operation(sq_rx.try_recv().expect("operation should succeed"));
        assert!(is_submit);
        assert_eq!(submission.source, SubmissionSource::Scheduler);
        assert_eq!(submission.sender_generation, 7);

        accept(
            &submission,
            &receipt_tx,
            SubmissionReceiptDisposition::AcceptedPending,
        );
        yield_times(10).await;
        assert!(list(&handle).await.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn one_shot_transfers_only_after_durable_receipt() {
        let (sq_tx, mut sq_rx) = mpsc::channel(8);
        let (handle, _join) = spawn_scheduler_actor(sq_tx, 0, CancellationToken::new());
        register_one_shot(&handle, "one", Duration::from_secs(1)).await;

        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;
        let (is_submit, submission, receipt_tx) =
            split_tracked_operation(sq_rx.try_recv().expect("operation should succeed"));
        assert!(is_submit);
        assert_eq!(submission.source, SubmissionSource::Scheduler);
        assert!(
            submission.items[0]
                .text
                .starts_with(SCHEDULED_FOLLOWUP_LABEL)
        );
        assert_eq!(list(&handle).await.len(), 1);

        accept(
            &submission,
            &receipt_tx,
            SubmissionReceiptDisposition::AcceptedPending,
        );
        yield_times(10).await;
        assert!(list(&handle).await.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn lost_ack_reconciles_the_exact_fire_identity() {
        let (sq_tx, mut sq_rx) = mpsc::channel(8);
        let (handle, _join) = spawn_scheduler_actor(sq_tx, 0, CancellationToken::new());
        register_one_shot(&handle, "lost-ack", Duration::from_secs(1)).await;

        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;
        let (_, submitted, _lost_receipt_tx) =
            split_tracked_operation(sq_rx.try_recv().expect("operation should succeed"));

        advance_delivery_retry().await;
        let (is_submit, reconciled, receipt_tx) =
            split_tracked_operation(sq_rx.try_recv().expect("operation should succeed"));
        assert!(!is_submit);
        assert_eq!(reconciled, submitted);

        accept(
            &reconciled,
            &receipt_tx,
            SubmissionReceiptDisposition::AlreadyAccepted {
                state: PendingSubmissionState::Running,
                turn_id: Some("turn-1".into()),
            },
        );
        yield_times(10).await;
        assert!(list(&handle).await.is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn closed_session_queue_retains_a_blocked_fire() {
        let (sq_tx, sq_rx) = mpsc::channel(1);
        let (handle, _join) = spawn_scheduler_actor(sq_tx, 0, CancellationToken::new());
        drop(sq_rx);
        register_one_shot(&handle, "closed", Duration::from_secs(1)).await;

        tokio::time::advance(Duration::from_secs(5)).await;
        yield_times(20).await;
        let tasks = list(&handle).await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].delivery_state(), "delivery-blocked");
    }

    #[tokio::test(start_paused = true)]
    async fn rejection_retries_without_changing_identity() {
        let (sq_tx, mut sq_rx) = mpsc::channel(8);
        let (handle, _join) = spawn_scheduler_actor(sq_tx, 0, CancellationToken::new());
        register_one_shot(&handle, "rejected", Duration::from_secs(1)).await;

        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;
        let (_, first, receipt_tx) =
            split_tracked_operation(sq_rx.try_recv().expect("operation should succeed"));
        accept(
            &first,
            &receipt_tx,
            SubmissionReceiptDisposition::Rejected {
                reason: SubmissionRejectionReason::LimitExceeded,
            },
        );

        yield_times(10).await;
        tokio::time::advance(DELIVERY_RETRY_DELAY).await;
        yield_times(20).await;
        let (is_submit, retry, _) =
            split_tracked_operation(sq_rx.try_recv().expect("operation should succeed"));
        assert!(is_submit);
        assert_eq!(retry, first);
        assert_eq!(list(&handle).await.len(), 1);
    }

    #[tokio::test(start_paused = true)]
    async fn recurring_does_not_replace_an_unresolved_fire() {
        let (sq_tx, mut sq_rx) = mpsc::channel(32);
        let (handle, _join) = spawn_scheduler_actor(sq_tx, 0, CancellationToken::new());
        let (response_tx, response_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::RegisterRecurring {
                id: Some("recurring".into()),
                message: "tick".into(),
                interval: Duration::from_secs(5),
                response_tx,
            })
            .await
            .expect("operation should succeed");
        assert!(response_rx.await.is_ok());

        tokio::time::advance(Duration::from_secs(6)).await;
        yield_times(10).await;
        let (_, first, _) =
            split_tracked_operation(sq_rx.try_recv().expect("operation should succeed"));

        advance_delivery_retry().await;
        let (is_submit, retry, receipt_tx) =
            split_tracked_operation(sq_rx.try_recv().expect("operation should succeed"));
        assert!(!is_submit);
        assert_eq!(
            retry.id, first.id,
            "later interval replaced unresolved fire"
        );
        assert!(sq_rx.try_recv().is_err());

        accept(
            &retry,
            &receipt_tx,
            SubmissionReceiptDisposition::AcceptedPending,
        );
        yield_times(20).await;

        tokio::time::advance(Duration::from_secs(4)).await;
        yield_times(10).await;
        assert!(sq_rx.try_recv().is_err());
        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;
        let (_, second, _) =
            split_tracked_operation(sq_rx.try_recv().expect("operation should succeed"));
        assert_ne!(second.id, first.id);
        assert!(second.id.ends_with(":fire:2"));
    }

    #[tokio::test(start_paused = true)]
    async fn cancellation_aborts_an_unaccepted_fire() {
        let (sq_tx, mut sq_rx) = mpsc::channel(8);
        let (handle, _join) = spawn_scheduler_actor(sq_tx, 0, CancellationToken::new());
        register_one_shot(&handle, "cancel", Duration::from_secs(1)).await;
        tokio::time::advance(Duration::from_secs(2)).await;
        yield_times(10).await;
        let _ = sq_rx.try_recv().expect("operation should succeed");

        let (response_tx, response_rx) = oneshot::channel();
        handle
            .send(ScheduleCommand::Cancel {
                id: "cancel".into(),
                response_tx,
            })
            .await
            .expect("operation should succeed");
        assert!(matches!(
            response_rx.await.expect("operation should succeed"),
            CancelResult::Cancelled
        ));
        tokio::time::advance(Duration::from_secs(10)).await;
        yield_times(20).await;
        assert!(sq_rx.try_recv().is_err());
    }

    #[test]
    fn tool_natures_remain_permission_safe() {
        let (tools, _pending) = create_scheduler_tools();
        assert_eq!(tools[0].nature(), ToolNature::Execute);
        assert_eq!(tools[1].nature(), ToolNature::Execute);
        assert_eq!(tools[2].nature(), ToolNature::Read);
        assert_eq!(tools[3].nature(), ToolNature::Execute);
    }
}
