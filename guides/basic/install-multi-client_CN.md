# 多端客户端安装指南 (Codex / Claude Code / Gemini)

## 1. 通用安装方式（不依赖 Python）

最通用的安装方式是 shell bootstrap。它会下载指定 release 的压缩包，并执行其中自带的安装脚本：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- \
  --project-dir /path/to/project \
  --target all
```

依赖：
- `bash`
- `curl` 或 `wget`
- `tar`

说明：
- 默认也会安装 shell CLI：`research-skills`、`rsk`、`rsw`。
- 默认 CLI 目录：`${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`。
- 如果是已有安装上的刷新/升级，请加 `--overwrite`。
- 如果你只想安装 workflow 资产，可加 `--no-cli`。
- 如果你要改 CLI 落盘目录，可用 `--cli-dir <path>`。
- `--doctor` 是可选项，只有检测到 `python3` 时才会执行。
- 远程 bootstrap 只支持 `--mode copy`。如果你需要 `--mode link`，请先 clone 仓库，再使用下面的本地安装脚本。

## 2. 可选：Python CLI

如果机器上已经有 Python，也可以继续使用 `pipx` 安装升级器 CLI：

```bash
pipx install research-skills-installer
rsk upgrade --target all --project-dir /path/to/project --doctor
```

## 3. 本地仓库安装脚本

如果你已经有仓库副本，可以直接运行统一安装脚本：

```bash
./scripts/install_research_skill.sh --target all --project-dir /path/to/project --install-cli --doctor
```

## 全局优先安装与修改产物

目前系统所有的安装与升级**默认全部是全局操作（Global-first）**，你的项目目录会被保持绝对干净。

安装器主要执行两步：
1. **安装核心技能包：** 把 `research-paper-workflow` 下载存进你本地 AI 客户端所在的专属配置目录（例如 `~/.claude/skills/` 和 `~/.gemini/skills/`）。
2. **注册快捷指令 (Slash Commands)：** 自动在客户端的发现路径里打下轻量级的软链接（例如 `~/.claude/commands/paper.md`）。

这意味着像 `/paper`、`/study-design` 这样的命令，**无论你当前在电脑的哪个文件夹下工作，AI 都能原生识别**。

_注：涉及你具体项目内文件的写入（比如需要注入 API Key 的 `.env`），只有在你显式运行 `rsk init --project-dir .` 时才会发生。_

## 常用参数

- `--install-cli`：安装 shell CLI 命令（`research-skills`、`rsk`、`rsw`）。
- `--no-cli`：跳过 shell CLI 安装。
- `--cli-dir <path>`：指定 shell CLI 安装目录（默认：`${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`）。
- `--overwrite`：替换现有的安装目标文件。
- `--dry-run`：仅预览安装操作。
- `--doctor`：在安装后运行 `python3 -m bridges.orchestrator doctor --cwd <project>` 进行环境预检；若系统没有 `python3` 会自动跳过。

## 极简使用指南（零配置）

有了全局化命令注册，现在开启一篇新论文的研究步骤非常简单：

1. 新建一个纯空目录：`mkdir my-new-paper && cd my-new-paper`
2. 唤出终端里的 AI：输入 `claude` 或 `gemini`
3. 直接调用命令：敲击 `/paper` 或 `/lit-review`

## 升级指南

- 检测更新：`rsk check --repo <owner>/<repo>`
- 一键升级（无需 fork 或 git clone）：`rsk upgrade --repo <owner>/<repo> --target all` 自动刷新全局环境。
- 完整升级指南：`guides/basic/upgrade-research-skills_CN.md`

## 验证安装

```bash
python3 -m bridges.orchestrator doctor --cwd /path/to/project
python3 scripts/validate_research_standard.py --strict
```
