# 多端客户端安装指南 (Codex / Claude Code / Gemini)

使用统一安装脚本：

```bash
./scripts/install_research_skill.sh --target all --project-dir /path/to/project --doctor
```

## 目标环境行为

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

## 常用参数

- `--mode copy|link`：复制文件或创建软链接 (symlinks)。
- `--overwrite`：替换现有的安装目标文件。
- `--dry-run`：仅预览安装操作。
- `--doctor`：在安装后运行 `python3 -m bridges.orchestrator doctor --cwd <project>` 进行环境预检。

## 升级指南

- 命令别名（pipx 安装后可用）：`rs` / `rsw`（等价于 `research-skills`）
- 可选默认上游（省略 `--repo`）：设置环境变量 `RESEARCH_SKILLS_REPO=<owner>/<repo>`，或在项目根目录添加 `research-skills.toml` 文件
- 检测更新：`rs check --repo <owner>/<repo>`（配置了默认上游后可直接使用 `rs check`；或使用源码脚本 `python3 scripts/research_skill_update.py check ...`）
- 一键升级（无需 fork 或 git clone）：`rs upgrade --repo <owner>/<repo> --project-dir /path/to/project --target all`（配置默认上游后可省略 `--repo`；或使用源码脚本 `python3 scripts/research_skill_update.py upgrade ...`）
- 完整升级指南：`guides/basic/upgrade-research-skills_CN.md`

## 验证安装

```bash
python3 -m bridges.orchestrator doctor --cwd /path/to/project
python3 scripts/validate_research_standard.py --strict
```
