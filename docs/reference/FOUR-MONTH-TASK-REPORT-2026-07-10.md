# 四个月架构与技术债清理任务报告

**项目**: Talos — Rust 原生本地编码 Agent
**计划文档**: `docs/tasks/2026-07-09-four-month-architecture-tech-debt-cleanup-plan.md`
**执行周期**: 2026-07-09 至 2026-07-10
**执行方**: glm-5.2 (外部 Agent) + gpt-5 (维护者手动补丁)
**提交范围**: b2c8d25 → b22b98a (45 commits)
**最终状态**: 全部完成

---

## 一、执行概览

### 1.1 提交统计

| 指标 | 数值 |
|---|---|
| 总提交数 | 45 |
| glm-5.2 提交 | 40 |
| gpt-5 提交 | 5 |
| 涉及 crate | talos-provider, talos-permission, talos-session, talos-tools, talos-cli, talos-tui, talos-agent, talos-conversation, talos-core, talos-config, talos-plugin |
| 新增 ADR | ADR-036 (zstd), ADR-037 (compact text format), ADR-038 (workspace trust) |
| 修订 ADR | ADR-034 v4 (reasoning display projection) |

### 1.2 验证结果

| 检查项 | 结果 |
|---|---|
| `cargo check --workspace` | ✅ exit 0 |
| `cargo clippy --workspace -- -D warnings` | ✅ clean |
| `cargo test --workspace` | ✅ 61 suites, 1852 tests, 0 failures |
| Git working tree | ✅ clean |

---

## 二、任务清单

### Month 1 (I110) — Provider 架构 + Session 格式基础

| Task | 描述 | 交付物 | 验证 |
|---|---|---|---|
| T100 | 分解 openai.rs (2365→313) | `openai_sse.rs` (2065行): SSE DTO + parse_sse_stream + 全部测试 | 96 provider 测试通过 |
| T101 | 分解 lib.rs Anthropic (1677→291) | `anthropic_request.rs` (462行) + `anthropic_stream.rs` (961行) | parse_text_tool_calls re-export 保持兼容 |
| T102 | SessionStore 抽象 | `store.rs` (201行): SessionStore trait + JsonlSessionStore; `segment_chain.rs` (288行): chain.tlog reader/writer | 117 session 测试通过 |
| T103 | 紧凑文本 .tlog 格式 | `compact_text.rs` (742行): TSV + length-prefix, 多格式管理器集成, 段链元数据, fork 格式保留, corruption resync | 26.3% 密度节省 vs JSONL |
| T104 | Month-1 收尾 | 执行证据, docs sync | ✅ |

### Month 2 (I111) — CLI/Permission/TUI 架构 + 导出服务

| Task | 描述 | 交付物 | 验证 |
|---|---|---|---|
| T110 | 分解 mode_runners.rs (2290→696) | `session_handlers.rs` (958行) + `mode_interactive.rs` (184行) + `mode_runners_tests.rs` (316行) + `dashboard_helpers.rs` (171行) | 166 CLI 测试通过 |
| T111 | 分解 permission/lib.rs (1630→454) | `rule.rs` (162行) + `resource.rs` (76行) + `permission_tests.rs` (970行) | 62 permission 测试通过 |
| T112 | 导出/转录服务 | `transcript.rs` (350行): JSON + Markdown 导出, 格式无关, 双格式兼容 | 16 tests |
| T113 | 分解 state.rs (1469→450) | `panel_state.rs` (537行): BottomPanelState + 全部 panel 类型; `state_tests.rs` (500行) | 252 TUI 测试通过 |
| T114 | Month-2 收尾 | 执行记录, docs sync | ✅ |

### Month 3 (I112) — Permission Sandbox + Session 压缩 + 工具清理

| Task | 描述 | 交付物 | 验证 |
|---|---|---|---|
| T120 | PERM-004 ADR | ADR-038: workspace trust sandbox boundary (Git 检测, 显式授权, Deny 优先, repo 外严格) | ADR 接受 |
| T121 | PERM-004 实现 | `workspace_trust.rs` (206行): WorkspaceTrustStore + is_git_workspace + is_within_repo; PermissionEngine.trusted_workspace 字段 + evaluate_facect auto-Allow; CLI `--trust` flag; trusted_workspaces.toml 持久化 | 62 permission 测试 + 4 trust 测试 |
| T122 | SESSION-004 Slice D 压缩归档引擎 | `compression.rs` (128行): SegmentCompressor trait + NoCompressor + ZstdCompressor (feature-gated); `compaction_engine.rs` (323行): CompactionEngine (should_compact/compact_segment/freeze/archive/chain update); 接入 session.rs turn loop (LLM collapse + 文件归档原子执行) | 4 compaction 测试 + 4 compression 测试 |
| T123 | 分解 git.rs (1285→660) | `git_write.rs` (454行): host-git helpers + GitAdd/Commit/Push/Pull/Checkout; `git_tests.rs` (227行) | 258 tools 测试通过 |
| T124 | TOOL-020 git diff ref-to-ref | GitDiffInput 新增 base_ref/head_ref; host git diff fallback; path 过滤; max_lines 截断 | 7 git 测试通过 |
| T125 | Month-3 收尾 | 执行记录, ARCH-030 更新 | ✅ |

### Month 4 (I113) — Session 压缩 + TUI Polish + 收尾

| Task | 描述 | 交付物 | 验证 |
|---|---|---|---|
| T130 | SESSION-004 Slice E 工具输出压缩 | `tool_output.rs` (96行): compress_tool_output(content, threshold) → ToolOutputCompression; SessionMetadata.raw_content 字段; 接入 agent turn loop (tool result > 4000 字节自动压缩, model 得到摘要, raw 保留) | 4 compression 测试 |
| T131 | PERF-001 Phase 1 build-time models.toml | `build.rs` 扩展: 编译时解析 models.toml → 生成 Rust 代码到 OUT_DIR; builtin_models()/builtin_providers() 不再运行时解析 TOML | 46 config 测试通过 |
| T132 | TUI-028 残留 | #28/#39: dashboard 通知改为 transient Tip (glm-5.2); #24/#25/#31: I114 迭代由 gpt-5 完成 (150ms 定时器, two-color ripple, display-width-aware truncation) | I114 Complete |
| T133 | TUI-029 决策 → 实施 | 初始拒绝 → 维护者推翻 → ADR-034 v4 → 全部 4 slices 实现 (见下文) | ✅ |
| T134 | HOOK-001 config-introduced hooks | HookConfig + HookDeclaration schema (event, name, description, enabled); /hooks 命令列出声明的 hooks; ConversationEngine.hook_declarations | 120 conversation 测试通过 |
| T135 | 最终收尾 | 执行记录更新, ARCH-030 更新 (6 个 resolved roots + 5 个 remaining), 计划文档 Complete | ✅ |

### TUI-029 实施 (ADR-034 v4, 4 Slices)

维护者 commit 36c95a6 推翻了初始拒绝, 接受 ADR-034 v4:

| Slice | Crates | 交付物 | Commit |
|---|---|---|---|
| A | talos-conversation | MessageRole::Reasoning + MessageSource::Reasoning; finalize_thinking() 在 TextDelta/ToolCallStarted/TurnEnd transition 时归档; Error/cancel 丢弃未完成 thinking | 6970af9 |
| B | talos-tui + talos-core | project_displayable_reasoning() centralized helper (只读 Thinking.text/Plain.text, 不暴露 signature/Redacted); hydrate_history() resume 时在 assistant entry 前重建 reasoning block | 4ebf73e |
| C | talos-conversation + talos-cli | /export <path> --include-thinking: filtered text 导出; transcript_plain_text_with_thinking(); 默认 /copy /export 排除 reasoning | 26211d3 |
| D | workspace | TUI-029 backlog Complete; BOARD 更新; PRODUCT-BACKLOG 更新; 执行记录 | 63b8e42 |

---

## 三、引擎接入生产代码路径

所有基础设施引擎均已接入生产代码路径, 不存在 dead code:

| 引擎 | 位置 | 接入点 | 状态 |
|---|---|---|---|
| CompactionEngine | talos-session/compaction_engine.rs | session.rs turn loop: should_compact → LLM collapse → compact_segment() (freeze + compress + chain update) | ✅ |
| compress_tool_output | talos-agent/tool_output.rs | agent lib.rs: tool result > 4000 bytes → split model summary + raw | ✅ |
| WorkspaceTrustStore | talos-permission/workspace_trust.rs | registry.rs TuiApprovalHandler::new_with_trust(): CLI 启动检测 Git + trust store; --trust flag | ✅ |
| SegmentCompressor | talos-session/compression.rs | CompactionEngine 内部使用 | ✅ |
| SessionStore (multi-format) | talos-session/store.rs | SessionManager: KNOWN_EXTENSIONS + is_session_file + store_for_path | ✅ |
| project_displayable_reasoning | talos-core/message.rs | hydrate_history() resume 路径; export --include-thinking | ✅ |

---

## 四、架构债清理结果

### 4.1 God Module 分解

| 文件 | 原始行数 | 最终行数 | 减少 | 新增模块 |
|---|---|---|---|---|
| openai.rs | 2365 | 313 | -87% | openai_sse.rs |
| lib.rs (provider) | 1677 | 291 | -83% | anthropic_request.rs, anthropic_stream.rs |
| lib.rs (permission) | 1630 | 454 | -72% | rule.rs, resource.rs, workspace_trust.rs |
| state.rs (tui) | 1469 | 450 | -69% | panel_state.rs |
| git.rs (tools) | 1285 | 660 | -49% | git_write.rs |
| mode_runners.rs (cli) | 2290 | 696 | -70% | session_handlers.rs, mode_interactive.rs, dashboard_helpers.rs |
| **合计** | **10716** | **2864** | **-73%** | 12 new modules |

**全部 6 个文件均低于 800 行目标。**

### 4.2 ADR 记录

| ADR | 标题 | 状态 |
|---|---|---|
| ADR-034 v4 | Provider Reasoning/Thinking Boundary — 新增 bounded visible-history projection | Accepted (v4 revision 2026-07-10) |
| ADR-036 | zstd Compression for Session Log Archival | Accepted |
| ADR-037 | Compact Text Session Log Format and Archival Architecture | Accepted (fork section updated: simple copy, no COW) |
| ADR-038 | Workspace Trust Sandbox Boundary | Accepted |

### 4.3 新增基础设施

| 模块 | 文件 | 用途 |
|---|---|---|
| SessionStore trait | store.rs | 格式无关的会话存储抽象 |
| CompactTextSessionStore | compact_text.rs | TSV + length-prefix 文本格式, 比 JSONL 节省 26.3% |
| SegmentCompressor | compression.rs | 压缩 trait + NoCompressor + ZstdCompressor (feature-gated) |
| CompactionEngine | compaction_engine.rs | 文件级归档: freeze → compact → compress → chain.tlog |
| WorkspaceTrustStore | workspace_trust.rs | 持久化 Git workspace trust, trusted_workspaces.toml |
| ToolOutputCompression | tool_output.rs | per-request 工具输出压缩: threshold → summary + raw |
| project_displayable_reasoning | talos-core/message.rs | 安全的 reasoning 文本投影 (不暴露 signature/Redacted) |
| SegmentChain | segment_chain.rs | chain.tlog reader/writer + SegmentMeta |
| Transcript Service | transcript.rs | 格式无关 JSON + Markdown 导出 |

---

## 五、Post-Plan 变更控制

### 5.1 TUI-029 决策变更

初始执行 (T133): 正式拒绝 thinking history archive (ADR-034 v3 transient boundary preserved)

维护者变更 (commit 36c95a6): 推翻拒绝, 接受 ADR-034 v4, 要求实现

后续执行: 全部 4 slices 按 ADR-034 v4 合同实现 (TUI029-A/B/C/D)

### 5.2 TUI-028 残留处理

初始执行: 推迟 #24/#25/#31 (理由: 需要 PTY 视觉测试)

维护者执行 (I114, commits c68fd08/823a8e0/072c726): 全部完成 — 150ms 定时器, two-color ripple, display-width-aware truncation

### 5.3 Fork COW 设计修正

ADR-037 原设计包含 COW snapshot reference (parent_ref + ref_count + delete protection)

实现中发现: compaction 保证 head.tlog 始终很小, 直接复制 entries 是更简单且足够的方案

修正: 移除 COW 代码 (-53行), ADR-037 fork section 更新为 simple copy

---

## 六、治理文档同步

| 文档 | 更新内容 |
|---|---|
| 计划文档 (tasks/2026-07-09) | 全部 22 任务 Planned → Complete |
| 执行记录 (FOUR-MONTH-PLAN-EXECUTION-RECORD) | 44 commits, 全部残留项关闭 |
| ARCH-030 残留寄存器 | 6 resolved roots + 5 remaining (app.rs, sqlite.rs, exploration, ingestion) |
| BOARD.md | TUI-029 → Complete; TUI-028 → Complete (I114) |
| PRODUCT-BACKLOG.md | TUI-029, TUI-028, MC-002 状态更新 |
| ADR-034 | v4 revision for bounded visible-history projection |
| ADR-037 | Fork section updated (COW removed, simple copy) |
| ADR README | ADR-036, 037, 038 added to index |
| EVOLUTION.md | TUI-029 approved entry |

---

## 七、未实现项 (设计如此, 非遗漏)

| 项目 | 原因 |
|---|---|
| raw_flag=2 外部 SQLite blob | ADR-037 设计为 future extension; 当前 raw_flag=0 (no compression) 和 raw_flag=1 (inline) 已实现 |
| Hook 执行载体 | ADR-027 门控: WASM runtime 未就绪; HOOK-001 第一 slice 只做 config schema + diagnostics |
| PERM-005 bash/exec sandbox | 独立 story, 需要 touched-path evidence; ADR-038 记录为 reversal trigger |
| 剩余 >800 行模块 (app.rs 1005, sqlite.rs 986, exploration 958) | ARCH-030 记录为 future decomposition targets; 本计划范围不包含 |

---

## 八、结论

全部 22 个计划任务 (T100-T135) + 4 个 TUI-029 slices + 5 个引擎接入 + 4 个治理文档同步 = **31 项工作单元全部完成**。

45 个 commit 推送到 main 分支, 涵盖 11 个 crate, 新增 12 个聚焦模块, 6 个 god module 减少 73% 代码量, 3 个新 ADR + 1 个 ADR 修订, 1852 个测试全部通过, clippy 零警告。

计划文档执行状态: **Complete**。
