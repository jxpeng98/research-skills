# 安装 Qiongli

> 原生 2.x 可[直接下载独立二进制包](cli-2x.md#standalone-binary-download)，
> 或使用 [npm / pip 安装](cli-2x.md#package-managers)。
> 下方 npm/Python 能力差异与安装命令属于 1.x。

> **产品线说明：**本页介绍 Qiongli 1.x 的 npm、Python、marketplace 与 bootstrap/shell
> 安装方式。保留维护的 App / Community Alpha 使用单独的
> [历史安装说明](../../alpha/install-2x.md)，原生 2.x CLI 不要求安装 App。

Qiongli 有多个安装入口，是因为不同用户需要的运行时能力不同。先选能满足目标的最小入口。

## 最新稳定版下载

当前稳定版是 [v1.17.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.17.0)。下面这些直达链接覆盖常见安装路径；需要 subject 专精 Desktop ZIP 或维护者 artifacts 时，再打开下载指南。

| 需求 | 链接或命令 |
|---|---|
| npm CLI | [`qiongli@1.17.0`](https://www.npmjs.com/package/qiongli/v/1.17.0)：`npm install -g qiongli@latest` |
| PyPI CLI | [`qiongli 1.17.0`](https://pypi.org/project/qiongli/1.17.0/)：`pipx install qiongli` |
| Claude Desktop 推荐插件 | [`qiongli-claude-desktop-plugin-v1.17.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-claude-desktop-plugin-v1.17.0.zip) |
| Claude Desktop/Web fallback skill ZIP | [`qiongli-claude-desktop-skill-core-v1.17.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-claude-desktop-skill-core-v1.17.0.zip) |
| Claude Desktop literature MCPB | [`qiongli-literature-provider-0.1.5.mcpb`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-literature-provider-0.1.5.mcpb) |
| Zotero Desktop companion | [`qiongli-zotero-companion-0.2.2.xpi`](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-zotero-companion-0.2.2.xpi) |
| 全部 release assets | [下载指南](https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-downloads-v1.17.0.md) 和 [GitHub Release](https://github.com/jxpeng98/qiongli/releases/tag/v1.17.0) |

## 安装入口对比

| 入口 | 定位 | 包含内容 | 能做什么 | 边界 | 是否要求 Python |
|---|---|---|---|---|---|
| Marketplace plugin / extension | 客户端原生、最少配置 | 客户端 plugin 和 `qiongli-workflow`；Codex / Claude Code 内置 Rust Lite literature MCP | 在单个客户端里使用 workflows、prompts、subject packages 和内置 literature-provider tools | 不暴露完整 Python orchestrator，也不负责 package self-update | skill 使用和内置 literature MCP 不要求 |
| Claude Desktop Skill ZIP | Desktop/Web 的 skill-only 路径 | 个人上传的 `qiongli` Skill，通常可搭配 literature MCPB | 不用终端或代码环境，在 Claude Desktop/Web 中使用 workflows | Skill ZIP 不保存 secrets，也不执行 provider 或 orchestrator calls | 否 |
| Claude Desktop Literature MCPB | Desktop 本地 provider 路径 | `qiongli-literature-provider.mcpb` 内置 Rust Lite MCP executable | provider config/status、本地 literature search、evidence export、Zotero import files | 只提供 provider；不安装 Qiongli workflows，也不运行 Python orchestrator | 否 |
| npm / npx | 免 Python 资产管理器 | npm CLI、默认预生成 skills；可用 `--surface plugin|both` 显式安装 plugin-lite assets；Node project commands | 脚本化安装、CI/dotfiles、当前 package asset refresh、`project init/status/set-subject`、bundled plugin-lite | 不升级 npm/Python package，不运行 `doctor`、`mcp serve`、provider setup 或 task orchestration | 否 |
| pipx / pip 完整运行时 | Python CLI 和受管理 full local runtime | Python CLI、setup wizard、完整 plugin surface、统一 MCP server、provider setup、doctor、task/orchestrator commands | 完整本地验证、provider 配置、MCP/orchestrator 工具、package self-update、release archive refresh | 真实 agent execution 前仍需要本地 Python 和对应 client/model CLI | 是，Python 3.12+ |
| Bootstrap `partial` | release script skills 安装 | 多客户端全局 skills 和 workflow discovery | 不走 package manager，直接安装 portable workflow assets | 不做完整 runtime validation 或 orchestration | 否 |
| Bootstrap `full` | release script full runtime 安装 | `partial` 加 shell CLI、MCP registration part 和 `doctor` 支持 | 通过 release scripts 安装完整运行时 | 不安装 Python；机器必须已有 Python 3.12+ | 是，Python 3.12+ |

用户可见的 skill 名称是 `qiongli`。安装目录仍然是 `qiongli-workflow`，这是为了兼容已有客户端和 release artifacts。普通完整运行时 CLI / local plugin 使用中，先安装稳定的 `core/complete` runtime，并把日常 subject 行为保存在 `.qiongli/guidance_manifest.yaml`。npm 默认安装 skills surface；只有明确需要 bundled/supported 的 plugin-lite 输出时，才使用 `--surface plugin` 或 `--surface both`。CLI/npm 专精安装默认使用 `coverage=complete`，但它们是 Desktop ZIP、focused package、release payload 和 install-surface testing 等 advanced compatibility 路径。

在 Python 完整运行时中，从 v1.9.0 开始，`qiongli install` 默认等价于 `--profile full --surface plugin`。Codex / Claude Code / Antigravity 会得到 CLI 管理的本地 plugin，并由 `qiongli mcp serve --transport stdio` 支撑；Antigravity 会按官方 plugin 结构在 plugin 根目录写入 `mcp_config.json`，Hermes 会写入受管理的 full MCP client config。npm/npx 仍是免 Python 路径，默认 `--surface skills`。

npm/npx 是免 Python 资产管理器入口。需要 `doctor`、`mcp serve`、`provider setup`、`task-plan`、`task-run` 或 `customize` 这类完整运行时命令时，先安装完整运行时：`pipx install qiongli`。

## 原生 Plugin 和 Extension

只需要在一个客户端里使用 Qiongli 时，优先选这个入口。

Codex 通过统一的 [Skillsplace](https://github.com/jxpeng98/skillsplace) marketplace 安装：

```bash
codex plugin marketplace add jxpeng98/skillsplace --ref main
codex plugin marketplace list
```

然后在 Codex plugin UI 中安装或启用 `qiongli`，这是默认 core package。也可以选择 `qiongli-economics`、`qiongli-accounting`、`qiongli-business`、`qiongli-finance`、`qiongli-political-economy`、`qiongli-geoeconomics`、`qiongli-economics-accounting` 这类 subject entry，它们会安装对应的 `subject/complete` package。

Codex plugin 自带 `.mcp.json` 和 `bin/qiongli-literature-provider` Rust Lite literature-provider MCP runtime。只使用这些内置文献 provider 工具时，桌面用户不需要安装 Node、Python、`qiongli` CLI，也不需要手写 MCP config。Provider key 不写入 plugin manifest；可以通过平台无关的本地设置工具 `qiongli_configure_provider` 配置，也可以用 `qiongli_save_provider_config` 保存，或者在已安装 CLI 时用 `qiongli mcp configure` / `qiongli provider setup` 配置。完整本地 Qiongli 使用 CLI 生成的本地 plugin：`qiongli install --profile full --target codex --surface plugin`。这个本地 plugin 保留 Codex 原生 plugin 容器，但它的 `.mcp.json` 会启动完整 Python-backed `qiongli mcp serve --transport stdio` server。

安装或升级 plugin 之后，请新开一个 Codex thread，或者重启本地 Codex 客户端，让 plugin 内置 MCP server 被重新加载。在 Codex CLI 中，`codex mcp list` 应该能看到 `qiongli` 或 `qiongli-next`，其 command 为 `./bin/qiongli-literature-provider`，`cwd` 指向 plugin cache 路径。在 Codex App 中，可以通过 Settings > Integrations & MCP 或当前 thread 的 MCP status view 确认 server 已启用，再测试 provider tools。

Codex 插件安装默认使用插件内置 MCP。安装器会把 `.mcp.json` 写在 Qiongli 插件目录内，并由插件 manifest 指向它；插件路径不会写入 `~/.codex/config.toml`。只有在需要 standalone MCP fallback 时，才运行 `qiongli install --target codex --parts mcp`。

Codex 目前会把 plugin-bundled MCP server 当作 plugin asset：设置页可以启用 server 和管理 tool policy，但不适合作为这个内置 server 的 provider key 注入入口。Claude Desktop MCPB、Claude Code、Cursor 类客户端和其他本地 stdio MCP client 也应使用同一个 Qiongli provider setup contract。请改用 Qiongli provider config：

1. 让 Codex 运行 `qiongli_config_status`，查看 redacted status 和 `config_path`。
2. 让当前客户端运行 `qiongli_configure_provider`，然后打开返回的 `127.0.0.1` URL。
3. 在本地浏览器页面里填写 OpenAlex API key、可选 OpenAlex email 和 Semantic Scholar API key。页面会写入共享 provider config，避免把密钥放进对话上下文。
4. 再运行 `qiongli_config_status` 或 `qiongli_literature_status`；结果只应显示 `configured` / `missing`，不应打印完整密钥。

不要把 provider key 写进 `.mcp.json`、`.codex-plugin/plugin.json`、release ZIP 或研究产物。Codex plugin MCP、Claude Code plugin MCP、Claude Desktop MCPB 和完整 CLI MCP server 读取的是同一个共享 provider config。

Claude Code 使用同一个 Skillsplace catalog：

```bash
claude plugin marketplace add jxpeng98/skillsplace@main
claude plugin install qiongli@skillsplace
# Subject 专精安装：
claude plugin install qiongli-economics@skillsplace
```

在 Claude Code 交互会话中，也可以使用 slash commands：

```text
/plugin marketplace add jxpeng98/skillsplace@main
/plugin install qiongli@skillsplace
/plugin install qiongli-economics@skillsplace
```

Claude Code marketplace plugin 也内置 `bin/qiongli-literature-provider` Rust Lite literature-provider MCP runtime，提供与 Codex plugin 相同的 provider、search、search-plan、evidence-export、Zotero import-file 和 status tools。只使用这些内置 literature-provider tools 时，不需要安装 Node、Python 或 `qiongli` CLI。`qiongli_task_run` 和 `qiongli_orchestrator_doctor` 这类 Python-backed orchestration tools 需要完整运行时：`pipx install qiongli`。然后运行 `qiongli install --profile full --target claude --surface plugin` 生成本地 Claude Code plugin，并由这个 plugin 启动统一的 `qiongli mcp serve --transport stdio` server。`--target antigravity` 会生成带 root `mcp_config.json` 的 Antigravity plugin，`--target hermes` 写入 Hermes MCP config；`--target all --surface plugin` 会让 Codex / Claude Code / Antigravity 使用本地 plugin，同时给 Hermes 写入受管理的 full MCP client 配置。

安装或升级 Claude Code plugin 之后，请运行 `/reload-plugins` 或新开一个 Claude Code session。然后打开 `/mcp`；Qiongli server 应显示为 plugin-provided MCP server，并且 tool count 不应为 0。Plugin-bundled MCP tool name 会带 plugin 和 server namespace，所以实际可调用名称会包含 plugin 前缀，而不只是裸的 `qiongli_literature_search`。

Claude Desktop 和 Claude.ai 不安装第三方 Claude Code plugin marketplace。对于 Claude Desktop，优先使用 direct plugin ZIP：

1. 从 GitHub Release assets 下载 `qiongli-claude-desktop-plugin-<tag>.zip`。
2. 通过 Claude Desktop 的 direct plugin 或 plugin upload 流程安装。
3. 重启 Claude Desktop，并确认用户可见入口是 `qiongli`。

direct plugin 是推荐的 Desktop 包，因为它把 `qiongli` skill、包含 `/qiongli` 的 workflow command wrappers，以及轻量 bundled literature MCP runtime 放在同一个 plugin-shaped asset 里。它不提供 Python-backed orchestrator；如果需要 `qiongli_orchestrator_route`、`qiongli_task_plan` 或 `qiongli_task_run`，仍然要单独安装完整运行时。

如果 direct plugin 安装不可用，再使用 fallback skill ZIP。只下载一个 focused skill ZIP，例如 `qiongli-claude-desktop-skill-core-<tag>.zip`、`qiongli-claude-desktop-skill-economics-<tag>.zip`、`qiongli-claude-desktop-skill-business-<tag>.zip`、`qiongli-claude-desktop-skill-finance-<tag>.zip`、`qiongli-claude-desktop-skill-political-economy-<tag>.zip`、`qiongli-claude-desktop-skill-geoeconomics-<tag>.zip` 或 `qiongli-claude-desktop-skill-economics-accounting-<tag>.zip`。在 Claude Desktop 中，把 ZIP 拖拽到 Skills 上传/安装流程中；也可以打开 `Customize > Skills`，点击 `+`，选择 `Create skill`，再选择 `Upload a skill`。Claude.ai 网页版使用同样的 `Customize > Skills` 上传流程。本阶段公开 Desktop ZIP subjects 是 `core`、`economics`、`business`、`finance`、`political-economy`、`geoeconomics` 和 `economics-accounting`；还没有 standalone accounting Desktop ZIP。

fallback skill ZIP 使用 `coverage=focused`，用于保持当前 180 文件上传预算。它是 subject 专精 Desktop/Web 包，不是降质删减版：保留可执行 workflows、prompts、templates、standards、所选 profiles、`skills-summary.md` 和 `skills-core.md`；专精 ZIP 还包含通过 layered overlays 生成的 selected effective skill markdown。这个 Desktop skill ZIP 是 skill-only asset：只包含 workflows/prompts/templates，不保存 secrets，也不执行 provider calls。完整 canonical source 可通过默认 `coverage=complete` 的 CLI/npm 安装、Codex / Claude Code plugin 包和源码仓库获得。

独立的 Qiongli Literature Provider `.mcpb`（`qiongli-literature-provider.mcpb`）才是 Claude Desktop 本地 provider asset。它在本地运行 Desktop literature search，支持 OpenAlex、Semantic Scholar、Crossref、PubMed 和 arXiv，并通过 Desktop 配置字段填写需要凭据的 provider；arXiv 默认可用，不需要凭据。敏感 key 交给 Claude Desktop sensitive-field handling，不写入 Desktop skill ZIP。Rust Lite 只承诺有界基础检索、规范化/去重、evidence export、Zotero 状态/导入文件与 preview-only route/task planning；领域深搜、citation expansion、Zotero library 检索/写入和 agent execution 需要 Full。这个 MCPB 自带 Rust Lite MCP executable，所以 Desktop 用户不需要安装 Node、Python、`qiongli` CLI 或运行 npm install。标记为 `current-host-only` 的 beta 只能安装到 identity file 所记录的 target triple；这类 beta 不会推进通用 marketplace dist ref。CLI、Codex、Claude Code、Antigravity 和 Hermes 用户仍然可以运行 `qiongli provider setup`，再用 `qiongli provider doctor` 检查当前是 `provider_connected` 还是 `strategy_only`。Desktop 用户需要 `qiongli-literature-provider` MCPB 或平台原生搜索能力，才能声称 `provider_connected`；如果没有 MCPB 或平台原生搜索能力，就把运行记录为 `strategy_only`，并把平台搜索或用户提供的 corpus 作为证据来源。

## 安装后如何使用

安装或升级后，先重启目标客户端。然后使用该客户端暴露的入口：

| 客户端 | 发现方式 | 调用方式 |
|---|---|---|
| Codex | `/skills` 应该能列出 `qiongli`，以及 `qiongli-lit-review` 这类 plugin workflow wrappers | `$qiongli <research task>` 或 `$qiongli-lit-review <topic>` |
| Claude Desktop / Claude.ai | `Customize > Skills` 应该能看到 `qiongli`，或 direct plugin entry 已启用 | 直接用自然语言描述研究任务；当 workflow commands 可见时使用 `/qiongli` |
| Claude Code | Plugin UI、`/plugin` 或全局 command discovery | `/qiongli`、`/paper`、`/lit-review`、`/paper-write`、`/code-build` |
| Shell | `qiongli check` | `qiongli doctor`、`qiongli upgrade`、`python3 -m bridges.orchestrator ...` |

Codex 不暴露自定义 `/qiongli` 或 `/lit-review` slash command。先用 `/skills` 确认主 skill 和生成的 wrapper skills 存在，再用 `$qiongli` 或 `$qiongli-lit-review` 这类 wrapper 调用。每个 wrapper 都会路由到 Claude Code slash command 使用的同一个 canonical workflow 文件。

当客户端 UI 和文件系统状态看起来不一致时，先运行 `qiongli check --json`。它会为每个客户端报告当前安装 surface：`plugin`、`mcp`、`legacy_skill` 或 `none`。运行时/orchestrator 健康检查使用 `qiongli doctor --cwd .`；它也会追加一个非致命的 `Client Integration` 摘要。

如果 Codex 在 Personal plugins 里显示 `qiongli`，但打开详情时显示 `Plugin detail unavailable`，说明 Codex 找到了 marketplace entry，但无法读取本地 plugin payload。常见原因是 `.codex-plugin/plugin.json` 不符合 schema、`skills/qiongli-workflow/SKILL.md` frontmatter 不是合法 YAML，或 plugin 路径缺失。刷新 CLI 管理的本地 plugin：

```bash
qiongli install --target codex --surface plugin --overwrite
```

## Bootstrap Partial

`partial` 用于安装跨客户端 workflow package，不要求 Python：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
```

Windows PowerShell 7+：

```powershell
winget install --id Microsoft.PowerShell --source winget
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.ps1 -OutFile .\bootstrap_qiongli.ps1
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile partial -ProjectDir "$PWD" -Target all
```

`partial` 会安装 workflow assets 和 discovery links，但不会运行完整 runtime validation。

## Bootstrap Full

需要本地验证或 orchestrated task execution 时，用 `full`：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/qiongli/main/scripts/bootstrap_qiongli.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

Windows PowerShell 7+：

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_qiongli.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

`full` 要求 Python 3.12+ 已经在 `PATH` 上。安装器不会安装 Python 或 `mise`。Codex 的完整本地安装入口也可以直接使用：

```bash
qiongli install --profile full --target codex --surface plugin
qiongli mcp doctor --json
```

安装后检查工作区：

```bash
qiongli doctor --cwd .
python3 -m bridges.orchestrator doctor --cwd .
qiongli mcp doctor --json
```

## npm / npx

如果你想用 Node 分发的安装器，并让 workflow payload 跟随 npm 包：

```bash
npm install -g qiongli
qiongli install --target all --surface skills
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project status --project-dir .
```

脚本兼容写法 `qiongli install --target all --surface skills --project-dir "$PWD"` 对默认安装面等价；普通项目 subject 行为仍然交给 `qiongli project ...`。plugin-lite 输出只在 bundled/supported 的位置通过 `--surface plugin` 或 `--surface both` 显式启用。

日常 npm asset / project 使用中，保持当前 npm package 稳定，把 subject 行为保存在 `.qiongli/guidance_manifest.yaml`。如果 manifest 不存在，Qiongli 使用隐式 `active_subject: auto`：先使用 core guidance，再根据当前任务临时推断 subject 或 method lens，并在改变项目本地状态前写出可审计 proposal。

在 npm/npx 下，用这些命令处理免 Python 资产刷新和项目状态：

```bash
qiongli update
qiongli refresh
qiongli upgrade --target all
qiongli project status --project-dir .
```

`qiongli update`、`qiongli refresh` 和 `qiongli upgrade` 都只会从当前已安装 npm package 重新应用 bundled assets；`upgrade` 是覆盖式刷新别名。它们不会升级 npm package，也不会升级完整 Python CLI。

一次性运行：

```bash
npx qiongli@latest install --target all --surface skills
npx qiongli@latest project status --project-dir .
npx qiongli@latest check --json
```

测试 prerelease 时仍可使用 `next` dist-tag：

```bash
npx qiongli@next install --target all --surface skills
```

Advanced compatibility、Desktop ZIP、focused package、release payload 和 install-surface testing 示例：

```bash
qiongli install --subject core --target all --project-dir "$PWD"
qiongli install --subject core --target all --surface skills --project-dir "$PWD"
qiongli install --subject economics --target all --surface skills --project-dir "$PWD"
qiongli install --subject accounting --target all --surface skills --project-dir "$PWD"
qiongli install --subject economics-accounting --target all --surface skills --project-dir "$PWD"
qiongli install --subject economics --coverage focused --target all --surface skills --project-dir "$PWD"
npx qiongli@latest install --subject economics --target all --surface skills --project-dir "$PWD"
npx qiongli@next install --subject economics --target all --surface skills --project-dir "$PWD"
```

如果要完全切换到 marketplace plugin，先移除 CLI 安装产生的资产：

```bash
qiongli remove --target all --dry-run
qiongli remove --target all
```

`qiongli remove` 默认只移除 CLI 安装的全局 workflow assets 和 discovery links。原生 marketplace plugin 仍由安装它的客户端/plugin manager 管理。

对于 npm plugin-lite root，只移除 npm 管理的 plugin assets：

```bash
qiongli remove --surface plugin --target claude
```

npm 会用 `.qiongli-npm-lite.json` 标记自己创建的 plugin-lite root；link 安装会使用 sidecar marker。完整运行时本地 plugin root 使用 `.qiongli-managed.json`，Codex marketplace entry 使用 `metadata.managedBy = "qiongli-cli"`。`remove` 会保留非受管 plugin 目录和 marketplace lite plugin。

## 推荐的 CLI Setup Wizard

通过 pipx、pip 或 bootstrap `full` profile 安装完整运行时后，先运行交互式 setup wizard，再手写安装参数：

```bash
pipx install qiongli
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
qiongli setup --provider-mode prompt --no-browser
```

wizard 会引导 CLI、Codex、Claude Code 和 Antigravity 用户完成：

- setup path：`install` 用于首次安装内置 assets，`upgrade` 用于从上游刷新
- runtime surface：CLI、Codex、Claude Code、Antigravity 或 multi-platform
- subject 选择
- coverage 选择：`complete` 或 `focused`
- install mode：普通用户使用 `--mode copy`，本地 checkout 开发使用 `--mode link`
- install scope：`all`、`globals`、`project` 或 `cli`
- 启用 CLI wrapper 时的 CLI 目录
- overwrite 策略：需要替换已管理安装时使用 `--overwrite`；升级但保留现有 managed files 时使用 `--no-overwrite`
- upgrade source：latest stable、latest beta、显式 `--ref` tag、显式 `--ref-type branch`，以及可选 `--repo`
- 可选 literature provider setup。默认会打开一个本地浏览器页面，一次配置 OpenAlex、Semantic Scholar、Crossref、PubMed，并展示 arXiv 不需要 API key；远程终端可用 `--no-browser` 只打印 URL，也可以用 `--provider-mode prompt` 回到终端逐项录入，或用 `--provider-mode skip` 跳过 provider setup
- doctor verification，除非设置 `--no-doctor`

每一步 prompt 都会打印简短的 `Tip:` 注释，解释这个选择会改变什么；不熟悉完整 CLI 参数的用户也可以按引导完成安装或升级。

通过 setup 保存的 provider 密钥使用与 `qiongli provider setup` 和 `qiongli provider doctor` 相同的 provider 配置。本地页面包含各 provider 获取 key 的链接和步骤；arXiv 标注为无需 API key。密钥会保存在生成的研究 artifacts 之外。provider 步骤用于配置凭据并执行 doctor/capability 检查；它不保证一定产生外部检索结果。

在 npm/npx 下，`qiongli setup` 是客户端资产安装快捷入口，不是完整交互式 wizard。它仍然属于面向客户端资产的免 Python 资产管理器路径。需要完整运行时命令时，先安装完整运行时：`pipx install qiongli`。如果只需要 Node-only asset installer，继续使用显式 `qiongli install ...` 命令。

## pipx / pip

如果你明确需要 Python 分发的 updater CLI，用 pipx：

```bash
pipx install qiongli
qiongli setup
qiongli install --target all
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project status --project-dir .
```

`qiongli setup` 可以交互式引导安装。脚本化安装可以用 `qiongli install --target auto` 只刷新检测到的客户端 CLI，也可以用 `--target all` 明确写入所有支持的平台路径；项目 subject 行为用 `qiongli project set-subject` 修改。

升级完整运行时：

```bash
qiongli update
qiongli update --dry-run
qiongli update --yes
```

在交互式终端中，裸 `qiongli update` 会打开 wizard，询问 stable/beta channel、刷新 target、surface、profile、是否刷新/检查，以及确认行为。手动 package-manager 更新仍然可用：运行 `pipx upgrade qiongli` 或 `python -m pip install --upgrade qiongli`，然后用 `qiongli install --target auto --surface plugin --profile full --overwrite` 刷新客户端安装面。`qiongli upgrade` 仍是你明确想下载上游 GitHub release 时使用的 release archive refresh 路径。

`--subject` 默认是 `core`，`--coverage` 默认是 `complete`，但 subject 安装参数是 advanced compatibility 控制项。它们用于有意选择的精简安装、Desktop/Web 等价包、release payload validation 和 install-surface testing。当前官方 subjects 是 `core`、`economics`、`accounting`、`business`、`finance`、`political-economy`、`geoeconomics` 和命名 composite subject `economics-accounting`；`political-economy` 和 `geoeconomics` 是两个独立 subject 选择，不是一个 composite。官方 composite subjects 不是任意逗号分隔叠加。普通项目 subject 行为用 `qiongli project set-subject` 修改；installed subject 或 coverage 只在有意刷新专精安装包时改变。`qiongli check --json` 会输出每个 target 当前安装的 subject 和 coverage；旧安装缺少 `SUBJECT_MANIFEST.json` 或 `SUBJECT` 文件时按 legacy `core` / `complete` 处理。

先创建 custom scaffold，再 materialize 本地 overlays：

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
```

本地自定义 overlays 通过源码 materializer 支持：

```bash
python3 scripts/materialize_subject_package.py \
  --subject economics \
  --coverage complete \
  --source . \
  --custom-dir /path/to/custom-qiongli \
  --out /tmp/qiongli-workflow
```

这个路径适合添加本地 overlays、profiles、registry entries 或 custom skill markdown。Custom overlays 只影响 generated output，不会改写 canonical source files。`qiongli customize` 加 `--custom-dir` materialization 面向 Python/source checkout 工作流；npm runtime installs 在这个阶段使用预生成 payloads，不支持 `--custom-dir`。

## 实际会安装什么

根据入口不同，Qiongli 可能安装：

- 客户端 home 目录下的 `qiongli-workflow` skill assets，用户侧显示为 `qiongli`
- 在支持该 discovery 模型的客户端中，提供 `/paper`、`/lit-review`、`/paper-write`、`/code-build` 等 workflow command discovery links
- shell 命令 `qiongli`、`ql` 和兼容别名 `research-skills`、`rsk`、`rsw`
- 只有显式运行 `qiongli init --project-dir .` 时才写入的项目集成文件

默认不会写入项目本地文件。全局 workflow package 可在任意研究工作区使用。

完整调用细节见 [使用 Agent Skills](/zh/guide/using-agent-skills)。

## 保持版本一致

如果你同时使用多个安装面，保持 plugin、global skill assets、npm payload 和 Python CLI 一致。对于 Python 完整运行时：

```bash
qiongli check
qiongli update
qiongli update --dry-run
qiongli update --yes
```

在 npm/npx 下，不使用这些完整运行时参数；用 `qiongli update`、`qiongli refresh` 或 `qiongli upgrade --target all` 从当前 package 刷新 assets。

如果你已经完全转向原生 plugin，不再需要旧的全局 skill 目录或 slash discovery，先 dry-run 清理：

```bash
qiongli remove --target all --dry-run
```
