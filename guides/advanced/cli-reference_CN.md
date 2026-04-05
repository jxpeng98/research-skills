# CLI 命令参考（research-skills）

本文件整理本仓库所有“可执行入口”（pipx CLI / Python module / Bash scripts），用于本地与 GitHub CI 保持一致的调用方式。

## 0) 命令名约定

- `research-skills`：主 CLI（pipx/venv 安装后可用，或通过 shell bootstrap 安装）
- `rsk` / `rsw`：短别名（与 `research-skills` 完全等价）

下文统一用 `rsk` 作为示例。

---

## 1) Upstream（上游仓库）如何确定（如何省略 `--repo`）

很多命令需要知道“去哪个 GitHub 仓库查询/下载 release”。`rsk` 的上游解析优先级如下（从高到低）：

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

## 2) `rsk`（安装/升级器 CLI）

这个 CLI 现在有两种分发方式：
- Python CLI：通过 `pip`/`pipx` 安装
- Shell CLI：由 `bootstrap_research_skill.sh` 默认安装到 `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`

共同支持的命令：`check`、`upgrade`、`align`

仅 Python CLI 提供：`doctor`、`init`

### 2.1 `rsk check`（检查版本/是否有更新）

用途：
- 输出 CLI 版本、本地 repo 版本（若在仓库内运行）、三端已安装版本
- 可选：查询上游最新 release tag，并判断是否需要升级

```bash
rsk check [--repo <owner/repo|url>] [--json] [--strict-network]
```

关键参数：
- `--repo`：指定上游（可省略，见“上游解析”）
- `--json`：只输出 JSON（便于 CI/脚本）
- `--strict-network`：如果上游查询失败则返回失败（默认仅提示并继续）

退出码约定：
- `0`：无更新/或跳过上游检查
- `1`：检测到更新可用
- `2`：参数错误

### 2.2 `rsk upgrade`（下载 release 并执行三端安装脚本）

用途：
- 下载上游 release（默认 latest tag 的 tar.gz）
- 解压后运行其中的 `scripts/install_research_skill.sh`

```bash
rsk upgrade \
  [--repo <owner/repo|url>] \
  [--ref <tag-or-branch>] \
  [--ref-type tag|branch] \
  [--target codex|claude|gemini|antigravity|all] \
  [--mode copy|link] \
  [--project-dir <path>] \
  [--install-cli] \
  [--no-cli] \
  [--cli-dir <path>] \
  [--parts globals,project,cli,doctor] \
  [--no-overwrite] \
  [--doctor] \
  [--dry-run]
```

说明：
- `upgrade` 现在默认是 global-first。项目接线建议走 `rsk init --project-dir .`；如果要在升级时顺手重写项目文件，再显式加 `--parts project`。
- `--project-dir` 主要在开启项目侧安装面时生效，例如 `--parts project`。
- `--parts` 用于收窄安装面，例如 `project` 表示只刷新项目资产，`project,doctor` 表示轻量刷新后立即校验。
- 全局安装后，`upgrade` 会自动创建工作流发现 symlink：`~/.claude/commands/*.md` 和 `~/.gemini/workflows/*.md`，可直接使用 `/paper`、`/lit-review` 等 slash 命令。
- Shell CLI 会通过随附的 bootstrap helper 执行升级，不依赖 Python。
- 退出码为底层安装器返回码（若安装失败，沿用其错误码）。

### 2.3 `rsk doctor`（仅 Python CLI）

用途：
- 用更短的命令运行 orchestrator doctor

```bash
rsk doctor [--cwd <path>]
```

### 2.4 `rsk init`（仅 Python CLI）

用途：
- 直接从已安装包初始化项目侧 workflow 资产
- 默认等价于 `--parts project`，适合给一个新项目快速接好 workflow 入口

```bash
rsk init \
  [--project-dir <path>] \
  [--target codex|claude|gemini|antigravity|all] \
  [--mode copy|link] \
  [--parts globals,project,cli,doctor] \
  [--overwrite] \
  [--doctor] \
  [--dry-run]
```

### 2.5 `rsk clean`（清理过期资产）

用途：移除旧版本安装留下的项目本地资产。

```bash
rsk clean [--project-dir <path>] [--dry-run] [--globals]
```

参数说明：
- `--project-dir`：要清理的目录（默认当前目录）。
- `--globals`：同时移除全局工作流发现 symlink（`~/.claude/commands/` 和 `~/.gemini/workflows/`）。只移除指向 `research-paper-workflow` 的 symlink，用户自建的命令不受影响。
- `--dry-run`：只显示将要移除的内容，不实际删除。

### 2.6 `rsk align`（快速参考）

用途：打印“pipx 安装了什么 / upgrade 会修改哪些路径 / 常见用法”。

```bash
rsk align [--repo <owner/repo|url>]
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
  - `--domain <name>`：把运行时领域 profile（例如 `econ`、`cs`、`psychology`）注入 task packet 和 prompts
  - `--venue <name>` / `--context <text>`
  - `--mcp-strict` / `--skills-strict`
  - `--profile-file <path>` + `--profile <name>`（以及 `--draft-profile` / `--review-profile` / `--triad-profile`）
  - `--focus-output <path>`（可重复）+ `--output-budget <n>`：把本次运行收敛到更小的 active outputs，其余 contract outputs 明确标记为 deferred，而不是继续扩写
  - `--research-depth standard|deep` + `--max-rounds <n>`：提高证据扩展强度，并把 review/revision loop 拉深
  - `--only-target <id>`（可重复）：针对结构化 Stage-I 任务 `I4`-`I8`，回读 `RESEARCH/[topic]/code/` 下的现有 artifact，并且只重跑指定 actionable target
  - 内置 profile 新增 `focused-delivery`、`deep-research`；原有 `default`、`rapid-draft`、`strict-review` 仍可用

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
- `task-plan`：从合同渲染依赖任务顺序（用于“从哪一步开始做”）
  ```bash
  python3 -m bridges.orchestrator task-plan --task-id F3 --paper-type empirical --topic your-topic --cwd .
  ```
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
  python3 -m bridges.orchestrator role --cwd . --codex-task "..." --claude-task "..." --gemini-task "..."
  ```

---

## 4) Bash 脚本入口（不依赖 pipx）

### 4.1 远程 bootstrap 安装器：`./scripts/bootstrap_research_skill.sh`

用途：
- 在没有 Python 的机器上完成安装或刷新。
- 下载 GitHub release/branch 压缩包，解压后转调其中的 `scripts/install_research_skill.sh`。

```bash
./scripts/bootstrap_research_skill.sh \
  --repo owner/repo \
  --target all \
  --project-dir /path/to/project \
  --overwrite
```

说明：
- 依赖 `bash` 和 `curl` 或 `wget`，以及 `tar`。
- 支持 `--ref <tag-or-branch>` 配合 `--ref-type tag|branch`。
- 默认会安装 shell CLI 命令：`research-skills`、`rsk`、`rsw`。
- 如果你不想安装 shell CLI，可加 `--no-cli`；如需改目录，可用 `--cli-dir <path>`。
- 远程 bootstrap 只支持 `--mode copy`。
- `--doctor` 在没有 `python3` 时会自动跳过。

### 4.2 安装脚本：`./scripts/install_research_skill.sh`

```bash
./scripts/install_research_skill.sh \
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
- 如果需要同时安装 shell CLI，可加 `--install-cli`；默认目录为 `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`，也可用 `--cli-dir <path>` 覆盖。
- `--doctor` 仅在系统存在 `python3` 时运行 `python3 -m bridges.orchestrator doctor --cwd <project>`。

### 4.3 Release 自动化：`./scripts/release_automation.sh`

```bash
./scripts/release_automation.sh publish --version 0.1.0 --from-tag v0.1.0-beta.X
./scripts/release_automation.sh pre  --tag v0.1.0-beta.X
./scripts/release_automation.sh post --tag v0.1.0-beta.X --create-release
```

也可单独运行：

```bash
./scripts/release_preflight.sh [--tag v0.1.0-beta.X] [--skip-smoke] [--no-strict]
./scripts/release_postflight.sh --tag v0.1.0-beta.X [--skip-remote] [--skip-ci-status] [--wait-ci] [--create-release]
```

### 4.4 Beta smoke：`./scripts/run_beta_smoke.sh`

```bash
./scripts/run_beta_smoke.sh
```

### 4.5 CI 注入打包默认上游：`./scripts/inject_project_toml.sh`

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
