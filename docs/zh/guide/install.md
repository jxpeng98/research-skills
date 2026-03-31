# 多端客户端安装指南 (Codex / Claude Code / Gemini)

::: warning 完整功能依赖
现在首装已经不要求你手动先装 Python，但完整运行时仍然依赖：

- `python3`
- `codex`
- `claude`
- `gemini`
- `OPENAI_API_KEY`、`ANTHROPIC_API_KEY`、`GOOGLE_API_KEY`

如果缺少这些依赖，资产安装和 shell `rsk` 维护命令仍可使用，但 orchestrator 执行、`doctor`、validator 和完整多模型流程会受限。
:::

## 1. 先选 `partial` 还是 `full`

如果你不传 `--profile`，bootstrap 会先解释这两种模式，再提示你选择。

| Profile | 安装内容 | 安装前是否要求 Python | 安装后结果 |
|---|---|---|---|
| `partial` | skills、workflows、项目集成文件 | 否 | 资产可用，但 orchestrator 还没准备好 |
| `full` | `partial` + shell CLI + 缺失时自动补 Python 3.12 + `doctor` | 否 | orchestrator 运行时可直接使用 |

`full` 的真实行为：

- 如果系统里已经有 `python3 >= 3.12`，bootstrap 会直接复用。
- 如果 Python 缺失或版本过低，bootstrap 会先安装 `mise`，再安装 `python@3.12`。
- Windows 上由 PowerShell 直接安装，只有在 shell CLI 包装器需要 Bash 时才会通过 `winget` 安装 Git for Windows。

## 2. 运行推荐的一键 bootstrap

### Linux / macOS

交互式选择 `partial` 或 `full`：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --project-dir "$PWD" --target all
```

`full` 模式如果自动安装 `mise`，会同时把 `mise` 和 `mise shims` 目录写入当前会话，以及当前 shell 对应的 rc 文件和 `~/.profile`。

强制 `partial`：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --profile partial --project-dir "$PWD" --target all
```

强制 `full`：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --profile full --project-dir "$PWD" --target all
```

安装最新 beta / prerelease：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --beta --profile full --project-dir "$PWD" --target all
```

### Windows PowerShell 7+

如果机器上还没有 `pwsh`，先安装：

```powershell
winget install --id Microsoft.PowerShell --source winget
```

`full` 模式如果自动安装 `mise`，会同时把 `mise` 的 bin 目录写入当前会话和用户级 `PATH`。

下载后交互式选择 `partial` 或 `full`：

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.ps1 -OutFile .\bootstrap_research_skill.ps1
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -ProjectDir "$PWD" -Target all
```

强制 `partial`：

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Profile partial -ProjectDir "$PWD" -Target all
```

强制 `full`：

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Profile full -ProjectDir "$PWD" -Target all
```

安装最新 beta / prerelease：

```powershell
pwsh -ExecutionPolicy Bypass -File .\bootstrap_research_skill.ps1 -Beta -Profile full -ProjectDir "$PWD" -Target all
```

Bootstrap 会安装：

- Codex / Claude Code / Gemini 的 workflow 资产
- `.agent/workflows/`、`CLAUDE.md`、`.gemini/` 等项目集成文件
- `full` 模式下的 shell CLI：`research-skills`、`rsk`、`rsw`

## 3. 可选：自己用 `mise` 准备 Python

只有在你想手动管理 Python，而不是交给 `full` bootstrap 自动处理时，才需要这一节。

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
# Windows
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

## 4. 可选：本地安装器

如果机器上已经有 Python，可以用跨平台本地安装器：

```bash
python3 scripts/bootstrap_research_skill.py --profile partial --project-dir .
python3 scripts/bootstrap_research_skill.py --profile full --project-dir .
```

如果你在 Linux 或 macOS 上已经有本地仓库副本，也可以直接用本地 shell 安装器：

```bash
./scripts/install_research_skill.sh --profile partial --target all --project-dir /path/to/project
./scripts/install_research_skill.sh --profile full --target all --project-dir /path/to/project
```

`pip` / `pipx` 路径仍然保留给升级器 CLI，但不再是推荐的首次安装入口：

```bash
pipx install research-skills-installer
rsk upgrade --target all --project-dir /path/to/project --doctor
```

## 5. 目标环境行为

- 项目级默认内容
  - 将 `.env.example` 复制为 `<project>/.env`。
- `codex`
  - 将 `research-paper-workflow` 安装到 `${CODEX_HOME:-~/.codex}/skills/research-paper-workflow`。
- `claude`
  - 将 `research-paper-workflow` 安装到 `${CLAUDE_CODE_HOME:-~/.claude}/skills/research-paper-workflow`。
  - 将 `.agent/workflows/*.md` 复制到 `<project>/.agent/workflows/`。
  - 将 `CLAUDE.md` 复制到 `<project>/CLAUDE.md`，如果目标已存在且未使用 `--overwrite`，则写入 `CLAUDE.research-skills.md`。
- `gemini`
  - 将 `research-paper-workflow` 安装到 `${GEMINI_HOME:-~/.gemini}/skills/research-paper-workflow`。
  - 将 `.gemini/research-skills.md` 复制到 `<project>/.gemini/research-skills.md`。
  - 将 `standards/agent-profiles.example.json` 复制到 `<project>/.gemini/agent-profiles.example.json`。
- `antigravity`
  - 将 workspace skill 安装到 `<project>/.agents/skills/research-paper-workflow`。
  - 将兼容旧目录的 workspace skill 安装到 `<project>/.agent/skills/research-paper-workflow`。
  - 当 `antigravity` CLI 可用时，将全局 skill 安装到 `${ANTIGRAVITY_HOME:-~/.gemini/antigravity}/skills/research-paper-workflow`。

## 6. 常用参数

- `--profile partial|full`：显式指定安装模式，跳过交互提示。
- `--target codex|claude|gemini|antigravity|all`：限制安装范围。
- `--beta`：在未传 `--ref` 时安装最新 beta / prerelease tag。
- `--mode copy|link`：复制文件或创建软链接。bootstrap 固定使用 `copy`。
- `--install-cli`：即使不是 `full` 也安装 shell CLI。
- `--no-cli`：即使是 `full` 也跳过 shell CLI。
- `--cli-dir <path>`：指定 shell CLI 安装目录。默认：`${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`。
- `--overwrite`：覆盖已有安装目标。
- `--dry-run`：只预览安装动作。
- `--doctor`：安装后运行 `python3 -m bridges.orchestrator doctor --cwd <project>`。

## 7. 升级与验证

刷新已有安装：

```bash
rsk upgrade --target all --project-dir . --doctor
```

验证环境：

```bash
python3 -m bridges.orchestrator doctor --cwd /path/to/project
python3 scripts/validate_research_standard.py --strict
```

Python 边界：

- shell `rsk check|upgrade|align` 不需要 Python
- `--doctor`、`python3 -m bridges.orchestrator ...`、validator 和 tests 仍然需要 Python
