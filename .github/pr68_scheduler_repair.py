from pathlib import Path
import re

path = Path("crates/talos-agent/src/scheduler.rs")
text = path.read_text()
if "enum TaskFireEvent" in text:
    print("one-shot repair already applied")
    raise SystemExit(0)

text = text.replace(
    "pub(crate) const MAX_INTERVAL_SECS: u64 = 3_600;\n",
    "pub(crate) const MAX_INTERVAL_SECS: u64 = 3_600;\n"
    "const ONE_SHOT_DELIVERY_TIMEOUT: Duration = Duration::from_secs(5);\n",
    1,
)
text = text.replace(
    "    /// When the task is scheduled to fire.\n"
    "    pub fire_at: Instant,\n",
    "    /// When the task is scheduled to fire.\n"
    "    pub fire_at: Instant,\n"
    "    /// Last bounded one-shot delivery failure, retained for inspection.\n"
    "    pub delivery_error: Option<String>,\n",
    1,
)
text = text.replace(
    "struct ActiveTask {\n"
    "    info: ScheduledTaskInfo,\n"
    "    handle: JoinHandle<()>,\n"
    "}\n",
    "struct ActiveTask {\n"
    "    info: ScheduledTaskInfo,\n"
    "    handle: JoinHandle<()>,\n"
    "}\n\n"
    "enum TaskFireEvent {\n"
    "    Delivered(String),\n"
    "    DeliveryFailed { task_id: String, reason: String },\n"
    "}\n",
    1,
)
text = text.replace(
    "    fired_tx: mpsc::UnboundedSender<String>,\n"
    "    fired_rx: mpsc::UnboundedReceiver<String>,\n",
    "    fired_tx: mpsc::UnboundedSender<TaskFireEvent>,\n"
    "    fired_rx: mpsc::UnboundedReceiver<TaskFireEvent>,\n",
    1,
)

old_event = '''                Some(task_id) = self.fired_rx.recv() => {
                    let kind = self.tasks.get(&task_id).map(|t| t.info.kind);
                    match kind {
                        Some(ScheduleKind::OneShot) => {
                            self.tasks.remove(&task_id);
                        }
                        Some(ScheduleKind::Recurring { interval }) => {
                            if let Some(task) = self.tasks.get_mut(&task_id) {
                                task.info.fire_at = Instant::now() + interval;
                            }
                        }
                        None => {}
                    }
                }
'''
new_event = '''                Some(event) = self.fired_rx.recv() => {
                    match event {
                        TaskFireEvent::Delivered(task_id) => {
                            let kind = self.tasks.get(&task_id).map(|task| task.info.kind);
                            match kind {
                                Some(ScheduleKind::OneShot) => {
                                    self.tasks.remove(&task_id);
                                }
                                Some(ScheduleKind::Recurring { interval }) => {
                                    if let Some(task) = self.tasks.get_mut(&task_id) {
                                        task.info.fire_at = Instant::now() + interval;
                                    }
                                }
                                None => {}
                            }
                        }
                        TaskFireEvent::DeliveryFailed { task_id, reason } => {
                            if let Some(task) = self.tasks.get_mut(&task_id) {
                                task.info.delivery_error = Some(reason);
                                task.info.fire_at = Instant::now();
                            }
                        }
                    }
                }
'''
if text.count(old_event) != 1:
    raise SystemExit("expected exactly one scheduler fire event block")
text = text.replace(old_event, new_event, 1)

old_fire = '''        let handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            match sq_tx.try_send(scheduled_submission(&task_id_for_fire, labeled_for_fire)) {
                Ok(()) => {}
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    tracing::warn!(
                        task_id = %task_id_for_fire,
                        "scheduled follow-up dropped: bounded session queue is full"
                    );
                }
                Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                    tracing::debug!(
                        task_id = %task_id_for_fire,
                        "scheduled follow-up fire: session queue closed"
                    );
                }
            }
            let _ = fired_tx.send(task_id_for_fire);
        });
'''
new_fire = '''        let handle = tokio::spawn(async move {
            tokio::time::sleep(delay).await;

            let operation = scheduled_submission(&task_id_for_fire, labeled_for_fire);
            let event = match tokio::time::timeout(
                ONE_SHOT_DELIVERY_TIMEOUT,
                sq_tx.send(operation),
            )
            .await
            {
                Ok(Ok(())) => TaskFireEvent::Delivered(task_id_for_fire),
                Ok(Err(_)) => TaskFireEvent::DeliveryFailed {
                    task_id: task_id_for_fire,
                    reason: "session queue closed before one-shot delivery".to_string(),
                },
                Err(_) => TaskFireEvent::DeliveryFailed {
                    task_id: task_id_for_fire,
                    reason: "timed out waiting for one-shot session queue capacity".to_string(),
                },
            };
            let _ = fired_tx.send(event);
        });
'''
if text.count(old_fire) != 1:
    raise SystemExit("expected exactly one one-shot fire block")
text = text.replace(old_fire, new_fire, 1)

literal = re.compile(r"ScheduledTaskInfo \{(?P<body>.*?)\n(?P<indent>\s*)\}", re.S)
seen = 0

def add_delivery_error(match):
    global seen
    seen += 1
    body = match.group("body")
    indent = match.group("indent")
    if "delivery_error:" in body:
        return match.group(0)
    lines = body.splitlines()
    insert_at = None
    field_indent = None
    for index, line in enumerate(lines):
        if re.match(r"\s*fire_at(?::|,)", line):
            insert_at = index + 1
            field_indent = line[: len(line) - len(line.lstrip())]
    if insert_at is None:
        raise SystemExit("ScheduledTaskInfo literal missing fire_at")
    lines.insert(insert_at, f"{field_indent}delivery_error: None,")
    return "ScheduledTaskInfo {" + "\n".join(lines) + "\n" + indent + "}"

text = literal.sub(add_delivery_error, text)
if seen < 4:
    raise SystemExit(f"expected at least four ScheduledTaskInfo occurrences, saw {seen}")

old_list = '''                for info in tasks.iter().take(display_count) {
                    text.push_str(&format!(
                        "  {} | {} | next: {}s\\n",
                        info.id,
                        info.kind,
                        info.remaining().as_secs(),
                    ));
                }
'''
new_list = '''                for info in tasks.iter().take(display_count) {
                    if let Some(error) = &info.delivery_error {
                        text.push_str(&format!(
                            "  {} | {} | delivery failed: {}\\n",
                            info.id, info.kind, error,
                        ));
                    } else {
                        text.push_str(&format!(
                            "  {} | {} | next: {}s\\n",
                            info.id,
                            info.kind,
                            info.remaining().as_secs(),
                        ));
                    }
                }
'''
if text.count(old_list) != 1:
    raise SystemExit("expected exactly one scheduler list rendering block")
text = text.replace(old_list, new_list, 1)

old_closed = '''        let snapshot = list_rx.await.unwrap();
        assert!(
            snapshot.is_empty(),
            "fired task must be cleaned up even when queue is closed"
        );
'''
new_closed = '''        let snapshot = list_rx.await.unwrap();
        assert_eq!(snapshot.len(), 1, "failed one-shot remains inspectable");
        assert!(
            snapshot[0]
                .delivery_error
                .as_deref()
                .is_some_and(|error| error.contains("queue closed")),
            "closed queue delivery failure must be observable: {snapshot:?}"
        );
'''
if text.count(old_closed) != 1:
    raise SystemExit("expected exactly one closed-queue assertion block")
text = text.replace(old_closed, new_closed, 1)

path.write_text(text)
