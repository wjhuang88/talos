# 2026-07-01 Month-3 交接词（Sisyphus session → architect review）

## 上下文

Month 3 执行的是四个月自举产品加固计划（`docs/tasks/2026-06-30-four-month-self-bootstrap-product-hardening-plan.md`）的 Week 9–12 任务，全部 17 个任务已关闭。

## 已完成

- **插件轨道（T40/T45/T46 + ADR-028/029/030/032）**：ToolProvenance 加了 `Plugin { name, version, carrier }` 变体，3 个 exhaustive match 站位全部通过编译器引导修复；manifest 解析器带"wasm-only"验证（拒绝 lua/dylib）；wasmtime v29.0.1 在 `wasm` Cargo feature 后面运行时默认不编译；ADR-032 补录了 wasmtime vs wasmer 6 维比较和拒绝理由。
- **Web 轨道（T42/T47 + ADR-031）**：新 crate `talos-dashboard`（axum 0.8.9）—— 127.0.0.1 loopback、per-process bearer token、4 个 GET 路由（/status /history /governance /config，api_key 已脱敏），adds 872 KB 到 release binary；新模块 `talos-tools/src/browser_page.rs` 是 BrowserPageRecord 类型 + 可 mock 的 BrowserPageConnector trait（仅 mock，无真实浏览器/cookie/storage 暴露）。
- **Track E（T43/T50/T51）**：新模块 `talos-memory/graph.rs`，SQLite v3 migration 加 `memory_graph_nodes` / `memory_graph_edges`，deterministic bounded multi-hop BFS（max 3 hops / min weight 0.3 / top-k fanout），7-day exponential decay；`CompressionMetrics` + `RetrievalMetrics` 跟踪 token 节省和召回命中。
- **Track G（T49/T53）**：`site/zh/` 7 页中文翻译 + 全 14 页 EN/ZH 互相链接；CSS 切换到 Nord 调色板（Frost accent、Polar Night 背景、Aurora status pills），SVG 资产加入六边形 mark + Nord 渐变。
- **Track A（T44/T52/T54）**：T44 通过现有 T14 ripgrep 引擎 + 12 个 parity test 关闭；T52 是**真实 Talos 驱动**的彩排（不是回顾性分析）—— Talos 自主跨 3 crate 加 `TestVariant` 变体，自举覆盖率从 T38 的 ~10% 升到 ~45%，编译器引导修复环被验证有效；月收尾报告含 REL-002 gap。
- **Issue 同步（#7/#8）+ Issue Sync Rule**：`TUI-016`（slash panel 自动执行）+ `TODO-001`（session-level todo list）已登记到 backlog；AGENTS.md 和 PRODUCT-BACKLOG 加了状态变更必须同步回原 issue 的规则。
- **Bug 修复**：T42 release 时发现 `crates/talos-tui/src/state.rs:226` 在 bash 命令审批弹窗用字节索引截断字符串，碰到中文（CJK 3 字节/字符）会 panic。修复用 `is_char_boundary` 回退到字符边界；同时给 TUI 加了 panic hook，今后 panic 会把 location 和 message 打到 stderr，不再被 raw mode 吞掉。

## 验证

| 检查 | 结果 |
|---|---|
| `cargo test --workspace` | **1347 passed**, 0 failed |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace -- -D warnings` | pass per-slice |
| `scripts/validate_project_governance.sh .` | 0 warnings |
| `scripts/check_publish_guard.sh .` | PASSED |
| `talos --version` | `talos 0.2.0` |

所有 16 次 commit 推到 `origin/main`（`f4423b9` → `8a4c3b5`）。

## 决策记录（需架构评审）

1. **ADR-032 补录 wasmtime vs wasmer 对比**：ADR-027 当时只"declared wasmtime"没有对比。现在 ADR-032 § Runtime Selection 补了 6 维对比（治理、纯 Rust、资源控制、安全记录、Component Model、依赖体积）+ 决定性因素 + Rejected Alternatives 加 wasmer 条目。
2. **Issue Sync Rule（治理新增）**：来源为 issue 的 backlog 项，状态变更必须在原 issue 上 comment 同步；close 时间点是 owner doc 标 Complete 或 Cancelled。已加到 PRODUCT-BACKLOG.md 和 AGENTS.md § Session End Checklist #7。
3. **PLUGIN-001 step 2 关闭**：Plugin 数据模型这一步实际由 T40/T45/T46 交付，下一步是 plugin 工具通过 AgentTool 注册（T59 安全审查之后）。

## 关键决策点（请架构判断）

| 问题 | 当前状态 | 需评审 |
|---|---|---|
| Dashboard 是否加 HTML 前端？ | T42 实现 API-only（JSON / 脱敏文本）。预案里 HTML 是 MVP 一部分。 | 是否要现在加（in ADR-031 边界内），还是保持 API-only 关闭 T47 |
| wasmtime 何时升级为默认依赖？ | TOOL-008 Phase 3 需要 tree-sitter WASM 加载 → wasmtime 常驻。 | Phase 3 实现前是否需要新 ADR？还是 ADR-032 已隐含 |
| 自举验证环缺失 | T52 显示 ~45%，缺**自主验证环**（agent 改完不跑 cargo check/test）。 | 单一最高 ROI 改进路径——批准 T61 后做"validate" 工具，还是 bash 工具 + 提示词够了 |
| T55/T56 发布 | 4 个 workspace dep 障碍已记录在 CRATE-PUBLICATION-MATRIX §A5。 | 维护者批准真实 publish |

## 留给 Month-4 的事项

按四个月计划，Month-4（Week 13–16）任务都是收官性质：
- T57 工具可靠性扫除（独立）
- T58 WEB-001/WEB-005 安全审查（T42/T47 MVP 在）
- T59 Plugin MVP 安全审查（T46 runtime adapter 在）
- T60 自动关联记忆注入决策（T31 建议 default-off）
- T61 第三次自举彩排（目标 >60%，自主验证环）
- T62 文档汇总 / T63 v1.0 readiness report / T64 final closeout / T65 交接产物

T55/T56 真实 publish 需要维护者显式批准，不在自动推进范围。

## 工程师交接

- 实现月报：`docs/tasks/2026-07-01-self-bootstrap-rehearsal-t52.md`
- 月收尾检查点：四个月计划的 § Month-3 Closeout
- 详细交接文档（包含决策征求清单）：`docs/tasks/2026-07-01-month3-handoff-for-architecture-review.md`

—— Sisyphus session, 2026-07-01
