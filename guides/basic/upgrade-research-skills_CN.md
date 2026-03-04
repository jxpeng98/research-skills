# Upgrade / Auto-Upgrade Guide (No Fork Required)

本指南说明如何在使用 `research-skills` 时：
1) 检测是否有新版本；2) 自动化升级；3) 在不 fork、不 git clone 的情况下完成升级。

## 0) 标准化成 pip 包（推荐）

本仓库已提供 `pyproject.toml`，可以作为 pip 包发布/安装（推荐用 `pipx` 安装 CLI）。

发布到 PyPI 后（或用内部源），用户侧的标准用法就是：

```bash
pipx install research-skills-installer
# 提供 3 个等价命令（任选其一）：
# - research-skills
# - rs
# - rsw
# 你也可以设置 `RESEARCH_SKILLS_REPO=<owner>/<repo>` 后省略 --repo
rsk check --repo <owner>/<repo>
rsk upgrade --repo <owner>/<repo> --project-dir /path/to/project --target all --doctor
```

> 注意：pip 安装/升级的是“升级器 CLI”；真正把 skill/workflows 覆盖到三端目录与 project 的动作，仍由 `rsk upgrade`（等价于 `research-skills upgrade`）来执行（显式可控、不会在 pip 安装时偷偷写用户目录）。

## 1) 你需要升级的到底是什么？

这个项目有两类“安装目标”：

- **三端本地 skill 安装目录**（让 Codex / Claude Code / Gemini 识别 skill）  
  - Codex: `${CODEX_HOME:-~/.codex}/skills/research-paper-workflow`
  - Claude: `${CLAUDE_CODE_HOME:-~/.claude}/skills/research-paper-workflow`
  - Gemini: `${GEMINI_HOME:-~/.gemini}/skills/research-paper-workflow`
- **项目内集成文件**（让 Claude Code 的 `/...` 命令在你的项目里可用）  
  - `<project>/.agent/workflows/*.md`
  - `<project>/CLAUDE.md`（或 `CLAUDE.research-skills.md`）
  - `<project>/.gemini/research-skills.md`

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
rsk upgrade --project-dir . --target all --doctor
```

---

## 3) 自动升级（不需要 fork，不需要 git clone）

直接下载 GitHub release 压缩包并执行其中的安装脚本：

```bash
# 如果已设置 RESEARCH_SKILLS_REPO，可省略 --repo
rsk upgrade \
  --repo <owner>/<repo> \
  --project-dir /path/to/your/project \
  --target all \
  --mode copy \
  --doctor

# 或在仓库内运行（等价）：
python3 scripts/research_skill_update.py upgrade \
  --repo <owner>/<repo> \
  --project-dir /path/to/your/project \
  --target all \
  --mode copy \
  --doctor
```

要点：
- 这个方式**不依赖 git**，也不要求你把仓库 clone 到本地。
- 私有仓库或遇到 API 限流时，建议设置：`GITHUB_TOKEN` 或 `GH_TOKEN`。
- 默认使用“最新 release tag”；也可指定版本：
  - `--ref v0.1.0-beta.6 --ref-type tag`
  - `--ref main --ref-type branch`

升级后建议重启客户端（Codex / Claude Code / Gemini CLI）。

---

## 4) 另一种“自动升级”：link 安装 + git pull（适合长期维护）

如果你愿意保留一份本地仓库（不需要 fork，只需 clone 一次），推荐：

1) 安装时用 `--mode link`（用软链接指向仓库，后续更新无需重复 install）：

```bash
./scripts/install_research_skill.sh --target all --mode link --project-dir /path/to/project --overwrite
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
rsk upgrade --repo <owner>/<repo> --project-dir /path/to/project --target all
```

如果你希望我把这套升级检测做成 Codex Automation（定期跑并生成 inbox 结果），告诉我运行频率和要覆盖的 project 路径即可。
