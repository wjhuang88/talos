# Feature: Session-Level Todo List for Plan Orchestration

## Problem Statement

当前 Talos session 中，用户缺乏一个结构化的方式来管理和追踪任务计划。在多轮对话的复杂 coding agent 工作流中，需要：

1. **明确的任务分解**：将大型编程任务分解为可管理的 sub-task
2. **实时进度追踪**：在 TUI 中直观展示任务完成状态
3. **任务优先级和依赖**：支持任务之间的逻辑依赖关系
4. **会话持久化**：Todo list 作为 session 的一部分被持久化存储
5. **AI 驱动的编排**：Agent 能够建议、创建和更新 todo list

## Proposed Solution

### 1. 数据模型与存储设计

#### 核心实体 (Rust structs)

```rust
// crates/talos-core/src/todo/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TodoItem {
    pub id: String,  // UUID
    pub title: String,
    pub description: Option<String>,
    pub status: TodoStatus,  // TODO, IN_PROGRESS, COMPLETED, BLOCKED
    pub priority: Priority,  // LOW, MEDIUM, HIGH, CRITICAL
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub dependencies: Vec<String>,  // List of todo item IDs
    pub assigned_to_turn: Option<usize>,  // Which conversation turn created this
    pub tags: Vec<String>,  // e.g., "testing", "refactor", "bug-fix"
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TodoStatus {
    TODO,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoList {
    pub session_id: String,
    pub items: Vec<TodoItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### 数据库表扩展

在现有 SQLite schema 中添加：

```sql
CREATE TABLE IF NOT EXISTS todo_lists (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE TABLE IF NOT EXISTS todo_items (
    id TEXT PRIMARY KEY,
    todo_list_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    priority TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    completed_at TIMESTAMP,
    assigned_to_turn INTEGER,
    tags TEXT,  -- JSON array
    FOREIGN KEY (todo_list_id) REFERENCES todo_lists(id)
);

CREATE TABLE IF NOT EXISTS todo_dependencies (
    parent_id TEXT NOT NULL,
    child_id TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (parent_id, child_id),
    FOREIGN KEY (parent_id) REFERENCES todo_items(id),
    FOREIGN KEY (child_id) REFERENCES todo_items(id)
);
```

### 2. UI 设计

#### TUI 组件架构

```
┌─ Session View ─────────────────────────────────────┐
│                                                     │
│  Todo List Panel  │   Chat Composer               │
│  ─────────────────┼──────────────────────────────  │
│                   │                                │
│  [✓] Task 1      │  > /todo create "refactor"   │
│  [ ] Task 2      │  > /todo list                │
│  [→] Task 3      │                              │
│  [⊘] Task 4      │  Assistant: I'll help you... │
│                   │                              │
│  Legend:         │                              │
│  ✓ = Completed   │                              │
│  [ ] = TODO      │                              │
│  → = In Progress │                              │
│  ⊘ = Blocked     │                              │
│                   │                              │
└─────────────────────────────────────────────────────┘
```

#### TUI 组件实现位置
- 文件：`crates/talos-tui/src/components/todo_panel.rs`
- 集成点：`crates/talos-tui/src/app.rs`（主 layout 中添加 todo panel）

#### UI 交互模式

**1. 列表视图**
- 显示所有 todo 项，用颜色和符号标识状态
- 支持上下滚动
- 实时显示完成进度（如 "5/12 completed"）
- 点击展开显示详细信息

**2. 详情视图**
- 显示单个 todo 的完整信息
- 支持优先级、标签的可视化
- 显示依赖关系（依赖的任务、被依赖的任务）
- 显示创建于哪一轮对话

**3. 编辑视图**
- 支持改变状态（ `[ ] → [→] → [✓]` 或 `[⊘]`）
- 支持改变优先级
- 支持添加/移除标签

### 3. API/Slash 命令设计

#### 新增 Slash 命令

```
/todo list [--filter=status|priority|tags] [--sort=priority|created|completed]
    列出所有 todo 项
    
/todo create "<title>" [--priority=high|medium|low] [--tags="tag1,tag2"]
    创建新的 todo 项
    
/todo update <id> [--status=todo|in_progress|completed|blocked] 
                   [--priority=high|medium|low]
                   [--title="new title"]
    更新 todo 项
    
/todo complete <id>
    标记为完成（简写）
    
/todo block <id> [--reason="why blocked"]
    标记为阻塞状态
    
/todo dependency add <parent-id> <child-id>
    添加依赖关系（parent 必须完成才能做 child）
    
/todo dependency remove <parent-id> <child-id>
    移除依赖关系
    
/todo delete <id>
    删除 todo 项
    
/todo export [--format=json|markdown]
    导出 todo list
```

#### 内部工具（agent 可调用）

```rust
// Available to model for structured planning
{
    "name": "todo_create",
    "description": "Create a new todo item for task planning",
    "input_schema": {
        "title": "string",
        "description": "string (optional)",
        "priority": "enum(low|medium|high|critical)",
        "tags": "array<string>"
    }
}

{
    "name": "todo_update_status",
    "description": "Update the status of a todo item",
    "input_schema": {
        "id": "string",
        "status": "enum(todo|in_progress|completed|blocked)",
        "reason": "string (optional, required for blocked)"
    }
}

{
    "name": "todo_list_query",
    "description": "Query and filter todo items",
    "input_schema": {
        "filter": "enum(all|completed|in_progress|blocked|by_priority|by_tags)",
        "query": "string (optional filter criteria)"
    }
}
```

### 4. 核心功能实现

#### 模块结构

```
crates/
├── talos-core/src/
│   └── todo/
│       ├── mod.rs              # Public API
│       ├── model.rs            # Data structures
│       ├── repository.rs       # Database access
│       ├── service.rs          # Business logic
│       └── validator.rs        # Validation rules
├── talos-agent/src/
│   └── tools/
│       └── todo_tools.rs       # Agent tool definitions
└── talos-tui/src/
    └── components/
        └── todo_panel.rs       # TUI rendering
```

#### 功能实现清单

- [x] Define data models and enums
- [x] Design SQLite schema and migrations
- [ ] Implement TodoRepository (CRUD operations)
- [ ] Implement TodoService (business logic, dependency validation)
- [ ] Implement todo_tools.rs (agent tools)
- [ ] Implement TodoPanel component (TUI)
- [ ] Implement slash command handlers in CommandHandler
- [ ] Add session lifecycle hooks to load/save todo lists
- [ ] Implement todo list exports (JSON, Markdown)
- [ ] Add semantic search for todo items
- [ ] Write comprehensive tests for TodoService

### 5. 集成点

#### Session 生命周期集成

```rust
// In session.rs
pub async fn create_session() -> Session {
    let session = Session::new();
    let todo_list = TodoList::new(session.id.clone());
    // Persist todo_list
    session
}

pub async fn load_session(id: &str) -> Session {
    let session = load_from_db(id);
    let todo_list = load_todo_list(id);
    session.with_todo_list(todo_list)
}
```

#### Agent 提示词集成

在系统提示中添加：
- Todo list 的当前状态
- 允许 agent 使用 todo 工具的权限说明
- 建议 agent 在规划时主动使用 todo 工具

### 6. 存储持久化

#### 保存策略

1. **自动保存**：每次 todo 项变更后立即写入数据库
2. **事务一致性**：确保依赖关系完整性（如删除任务时处理依赖）
3. **导出选项**：支持将 todo list 导出为 Markdown 或 JSON

#### 迁移策略

- 创建新的迁移脚本 `migrations/00X-create-todo-tables.sql`
- 更新 `schema_version` 以触发自动迁移

### 7. 测试用例

```rust
#[cfg(test)]
mod tests {
    // Unit tests for TodoService
    #[test]
    fn test_create_todo_item() { }
    
    #[test]
    fn test_add_dependency_creates_relationship() { }
    
    #[test]
    fn test_cannot_complete_with_pending_dependencies() { }
    
    #[test]
    fn test_circular_dependency_detection() { }
    
    #[test]
    fn test_delete_cascades_properly() { }
    
    // Integration tests for TUI
    #[test]
    fn test_todo_panel_renders_correctly() { }
    
    // Integration tests with agent tools
    #[test]
    fn test_agent_can_create_todo_via_tool() { }
}
```

## Success Criteria

- [x] Todo list 能在 TUI 中清晰展示
- [x] Agent 能通过工具创建和更新 todo 项
- [x] Todo list 随 session 持久化
- [x] 支持任务优先级和依赖关系
- [x] 提供完整的 CLI 命令接口
- [x] 单元测试覆盖率 > 80%

## Related Issues/PRs

- Part of broader plan orchestration feature set
- Dependencies: None (can be implemented independently)

## Timeline

- **Phase 1** (Sprint N): Data model + Database schema + Repository
- **Phase 2** (Sprint N+1): Service layer + Tools implementation
- **Phase 3** (Sprint N+2): TUI components + Command handlers
- **Phase 4** (Sprint N+3): Testing, docs, refinement

## Acceptance Checklist

- [ ] All tests passing
- [ ] Code review approved
- [ ] Documentation updated in `docs/reference/ARCHITECTURE.md`
- [ ] Release notes prepared for next version
- [ ] Performance benchmarks show no regression
