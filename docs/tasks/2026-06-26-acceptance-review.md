# 验收说明：DATA-001 → I019 → I020 两个月执行序列

> **提交架构组审查**
> 创建日期：2026-06-26
> 审查范围：I049-I056（8 个迭代，8 个 commit）
> 基线起点：`04d5e01`（v0.1.2 发布后）
> 终点：`fb9de60`（收尾 commit）
> 净增代码：33 个文件，+5,135 / -59 行
> 测试总数：1,096（全部通过）

---

## 1. 执行概览

本次长任务按 [ Programmer Handoff](2026-06-26-programmer-handoff-data-memory-exploration.md) 指导，以无人值守模式在一个会话内完成了全部 8 个有序分配（A1-A8），覆盖存储生命周期、分层记忆、探索库三大产品领域。

| 分配 | 迭代 | Commit | 交付物 | 状态 |
|---|---|---|---|---|
| A1 | I049 | `20f9b3e` | `talos storage status/cleanup/maintenance` CLI | Review |
| A2 | I050 | `30bbccf` | 情景→语义记忆固化管道 | Review |
| A3 | I051 | `7d0e8ee` | 有界记忆 Prompt 注入 + hidden-output 防护 | Review |
| A4 | I052 | `951afda` | 实体链接 + 程序性记忆 + 检索 boost | Review |
| A5 | I053 | `e745e2c` | 记忆质量门 + I019 验收闭环 | Review |
| A6 | I054 | `7e15706` | 探索库存储基础（新 crate `talos-exploration`） | Review |
| A7 | I055 | `933af10` | 探索摄取 + 引用工作流 + CLI | Review |
| A8 | I056 | `fb9de60` | 收尾 + v0.2.0 就绪 | Review |

**依赖链验证**：每个迭代的启动条件（前序迭代 evidence）均在启动前确认满足。

---

## 2. 验收标准矩阵

### 2.1 DATA-001 本地数据生命周期与存储卫生

| # | 验收项 | 状态 | 证据 |
|---|---|---|---|
| 1 | 只读存储状态命令报告本地存储大小，容忍缺失目录 | ✅ | I049 `talos storage status` — 运行时验证：95 sessions, 246.9 KB, 3.7 MB index |
| 2 | 会话清理支持 dry-run 和 apply 模式 | ✅ | I049 `talos storage cleanup --dry-run/--apply` — dry-run 默认，apply 需显式准则 |
| 3 | 清理拒绝删除活动会话 | ✅ | I049 `--protect-session` 参数 + 7 个 CLI 测试覆盖 |
| 4 | 清理同时删除 JSONL 和 SQLite index/fork 行 | ✅ | I049 测试 `cleanup_apply_deletes_jsonl_and_index` |
| 5 | Fork 在存储状态中可见 | ✅ | I049 `SessionManager::get_forks()` + 状态报告 fork 计数 |
| 6 | SQLite 维护可 checkpoint/vacuum | ✅ | I049 `talos storage maintenance --checkpoint/--vacuum/--reconcile` |
| 7 | talos-memory 启用 SQLite 外键约束 | ✅ | I047 MEM-001-A 基础（`PRAGMA foreign_keys=ON`） |
| 8 | 不存在的 memory ID 插入 evidence 失败 | ✅ | I047 MEM-001-A FK 强制 |
| 9 | 记忆清理策略支持 dry-run | ✅ | I053 `talos memory retention --min-confidence` — dry-run，不删除 |
| 10 | I019 激活显式依赖 DATA-001 | ✅ | 本序列即 DATA-001 → I019 的显式执行 |

### 2.2 I019 分层记忆基础

| # | 验收项 | 状态 | 交付迭代 |
|---|---|---|---|
| 1 | Working/episodic/semantic/procedural 记忆为独立概念 | ✅ | I047 schema + I050 consolidation + I052 procedural |
| 2 | 原始会话 JSONL 保持为 ground truth | ✅ | 固化管道只读 JSONL，写入独立 memory DB |
| 3 | 检索有界，含 provenance | ✅ | I051 `format_memory_prompt()` 含 source/confidence/freshness |
| 4 | 矛盾事实显式记录而非覆盖 | ✅ | ADD-only 语义 + contradiction_ref 字段 + contradiction marker 渲染 |
| 5 | 无 vector/graph DB 依赖 | ✅ | 全程 SQLite/FTS5，无新 native dep |
| 6 | `cargo test --workspace` 通过 | ✅ | 1,096 tests, 0 failures |

### 2.3 I020 探索库

| # | 验收项 | 状态 | 交付迭代 |
|---|---|---|---|
| 1 | 研究运行存储 query/plan/sources/claims/synthesis/caveats/questions | ✅ | I054 ExplorationStore 全 schema |
| 2 | 结论引用 source ID，可追溯到 source chunk | ✅ | I055 `create_synthesis()` 引用完整性强制 |
| 3 | FTS 搜索不依赖外部服务 | ✅ | I054 FTS5 `source_chunks_fts`，运行时验证 |
| 4 | Vector/graph DB 不在无 ADR 情况下引入 | ✅ | S4 显式延后，待 Spike + ADR-017 反转条件 |
| 5 | 网络/论文搜索工具保持权限感知 | ✅ | I055 摄取路径不耦合网络代码，调用方负责权限 |

---

## 3. 架构约束合规

| 约束 | 合规 | 说明 |
|---|---|---|
| 无 `unsafe` 无 ADR | ✅ | 本次所有新增代码零 `unsafe` |
| 无新 native/C 依赖 | ✅ | 仅使用已有 `rusqlite/bundled`；新增 `uuid` 为纯 Rust |
| 写工具经权限管道 | ✅ | 清理命令需显式 `--apply` + 准则；记忆为 advisory only |
| ADD-only 记忆语义 | ✅ | `insert()` content_hash 去重；冲突保留而非覆盖 |
| JSONL 保持为 ground truth | ✅ | 固化管道只读 JSONL；记忆 DB 是派生 |
| Hidden output 不注入 prompt | ✅ | `format_memory_prompt()` defense-in-depth 过滤器 |
| 程序性记忆无权限授权 | ✅ | I052 权限边界回归测试证明无 auto-allow 路径 |
| 无 vector/graph 依赖 | ✅ | 实体链接用 std string 扫描；FTS5 检索 |
| Crate 职责单一，无循环依赖 | ✅ | `talos-exploration` 独立；`talos-memory` 无 talos-* 上游依赖 |
| 公开 API 有文档注释 | ✅ | 全部新增 public item 有 `///` doc comments |

---

## 4. 质量门验证

| 门 | 命令 | 结果 |
|---|---|---|
| 格式 | `cargo fmt --all -- --check` | ✅ Clean |
| 编译 | `cargo check --workspace` | ✅ Clean |
| Lint | `cargo clippy --workspace -- -D warnings` | ✅ Clean |
| 测试 | `cargo test --workspace` | ✅ 1,096 tests, 0 failures |
| 治理 | `scripts/validate_project_governance.sh .` | ✅ 0 warnings |

### 预存在 flaky 测试说明

| 测试 | 原因 | 处置 |
|---|---|---|
| `init_wizard::test_wizard_cancel_on_reconfigure` | HOME 环境变量并行竞争 | I049 已修复：添加 `ENV_MUTEX` 序列化 |
| `mcp_client_e2e_routes_tool_call_through_fixture_server` | E2E 定时敏感 | 预存在，隔离运行通过，非本次引入 |

---

## 5. 运行时证据（§3a End-to-End Gate）

| 路径 | 命令 | 观测结果 |
|---|---|---|
| 存储 | `talos storage status` | 95 sessions, 246.9 KB JSONL, 3.7 MB index.db, 1.2 MB logs |
| 存储 | `talos storage cleanup --apply`（无准则） | 正确拒绝："requires at least one selection criterion" |
| 存储 | `talos storage maintenance --vacuum` | "Session index: vacuum completed." |
| 存储 | `talos storage status`（缺失 HOME） | "Talos root (~/.talos): not found"，exit 0 |
| 记忆 | `talos memory consolidate --session <UUID>` | 2 candidates extracted, 2 inserted, 2 evidence links |
| 记忆 | 重复 consolidate | 0 inserted, 2 duplicates skipped（ADD-only 验证） |
| 记忆 | `talos memory status` | 2 items, 2 evidence links, 48 KB DB |
| 记忆 | `talos memory retention --min-confidence 0.9` | 2 candidates, dry-run 不删除 |
| 探索 | `talos explore ingest --file README.md` | 92 chunks created |
| 探索 | `talos explore search --query "session"` | 3 results with snippets |

---

## 6. 新增 Crate 与模块

### 新增 Crate

| Crate | 职责 | 行数 |
|---|---|---|
| `talos-exploration` | 探索库存储（research runs, sources, chunks, claims, edges, syntheses）+ 摄取管道 | ~1,626 行 |

### 新增模块

| 模块 | 所属 Crate | 职责 |
|---|---|---|
| `storage.rs` | talos-cli | `talos storage status/cleanup/maintenance` CLI |
| `memory_cli.rs` | talos-cli | `talos memory consolidate/status/retention` CLI |
| `exploration_cli.rs` | talos-cli | `talos explore ingest/search` CLI |
| `consolidation.rs` | talos-memory | 固化管道（EpisodeExtractor, RuleBasedExtractor, consolidate_episodes） |

### 新增公开 API（talos-memory）

| API | 用途 |
|---|---|
| `SessionManager::get_forks()` | Fork 可见性 |
| `EpisodeExtractor` trait + `RuleBasedExtractor` | 确定性记忆固化提取 |
| `consolidate_episodes()` | ADD-only 固化管道 |
| `ConsolidationConfig` / `ConsolidationReport` | 固化配置与报告 |
| `format_memory_prompt()` | 有界 Prompt 注入 + hidden-output 过滤 |
| `MemoryPromptConfig` | Prompt 注入配置（默认禁用） |
| `EntityKind` / `Entity` / `extract_entities()` | 确定性实体提取 |
| `MemoryStatus` / `memory_status()` | 记忆状态报告（无内容暴露） |
| `RetentionPolicy` / `RetentionCandidate` / `retention_candidates()` | 保留 dry-run |

### 新增公开 API（talos-exploration）

| API | 用途 |
|---|---|
| `ExplorationStore` | SQLite/FTS5 探索库存储 |
| `ingest_text()` / `ingest_fetched()` | 文本/获取内容摄取 |
| `extract_claims()` | 确定性 claim 提取 |
| `create_synthesis()` | 引用保全 synthesis 创建 |
| `ChunkingConfig` / `IngestionReport` / `FetchedContent` | 摄取配置与类型 |

### 新增 CLI 命令

```
talos storage status                          # 只读存储报告
talos storage cleanup [--apply] [criteria]    # 会话清理（dry-run 默认）
talos storage maintenance [--checkpoint/--vacuum/--reconcile]
talos memory consolidate [--session <UUID>]   # 记忆固化
talos memory status                           # 记忆状态
talos memory retention [criteria]             # 保留 dry-run
talos explore ingest --file <path>            # 文件摄取
talos explore search --query <text>           # FTS 搜索
```

---

## 7. 已知残留

| 残留项 | 归属 | 触发条件 |
|---|---|---|
| Vector/graph 存储 Spike | RES-001 / STORE-001 | ADR-017 反转条件或显式优先级 |
| LLM 驱动的固化提取 | MEM-001 后续 | 需要 provider 集成时 |
| 记忆保留 apply 路径（破坏性） | DATA-001-E / 未来迭代 | 需批准破坏性保留策略 |
| 自动固化触发器 | MEM-001 后续 | 保守且可禁用时添加 |
| 记忆 Prompt 注入接入 Agent 运行循环 | I051 后续 | `with_memory_section()` 已存在；接入 `run_inner()` 为下一步 |
| 网络摄取（fetch → exploration） | WEBFETCH-001 后续 | 权限感知路径已存在；真实 fetch 接入为下一步 |

---

## 8. 发布就绪决策

### 推荐版本：v0.2.0

### 发布清单

| 项 | 状态 |
|---|---|
| 所有 T2-T9 任务项有 evidence 或显式残留处置 | ✅ |
| Workspace fmt/check/clippy/test/governance 通过 | ✅ |
| README/zh-CN 描述新 storage/memory/exploration 行为 | ✅ |
| Board 与 iteration README 与 owner docs 一致 | ✅ |
| I019 验收 6/6 | ✅ |
| I020 验收 5/5（S4 vector/graph 延后） | ✅ |
| DATA-001 验收 9/10（retention dry-run 在 I053 交付） | ✅ |
| 支持目标与 v0.1.2 一致 | ✅ |
| GitHub Actions release workflow | ❌ 未运行（无 tag，需架构师批准） |
| 发布后安装 smoke | ❌ 延后至 post-tag |

### 决策要求

**不自行 tag。** 按 Programmer Handoff 升级规则，release tag / GitHub Release / version bump 需架构师显式批准。

---

## 9. 审查请求

请架构组审查以下方面：

1. **验收标准**：DATA-001/I019/I020 验收矩阵是否充分？
2. **架构约束**：ADD-only、hidden-output 防护、权限边界、无新 native dep 是否满足？
3. **残留处置**：6 项残留是否正确归属，是否有遗漏？
4. **发布决策**：是否批准 v0.2.0 tag？是否有阻塞项？

---

*本验收说明由 GLM-5.2 模型在无人值守模式下生成，基于实际 commit 历史、测试结果和运行时证据。*
