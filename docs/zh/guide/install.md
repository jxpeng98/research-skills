# 多端客户端安装指南 (Codex / Claude Code / Gemini)

::: warning 完整功能依赖
通用安装本身不依赖 Python，但完整功能依赖。

如果你要启用完整系统，请安装并配置：

- `python3`
- `codex`
- `claude`
- `gemini`
- `OPENAI_API_KEY`、`ANTHROPIC_API_KEY`、`GOOGLE_API_KEY`

如果缺少这些依赖，安装流程和 shell `rsk` 维护命令仍可使用，但 orchestrator 执行、`doctor`、validator 和完整多模型流程会不完整。
:::

## 0. Preliminary：先安装 Python（推荐）

Python 主要是为了启用 orchestrator 运行时，而不是为了单纯复制 skills 文件。如果你只想把 workflow 资产装到项目里，`partial` 安装可以不依赖 Python。如果你要 `doctor`、validator 和 `python3 -m bridges.orchestrator ...`，建议先安装 `Python >= 3.12`。仓库当前的 Python 包要求也是 `requires-python = ">=3.12"`。

推荐使用 `mise`：

如果机器上还没有 `mise`，先安装并激活它：

```bash
# Linux / macOS
curl https://mise.run | sh
```

```bash
# bash
echo 'eval "$(mise activate bash)"' >> ~/.bashrc
source ~/.bashrc
```

```bash
# zsh
echo 'eval "$(mise activate zsh)"' >> "${ZDOTDIR-$HOME}/.zshrc"
source "${ZDOTDIR-$HOME}/.zshrc"
```

```powershell
# Windows (PowerShell)
scoop install mise
```

```powershell
# Windows 备用方式
winget install jdx.mise
```

```bash
mise install python@3.12
mise use -g python@3.12
python3 --version
```

说明：
- 如果你的机器还没有 `mise`，需要先把 `mise` 预装好，再执行上面的 Python 安装命令。
- 这样后续的 `python3 -m bridges.orchestrator ...`、`--doctor`、validator 和 tests 都能直接运行。
- 即使当前只准备走 shell bootstrap，也推荐先把 Python 配好，避免后面切到升级、预检或排障时再补环境。

## 1. 零 Python Bootstrap（首次安装推荐）

如果机器上可能还没有 Python，首装建议先走 bootstrap 入口。如果你不传 `--profile`，脚本会先解释 `partial` 和 `full` 的区别，再提示你选择。

统一 profile 行为：

| profile | 安装内容 | 安装前是否要求 Python | 安装后是否可直接跑 orchestrator |
|---|---|---|---|
| `partial` | skills、workflows、项目集成文件 | 否 | 否 |
| `full` | `partial` + shell CLI + 缺失时通过 `mise` 安装 Python 3.12 + `doctor` | 否 | 是 |

Linux / macOS，一键 bootstrap：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- \
  --project-dir /path/to/project \
  --target all
```

Windows PowerShell，一键 bootstrap：

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.ps1 -OutFile .\bootstrap_research_skill.ps1
powershell -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -ProjectDir "C:\path\to\project" -Target all
```

profile 行为：
- `partial`：只安装 workflow 资产和项目集成文件。
- `full`：安装 workflow 资产，启用 shell CLI，自动通过 `mise` 补齐 Python 3.12，并运行 doctor，让 orchestrator 运行时可用。Windows bootstrap 现在由 PowerShell 直接完成安装，只有在 shell CLI 包装器需要 Bash 时才会通过 `winget` 安装 Git for Windows。

## 2. 跨平台 Python 安装器（已有 Python 时可选）

如果机器上已经有 Python，也可以使用本地 Python 安装器。它在 Linux、macOS 和 Windows 上都能工作，不依赖 Bash。

部分功能安装：

```bash
python3 scripts/bootstrap_research_skill.py --profile partial --project-dir .
```

完整功能安装：

```bash
python3 scripts/bootstrap_research_skill.py --profile full --project-dir .
```

profile 行为：
- `partial`：只安装 workflow 资产和项目集成文件。
- `full`：安装 workflow 资产，尽量安装 shell CLI，打印完整依赖提示，并运行 doctor。

说明：
- shell bootstrap 路径在 Linux/macOS 上仍然需要 `bash`。
- `rsk upgrade` 现在已经是纯 Python 路径，不再依赖 Bash。
- Windows PowerShell bootstrap 现在由 PowerShell 直接完成安装，只有在 shell CLI 包装器需要 Bash 时才会通过 `winget` 安装 Git for Windows。
- shell bootstrap 路径在 `full` 模式下默认安装 shell CLI，在 `partial` 模式下默认跳过。

## 3. 可选：Python CLI

如果机器上已经有 Python，也可以继续使用 `pipx` 安装升级器 CLI：

```bash
pipx install research-skills-installer
rsk upgrade --target all --project-dir /path/to/project --doctor
```

这条 `pip` / `pipx` 路径现在主要保留为升级器 CLI 的兼容性分发方式，不再是推荐的首次安装入口。

## 4. 本地仓库安装脚本

如果你已经有仓库副本，可以直接运行统一安装脚本：

```bash
./scripts/install_research_skill.sh --profile partial --target all --project-dir /path/to/project
./scripts/install_research_skill.sh --profile full --target all --project-dir /path/to/project
```

## 目标环境行为

- 项目级默认内容
  - 默认将 `.env.example` 复制为 `<project>/.env`，让项目开箱就有可编辑的运行时配置文件。
- `codex`
  - 将 `research-paper-workflow` 安装到 `${CODEX_HOME:-~/.codex}/skills/research-paper-workflow`。
- `claude`
  - 将 `research-paper-workflow` 安装到 `${CLAUDE_CODE_HOME:-~/.claude}/skills/research-paper-workflow`。
  - 将 `.agent/workflows/*.md` 复制到 `<project>/.agent/workflows/`。
  - 将 `CLAUDE.md` 复制到 `<project>/CLAUDE.md`（如果 `CLAUDE.md` 已存在且未使用 `--overwrite`，则另存为 `CLAUDE.research-skills.md`）。
- `gemini`
  - 将 `research-paper-workflow` 安装到 `${GEMINI_HOME:-~/.gemini}/skills/research-paper-workflow`。
  - 在 `<project>/.gemini/research-skills.md` 中创建 orchestrator 快速启动命令参考。
  - 将 `standards/agent-profiles.example.json` 复制到 `<project>/.gemini/agent-profiles.example.json`。
- `antigravity`
  - 在写入全局目录前，先检查 `PATH` 中是否存在 `antigravity` CLI。
  - 将 workspace skill 安装到 `<project>/.agents/skills/research-paper-workflow`。
  - 同时写入兼容旧目录的 `<project>/.agent/skills/research-paper-workflow`。
  - 若 CLI 可用，则把全局 skill 安装到 `${ANTIGRAVITY_HOME:-~/.gemini/antigravity}/skills/research-paper-workflow`。

## 常用参数

- `--mode copy|link`：复制文件或创建软链接 (symlinks)。
- `--install-cli`：安装 shell CLI 命令（`research-skills`、`rsk`、`rsw`）。
- `--no-cli`：跳过 shell CLI 安装。
- `--cli-dir <path>`：指定 shell CLI 安装目录（默认：`${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`）。
- `--overwrite`：替换现有的安装目标文件。
- `--dry-run`：仅预览安装操作。
- `--doctor`：在安装后运行 `python3 -m bridges.orchestrator doctor --cwd <project>` 进行环境预检；若系统没有 `python3` 会自动跳过。

## 升级指南

- 命令别名（pipx 安装后可用）：`rsk` / `rsw`（等价于 `research-skills`）
- shell CLI 别名（bootstrap 安装后可用）：`rsk` / `rsw` / `research-skills`
- 可选默认上游（省略 `--repo`）：设置环境变量 `RESEARCH_SKILLS_REPO=<owner>/<repo>`，或在项目根目录添加 `research-skills.toml` 文件
- 无 Python 刷新：重新执行 `bootstrap_research_skill.sh --overwrite`
- 检测更新：`rsk check --repo <owner>/<repo>`（shell CLI 或 Python CLI 均可；或使用源码脚本 `python3 scripts/research_skill_update.py check ...`）
- 一键升级（无需 fork 或 git clone）：`rsk upgrade --repo <owner>/<repo> --project-dir /path/to/project --target all`（shell CLI 或 Python CLI 均可；或使用源码脚本 `python3 scripts/research_skill_update.py upgrade ...`）
- 完整升级指南：[升级](/zh/guide/upgrade)

## 验证安装

```bash
python3 -m bridges.orchestrator doctor --cwd /path/to/project
python3 scripts/validate_research_standard.py --strict
```
