# Unified Event Stream Architecture

**Status**: Proposal  
**Priority**: 🔴 **CRITICAL** — foundational architecture for session management  
**Created**: 2026-06-02  
**Author**: System  
**Estimated effort**: 7-10 days  
**Dependencies**: Session contamination fix (session-context-contamination.md)

## Problem Statement

Talos currently has **four separate type systems** for managing conversation flow:

1. **Message** (talos-core) — User/Assistant/Tool messages
2. **AgentEvent** (talos-core) — Streaming events (TurnStart, TextDelta, etc.)
3. **HookEvent** (talos-plugin) — 20 lifecycle events for hooks
4. **SessionEntry** (talos-session) — Persistent storage format

This fragmentation causes:
- **Context loss**: TUI mode doesn't load conversation history
- **Inconsistent state**: Different components see different views of the same turn
- **Complex wiring**: Manual event forwarding between systems
- **No unified timeline**: Cannot reconstruct "what happened when"
- **Hallucination risk**: LLM fabricates memories when context is missing

## Solution: Unified Event Stream

Design a **single, ordered event stream** that all information flows through, with:
- **One canonical event type** that encompasses all information
- **Ordered delivery** with monotonically increasing sequence numbers
- **Multiple consumers** with filtering capabilities
- **Persistent storage** as a first-class concern
- **Replay capability** for context reconstruction

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Event Producers                         │
├─────────────────────────────────────────────────────────────┤
│  User Input  │  Model Output  │  Tool Calls  │  System      │
│  (TUI/CLI)   │  (Provider)    │  (Executor)  │  (Hooks)     │
└──────┬───────┴───────┬────────┴──────┬───────┴──────┬───────┘
       │               │               │              │
       └───────────────┴───────────────┴──────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │   Event Stream      │
                    │   (Ordered Queue)   │
                    │                     │
                    │  ┌───────────────┐  │
                    │  │ Event {       │  │
                    │  │   id: u64,    │  │
                    │  │   turn: u64,  │  │
                    │  │   kind: Kind, │  │
                    │  │   data: Data, │  │
                    │  │   ts: i64,    │  │
                    │  │ }             │  │
                    │  └───────────────┘  │
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
              ▼                ▼                ▼
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │   Session    │  │     TUI      │  │    Agent     │
    │   Storage    │  │   Display    │  │   Context    │
    │              │  │              │  │              │
    │ - Persist    │  │ - Render     │  │ - Build      │
    │ - Index      │  │ - Stream     │  │   messages   │
    │ - Query      │  │ - Update     │  │ - Filter     │
    └──────────────┘  └──────────────┘  └──────────────┘
```

## Core Design

### 1. Unified Event Type

```rust
/// A single event in the conversation stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Monotonically increasing sequence number (global ordering).
    pub id: u64,
    
    /// Turn this event belongs to (for grouping).
    pub turn_id: u64,
    
    /// When this event occurred.
    pub timestamp: i64,
    
    /// What kind of event this is.
    pub kind: EventKind,
    
    /// Event-specific data.
    pub data: EventData,
    
    /// Optional metadata (model name, token count, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<EventMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    // Turn lifecycle
    TurnStart,
    TurnEnd,
    
    // User interaction
    UserMessage,
    
    // Model output
    TextDelta,
    TextComplete,
    
    // Tool execution
    ToolCallRequested,
    ToolCallApproved,
    ToolCallDenied,
    ToolCallStarted,
    ToolCallCompleted,
    
    // System events
    SystemPrompt,
    ContextLoaded,
    Error,
    Warning,
    
    // Hook events (subset)
    HookTriggered(String),  // hook name
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    /// No data (lifecycle events).
    None,
    
    /// Text content (messages, deltas).
    Text(String),
    
    /// Tool call details.
    ToolCall {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    
    /// Tool result.
    ToolResult {
        call_id: String,
        output: String,
        is_error: bool,
    },
    
    /// Error information.
    Error {
        code: String,
        message: String,
        recoverable: bool,
    },
    
    /// Arbitrary metadata.
    Metadata(serde_json::Value),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<TokenUsage>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
}
```

### 2. Event Stream (Ordered Queue)

```rust
/// Ordered stream of events with multiple consumers.
pub struct EventStream {
    /// Next sequence number to assign.
    next_id: AtomicU64,
    
    /// All events in order (for replay).
    events: Vec<Event>,
    
    /// Active subscribers.
    subscribers: Vec<EventSubscriber>,
    
    /// Persistence backend.
    storage: Box<dyn EventStorage>,
}

impl EventStream {
    /// Emit a new event to the stream.
    pub fn emit(&mut self, turn_id: u64, kind: EventKind, data: EventData) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let event = Event {
            id,
            turn_id,
            timestamp: chrono::Utc::now().timestamp_millis(),
            kind,
            data,
            metadata: None,
        };
        
        // Persist first (crash safety).
        self.storage.append(&event);
        
        // Store for replay.
        self.events.push(event.clone());
        
        // Notify subscribers.
        for sub in &mut self.subscribers {
            if sub.filter.matches(&event) {
                sub.sender.send(event.clone());
            }
        }
        
        id
    }
    
    /// Subscribe to events with a filter.
    pub fn subscribe(&mut self, filter: EventFilter) -> EventReceiver {
        let (sender, receiver) = mpsc::unbounded_channel();
        self.subscribers.push(EventSubscriber { filter, sender });
        receiver
    }
    
    /// Replay events from a starting ID.
    pub fn replay(&self, from_id: u64) -> impl Iterator<Item = &Event> {
        self.events.iter().filter(move |e| e.id >= from_id)
    }
    
    /// Get all events for a specific turn.
    pub fn turn_events(&self, turn_id: u64) -> Vec<&Event> {
        self.events.iter().filter(|e| e.turn_id == turn_id).collect()
    }
}
```

### 3. Event Filters

```rust
/// Filter for event subscriptions.
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Only events of these kinds.
    pub kinds: Option<Vec<EventKind>>,
    
    /// Only events from these turns.
    pub turns: Option<Vec<u64>>,
    
    /// Only events after this ID.
    pub after_id: Option<u64>,
    
    /// Exclude these kinds.
    pub exclude_kinds: Option<Vec<EventKind>>,
}

impl EventFilter {
    pub fn matches(&self, event: &Event) -> bool {
        if let Some(ref kinds) = self.kinds {
            if !kinds.contains(&event.kind) {
                return false;
            }
        }
        
        if let Some(ref turns) = self.turns {
            if !turns.contains(&event.turn_id) {
                return false;
            }
        }
        
        if let Some(after) = self.after_id {
            if event.id <= after {
                return false;
            }
        }
        
        if let Some(ref exclude) = self.exclude_kinds {
            if exclude.contains(&event.kind) {
                return false;
            }
        }
        
        true
    }
}
```

### 4. Storage Backend

```rust
/// Trait for event persistence.
pub trait EventStorage: Send + Sync {
    /// Append an event to storage.
    fn append(&mut self, event: &Event);
    
    /// Load events from storage.
    fn load(&self) -> Vec<Event>;
    
    /// Load events for a specific turn.
    fn load_turn(&self, turn_id: u64) -> Vec<Event>;
    
    /// Search events by content.
    fn search(&self, query: &str) -> Vec<Event>;
}

/// JSONL file storage (current session format).
pub struct JsonlStorage {
    path: PathBuf,
}

impl EventStorage for JsonlStorage {
    fn append(&mut self, event: &Event) {
        let line = serde_json::to_string(event).unwrap();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .unwrap();
        writeln!(file, "{}", line).unwrap();
    }
    
    fn load(&self) -> Vec<Event> {
        let file = File::open(&self.path).unwrap();
        let reader = BufReader::new(file);
        reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| serde_json::from_str(&line).ok())
            .collect()
    }
    
    // ... other methods
}
```

## Integration Points

### 1. User Input (TUI/CLI)

```rust
// In TUI mode, when user presses Enter:
fn on_user_input(&mut self, text: String) {
    let turn_id = self.current_turn_id;
    
    self.event_stream.emit(
        turn_id,
        EventKind::UserMessage,
        EventData::Text(text),
    );
}
```

### 2. Model Output (Provider)

```rust
// In Agent::run_inner, when receiving streaming response:
while let Some(event) = rx.recv().await {
    match event {
        AgentEvent::TextDelta { delta } => {
            self.event_stream.emit(
                turn_id,
                EventKind::TextDelta,
                EventData::Text(delta),
            );
        }
        AgentEvent::TurnEnd { stop_reason, usage } => {
            self.event_stream.emit(
                turn_id,
                EventKind::TurnEnd,
                EventData::Metadata(serde_json::json!({
                    "stop_reason": stop_reason,
                    "usage": usage,
                })),
            );
        }
        // ... other events
    }
}
```

### 3. Tool Execution

```rust
// When tool call is requested:
let call_id = self.event_stream.emit(
    turn_id,
    EventKind::ToolCallRequested,
    EventData::ToolCall {
        id: tool_call.id.clone(),
        name: tool_call.name.clone(),
        input: tool_call.input.clone(),
    },
);

// When tool call is approved:
self.event_stream.emit(
    turn_id,
    EventKind::ToolCallApproved,
    EventData::Metadata(serde_json::json!({
        "call_id": tool_call.id,
    })),
);

// When tool execution completes:
self.event_stream.emit(
    turn_id,
    EventKind::ToolCallCompleted,
    EventData::ToolResult {
        call_id: tool_call.id,
        output: result.content,
        is_error: result.is_error,
    },
);
```

### 4. Agent Context Building

```rust
// When building context for the next turn:
fn build_context(&self, turn_id: u64) -> Vec<Message> {
    let mut messages = Vec::new();
    
    // Load all events from previous turns.
    for event in self.event_stream.replay(0) {
        if event.turn_id >= turn_id {
            break;  // Stop at current turn.
        }
        
        match event.kind {
            EventKind::UserMessage => {
                if let EventData::Text(text) = &event.data {
                    messages.push(Message::User { content: text.clone() });
                }
            }
            EventKind::TextComplete => {
                if let EventData::Text(text) = &event.data {
                    messages.push(Message::Assistant {
                        content: text.clone(),
                        tool_calls: vec![],
                    });
                }
            }
            EventKind::ToolCallRequested => {
                if let EventData::ToolCall { id, name, input } = &event.data {
                    // Add to last assistant message's tool_calls.
                    if let Some(Message::Assistant { tool_calls, .. }) = messages.last_mut() {
                        tool_calls.push(ToolCall {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                        });
                    }
                }
            }
            EventKind::ToolCallCompleted => {
                if let EventData::ToolResult { call_id, output, is_error } = &event.data {
                    messages.push(Message::Tool {
                        result: ToolResult {
                            tool_use_id: call_id.clone(),
                            content: output.clone(),
                            is_error: *is_error,
                        },
                    });
                }
            }
            _ => {}  // Ignore other events.
        }
    }
    
    messages
}
```

### 5. TUI Display

```rust
// Subscribe to events for rendering:
let filter = EventFilter {
    kinds: Some(vec![
        EventKind::UserMessage,
        EventKind::TextDelta,
        EventKind::ToolCallRequested,
        EventKind::ToolCallCompleted,
    ]),
    turns: None,
    after_id: None,
    exclude_kinds: None,
};

let mut event_rx = event_stream.subscribe(filter);

// In TUI event loop:
while let Some(event) = event_rx.recv().await {
    match event.kind {
        EventKind::UserMessage => {
            if let EventData::Text(text) = &event.data {
                self.append_user_message(text);
            }
        }
        EventKind::TextDelta => {
            if let EventData::Text(delta) = &event.data {
                self.append_text_delta(delta);
            }
        }
        // ... other events
    }
}
```

## Migration Strategy

### Phase 1: Add Event Stream (2-3 days)

1. Create `talos-event-stream` crate with core types
2. Implement `EventStream`, `EventFilter`, `JsonlStorage`
3. Add unit tests for event ordering and filtering

### Phase 2: Integrate with Agent (2-3 days)

1. Add `EventStream` to `Agent` struct
2. Emit events in `run_inner` for all AgentEvents
3. Build context from event stream instead of manual message tracking
4. Update `run_streaming` to use event stream

### Phase 3: Integrate with TUI (1-2 days)

1. Subscribe TUI to event stream with appropriate filter
2. Replace manual event handling with event stream subscription
3. Add session management (create/resume sessions)

### Phase 4: Integrate with Session Storage (1-2 days)

1. Replace `SessionEntry` with `Event` in JSONL files
2. Update `SessionManager` to use event stream
3. Add backward compatibility for old JSONL format

### Phase 5: Integrate with Hooks (1 day)

1. Emit `HookTriggered` events for hook lifecycle
2. Allow hooks to subscribe to event stream
3. Update `HookRegistry` to use event stream

## Concurrent Event Sources & Queueing

### Problem

Multiple sources can emit events simultaneously:
- **User typing** while model is streaming
- **Parallel tool calls** (batch execution)
- **Hook events** firing during provider calls
- **System events** (context loading, errors)

Without proper queueing, events can:
- Arrive out of order (confusing UI)
- Be lost (race conditions)
- Cause inconsistent state (partial updates)

### Solution: Priority-Based Event Queue

```rust
/// Priority levels for event queueing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    /// System-critical events (errors, turn lifecycle).
    Critical = 0,
    
    /// User-initiated events (messages, interrupts).
    User = 1,
    
    /// Model output (text deltas, tool calls).
    Model = 2,
    
    /// Tool execution (results, progress).
    Tool = 3,
    
    /// Informational events (hooks, metadata).
    Info = 4,
}

/// Queued event waiting to be processed.
#[derive(Debug, Clone)]
pub struct QueuedEvent {
    pub priority: EventPriority,
    pub event: Event,
    pub enqueue_time: i64,
}

/// Thread-safe event queue with priority ordering.
pub struct EventQueue {
    /// Priority queue (min-heap by priority, then by enqueue time).
    queue: BinaryHeap<Reverse<QueuedEvent>>,
    
    /// Maximum queue size (backpressure).
    max_size: usize,
    
    /// Current queue size.
    current_size: AtomicUsize,
    
    /// Event stream to emit to when dequeued.
    event_stream: Arc<Mutex<EventStream>>,
}

impl EventQueue {
    /// Enqueue an event with priority.
    ///
    /// Returns `Err` if queue is full (backpressure).
    pub fn enqueue(
        &self,
        priority: EventPriority,
        turn_id: u64,
        kind: EventKind,
        data: EventData,
    ) -> Result<(), QueueFullError> {
        let current = self.current_size.load(Ordering::SeqCst);
        if current >= self.max_size {
            return Err(QueueFullError { current, max: self.max_size });
        }
        
        let event = Event {
            id: 0,  // Will be assigned by EventStream.
            turn_id,
            timestamp: chrono::Utc::now().timestamp_millis(),
            kind,
            data,
            metadata: None,
        };
        
        let queued = QueuedEvent {
            priority,
            event,
            enqueue_time: chrono::Utc::now().timestamp_millis(),
        };
        
        self.queue.lock().unwrap().push(Reverse(queued));
        self.current_size.fetch_add(1, Ordering::SeqCst);
        
        Ok(())
    }
    
    /// Process all queued events in priority order.
    pub fn drain(&self) {
        let mut queue = self.queue.lock().unwrap();
        let mut stream = self.event_stream.lock().unwrap();
        
        while let Some(Reverse(queued)) = queue.pop() {
            let id = stream.emit(
                queued.event.turn_id,
                queued.event.kind,
                queued.event.data,
            );
            self.current_size.fetch_sub(1, Ordering::SeqCst);
        }
    }
}
```

### Queueing Strategies

#### 1. FIFO Within Priority

Events of the same priority are processed in arrival order:

```rust
impl Ord for QueuedEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // First by priority (lower = higher priority).
        self.priority.cmp(&other.priority)
            // Then by enqueue time (earlier = first).
            .then(self.enqueue_time.cmp(&other.enqueue_time))
    }
}
```

#### 2. Turn-Based Ordering

Events from the same turn are grouped together:

```rust
/// Drain events for a specific turn.
pub fn drain_turn(&self, turn_id: u64) {
    let mut queue = self.queue.lock().unwrap();
    let mut stream = self.event_stream.lock().unwrap();
    
    let mut turn_events = Vec::new();
    
    // Extract all events for this turn.
    queue.retain(|Reverse(queued)| {
        if queued.event.turn_id == turn_id {
            turn_events.push(queued.clone());
            false  // Remove from queue.
        } else {
            true  // Keep in queue.
        }
    });
    
    // Sort by priority, then by enqueue time.
    turn_events.sort();
    
    // Emit in order.
    for queued in turn_events {
        stream.emit(
            queued.event.turn_id,
            queued.event.kind,
            queued.event.data,
        );
        self.current_size.fetch_sub(1, Ordering::SeqCst);
    }
}
```

#### 3. Backpressure Handling

When queue is full, producers must wait or drop:

```rust
/// Enqueue with backpressure (blocks if full).
pub async fn enqueue_blocking(
    &self,
    priority: EventPriority,
    turn_id: u64,
    kind: EventKind,
    data: EventData,
) {
    loop {
        match self.enqueue(priority, turn_id, kind.clone(), data.clone()) {
            Ok(()) => return,
            Err(QueueFullError { current, max }) => {
                tracing::warn!(
                    "Event queue full ({}/{}), waiting...",
                    current,
                    max
                );
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}

/// Enqueue with drop-if-full (for low-priority events).
pub fn enqueue_or_drop(
    &self,
    priority: EventPriority,
    turn_id: u64,
    kind: EventKind,
    data: EventData,
) -> bool {
    match self.enqueue(priority, turn_id, kind, data) {
        Ok(()) => true,
        Err(_) => {
            tracing::debug!("Dropped low-priority event due to queue full");
            false
        }
    }
}
```

### Concurrent Scenarios

#### Scenario 1: User Interrupts Model

```rust
// Model is streaming TextDelta events.
// User presses Ctrl+C to interrupt.

// Model emits (priority = Model):
queue.enqueue(EventPriority::Model, turn_id, EventKind::TextDelta, EventData::Text("Hello".into()));

// User interrupt emits (priority = User, higher):
queue.enqueue(EventPriority::User, turn_id, EventKind::UserMessage, EventData::Text("[interrupted]".into()));

// Queue processes User event first, then Model events.
// Result: Interrupt is visible immediately, model output is truncated.
```

#### Scenario 2: Parallel Tool Calls

```rust
// Agent requests 3 tool calls in parallel.
let tool_calls = vec![
    ToolCall { id: "1", name: "read", input: json!({"path": "a.txt"}) },
    ToolCall { id: "2", name: "read", input: json!({"path": "b.txt"}) },
    ToolCall { id: "3", name: "read", input: json!({"path": "c.txt"}) },
];

// Emit all tool call requests (priority = Model).
for call in &tool_calls {
    queue.enqueue(
        EventPriority::Model,
        turn_id,
        EventKind::ToolCallRequested,
        EventData::ToolCall {
            id: call.id.clone(),
            name: call.name.clone(),
            input: call.input.clone(),
        },
    );
}

// Execute tools in parallel.
let results = futures::future::join_all(
    tool_calls.iter().map(|call| execute_tool(call))
).await;

// Emit results as they complete (priority = Tool).
for result in results {
    queue.enqueue(
        EventPriority::Tool,
        turn_id,
        EventKind::ToolCallCompleted,
        EventData::ToolResult {
            call_id: result.call_id,
            output: result.output,
            is_error: result.is_error,
        },
    );
}

// Queue processes all requests first, then results in completion order.
```

#### Scenario 3: Hook Events During Provider Call

```rust
// Provider is streaming response.
// Hooks fire lifecycle events.

// Hook emits (priority = Info, lowest):
queue.enqueue(
    EventPriority::Info,
    turn_id,
    EventKind::HookTriggered("before_provider_call".into()),
    EventData::None,
);

// Model emits (priority = Model, higher):
queue.enqueue(
    EventPriority::Model,
    turn_id,
    EventKind::TextDelta,
    EventData::Text("Response".into()),
);

// Queue processes Model events first, then Info events.
// Result: User sees response immediately, hook events are logged but not blocking.
```

### Queue Configuration

```rust
/// Configuration for event queue.
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Maximum queue size before backpressure.
    pub max_size: usize,
    
    /// Whether to drop low-priority events when full.
    pub drop_low_priority: bool,
    
    /// Drain interval (milliseconds).
    pub drain_interval_ms: u64,
    
    /// Whether to group events by turn.
    pub group_by_turn: bool,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            drop_low_priority: true,
            drain_interval_ms: 10,
            group_by_turn: false,
        }
    }
}
```

### Integration with Event Stream

```rust
/// Event stream with integrated queue.
pub struct EventStreamWithQueue {
    stream: Arc<Mutex<EventStream>>,
    queue: Arc<EventQueue>,
    config: QueueConfig,
}

impl EventStreamWithQueue {
    /// Start background drain task.
    pub fn start_drain_task(&self) -> JoinHandle<()> {
        let queue = self.queue.clone();
        let interval = Duration::from_millis(self.config.drain_interval_ms);
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                queue.drain();
            }
        })
    }
    
    /// Emit event directly (bypasses queue, for critical events).
    pub fn emit_direct(&self, turn_id: u64, kind: EventKind, data: EventData) -> u64 {
        let mut stream = self.stream.lock().unwrap();
        stream.emit(turn_id, kind, data)
    }
    
    /// Emit event through queue (for normal events).
    pub async fn emit_queued(
        &self,
        priority: EventPriority,
        turn_id: u64,
        kind: EventKind,
        data: EventData,
    ) {
        if priority == EventPriority::Critical {
            // Critical events bypass queue.
            self.emit_direct(turn_id, kind, data);
        } else if self.config.drop_low_priority && priority >= EventPriority::Info {
            // Drop low-priority events if queue is full.
            self.queue.enqueue_or_drop(priority, turn_id, kind, data);
        } else {
            // Block for other priorities.
            self.queue.enqueue_blocking(priority, turn_id, kind, data).await;
        }
    }
}
```

## Benefits

1. **Unified timeline**: All events in one ordered stream
2. **Context reconstruction**: Replay events to build conversation history
3. **Flexible consumers**: Each component subscribes to what it needs
4. **Crash safety**: Events persisted before processing
5. **Debugging**: Full event log for troubleshooting
6. **Extensibility**: Easy to add new event types
7. **Performance**: Filtering reduces unnecessary processing

## Trade-offs

1. **Complexity**: More types and abstractions
2. **Memory**: Storing all events in memory (mitigated by streaming from disk)
3. **Migration**: Need to support old JSONL format
4. **Learning curve**: Developers need to understand event stream model

## Success Criteria

1. TUI mode loads conversation history correctly
2. No LLM hallucinations about previous messages
3. All events are persisted and replayable
4. Context building is accurate and complete
5. Performance is acceptable (< 100ms overhead per turn)

## Related Proposals

- [Session Context Contamination](session-context-contamination.md) — This proposal fixes the root cause
- [Provider Plugin Architecture](provider-plugin-architecture.md) — Event stream enables plugin system
- [Reasoning Thinking Field](reasoning-thinking-field.md) — Event stream can capture reasoning traces

## Next Steps

1. Review and refine this proposal
2. Create `talos-event-stream` crate
3. Implement Phase 1 (core types)
4. Prototype integration with Agent
5. Validate with real conversation scenarios
