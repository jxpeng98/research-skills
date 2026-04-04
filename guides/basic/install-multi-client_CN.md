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

## 目标环境行为

- 项目级默认内容
  - 默认将 `.env.example` 复制为 `<project>/.env`，方便你直接填写运行时配置。
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
  - 将 workspace-local skill 安装到 `<project>/.agents/skills/research-paper-workflow`。
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
- 一键升级（无需 fork 或 git clone）：先执行 `rsk upgrade --repo <owner>/<repo> --target all` 刷新全局 skill，再在需要的项目里执行 `rsk init --project-dir /path/to/project` 接线项目资产（shell CLI 或 Python CLI 均可；或使用源码脚本 `python3 scripts/research_skill_update.py upgrade ...`）
- 完整升级指南：`guides/basic/upgrade-research-skills_CN.md`

## 验证安装

```bash
python3 -m bridges.orchestrator doctor --cwd /path/to/project
python3 scripts/validate_research_standard.py --strict
```
