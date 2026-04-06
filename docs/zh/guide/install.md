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
| `partial` | 仅全局 skills | 否 | 资产可用，但 orchestrator 还没准备好 |
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
- `.agent/workflows/`、`CLAUDE.md`、`.gemini/` 等项目集成文件，仅在执行 `rsk init` 或 `--parts project` 时写入
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
rsk upgrade --target all --doctor
rsk init --project-dir /path/to/project
```

## 5. 全局优先安装与修改产物

目前系统所有的安装与升级**默认全部是全局操作（Global-first）**，你的项目目录会被保持绝对干净。

安装器主要执行两步：
1. **安装核心技能包：** 把 `research-paper-workflow` 下载存进你本地 AI 客户端所在的专属配置目录（例如 `~/.claude/skills/` 和 `~/.gemini/skills/`）。
2. **注册快捷指令 (Slash Commands)：** 自动在客户端的发现路径里打下轻量级的软链接（例如 `~/.claude/commands/paper.md`）。

这意味着像 `/paper`、`/study-design` 这样的命令，**无论你当前在电脑的哪个文件夹下工作，AI 都能原生识别**。

_注：涉及你具体项目内文件的写入（比如需要注入 API Key 的 `.env`），只有在你显式运行 `rsk init --project-dir .` 时才会发生。_

## 6. 常用参数

- `--profile partial|full`：显式指定安装模式，跳过交互提示。
- `--target codex|claude|gemini|antigravity|all`：限制安装范围。
- `--beta`：在未传 `--ref` 时安装最新 beta / prerelease tag。
- `--install-cli`：即使不是 `full` 也安装 shell CLI。
- `--no-cli`：即使是 `full` 也跳过 shell CLI。
- `--cli-dir <path>`：指定 shell CLI 安装目录。默认：`${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`。
- `--overwrite`：覆盖已有安装目标。
- `--dry-run`：只预览安装动作。
- `--doctor`：安装后运行 `python3 -m bridges.orchestrator doctor --cwd <project>`。

## 7. 极简使用指南（零配置）

有了全局化命令注册，现在开启一篇新论文的研究步骤非常简单：

1.新建一个纯空目录：`mkdir my-new-paper && cd my-new-paper`
2. 唤出终端里的 AI：输入 `claude` 或 `gemini`
3. 直接调用命令：敲击 `/paper` 或 `/lit-review`

模型会自动无缝调取全局后台存放的技能体系，不再往你的工作区里丢一堆恶心的模板文件。

## 8. 日常升级与故障排查

要将你电脑上所有 AI 客户端的学术引擎统一升级到远端最新版本（你不用再去分别挨个项目升级了）：

```bash
rsk upgrade --target all --doctor
```

_注：终端工具（如 `rsk check`, `rsk upgrade`, `rsk clean`）现在自身不再依赖 Python 即可运行。但底层的核心验证器（`--doctor`，单元测试 以及 `bridges.orchestrator`）仍需合法的 Python 3 解释器来工作。_
