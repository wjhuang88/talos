use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use talos_core::message::ToolCall;
use talos_permission::PermissionDecision;
use talos_plugin::{
    HookContext, HookEvent, HookEventKind, HookHandler, HookRegistry, HookResult, TurnId,
};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone, Default)]
struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

impl<'a> MakeWriter<'a> for SharedBuffer {
    type Writer = SharedWriter;

    fn make_writer(&'a self) -> Self::Writer {
        SharedWriter(self.0.clone())
    }
}

struct SharedWriter(Arc<Mutex<Vec<u8>>>);

impl io::Write for SharedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("buffer lock").extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct ModifyPermissionHandler;

#[async_trait]
impl HookHandler for ModifyPermissionHandler {
    fn name(&self) -> &str {
        "modifier"
    }

    fn subscribed(&self) -> &'static [HookEventKind] {
        &[HookEventKind::BeforePermissionCheck]
    }

    async fn on_event(&self, _ctx: &HookContext, _event: &mut HookEvent<'_>) -> HookResult {
        let leaked_call = Box::leak(Box::new(ToolCall {
            id: "call-2".to_string(),
            name: "write".to_string(),
            input: serde_json::json!({"path": "src/lib.rs"}),
        }));
        HookResult::Modify(HookEvent::BeforePermissionCheck { call: leaked_call })
    }
}

#[test]
fn modify_on_permission_boundary_is_dropped_and_logged() {
    let mut registry = HookRegistry::new();
    registry.register(Arc::new(ModifyPermissionHandler));

    let call = ToolCall {
        id: "call-1".to_string(),
        name: "read".to_string(),
        input: serde_json::json!({"path": "Cargo.toml"}),
    };
    let context = HookContext::new(TurnId::new(), PathBuf::from("."));
    let buffer = SharedBuffer::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(buffer.clone())
        .without_time()
        .with_ansi(false)
        .finish();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("runtime");

    tracing::subscriber::with_default(subscriber, || {
        let outcome = runtime.block_on(async {
            registry
                .dispatch(&context, HookEvent::BeforePermissionCheck { call: &call })
                .await
        });

        match outcome.into_event() {
            HookEvent::BeforePermissionCheck { call } => assert_eq!(call.id, "call-1"),
            other => panic!("unexpected event: {other:?}"),
        }
    });

    let logs = String::from_utf8(buffer.0.lock().expect("buffer lock").clone()).expect("utf8 logs");
    assert!(logs.contains("hook modify ignored for permission boundary"));
}

#[test]
fn after_permission_check_remains_read_only() {
    let decision = PermissionDecision::Allow;
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "read".to_string(),
        input: serde_json::json!({"path": "Cargo.toml"}),
    };
    let event = HookEvent::AfterPermissionCheck {
        call: &call,
        decision,
    };
    assert!(event.is_permission_boundary());
}
