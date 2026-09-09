# CLI 命令参考（qiongli）

> 原生 2.x 的命令以 [CLI 安装与命令指南](../../guide/cli-2x.md)为准；不需要 App。

> **旧产品线：**本页描述 Qiongli 1.x 的 Python、npm 与 bootstrap/shell CLI，不是原生
> 2.x CLI 契约。2.x CLI 只能按 [2.x Alpha 安装说明](../../alpha/install-2x.md)安装和验证。

本文件整理本仓库所有“可执行入口”（pipx CLI / Python module / Bash scripts），用于本地与 GitHub CI 保持一致的调用方式。

## 0) 命令名约定

- `qiongli`：主 CLI（pipx/venv 安装后可用，或通过 shell bootstrap 安装）
- `ql`：短主别名。`research-skills`、`rsk`、`rsw`：旧兼容别名，与 `qiongli` 等价

下文统一用 `qiongli` 作为示例。

---

## 1) Upstream（上游仓库）如何确定（如何省略 `--repo`）

很多命令需要知道“去哪个 GitHub 仓库查询/下载 release”。`qiongli` 的上游解析优先级如下（从高到低）：

1. CLI 参数：`--repo <owner/repo|Git URL>`
2. 环境变量：`QIONGLI_REPO=<owner/repo|Git URL>`
3. 旧环境变量 fallback：`RESEARCH_SKILLS_REPO=<owner/repo|Git URL>`
4. 项目配置文件（从当前目录或 `--project-dir` 向上搜索）：
   - `qiongli.toml`
   - `.qiongli.toml`
5. 打包默认（pipx 安装的包内）：`qiongli/project.toml`（由 CI 注入）
6. 如果你正在 `qiongli` 仓库 clone 内运行：从 git remote 推断（优先 `upstream`，其次 `origin`）

支持的 repo 形式：

- `owner/repo`
- `https://github.com/owner/repo.git`
- `git@github.com:owner/repo.git`

推荐把上游提交到你的项目仓库（适合 CI）：

```toml
# qiongli.toml
[upstream]
repo = "owner/repo"   # 或 url = "https://github.com/owner/repo.git"
```

---

## 2) `qiongli`（安装/升级器 CLI）

这个 CLI 现在有三种分发方式：
- Python CLI：通过 `pip`/`pipx` 安装
- Shell CLI：由 `bootstrap_qiongli.sh` 默认安装到 `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}`
- npm launcher：通过 `npm install -g qiongli` 安装；它是面向 `install`、`setup`、`update`、`refresh`、`upgrade`、`remove`、`check`、`clean`、`runtime doctor` 和 `project ...` 的免 Python 资产管理器；完整运行时命令需要 `pipx install qiongli`

### 2.0 默认安装模型

Python 完整运行时默认是 plugin-first。最短命令：

```bash
qiongli install
qiongli upgrade
```

在 Python 完整运行时中会展开为完整本地 plugin surface：

| 接口 | 默认值 |
|---|---|
| Target | `--target all` |
| Subject package | `--subject core --coverage complete` |
| Runtime profile | `--profile full`；如果显式写了 `--surface skills` 且没有指定 profile，则 CLI 使用 `partial` |
| Output surface | `install` 和 `upgrade` 默认 `--surface plugin` |
| Install mode | `--mode copy` |
| Project directory | 当前工作目录；只有请求 project-facing parts 时才会写项目文件 |
| Shell CLI wrapper | effective `full` profile 会启用；用 `--no-cli` 跳过 wrapper 刷新 |
| MCP registration | effective `full` profile 会启用；Codex/Claude 的 plugin-owned MCP entry 会跳过，因为本地 plugin 自己持有 MCP 启动配置 |
| Doctor | 直接运行 `install` / `upgrade` 时默认不跑；显式传 `--doctor` 或在 `--parts` 里包含 `doctor` 才运行 |
| Overwrite | `install` 默认不覆盖；`upgrade` 默认覆盖，并支持 `--no-overwrite` |

`--surface` 是高层输出形态选择：

| Surface | 安装内容 |
|---|---|
| `plugin` | Codex / Claude Code / Antigravity 的 CLI-managed local plugin；Antigravity plugin 包含 root `mcp_config.json`；target 包含 Hermes 时写入受管理 MCP config |
| `skills` | 旧版全局 `qiongli-workflow` skill 目录，以及客户端支持的 workflow discovery |
| `both` | 同时安装旧版全局 skills 和本地 plugin surface |

`--parts` 是精确覆盖。在完整运行时中，它会替代由 surface/profile 推导出的安装集合，只执行你列出的 `globals`、`plugin`、`project`、`cli`、`mcp` 或 `doctor`。npm/npx 只接受免 Python 资产 parts：`globals`、`project`、`cli` 和 `mcp`；npm plugin-lite assets 使用 `--surface plugin`，不要用 `--parts plugin`。`all` 和 `*` 会展开成该运行时支持的全部 parts。

npm/npx 不同：它是免 Python 资产管理器，默认 `--surface skills`；只有 bundled/supported 的 plugin-lite 输出需要用 `--surface plugin` 或 `--surface both` 显式启用。npm 的 `update`、`refresh` 和 `upgrade` 只会从当前已安装 npm package 重新应用 bundled assets，不会升级 npm package 或完整 Python CLI。

### 2.1 `qiongli check`（检查版本/是否有更新）

用途：
- 输出 CLI 版本、本地 repo 版本（若在仓库内运行）、受支持客户端安装 surface 的已安装版本
- 可选：查询上游最新 release tag，并判断是否需要升级

```bash
qiongli check [--repo <owner/repo|url>] [--json] [--strict-network]
```

关键参数：
- `--repo`：指定上游（可省略，见“上游解析”）
- `--json`：只输出 JSON（便于 CI/脚本）
- `--strict-network`：如果上游查询失败则返回失败（默认仅提示并继续）

`qiongli check` 现在会识别 plugin-first 安装形态：Codex / Claude Code / Antigravity 的 CLI-managed local plugin 会显示 `surface=plugin`，Hermes 或只安装 MCP 的受管理 config 会显示 `surface=mcp`，旧版全局 skill 目录会显示 `surface=legacy_skill`，未发现时显示 `surface=none`。JSON 仍保留兼容字段 `installed`、`version`、`subject`、`coverage` 和 `path`，同时增加 `plugin`、`skill`、`mcp` 三个诊断对象。可通过客户端 CLI 查询时，plugin 诊断还会包含 `active`、`enabled`、`plugin_id` 和 `activation_detail`，用于区分“文件已安装”和“客户端已启用 plugin”。旧 managed install 如果没有 `SUBJECT_MANIFEST.json` 或 `SUBJECT` marker，会按 legacy `core` / `complete` 处理。

对于 Codex 插件安装，`mcp` 表示当前有效 MCP 来源。`plugin_mcp` 表示插件本地 `.mcp.json`，`standalone_mcp` 表示 `~/.codex/config.toml`。当 `plugin_mcp.installed` 为 true 时，`standalone_mcp.installed` 为 false 是预期状态。只有在重启 Codex 后仍看不到插件内置 MCP 时，才使用 standalone MCP fallback。

如果 Codex Personal marketplace 里能看到 `qiongli`，但打开详情时显示 `Plugin detail unavailable`，说明 marketplace entry 被发现了，但 Codex 无法读取本地 plugin payload。先看 `qiongli check --json` 输出的本地 plugin root；常见原因是 `.codex-plugin/plugin.json` 不符合当前 schema、`skills/qiongli-workflow/SKILL.md` frontmatter 不是合法 YAML，或本地路径缺失。刷新 CLI 管理的 Codex plugin：

```bash
qiongli install --target codex --surface plugin --overwrite
```

Codex plugin-first 安装会把 Qiongli 暴露成名为 `qiongli` 的主 `/skills` 入口，并生成 `qiongli-lit-review`、`qiongli-academic-write`、`qiongli-paper-read` 等 workflow wrapper skills。打包进去的 `commands/*.md` 文件仍用于跨客户端一致性，但当前 Codex 不会把它们显示成独立的 `/lit-review`、`/academic-write` 或 `/paper` slash command。请使用 `$qiongli-lit-review <topic>`、`$qiongli run lit-review on <topic>`，或直接输入自然语言学术请求。

退出码约定：
- `0`：无更新/或跳过上游检查
- `1`：检测到更新可用
- `2`：参数错误

### 2.2 `qiongli setup`（交互式 CLI setup wizard）

用途：
- 通过 pipx、pip 或 bootstrap `full` 安装完整运行时后的推荐第一个命令。
- 引导 CLI、Codex、Claude Code、Antigravity 和 Hermes 用户选择 install/upgrade、runtime surface、subject、coverage、install mode、install scope、overwrite 策略、upgrade source、可选 provider key setup，并执行 doctor verification。

```bash
qiongli setup [--project-dir <path>] [--dry-run] [--no-doctor] [--provider-mode page|prompt|skip] [--no-browser]
```

示例：

```bash
pipx install qiongli
qiongli setup
qiongli setup --dry-run
qiongli setup --project-dir "$PWD" --no-doctor
qiongli setup --provider-mode prompt --no-browser
```

在 npm/npx 下，`qiongli setup` 是客户端资产安装快捷入口，不是完整交互式 wizard。它仍然属于面向客户端资产的免 Python 资产管理器路径。需要 `qiongli mcp serve --transport stdio`、`qiongli provider setup` 或 `qiongli customize` 这类完整运行时命令时，先安装完整运行时：`pipx install qiongli`。显式 `qiongli install ...` npm 命令仍可用于 Node-only asset installation。

wizard 选项：
- Setup path：`install` 或 `upgrade`。
- Runtime surface：`cli`、`codex`、`claude-code`、`antigravity`、`hermes` 或 `multi-platform`。
- Subject：`core`、`economics`、`accounting`、`business`、`finance`、`political-economy`、`geoeconomics` 或 `economics-accounting`。
- Coverage：`complete` 或 `focused`。
- Install mode：普通用户用 `--mode copy`，本地开发 checkout 用 `--mode link`。
- Install scope：`all`、`globals`、`project` 或 `cli`。
- 所选 scope 包含 CLI wrapper 时的 CLI 目录。
- Overwrite 策略：install refresh 使用 `--overwrite`；升级但不替换 managed files 时使用 `--no-overwrite`。
- Upgrade source：latest stable、latest beta、可选 `--repo`、显式 `--ref`，以及 `--ref-type tag|branch`。
- 可选 literature provider setup。默认 `page` 模式会打开一个本地浏览器页面，一次配置 OpenAlex、Semantic Scholar、Crossref、PubMed，并显示 arXiv 不需要 API key；远程终端可用 `--no-browser` 只打印本地 URL，也可以用 `--provider-mode prompt` 回到终端逐项录入，或用 `--provider-mode skip` 跳过 provider setup。
- Doctor verification，除非设置 `--no-doctor`。

每一步 prompt 都包含简短的 `Tip:` 注释，解释这个选择为什么重要，以及会改变哪种安装或升级行为。

通过 setup 保存的 provider 密钥使用与 `qiongli provider setup` 和 `qiongli provider doctor` 相同的 provider 配置。本地设置页包含各 provider 获取 key 的链接和步骤；arXiv 标注为无需 API key。密钥保存在生成的研究 artifacts 之外。setup 会配置凭据并执行 doctor/capability 检查；它不承诺一定会运行外部 literature search。

### 2.2.1 `qiongli update` / `qiongli self-update`（完整运行时自更新）

用途：
- 适用于 `pipx install qiongli` 这类完整运行时安装。
- npm/npx 中的普通 `qiongli update` 只是资产刷新；下面这些旧参数不属于免 Python npm 资产流。
- 通过最初安装它的 package manager 更新 CLI package。
- package 更新成功后，用新 package 内置 payload 刷新本地 full plugin / MCP 安装面。
- 不管理原生 marketplace plugin；Codex、Claude Code 或对应客户端安装的 marketplace plugin 仍由客户端 plugin manager 管理。

完整运行时的常规升级使用 `qiongli update`。它会先检查当前安装的 Python qiongli CLI/package 是否有新版本；如有，会询问是否升级。CLI/package 升级成功后，它会再询问是否用新 package 内的 payload 刷新本地 plugin/assets。脚本或 CI 使用 `qiongli update --yes`，它会把两个确认都视为 yes。只想升级完整运行时 package、不刷新本地内容时使用 `qiongli update --no-refresh`。

```bash
qiongli update [--channel stable|next] [--beta] [--dry-run] [--yes] [--no-refresh] [--skip-check]
qiongli self-update [--channel stable|next] [--beta] [--dry-run] [--yes] [--no-refresh] [--skip-check]
```

默认行为：
- 在交互式终端中，裸 `qiongli update` 或 `qiongli self-update` 会打开一个简短 wizard，询问 channel、刷新 target、surface、profile、是否刷新/检查，以及确认行为。只要传入任意显式参数，就走脚本化 CLI 路径。
- `--channel stable` 会根据检测到的完整运行时安装渠道委托给 `pipx upgrade qiongli` 或 `python -m pip install --upgrade qiongli`。
- `--channel next` 会在 Python 包管理器路径上启用 prerelease `--pre`。
- `--beta` 是 `--channel next` 的快捷别名。
- 刷新安装面默认执行 `qiongli install --target all --surface plugin --profile full --overwrite`。这是有意设计：package manager 已经更新了 CLI package，本地 payload 已经是新的，不需要再下载 release archive。
- 交互式 wizard 的刷新 target 默认是 `auto`：检测 `PATH` 上已安装的 Codex、Claude Code、Antigravity 和 Hermes CLI，只刷新检测到的客户端安装面。
- `--dry-run` 只打印检测到的渠道和将要执行的命令。
- 不传 `--yes` 时，会先询问是否执行 package-manager 更新；CLI/package 升级成功后，再询问是否刷新本地 plugin/assets。
- `--no-refresh` 跳过安装面刷新，`--skip-check` 跳过最后的 `qiongli check`。

源码 checkout 不会自我修改。检测到 source mode 时，先用 `git pull` 更新源码，再运行 `qiongli install --overwrite` 刷新需要的安装面。

### 2.2.2 `qiongli doctor`（运行时和客户端集成健康检查）

用途：
- 对指定项目目录运行 Python orchestrator doctor。
- 使用与 `qiongli check` 相同的 plugin/MCP/legacy skill discovery，追加一个非致命的 client integration 摘要。

```bash
qiongli doctor --cwd .
```

`doctor` 检查 project 文件、provider/orchestrator readiness、本地模型 CLI 等运行时条件。它不会安装或删除 plugin。缺少可选客户端集成只会出现在摘要里，不会单独让 `doctor` 失败；退出码仍然来自 orchestrator doctor。

### 2.2.3 `qiongli mcp`（跨平台 MCP server）

用途：
- 需要完整运行时。npm/npx 不会运行统一 MCP server。
- 为支持 MCP 的桌面或 agent 客户端启动本地 Qiongli MCP server。
- 让 CLI 用户和 Desktop-only 用户配置同一套 provider key。
- 生成不包含 secret 的客户端配置示例。

```bash
qiongli mcp serve --transport stdio
qiongli mcp serve --transport http --host 127.0.0.1 --port 8765
qiongli mcp configure --provider openalex --field email --value you@example.com
qiongli mcp doctor --json
qiongli mcp config example --target codex --json
qiongli mcp config example --target claude-code --json
qiongli mcp config example --target antigravity --json
qiongli mcp config example --target hermes --json
qiongli mcp wizard
```

full Python server 暴露的 MCP tools 包括 `qiongli_literature_status`、`qiongli_search_plan`、`qiongli_literature_search`、`qiongli_literature_export_evidence`、`qiongli_config_status`、`qiongli_save_provider_config`、`qiongli_configure_provider`、`qiongli_open_config_wizard`、`qiongli_list_provider_env`、`qiongli_test_provider`、`qiongli_collect_evidence`、`qiongli_subject_status`、`qiongli_subject_update`、`qiongli_orchestrator_route`、`qiongli_orchestrator_doctor`、`qiongli_task_plan` 和 `qiongli_task_run`。其中 `qiongli_collect_evidence` 是 filesystem / builtin / external-command evidence adapter；不要用它判断 OpenAlex、Semantic Scholar、Crossref、PubMed 或 arXiv 的 provider config。直接传入 `openalex` 这类 provider name 时，它检查的是 `RESEARCH_MCP_<PROVIDER>_CMD` 外部命令。

默认 `stdio` 模式是本地进程，不需要远端 server。Codex、Claude Code、Antigravity、Hermes 或其他本地 MCP client 可以先调用 `qiongli_orchestrator_route`，决定是否从 skill-only routing 升级到 full orchestrator tools。`qiongli_task_run` 默认是 preview mode；只有 MCP caller 显式传入 JSON boolean `run_agents: true` 时，才会启动本地模型 CLI。

### 2.3 `qiongli install`（安装包内 subject payload）

用途：
- 把 PyPI/npm/source checkout 内携带的 subject payload 安装成当前本地 Qiongli surface。
- 在 Python 完整运行时中，默认是完整 plugin surface：Codex / Claude Code / Antigravity 本地 plugin、Antigravity plugin 内置 MCP config、Hermes 受管理 MCP config，并刷新 shell CLI wrapper，除非显式传 `--no-cli`。
- 在 npm/npx 中，默认是 skills surface；plugin-lite 输出只在 bundled/supported 的位置通过 `--surface plugin` 或 `--surface both` 显式启用。
- 默认不会迁移或删除旧版 global skills；自动迁移走 `qiongli upgrade`，显式清理走 `qiongli remove`。

```bash
qiongli install \
  [--profile partial|full] \
  [--subject core|economics|accounting|business|finance|political-economy|geoeconomics|economics-accounting] \
  [--coverage complete|focused] \
  [--target codex|claude|antigravity|hermes|all|auto] \
  [--surface skills|plugin|both] \
  [--mode copy|link] \
  [--project-dir <path>] \
  [--install-cli | --no-cli] \
  [--cli-dir <path>] \
  [--overwrite] \
  [--doctor] \
  [--parts globals,plugin,project,cli,mcp,doctor] \
  [--dry-run]
```

示例：

```bash
qiongli install --target all
qiongli install --target auto
qiongli install --surface skills --profile partial --target all
qiongli install --profile full --target codex --surface plugin
qiongli install --profile full --target all --surface plugin
qiongli install --profile full --target all --surface both
qiongli install --parts mcp --target hermes
```

日常 CLI / local plugin 使用中，先安装一次 Qiongli，再用 `qiongli project ...` 在每个项目里设置 subject 行为。`--subject` 安装参数保留给 legacy 和 advanced compatibility 场景：focused Claude Desktop/Web ZIP、有意选择的窄包、release payload，以及安装面兼容性测试。

`--target auto` 会检测 `PATH` 上已安装的受支持客户端 CLI，只安装这些客户端对应的 surface。明确想写入所有支持平台路径时，继续使用 `--target all`。

Subject package 是专精安装包，不是降质删减版。默认安装是 `core/complete`。`--subject economics`、`--subject business`、`--subject finance`、`--subject political-economy` 和 `--subject geoeconomics` 表示 complete 专精安装，不是缩水包；`--subject accounting` 表示 `accounting/complete`，即全量框架加 accounting 专精。`focused` coverage 只选择该 subject 的 profiles 和 active effective skills，用于有意选择的精简安装和 Desktop/Web ZIP。当前官方 subjects 是 `core`、`economics`、`accounting`、`business`、`finance`、`political-economy`、`geoeconomics` 和命名 composite subject `economics-accounting`；`political-economy` 和 `geoeconomics` 是两个独立 subject 选择，不是一个 composite。官方 composite subjects 不是任意逗号分隔叠加。本阶段公开 Desktop ZIP subjects 是 `core`、`economics`、`business`、`finance`、`political-economy`、`geoeconomics` 和 `economics-accounting`，还没有 standalone accounting Desktop ZIP。普通项目的 subject 行为用 `qiongli project set-subject` 修改；只有在明确刷新专精安装包时，才通过 `install` 或 `upgrade` 切换 installed subject 或 coverage。

在 Python 完整运行时中，从 v1.9.0 开始，`qiongli install` 默认等价于 `--profile full --surface plugin`。Codex 会写入 personal marketplace entry，把 plugin payload 放到 `~/plugins/qiongli`，写入启动 `qiongli mcp serve --transport stdio` 的 plugin `.mcp.json`，并在 Codex CLI 可用时执行 `codex plugin add qiongli@personal`。Claude Code 会在 `~/.qiongli/plugins/claude-code` 写入本地 marketplace，把 plugin payload 放到 `plugins/qiongli`，并在 Claude CLI 可用时执行 `claude plugin marketplace add ...` 和 `claude plugin install qiongli@qiongli-local --scope user`。Antigravity 会在 `~/.qiongli/plugins/antigravity/qiongli` 写入带 root `plugin.json` 的 plugin bundle，并在 plugin 根目录写入 full MCP server 配置 `mcp_config.json`，在 CLI 可用时执行 `antigravity plugin install <path>`。`--target all` 会让 Codex / Claude Code / Antigravity 使用本地 plugin，同时给 Hermes 写入受管理的 full MCP client 配置。Marketplace 安装的 plugin 仍然保持 lite/no-Python 路径，使用内置 Rust Lite literature provider。需要旧版 skills-only 布局时，使用 `--surface skills --profile partial`。npm/npx 默认 skills，只有显式传 `--surface plugin` 或 `--surface both` 时才写 plugin-lite assets。

安装行为细节：
- `--surface plugin --target all` 会给 Codex / Claude Code / Antigravity 安装 CLI-managed local plugin；Antigravity plugin 会包含 root `mcp_config.json`，Hermes 会写入受管理的 client-level MCP config。
- `--surface skills --profile partial` 只安装旧版 skill 目录和 workflow discovery；除非使用 `--parts cli` 或 `--parts mcp`，否则不会安装 shell wrapper 或 MCP config。
- `--surface both` 会保留旧版 global skills，同时安装本地 plugin；只有明确需要两个发现路径同时存在时才使用。
- `--parts` 优先于 `--surface`。例如 `--parts mcp --target antigravity` 只写 Antigravity MCP config，`--parts project` 只写项目侧文件。
- 直接运行 `qiongli install` 时，`--doctor` 必须显式传入。交互式 setup wizard 可能默认建议 doctor verification，但 `install` 命令本身不会自动跑 doctor。

### 2.4 `qiongli upgrade`（刷新 assets）

用途：
- 在 Python 完整运行时中，下载上游 release（默认 latest tag 的 tar.gz）、解压后运行包内 Python installer
- 在 npm/npx 中，只从当前已安装 npm package 重新应用 bundled assets；不会下载 release archive，也不会升级 npm package
- 完整运行时默认使用完整本地 plugin surface，并在新安装成功后迁移清理旧 global skills 和 Codex/Claude 独立 MCP config

```bash
qiongli upgrade \
  [--repo <owner/repo|url>] \
  [--ref <tag-or-branch>] \
  [--ref-type tag|branch] \
  [--profile partial|full] \
  [--subject core|economics|accounting|business|finance|political-economy|geoeconomics|economics-accounting] \
  [--coverage complete|focused] \
  [--target codex|claude|antigravity|hermes|all|auto] \
  [--surface skills|plugin|both] \
  [--project-dir <path>] \
  [--install-cli | --no-cli] \
  [--cli-dir <path>] \
  [--overwrite | --no-overwrite] \
  [--doctor] \
  [--parts globals,plugin,project,cli,mcp,doctor] \
  [--dry-run]
```

说明：
- `qiongli upgrade` 是内容/assets 刷新命令，不会升级 npm、pipx 或 pip 中安装的 qiongli CLI package。在 npm/npx 下，`update`、`refresh` 和 `upgrade` 都只从当前 npm package 重新应用 assets；当前 parser 把 `upgrade` 当作覆盖式刷新别名。指定上游 release archive、channel/package self-update 和 `qiongli self-update` 属于完整运行时：先 `pipx install qiongli`。
- `--project-dir` 主要在你显式请求项目侧安装面时生效，例如 `--parts project`。
- 完整运行时的 `upgrade` 等价于 `--profile full --surface plugin`：刷新 Codex / Claude Code / Antigravity 本地 plugin，把 Antigravity MCP config 打包进 plugin，给 Hermes 写入受管理 MCP config，并在安装成功后清理旧 global skills 和 Codex/Claude/Antigravity 独立 MCP config。
- 如果要保留旧版 skills-only 升级路径，使用 `qiongli upgrade --surface skills --profile partial ...`。
- 如果明确想让旧版 global skills 和 plugin surface 并存，使用 `qiongli upgrade --surface both ...`；这个路径不会执行 plugin migration cleanup。
- migration cleanup 只会在 effective `--surface plugin` 升级成功后运行，并且 selected parts 为空或包含 `plugin`。安装失败时绝不会删除旧资产。
- migration cleanup 会删除旧版全局 `qiongli-workflow` skill 目录、Claude Code workflow discovery links，以及 Codex/Claude/Antigravity 独立 MCP config。Hermes MCP config 会保留，因为 Hermes 仍然使用 client-level 受管理 MCP config。
- 项目接线建议走 `qiongli init --project-dir .`；如果确实要在升级时重写项目文件，再显式加 `--parts project`。
- `--subject` 默认是 `core`，`--coverage` 默认是 `complete`；subject 安装参数主要用于刷新专精安装包、focused Desktop/Web ZIP payload 或兼容性测试。普通项目级 subject 选择请使用 `qiongli project set-subject`。
- 示例：`qiongli upgrade --subject accounting --target all`。
- 示例：`qiongli upgrade --target all` 会刷新本地 full plugin surface，不会切换到 marketplace lite plugin。
- 只有 legacy skills-only 升级路径会创建 Claude Code 工作流发现 symlink：`~/.claude/commands/*.md`，可直接使用 `/paper`、`/lit-review` 等 slash 命令。
- Shell CLI 会通过随附的 bootstrap helper 执行升级。完整 plugin/MCP 路径要求可用的 Python-backed `qiongli` runtime，因为 plugin 会启动 `qiongli mcp serve --transport stdio`。
- 退出码为底层安装器返回码（若安装失败，沿用其错误码）。

### 2.5 `qiongli align`（快速参考）

用途：打印“pipx 安装了什么 / upgrade 会修改哪些路径 / 常见用法”。

```bash
qiongli align [--repo <owner/repo|url>]
```

### 2.6 `qiongli init`（项目初始化）

用途：在项目目录中创建 `.env` 等项目配置。

```bash
qiongli init \
  [--project-dir <path>] \
  [--target all|auto|codex|claude|antigravity|hermes] \
  [--mode copy|link] \
  [--overwrite] \
  [--doctor] \
  [--parts project,doctor] \
  [--dry-run]
```

说明：
- 默认等价于 `--parts project`，只创建 project-facing assets（`.env`）。除非显式传 parts，否则不会触碰 global skill 目录、本地 plugin 或 MCP config。
- 可重复运行；除非显式传 `--overwrite`，否则不会覆盖已有文件。

### 2.7 `qiongli remove`（移除 CLI 安装的资产）

用途：移除 CLI 安装产生的资产，方便在 npm/PyPI/bootstrap 安装和原生 marketplace plugin 之间切换。

```bash
qiongli remove \
  [--target codex|claude|antigravity|hermes|all|auto] \
  [--surface skills|plugin|both] \
  [--parts globals|project|cli|mcp|plugin] \
  [--project-dir <path>] \
  [--cli-dir <path>] \
  [--dry-run]
```

示例：

```bash
qiongli remove --target all --dry-run
qiongli remove --target codex
qiongli remove --target codex --surface plugin
qiongli remove --parts globals,project --project-dir "$PWD"
qiongli remove --parts cli --cli-dir ~/.local/bin
qiongli uninstall --target all
qiongli delete --target claude
```

说明：
- `remove` 默认等价于 `--parts globals`，会移除 CLI 安装的 `qiongli-workflow` skill 目录和生成的 workflow discovery links。
- 如果某个 `qiongli-workflow` 目录不像 Qiongli package payload，会跳过，避免删除用户自建内容。
- npm/npx plugin removal 只删除带 `.qiongli-npm-lite.json` 或 link-mode sidecar marker 的 npm-managed plugin-lite root。
- 完整运行时 plugin removal 只删除带 `.qiongli-managed.json` 的 CLI-managed 本地 full plugin root，以及带 `metadata.managedBy = "qiongli-cli"` 的 Codex marketplace entry。
- `--surface plugin` 只删除 CLI-managed local plugin surface，不会删除 MCP client config；需要删 MCP config 时使用 `--parts mcp`。
- `--surface both` 会删除旧版 global skills 和 CLI-managed local plugin，但仍不会删除 MCP config，除非同时包含 `--parts mcp`。
- 它不会卸载 `qiongli` 或 `qiongli-next` 这类 marketplace plugin；这些需要在 Codex、Claude Code 或 Claude Desktop 的 plugin manager 中移除。
- 需要同时清理旧项目本地文件时，使用 `--parts project`。
- 只有通过 full CLI/bootstrap 安装过 shell wrapper 时，才需要使用 `--parts cli`。

### 2.8 `qiongli clean`（清理过期资产）

用途：移除旧版本安装留下的项目本地资产。

```bash
qiongli clean [--project-dir <path>] [--dry-run] [--globals]
```

参数说明：
- `--project-dir`：要清理的目录（默认当前目录）。
- `--globals`：同时移除全局工作流发现 symlink（例如 `~/.claude/commands/`）。也会清理旧版本遗留的 Gemini workflow symlink；只移除指向 `qiongli-workflow` 的 symlink，用户自建的命令不受影响。
- `--dry-run`：只显示将要移除的内容，不实际删除。

### 2.9 `qiongli doctor`（环境预检）

```bash
qiongli doctor [--cwd <path>]
```

### 2.10 `qiongli customize`（创建 custom subject overlay）

用途：
- 为 Python/source checkout materialization 工作流创建本地 custom overlay scaffold。
- Custom overlays 只影响 generated output，不会改写 canonical source files。
- npm runtime installs 在这个阶段使用预生成 payloads，不会在 install 时 materialize `--custom-dir` overlays。

```bash
qiongli customize --subject economics --name my-econ-lab --out ./qiongli-custom/econ-lab
python3 scripts/materialize_subject_package.py --subject economics --custom-dir ./qiongli-custom/econ-lab --source . --out /tmp/qiongli-workflow
```

开发或加深一个 subject 时，需要同步更新 `subjects/catalog.yaml`、subject overlays、subject-specific registry and markdown、选定的 domain and venue profiles、subject eval fixtures、specialization audit expected terms、materializer tests、基于 staged materialization 的 npm package contract tests（当该 subject 可通过 npm 安装时），以及该 subject 有 Desktop/Web artifact 时的 release validation。

---

## 3) 编排器 CLI：`python3 -m bridges.orchestrator`

这是“三端并发/降级 + task-run 标准合同落盘”的执行入口。

```bash
python3 -m bridges.orchestrator <mode> [args...]
```

mode 列表：

- `doctor`：环境预检
  ```bash
  python3 -m bridges.orchestrator doctor --cwd .
  ```
- `parallel`：三端并发分析 + 总结端综合（自动降级为双端/单端）
  ```bash
  python3 -m bridges.orchestrator parallel \
    --prompt "Analyze this study design" \
    --cwd . \
    --summarizer claude \
    --profile-file standards/agent-profiles.example.json \
    --profile default
  ```
- `task-run`：按 Task ID 跑标准链（plan -> evidence -> draft -> review -> gates -> 写入 RESEARCH/）
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id F3 \
    --paper-type empirical \
    --topic your-topic \
    --cwd . \
    --triad
  ```
  常用可选参数：
  - `--domain <name>`：把运行时领域 profile（例如 `econ`、`cs`、`psychology`）注入 task packet 和 prompts
  - `--venue <name>` / `--context <text>`
  - `--mcp-strict` / `--skills-strict`
  - `--profile-file <path>` + `--profile <name>`（以及 `--draft-profile` / `--review-profile` / `--triad-profile`）
  - `--focus-output <path>`（可重复）+ `--output-budget <n>`：把本次运行收敛到更小的 active outputs，其余 contract outputs 明确标记为 deferred，而不是继续扩写
  - `--research-depth standard|deep` + `--max-rounds <n>`：提高证据扩展强度，并把 review/revision loop 拉深
  - `--only-target <id>`（可重复）：针对结构化 Stage-I 任务 `I4`-`I8`，回读 `RESEARCH/[topic]/code/` 下的现有 artifact，并且只重跑指定 actionable target
  - `--skip-validation`：关闭严格的 MCP/skill 可用性校验，并跳过 artifact validator gate；运行结果会明确给出 warning，同时把 `validator_gate.skipped=true` 写进结果数据
  - `--guidance-mode off|read|propose|apply`：控制项目本地 `.qiongli/` 指导层；默认 `propose` 会在存在指导文件时读取它，写入 trace bundle，并生成保守的 guidance update proposal
  - `--update-academic-context`：对支持的阶段收口任务（`A5`、`B6`、`C5`、`D3`、`E5`、`F6`、`H4`），把 `context/research_state.md` 和 `context/decision_log.md` 追加进本次 active outputs，并向 draft prompt 注入阶段化的 academic continuity 更新约束
  - 内置 profile 新增 `focused-delivery`、`deep-research`；原有 `default`、`rapid-draft`、`strict-review` 仍可用

  正式研究产物仍然属于 `RESEARCH/[topic]/...`。项目 subject context 来自 `.qiongli/guidance_manifest.yaml`；缺失时有效 subject 是 `active_subject: auto`。第一次非 `off` 的 task-run 会在缺失时自动初始化 `.qiongli/local_guidance.md` 和 `.qiongli/trace/`。orchestrator 会要求运行时 agent 创建这些 required files；如果 agent 只返回文本而没有真正写入文件，validator 会把它们标为 missing。`.qiongli/trace/runs/<run_id>/` 是独立追溯目录，即使正式产物不完整，也会记录 task packet、draft、review、validator gate 和 guidance proposal。

  示例：减少辅助文件，但保持更强的深度审查
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id F3 \
    --paper-type empirical \
    --topic your-topic \
    --cwd . \
    --focus-output manuscript/manuscript.md \
    --research-depth deep \
    --draft-profile deep-research \
    --review-profile strict-review \
    --triad-profile deep-research \
    --triad \
    --max-rounds 4
  ```
  示例：只重跑某个 Stage-I planning step
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id I6 \
    --paper-type methods \
    --topic llm-bias \
    --cwd . \
    --only-target S1
  ```
  示例：在阶段收口任务中强制刷新项目级学术上下文连续性产物
  ```bash
  python3 -m bridges.orchestrator task-run \
    --task-id F6 \
    --paper-type empirical \
    --topic your-topic \
    --cwd . \
    --update-academic-context
  ```
- `task-plan`：从合同渲染依赖任务顺序（用于“从哪一步开始做”）
  ```bash
  python3 -m bridges.orchestrator task-plan --task-id F3 --paper-type empirical --topic your-topic --cwd .
  ```

### 项目级 subject guidance：`qiongli project`

用 `qiongli project` 把 subject、venue、method lens 和 strictness 上下文保存在项目里，而不是为了每篇论文重新安装 Qiongli。

```bash
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project set-venue journal-of-finance --project-dir .
qiongli project set-method-lens event-study --project-dir .
qiongli project status --project-dir .
```

这些命令读取和写入 `.qiongli/guidance_manifest.yaml`。manifest 可以包含 `active_subject`、`secondary_subjects`、`venue_profiles`、`method_lenses` 和 `strictness`。如果文件不存在，有效默认值是 `active_subject: auto`：Qiongli 不需要 setup 也能运行，会使用 core guidance，并且可以根据当前任务临时推断 subject 或 method lens。

Qiongli 不会静默持久切换 subject。持久的项目变更只来自显式的 `qiongli project ...` 命令，或已接受的 guidance proposal。Task run 可以生成 manifest 或 local guidance 更新 proposal 供审计；未接受的 proposal 不会改变项目本地状态。

- `guidance`：管理项目本地 guidance 和 trace
  ```bash
  qiongli guidance init --project-dir .
  qiongli guidance show --project-dir .
  qiongli guidance add --project-dir . --name writing-style
  qiongli guidance list --project-dir .
  qiongli guidance lint --project-dir .
  qiongli guidance trace --project-dir .
  qiongli guidance apply \
    --project-dir . \
    --proposal .qiongli/trace/runs/<run_id>/guidance_update_proposal.md
  ```
  项目 subject context 写在 `.qiongli/guidance_manifest.yaml`；项目本地定制写在 `.qiongli/local_guidance.md`；运行追溯写在 `.qiongli/trace/index.jsonl` 和 `.qiongli/trace/runs/<run_id>/`。这些文件不会修改 canonical workflow contract、内置 skills 或 release payload。
  Guidance proposal 默认是 project-local。proposal 可以建议 `user-global` 或 `canonical-candidate`，但 `qiongli guidance apply` 只会写 `.qiongli/local_guidance.md`。将规则提升到 `~/.qiongli/preferences.md` 或 canonical source，需要显式的后续命令或正常 repository PR。
- `code-build`：学术代码工作流入口
  ```bash
  python3 -m bridges.orchestrator code-build \
    --method "Staggered DID" \
    --topic policy-effects \
    --domain econ \
    --focus full \
    --cwd .
  ```
  关键参数：
  - `--topic <slug>`：提供后会进入严格 Stage I 工作流；不提供时才回落到 legacy prompt-only 模式
  - `--focus <name>`：映射到 `I1`/`I2`/`I3`/`I4`/`I5`/`I6`/`I7`/`I8`，或用 `full` 跑 `I5 -> I6 -> I7 -> I8`
  - `--domain <name>`：注入对应的 `skills/domain-profiles/*.yaml`
  - `--paper-type <type>`：严格 Stage-I 路由使用的论文类型
  - `--triad`：在最终严格 review 阶段追加第三个独立审计
  - `--paper <path-or-url>`：可选论文引用，会带入任务上下文
  - `--only-target <selector>`（可重复）：定向 follow-up 模式
    - 单阶段 focus：直接用 `S1`、`P1-01` 这类 target ID
    - `--focus full`：必须写成 `STAGE_ID:TARGET`，例如 `I5:decision-1`、`I8:P1-01`

  示例：只跑高级 CS 方法的 spec 阶段
  ```bash
  python3 -m bridges.orchestrator code-build \
    --method "Transformer Fine-Tuning" \
    --topic llm-bias \
    --domain cs \
    --tier advanced \
    --focus code_specification \
    --paper-type methods \
    --cwd .
  ```
  示例：在 full 流程里只重跑特定 target
  ```bash
  python3 -m bridges.orchestrator code-build \
    --method "Transformer Fine-Tuning" \
    --topic llm-bias \
    --domain cs \
    --focus full \
    --only-target I5:decision-1 \
    --only-target I8:P1-01 \
    --cwd .
  ```
- `single`：单模型执行（调试/快速跑）
  ```bash
  python3 -m bridges.orchestrator single --prompt "..." --cwd . --model codex
  ```
- `chain`：一端生成、另一端验证
  ```bash
  python3 -m bridges.orchestrator chain --prompt "..." --cwd . --generator codex
  ```
- `role`：按专长拆分任务
  ```bash
  python3 -m bridges.orchestrator role --cwd . --codex-task "..." --claude-task "..."
  ```

---

## 4) Bash 脚本入口（不依赖 pipx）

### 4.1 远程 bootstrap 安装器：`./scripts/bootstrap_qiongli.sh`

用途：
- 在没有 Python 的机器上完成安装或刷新。
- 下载 GitHub release/branch 压缩包，解压后转调其中的 `scripts/install_qiongli.sh`。

```bash
./scripts/bootstrap_qiongli.sh \
  --repo owner/repo \
  --target all \
  --project-dir /path/to/project \
  --overwrite
```

说明：
- 依赖 `bash` 和 `curl` 或 `wget`，以及 `tar`。
- 支持 `--ref <tag-or-branch>` 配合 `--ref-type tag|branch`。
- 默认会安装 shell CLI 命令：`qiongli`、`ql`、`research-skills`、`rsk`、`rsw`。
- 如果你不想安装 shell CLI，可加 `--no-cli`；如需改目录，可用 `--cli-dir <path>`。
- 远程 bootstrap 只支持 `--mode copy`。
- `--doctor` 在没有 `python3` 时会自动跳过。

### 4.2 安装脚本：`./scripts/install_qiongli.sh`

```bash
./scripts/install_qiongli.sh \
  --target all \
  --mode copy \
  --project-dir /path/to/project \
  --install-cli \
  --overwrite \
  --doctor
```

说明：
- 这是本地仓库安装器。
- `copy/link` 安装路径本身不再依赖 Python。
- 如果需要同时安装 shell CLI，可加 `--install-cli`；默认目录为 `${QIONGLI_BIN_DIR:-${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}}`，也可用 `--cli-dir <path>` 覆盖。
- `--doctor` 仅在系统存在 `python3` 时运行 `python3 -m bridges.orchestrator doctor --cwd <project>`。

### 4.3 Release 自动化：`./scripts/release_automation.sh`

```bash
./scripts/release_automation.sh publish --version 0.1.0 --from-tag v0.1.0-beta.X
./scripts/release_automation.sh pre  --tag v0.1.0-beta.X
./scripts/release_automation.sh post --tag v0.1.0-beta.X --create-release
```

推荐方式：

- 日常发布只用 `publish`
- 只有诊断或恢复时才用 `pre` / `post`
- 让 `publish` 统一负责 commit、推送 branch、branch CI/check 门禁、推送 tag、等待 tag publish、GitHub Release 和 acceptance receipt
- release-prep commit 通过 `CI` 和 `Checkout Install Check` 之前，不创建也不推送 release tag
- stable 正式版从 `CHANGELOG.md` 对应章节发布
- beta / prerelease 继续从 `tooling/release/<tag>.md` 发布

也可单独运行：

```bash
./scripts/release_preflight.sh [--tag v0.1.0-beta.X] [--quick] [--skip-smoke] [--maintainer-smoke] [--no-strict]
./scripts/release_postflight.sh --tag v0.1.0-beta.X [--skip-remote] [--skip-ci-status] [--wait-ci] [--ci-timeout-seconds 900] [--ci-timeout-mode soft] [--create-release]
```

`publish` 始终使用 hard CI 门禁：tag 创建前必须通过 branch checks，GitHub Release 创建前也必须确认 tag publish workflows 通过。`--ci-timeout-mode soft` 只用于手动 `post` 诊断或恢复，可把未完成 CI 记录为 acceptance receipt 里的 `pending`，不再用于日常 publish。

### 4.4 Beta smoke：`./scripts/run_beta_smoke.sh`

```bash
./scripts/run_beta_smoke.sh
./scripts/run_beta_smoke.sh --tier release
./scripts/run_beta_smoke.sh --tier maintainer
```

这个主 smoke 入口现在支持两档：

- `release`：内置 literature pipeline smoke + `doctor`
- `maintainer`：包含 `release` 全部内容，并额外执行 `parallel` 和 `task-run` 的 profile 路径检查

release preflight 默认只跑 `release` 档。只有在你明确想补跑维护者级别检查时，才加 `--maintainer-smoke`。

### 4.5 Literature smoke：`./scripts/run_literature_smoke.sh`

```bash
./scripts/run_literature_smoke.sh
```

### 4.6 CI 注入打包默认上游：`./scripts/inject_project_toml.sh`

GitHub Actions 构建时会运行它，把当前仓库 slug 写入 `qiongli/project.toml`，让 pipx 安装后的 CLI 默认指向正确上游。

```bash
bash scripts/inject_project_toml.sh

# 或覆盖（用于构建时切换到别的 upstream repo）
QIONGLI_REPO_SLUG="other-owner/other-repo" bash scripts/inject_project_toml.sh
```

---

## 5) 校验器（推荐在 CI/发布前运行）

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
```

项目产物校验（在你的项目里跑）：

```bash
python3 scripts/validate_project_artifacts.py \
  --cwd /path/to/project \
  --topic your-topic \
  --task-id H1 \
  --strict
```
