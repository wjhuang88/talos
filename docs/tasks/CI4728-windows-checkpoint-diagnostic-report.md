## CI4728 Windows 隔离差分诊断（#1024 / I338）

**首轮本地实验未复现。最终托管实验在 Main 和 PR 均有复现，详见下文。** 对 `session::tests::registered_model_checkpoint_discovers_admits_and_applies_through_actor` 串行执行三轮 Base → PR。每次精确匹配一个父测试，退出码均为 0，restart 子进程包含在父测试内。

| 组别 | 原始提交 | detached 诊断提交 | 成功 / 失败 | 三次墙钟耗时（秒） |
|---|---|---|---|---|
| Base | `2eeca48febc9a802be1317aac188252d7e049e14` | `6514c9a03cadcc8d774cb3c546418daf08691438` | 3 / 0 | 26.83、23.91、24.01 |
| PR | `755c963612d4de2dfa0b879c371738e70b633ed4` | `9313924c74477f9830db1970af30334c3f780fe4` | 3 / 0 | 26.05、23.88、23.60 |

**实验条件。** 同一台 Windows 11 build 26200，显式选择 Rust 1.99.0 MSVC。两份诊断提交各自仅增加 91 行测试诊断，两侧测试文件 Git blob 相同。两边 rustfmt 检查和锁定依赖的 talos-agent 库测试预编译均通过。使用 full fixture（8300 条填充记录），dev/test debug=0，incremental=0，color=always，父进程没有设置 restart root。

**本地观察。** 六份日志的全部诊断行内容和顺序一致，未发现首个行为差异。第 6 步历史检索均为 `user_search_error=false unavailable=false`。每次捕获 47 个事件，`provider_step=13 verification_calls=1 actor_finished=false`。Checkpoint A 在事件 11 返回 Applied（746767 → 10599），B 在事件 20 返回 Applied（747426 → 11264），事件 46 的回合终态为 Success。

**原始 CI 证据。** 交接记录中 Main 单次通过（31.97 秒），PR 单次失败（40.12 秒）。PR 首个已知异常在原测试 tests.rs:7118，历史检索返回 `history/checkpoint execution unavailable`，随后 tests.rs:7811 的第二次 Submit 成功断言失败。原始失败缺少本次插桩提供的完整事件序列和拒绝原因。[Main CI4727](https://github.com/wjhuang88/talos/actions/runs/46117964530/job/140895958023) · [PR CI4728](https://github.com/wjhuang88/talos/actions/runs/46118283216/job/140896945223)

**归因与后续。** 本机三对样本没有提供 I338 导致失败的因果证据。CI 的首次拒绝原因、CI 主机负载及复现频率仍未知，缺陷状态仍待验证。建议由 Agent/checkpoint 负责人协调在同类 Windows CI runner 上交替执行这两个诊断 SHA，复现后从第一条异常事件核对脱敏拒绝原因，再决定修复归属。
## 托管 Windows 追加实验

本地最终 Base 10/10、PR 10/10 通过。托管 Windows 使用相同测试专用诊断分别运行了 1、2、10 对样本。三组独立 runner 合计 Base 12 成功、1 失败；PR 12 成功、1 失败。restart 子进程未另计。最后十对均通过。

基线失败时，restart 查询得到结果，交付前剩余 464 ms，随后记录 delivery_source_or_snapshot_unavailable。另一台 runner 上 PR 的 restart 更早报告 linear_index_build_failed、archive_evidence_unavailable，两次查询得到空结果，均剩 1999 ms。尚未证实 SQLite 锁冲突或具体底层错误。原 CI4728 的异常出现在父进程第二次 Submit，仍缺该次的内部原因；这两次 restart 失败无法证明原失败同源。

复现同一测试失败已不依赖 I338。Agent/checkpoint 独立修复认领 #1053 待生效。托管脱敏日志保存在 hosted-*.log。
https://github.com/wjhuang88/talos/actions/runs/46123829691
https://github.com/wjhuang88/talos/actions/runs/46125021272
https://github.com/wjhuang88/talos/actions/runs/46125038376
