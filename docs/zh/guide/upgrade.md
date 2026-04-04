# Upgrade / Auto-Upgrade Guide (No Fork Required)

本指南说明如何在使用 `research-skills` 时：
1) 检测是否有新版本；2) 自动化升级；3) 在不 fork、不 git clone 的情况下完成升级。

## 0) 选择升级入口

### 方案 A：Shell bootstrap（不依赖 Python）

这个路径只需要 `bash` 和 `curl`/`wget`、`tar`：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- \
  --repo <owner>/<repo> \
  --project-dir /path/to/project \
  --target all \
  --overwrite
```

说明：
- bootstrap 会下载所选 release 压缩包，并执行其中自带的 `scripts/install_research_skill.sh`。
- 默认也会安装 shell CLI：`research-skills`、`rsk`、`rsw`。
- 默认 shell CLI 目录：`${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`。
- 如果只想更新 workflow 资产，可加 `--no-cli`；如果要改安装位置，可加 `--cli-dir <path>`。
- `--doctor` 是可选项，只有系统存在 `python3` 时才会执行。
- 远程 bootstrap 只支持 `--mode copy`；如果你需要 `--mode link`，请保留本地 clone。

### 方案 B：Python CLI（可选）

本仓库也提供 `pyproject.toml` 包，适合需要可复用升级器 CLI 的场景：

```bash
pipx install research-skills-installer
# 提供 3 个等价命令（任选其一）：
# - research-skills
# - rsk
# - rsw
# 你也可以设置 `RESEARCH_SKILLS_REPO=<owner>/<repo>` 后省略 --repo
rsk check --repo <owner>/<repo>
rsk upgrade --repo <owner>/<repo> --target all --doctor
rsk init --project-dir /path/to/project
```

> 注意：pip 安装/升级的是“升级器 CLI”；真正刷新三端全局 skill 目录的动作，仍由 `rsk upgrade`（等价于 `research-skills upgrade`）来执行。项目内文件现在改为显式更新：需要时使用 `rsk init` 或 `rsk upgrade --parts project ...`。

## 1) 你需要升级的到底是什么？

这个项目有两类“安装目标”：

- **本地 skill 安装目录**（让 Codex / Claude Code / Gemini / Antigravity 识别 skill）  
  - Codex: `${CODEX_HOME:-~/.codex}/skills/research-paper-workflow`
  - Claude: `${CLAUDE_CODE_HOME:-~/.claude}/skills/research-paper-workflow`
  - Gemini: `${GEMINI_HOME:-~/.gemini}/skills/research-paper-workflow`
  - Antigravity（全局）: `${ANTIGRAVITY_HOME:-~/.gemini/antigravity}/skills/research-paper-workflow`
- **项目内集成文件**（让 Claude Code 的 `/...` 命令在你的项目里可用）  
  - `<project>/.agent/workflows/*.md`
  - `<project>/CLAUDE.md`（或 `CLAUDE.research-skills.md`）
  - `<project>/.gemini/research-skills.md`
  - `<project>/.agents/skills/research-paper-workflow`
  - `<project>/.agent/skills/research-paper-workflow`

升级的本质就是：**把这些目标路径覆盖为新版本**（通常需要 `--overwrite`）。

---

## 2) 检测是否有新版本（推荐）

```bash
# 如果已设置 RESEARCH_SKILLS_REPO，可省略 --repo
rsk check --repo <owner>/<repo>
# 或在仓库内运行（等价）：
python3 scripts/research_skill_update.py check --repo <owner>/<repo>
```

说明：
- `--repo` 用于查询 GitHub 最新 release tag。
- 若检测到“本地/已安装版本 < 最新版本”，该命令会返回 exit code `1`（方便写自动化）。
- 你可以设置默认上游来省略 `--repo`：
  - 环境变量：`export RESEARCH_SKILLS_REPO=<owner>/<repo>`
  - 若你在 `research-skills` 仓库 clone 里运行，且已配置 git remote（优先 `upstream`，其次 `origin`），也可省略 `--repo`
  - 或在你的项目根目录添加 `research-skills.toml`（便于提交到项目仓库，适合 CI）

示例（项目根目录）：

```toml
# research-skills.toml
[upstream]
repo = "<owner>/<repo>" # 或 Git URL
```

此后可直接运行：

```bash
rsk check
rsk upgrade --target all --doctor
rsk init --project-dir .
```

---

## 3) 自动升级（不需要 fork，不需要 git clone）

直接下载 GitHub release 压缩包并执行其中的安装脚本：

```bash
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- \
  --repo <owner>/<repo> \
  --project-dir /path/to/your/project \
  --target all \
  --overwrite
```

如果机器上有 Python，也可以继续使用 CLI：

```bash
# 如果已设置 RESEARCH_SKILLS_REPO，可省略 --repo
rsk upgrade \
  --repo <owner>/<repo> \
  --target all \
  --mode copy \
  --doctor

# 或在仓库内运行（等价）：
python3 scripts/research_skill_update.py upgrade \
  --repo <owner>/<repo> \
  --target all \
  --mode copy \
  --doctor
```

要点：
- 这个方式**不依赖 git**，也不要求你把仓库 clone 到本地。
- shell bootstrap 路径**不依赖 Python**。
- shell CLI 本身也可以在无 Python 环境下执行 `check`、`upgrade`、`align`。
- 默认 upgrade 现在是 global-first；只有显式加 `--parts project` 时，才会刷新项目内 workflow 资产。
- 私有仓库或遇到 API 限流时，建议设置：`GITHUB_TOKEN` 或 `GH_TOKEN`。
- 默认使用“最新 release tag”；shell bootstrap 和 `rsk upgrade` 也都支持显式指定版本：
  - `--ref v0.1.0-beta.6 --ref-type tag`
  - `--ref main --ref-type branch`

升级后建议重启客户端（Codex / Claude Code / Gemini CLI）。

---

## 4) 另一种“自动升级”：link 安装 + git pull（适合长期维护）

如果你愿意保留一份本地仓库（不需要 fork，只需 clone 一次），推荐：

1) 安装时用 `--mode link`（用软链接指向仓库，后续更新无需重复 install）：

```bash
./scripts/install_research_skill.sh --target all --mode link --overwrite
python3 -m research_skills.cli init --project-dir /path/to/project --target all --overwrite
```

2) 更新时只需：

```bash
git pull
```

因为安装目标是软链接，仓库内容更新后，三端 skill 与 workflows 会自动同步到最新版本。

---

## 5) 自动化建议（可选）

你可以用 cron/CI 做“每周检查 + 有更新则升级”：

1) 定期 check：
```bash
rsk check --repo <owner>/<repo>
```
2) 返回码为 1 时执行 upgrade：
```bash
rsk upgrade --repo <owner>/<repo> --target all
rsk init --project-dir /path/to/project
```

如果你希望我把这套升级检测做成 Codex Automation（定期跑并生成 inbox 结果），告诉我运行频率和要覆盖的 project 路径即可。
