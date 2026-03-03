# CLI 命令参考（research-skills）

本文件整理本仓库所有“可执行入口”（pipx CLI / Python module / Bash scripts），用于本地与 GitHub CI 保持一致的调用方式。

## 0) 命令名约定

- `research-skills`：主 CLI（pipx/venv 安装后提供）
- `rs` / `rsw`：短别名（与 `research-skills` 完全等价）

下文统一用 `rs` 作为示例。

---

## 1) Upstream（上游仓库）如何确定（如何省略 `--repo`）

很多命令需要知道“去哪个 GitHub 仓库查询/下载 release”。`rs` 的上游解析优先级如下（从高到低）：

1. CLI 参数：`--repo <owner/repo|Git URL>`
2. 环境变量：`RESEARCH_SKILLS_REPO=<owner/repo|Git URL>`
3. 项目配置文件（从当前目录或 `--project-dir` 向上搜索）：
   - `research-skills.toml`
   - `.research-skills.toml`
4. 打包默认（pipx 安装的包内）：`research_skills/project.toml`（由 CI 注入）
5. 如果你正在 `research-skills` 仓库 clone 内运行：从 git remote 推断（优先 `upstream`，其次 `origin`）

支持的 repo 形式：

- `owner/repo`
- `https://github.com/owner/repo.git`
- `git@github.com:owner/repo.git`

推荐把上游提交到你的项目仓库（适合 CI）：

```toml
# research-skills.toml
[upstream]
repo = "owner/repo"   # 或 url = "https://github.com/owner/repo.git"
```

---

## 2) `rs`（安装/升级器 CLI）

### 2.1 `rs check`（检查版本/是否有更新）

用途：
- 输出 CLI 版本、本地 repo 版本（若在仓库内运行）、三端已安装版本
- 可选：查询上游最新 release tag，并判断是否需要升级

```bash
rs check [--repo <owner/repo|url>] [--json] [--strict-network]
```

关键参数：
- `--repo`：指定上游（可省略，见“上游解析”）
- `--json`：只输出 JSON（便于 CI/脚本）
- `--strict-network`：如果上游查询失败则返回失败（默认仅提示并继续）

退出码约定：
- `0`：无更新/或跳过上游检查
- `1`：检测到更新可用
- `2`：参数错误

### 2.2 `rs upgrade`（下载 release 并执行三端安装脚本）

用途：
- 下载上游 release（默认 latest tag 的 tar.gz）
- 解压后运行其中的 `scripts/install_research_skill.sh`

```bash
rs upgrade \
  [--repo <owner/repo|url>] \
  [--ref <tag-or-branch>] \
  [--ref-type tag|branch] \
  [--target codex|claude|gemini|all] \
  [--mode copy|link] \
  [--project-dir <path>] \
  [--no-overwrite] \
  [--doctor] \
  [--dry-run]
```

说明：
- `--project-dir` 用于写入项目内集成文件（例如 `.agent/workflows/`、`CLAUDE.md`、`.gemini/`）。
- `--mode link` 适合“长期维护一个本地 clone”的场景（软链接安装）；`--mode copy` 更适合一次性安装/CI。
- 退出码为安装脚本返回码（若安装失败，沿用其错误码）。

### 2.3 `rs align`（快速参考）

用途：打印“pipx 安装了什么 / upgrade 会修改哪些路径 / 常见用法”。

```bash
rs align [--repo <owner/repo|url>]
```

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
  - `--venue <name>` / `--context <text>`
  - `--mcp-strict` / `--skills-strict`
  - `--profile-file <path>` + `--profile <name>`（以及 `--draft-profile` / `--review-profile` / `--triad-profile`）
- `task-plan`：从合同渲染依赖任务顺序（用于“从哪一步开始做”）
  ```bash
  python3 -m bridges.orchestrator task-plan --task-id F3 --paper-type empirical --topic your-topic --cwd .
  ```
- `code-build`：研究代码构建入口（与 Task ID 的 code stage 协同）
  ```bash
  python3 -m bridges.orchestrator code-build --method "Staggered DID" --cwd . --domain econ
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
  python3 -m bridges.orchestrator role --cwd . --codex-task "..." --claude-task "..." --gemini-task "..."
  ```

---

## 4) Bash 脚本入口（不依赖 pipx）

### 4.1 安装脚本：`./scripts/install_research_skill.sh`

```bash
./scripts/install_research_skill.sh \
  --target all \
  --mode copy \
  --project-dir /path/to/project \
  --overwrite \
  --doctor
```

### 4.2 Release 自动化：`./scripts/release_automation.sh`

```bash
./scripts/release_automation.sh pre  --tag v0.1.0-beta.X
./scripts/release_automation.sh post --tag v0.1.0-beta.X
./scripts/release_automation.sh full --tag v0.1.0-beta.X
```

也可单独运行：

```bash
./scripts/release_preflight.sh [--tag v0.1.0-beta.X] [--skip-smoke] [--no-strict]
./scripts/release_postflight.sh --tag v0.1.0-beta.X [--skip-remote] [--skip-ci-status]
```

### 4.3 Beta smoke：`./scripts/run_beta_smoke.sh`

```bash
./scripts/run_beta_smoke.sh
```

### 4.4 CI 注入打包默认上游：`./scripts/inject_project_toml.sh`

GitHub Actions 构建时会运行它，把当前仓库 slug 写入 `research_skills/project.toml`，让 pipx 安装后的 CLI 默认指向正确上游。

```bash
bash scripts/inject_project_toml.sh

# 或覆盖（用于构建时切换到别的 upstream repo）
RESEARCH_SKILLS_REPO_SLUG="other-owner/other-repo" bash scripts/inject_project_toml.sh
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
