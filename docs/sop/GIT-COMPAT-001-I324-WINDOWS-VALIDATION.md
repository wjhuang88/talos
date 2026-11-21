# I324 / #919 — Windows Git 2.36.1 原生验证操作手册

> 状态：待执行的本机诊断操作；**不是**产品验收通过凭据。
>
> 任务：GIT-COMPAT-001 / [Issue #919](https://github.com/wjhuang88/talos/issues/919)；实施候选 [Draft PR #928](https://github.com/wjhuang88/talos/pull/928)；ADR 安全/API 审查 [Draft PR #931](https://github.com/wjhuang88/talos/pull/931)。
>
> 当前范围：**保留旧版 Git 能运行、且已获 ADR-089 授权的 CLI**。仅为实测不支持的查询或选项设计局部替代。旧版命令矩阵不等于 Prepared Checkout / Commit 验收。

## 1. 准备与源码选择

使用 **Windows 10/11 x64、PowerShell 7+**。第一阶段只需要联网下载 Git for Windows 官方归档；第二阶段可选，需要 Rust 1.99、Cargo、MSVC/C++ Build Tools（\`x86_64-pc-windows-msvc\`）。无需管理员权限。请使用个人空白测试目录，**不要在真实业务仓库运行 Git 写操作**。

本验证文档和可执行脚本目前在 **PR #928 分支，不在 main**。在 PowerShell 7 运行：

\`\`\`powershell
git clone https://github.com/wjhuang88/talos.git
Set-Location talos
git fetch origin pull/928/head:validation-i324
git switch validation-i324
git rev-parse HEAD
\`\`\`

如已有仓库，省略 clone，在已有仓库内 fetch 即可。如果本地 \`validation-i324\` 分支已存在，请不要用强制重置覆盖任何本地工作；改用另一临时分支或 detached FETCH_HEAD。

通过 \`git rev-parse HEAD\` **记录实际被测提交**；本文件与脚本可能继续更新，反馈时务必同时提供完整 40 位 SHA。若需重现历史固定版本，可使用已验证的提交 SHA，但不要将旧 CI 的绿色状态移植到新 SHA。

## 2. 第一阶段：固定 Git 2.36.1 命令矩阵（优先执行）

执行仓库现有脚本 \`scripts/test_i324_legacy_git_query_matrix.ps1\`，而不是复制旧会话中的脚本片段：

\`\`\`powershell
$PSVersionTable.PSVersion
$probeLog = Join-Path $env:TEMP 'i324-git236-pinned-result.txt'
./scripts/test_i324_legacy_git_query_matrix.ps1 -UsePinnedMinGit |
    Tee-Object -FilePath $probeLog
$probeLog
\`\`\`

脚本下载 Git for Windows 官方 \`MinGit-2.36.1-64-bit.zip\`，按官方 release 所列 SHA-256 \`a7a78c306dea018cc7ca3efe6a0d87c1dd7a43762705ccb0c5e31e3e44349207\` 校验后执行；**不会安装或替换系统 Git，不修改 PATH/HOME/Git 用户配置**。使用绝对路径启动隔离 MinGit，在临时测试仓库构造不同的 committed/index/worktree 属性，结果只打印变量名、退出码、支持性类别，不打印 Git 返回的路径或配置值。测试清理临时目录。

关注 \`I324_LEGACY_FIXTURE\`、五条 \`I324_LEGACY_VAR\`、\`I324_LEGACY_CHECK_ATTR\` 和 \`I324_LEGACY_QUERY_MATRIX\`。**不可把 \`--cached\` 当作目标提交 \`--source=HEAD\` 的同义操作**。任何 \`unexpected-error\` 都应保留为未分类错误，不能算作“旧版不支持”。

这里特意屏蔽外部 Git 配置及 \`GIT_INSTALL_ROOT\`，得到的是**旧版二进制命令能力证据**，并非原始 Scoop 安装 / 全局 attributes 默认路径与最终 Git 写操作语义的证明。

## 3. 第二阶段：所选 Windows 系统 Git（建议执行）

先记录软件版本与敏感环境变量的**存在性**（不要发实际路径或变量值）：

\`\`\`powershell
git --version
rustc --version
cargo --version
[pscustomobject]@{
    GitInstallRootPresent = [bool]$env:GIT_INSTALL_ROOT
    GitPagerPresent = [bool]$env:GIT_PAGER
    GitConfigGlobalPresent = [bool]$env:GIT_CONFIG_GLOBAL
}
\`\`\`

如果机器实际安装 Git 2.36.1（包括 Scoop shim），再用同一个隔离脚本直接指定这个可执行文件：

\`\`\`powershell
$selectedGit = (Get-Command git.exe -CommandType Application).Source
$nativeLog = Join-Path $env:TEMP 'i324-selected-git-result.txt'
./scripts/test_i324_legacy_git_query_matrix.ps1 -GitExecutable $selectedGit |
    Tee-Object -FilePath $nativeLog
\`\`\`

注意：此模式也会隔离环境，不会重现 Scoop 的全部动态语义。若机器的 \`git --version\` 并非 2.36.1，将其标注为“新版 Git 对照样本”，不能当旧版验收。

## 4. 第三阶段：Rust 回归与 Prepared Checkout / Commit（可选）

**这一步不需要强行让当前 Draft 通过**：目前 #928 只有诊断候选，尚未替换生产预检。必须在上一阶段明确保存实际 SHA、所选 Git 版本和测试环境。

\`\`\`powershell
rustup toolchain install 1.99.0-x86_64-pc-windows-msvc
cargo +1.99.0-x86_64-pc-windows-msvc fmt --all -- --check

$candidateLog = Join-Path $env:TEMP 'i324-gix-candidate-tests.log'
cargo +1.99.0-x86_64-pc-windows-msvc test --locked -p talos-tools --features git --lib gix_preflight_candidate_tests -- --nocapture 2>&1 |
    Tee-Object -FilePath $candidateLog
$candidateExit = $LASTEXITCODE
"candidate_exit=$candidateExit"

$checkoutLog = Join-Path $env:TEMP 'i324-large-checkout-commit.log'
cargo +1.99.0-x86_64-pc-windows-msvc test --locked -p talos-tools --features git --lib checkout_and_commit_support_tracked_content_over_16_mib -- --nocapture 2>&1 |
    Tee-Object -FilePath $checkoutLog
$checkoutExit = $LASTEXITCODE
"checkout_commit_exit=$checkoutExit"
\`\`\`

大文件测试使用约 17 MiB tracked 内容。若失败，请如实记录**第一个失败原因、退出码和未发生的实际变更**。目前正式 Prepared Checkout / Commit 仍未通过 Git 2.36.1 原始 Scoop 环境验收；不能因为 gix 候选测试通过就将该任务标为完成。

**如果系统 \`git\` 是新版**，不要为了运行上述测试而随意替换 PATH 或修改其他程序使用的 Git。可以先完成前两阶段，待确定旧版系统 Git 的可复现安装配置后再做最终产品验收。

## 5. 反馈及数据安全

请把以下信息反馈到 [Issue #919](https://github.com/wjhuang88/talos/issues/919) 或直接发回当前会话：

1. Windows 版本、PowerShell 版本、实际 \`git --version\`、被测 \`git rev-parse HEAD\` 完整 SHA。
2. **完整的第一阶段分类输出**（无需 Git 返回值）。
3. 第二阶段若执行：选定 Git 版本、\`GIT_INSTALL_ROOT\` 是否存在、与固定 MinGit 的分类差异。
4. 第三阶段若执行：Rust/Cargo 版本、候选与 17 MiB 测试的退出码、失败测试名称和脱敏后的首个错误。

日志写在 \`$env:TEMP\`，**不要提交原始日志、HOME、令牌、凭据、绝对路径、Git 配置内容或工作区敏感数据到仓库**。出现异常时保留原始日志在本机；反馈前人工脱敏。命令矩阵只能说明**具体命令形式**支持与否；真正的系统 attributes 编译默认来源、permission/Read/Execute 权限边界、变更后重验和原始环境 Checkout/Commit 仍须单独审查。

历史本地 Windows 失败证据见 [2026-11-17 原生验证报告](../reference/WINDOWS-VALIDATION-2026-11-17-I324.md)。此手册本身不修改生产实现、不接受 ADR-089 新权限、不关闭 #919。
