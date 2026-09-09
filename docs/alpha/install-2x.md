# Retained Qiongli 2.x App installation (historical lane)

> Current CLI-first releases use the [CLI installation guide](../guide/cli-2x.md).
> This page preserves the App/Community Alpha procedure for that retained lane;
> it does not govern GitHub CLI archives, npm, PyPI or Cargo installation.

The 2.x release is a self-contained desktop product. The packaged App carries the native CLI,
Qiongli Skills, the Lite and Full MCP content, and the Codex and Claude Code integration payloads.
It does not require Rust, Cargo, Python, Node.js, npm, or pip at runtime.

## 1. Install the packaged App

Download the asset for your platform from the matching GitHub prerelease and verify the published
integrity material before opening it.

| Platform | App asset | CLI delivery |
|---|---|---|
| macOS arm64 | `Qiongli-<version>-macOS-arm64.dmg` or signed ZIP | Install from the App's About page |
| Windows x86_64 | native portable package | Bundled in the same accepted package |
| Linux x86_64 | `Qiongli-<version>-Linux-x64.AppImage` | Bundled portable CLI archive or accepted App package |

Only a verified packaged product may install the CLI or change client integrations. A source build
is intentionally inspection-only and cannot be promoted by copying files or changing an
environment variable.

Open **About** and confirm that the product trust state says that the packaged product is verified,
the displayed version matches the downloaded release, and the source commit matches the release
receipt. Stop if any of these values disagree.

## 2. Install and verify the native CLI

In **About → Qiongli CLI**:

1. Choose **Preview CLI installation** and inspect the target and approval digest.
2. Confirm the filesystem write. Qiongli installs the exact CLI bytes bundled with the App at
   `~/.local/bin/qiongli` on macOS and Linux. If an unmanaged file already occupies that exact
   target, Qiongli retains a private, digest-bound predecessor for a possible later restore.
3. If the page reports **Shell PATH not configured**, choose **Configure login PATH**. The preview
   may add only one Qiongli-owned marker block to `.zprofile`, `.bash_profile`, or `.profile`.
4. Choose **Test in new shell**. Readiness requires a fresh login shell to resolve the installed
   native binary and the exact App version; the App process PATH is not accepted as evidence.

The profile operation refuses symbolic links, non-UTF-8 or oversized profiles, and any content
that changes after preview. The CLI install and removal operations similarly refuse unowned,
drifted, symlinked, or shadowed targets.

For bounded automation, the installed native CLI exposes the same preview/apply family as the App:

```text
qiongli app snapshot
qiongli app plan cli-install
qiongli app plan cli-path-configure
qiongli app plan cli-remove
```

Use the returned plan path, digest, and required approval with `qiongli app apply`; never construct
or edit a plan by hand. A source-built CLI remains read-only.

## 3. Install Codex and Claude Code integrations

Open **Client integrations**, refresh discovery, and select only the clients you intend to change.
Qiongli reports the Plugin source, Skills, marketplace/direct package, registration, activation,
and MCP attachment as separate observations.

1. Preview and confirm **Install selected**.
2. Follow the structured Host action shown by the App. The native snapshot, not frontend text,
   owns the exact command and scope. Claude Code operations use `--scope user`; Codex uses the
   personal marketplace.
3. Restart or reload the client as instructed, then return to Qiongli and choose **Verify**.
4. Accept **Ready** only when a new Host probe observes both plugin activation and MCP attachment.
   Installed files alone are not a successful integration.

For Codex, start a new task after activation. For Claude Code, reload plugins or start a new
session. Unsupported client versions fail closed. The release notes state the minimum supported
and exact versions tested for each Alpha release.

The supported 2.x path and profile contract is:

| Surface | Codex | Claude Code |
|---|---|---|
| User Skills | `~/.agents/skills` | `${CLAUDE_CONFIG_DIR:-~/.claude}/skills` |
| Project Skills | `<project>/.agents/skills` | `<project>/.claude/skills` |
| Qiongli-managed Plugin source | `~/.qiongli/plugins/codex/qiongli-next` | `~/.qiongli/plugins/claude-code/qiongli-local/plugins/qiongli-next` |
| Registration | personal marketplace at `~/.agents/plugins/marketplace.json` | local marketplace at `~/.qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json`, installed with user scope |
| Skill inside the Plugin | `skills/qiongli-workflow/SKILL.md` | `skills/qiongli-workflow/SKILL.md` |
| Production MCP entry | Plugin `.mcp.json` starts the bundled native executable with `--profile full` | Plugin MCP descriptor starts the bundled native executable with `--profile full` |
| Lite compatibility entry | official client MCP registration starts the same executable with `--profile lite` | official client MCP registration starts the same executable with `--profile lite` |

`.agents` is plural; `.agent` is not a Qiongli 2 target. A legacy
`.codex/skills/qiongli-workflow` directory is migration input only, not the current Codex Skill
root. Client caches are versioned, Host-owned implementation details: Qiongli verifies their
receipt and content but never writes those cache directories directly.

The Plugin's Full MCP remains the production readiness boundary. Lite registration is a bounded
compatibility check performed in an isolated client configuration; normal installation does not
install Lite and Full side by side. Codex and Claude Code are the only Agent hosts covered by this
matrix. Other Agent directories are not a support claim.

## 4. Skills and MCP boundaries

- **Bundled Skills** are the academic workflow content made visible to the selected client.
- **Lite MCP** is dependency-free and provides the bounded embedded tool surface.
- **Full MCP** is the native full research surface and must be registered and attached separately.
- A detected legacy Skill or a copied directory cannot satisfy 2.x Plugin or MCP readiness.
- Project-scoped Skills materialization is separate from Host Plugin installation and remains
  bound to the selected project and receipt.

Verify each component in Client integrations after the client restart. Do not infer MCP health from
the presence of a Skill, or Skill readiness from an MCP server entry.

## 5. Repair, remove, restore, and migrate

- Use **Reconcile selected** when Qiongli reports a receipt-owned installation as drifted or
  incomplete. Reconciliation preserves unrelated marketplaces, Plugins, Skills, MCP entries, and
  user configuration.
- Use **Remove selected** only for Qiongli-owned client state. Unowned or mixed state is retained and
  reported for explicit review.
- Use **Remove CLI** from About to remove the receipt-owned native CLI. If the original unmanaged
  command was retained and both its receipt and bytes still match, the operation restores it;
  otherwise removal fails closed.
- Use the in-App legacy migration flow for detected 1.x state. Review provider conflicts and the
  exact cleanup preview. The migration never treats 1.x files as proof that 2.x is installed.

## 6. Troubleshooting PATH and stale 1.x commands

Run these commands in a new terminal, not the terminal that launched the App:

```bash
type -a qiongli
command -v qiongli
"$HOME/.local/bin/qiongli" --version
qiongli --version
```

If `command -v` points to mise, pipx, npm, Cargo, pyenv, or another shim before
`~/.local/bin/qiongli`, the old command is shadowing the native 2.x CLI. Do not delete it blindly.
First verify its owner, use the App's migration/removal preview where available, and place the
Qiongli marker after the final shim activation in the supported login profile. Then open another
fresh login shell and test again.

If the direct target version is correct but the shell version is not, the problem is PATH ordering.
If the direct target bytes or version differ from the App, use **Preview CLI update**. If the App
reports the target as drifted, symlinked, or unowned, preserve it and resolve ownership before
retrying.

## 中文说明

本页是原生 Qiongli 2.x Community Alpha 唯一的安装依据。仓库其他位置的 npm、Python、
bootstrap/shell CLI 文档属于仍受维护的 1.x 产品线，不能安装或证明 2.x App 已就绪。

2.x 必须从对应 GitHub 预发布下载并校验平台安装包。打开 App 后，先在 **About** 中确认
版本、源码提交和“已验证打包产品”信任状态一致。源码构建只能检查，不能安装 CLI、Plugin
或修改客户端配置。

安装原生 CLI 时，依次使用 **预览 CLI 安装**、**配置登录 PATH** 和 **在新 Shell 中测试**。
测试会启动新的登录 Shell，而不会借用 GUI 进程的 PATH。安装目标、profile 文件、内容摘要
和批准摘要都会在写入前展示；漂移、符号链接、过大或非 UTF-8 profile、被旧命令遮蔽的目标
都会被拒绝。

Codex 与 Claude Code 应在 **Client integrations** 中分别选择、预览安装、执行 App 展示的
原生结构化 Host action、重启/重载客户端，再执行验证。Claude Code 的命令必须使用 user
scope。只有新的 Host 探测同时观察到 Plugin 激活和 MCP attachment，状态才可以显示 Ready。

2.x 的标准路径为：Codex 用户与项目 Skill 分别位于 `~/.agents/skills` 和
`<project>/.agents/skills`；Claude Code 分别位于
`${CLAUDE_CONFIG_DIR:-~/.claude}/skills` 和 `<project>/.claude/skills`。`.agents` 必须使用复数，
`.agent` 不是 Qiongli 2 目标；旧的 `.codex/skills/qiongli-workflow` 只作为迁移输入。Codex
使用 `~/.agents/plugins/marketplace.json` 注册个人 marketplace，Claude Code 使用 Qiongli
管理的本地 marketplace 并以 user scope 安装。客户端版本缓存由 Host 管理，Qiongli 只验证
收据与内容，不直接写缓存目录。

Plugin 中的 Full MCP 仍是生产 Ready 边界。Lite MCP 只在隔离客户端配置中使用同一个原生
可执行文件验证兼容性，正常安装不会同时写入 Lite 与 Full。当前矩阵只承诺 Codex 和 Claude
Code；其他 Agent 的目录约定不构成支持声明。

修复请使用 **Reconcile selected**，移除请使用 **Remove selected**；它们只处理收据归属的
Qiongli 状态。About 中的 **Remove CLI** 会在摘要仍匹配时移除 2.x CLI，并且只在原始 1.x
文件及其收据摘要仍完全匹配时恢复旧文件。不要手工删除未知来源的 shim 或用户配置。

若 Shell 仍找到旧版本，请在新终端运行 `type -a qiongli`、`command -v qiongli`、
`"$HOME/.local/bin/qiongli" --version` 和 `qiongli --version`。如果 mise、pipx、npm、Cargo
或 pyenv 的路径排在前面，应先确认归属，再调整受支持登录 profile 中最后一条 PATH 设置，
随后重新打开登录 Shell 验证。
