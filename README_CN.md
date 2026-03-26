# Academic Deep Research Skills

一套面向 Codex、Claude Code 与 Gemini 的契约驱动学术工作流系统，把安装、任务规划、文献工作、论文写作与严格 Stage-I 研究代码执行统一到同一套标准合同之下。

<div align="center">
  <a href="#-快速开始-0--1">🚀 快速开始</a> | 
  <a href="docs/zh/reference/cli.md">💻 CLI 命令大全</a> | 
  <a href="docs/zh/architecture.md">🏗 系统架构</a> | 
  <a href="docs/zh/advanced/agent-skill-collaboration.md">🤝 代理人协同指南</a> | 
  <a href="docs/zh/advanced/extend-research-skills.md">🛠️ 如何二次开发/贡献</a> | 
  <a href="TODO_ROADMAP.md">🗺️ Roadmap 蓝图</a>
</div>

## 功能特性

- 📚 **系统性文献综述** - 遵循 PRISMA 2020 方法论
- 📖 **论文深度阅读** - 结构化笔记 + BibTeX 引用生成
- 🧪 **证据综合与 Meta 分析** - 叙述/定性/定量综合（对齐 PRISMA）
- 📝 **完整论文草稿** - 大纲→全文→claim-evidence 校验→图表规划
- 🧩 **研究设计到发表** - 研究设计、伦理/IRB、投稿打包、rebuttal 返修流程
- 🔍 **研究 Gap 识别** - 5类学术空白深度分析
- 🧠 **理论框架构建** - 概念关系映射与假设推导
- ✍️ **学术写作辅助** - 严格对齐各领域的学术语言规范
- 🧑‍⚖️ **多角色专家互审** - 平行独立审稿模拟（Methodologist, Domain Expert, "Reviewer 2"）
- 🔎 **AI 去痕与终审校对** - 多 AI 协作去 AI 化改写、降重检测、终审校对
- 🚀 **严格 Stage-I 学术代码流** - `I5 -> I6 -> I7 -> I8` 结构化 spec/plan/execute/review 产物，以及定向 follow-up 重跑
- 🎤 **学术报告制作** - 故事线设计 → 幻灯片内容定义 → 输出到 Slidev（scholarly 主题）、LaTeX Beamer 或 PPTX
- 🤖 **多模型（Multi-Model）协同** - 混合调度 Codex、Claude、Gemini 跨阶段作业
- ⚡ **Token 深度优化** - 采用分层结构，指令 Token 开销降低 ~90%

> [!WARNING]
> 如果你要使用“完整功能集”，需要真实安装并配置：
> `python3`、`codex`、`claude`、`gemini` 四个运行时入口，以及对应的 `OPENAI_API_KEY`、`ANTHROPIC_API_KEY`、`GOOGLE_API_KEY`。
> 如果缺少这些依赖，你仍然可以完成 shell 安装和 `rsk check|upgrade|align`，但 `doctor`、validator、tests 和完整 orchestrator 多模型执行链会受限或不可用。

## 设计借鉴与相关项目

这个仓库不是凭空长出来的，两个外部项目对它的设计方向尤其重要：

- [fengshao1227/ccg-workflow](https://github.com/fengshao1227/ccg-workflow)
  - 本项目借鉴了它“强阶段隔离”的流程思想：spec -> plan -> execute -> review。
  - 也借鉴了“通过流程约束减少模型即兴发挥”的思路，而不是把整个任务塞进一个大 prompt。
  - 但两者目标不同：CCG 更偏工程开发协作；本仓库把这些思想本地化成学术场景里的 `I5 -> I6 -> I7 -> I8` Stage-I 任务，以及 `RESEARCH/[topic]/` 下的合同化产物。
- [GuDaStudio/skills](https://github.com/GuDaStudio/skills)
  - 这个项目对 Claude-oriented skill 打包方式，以及 Codex / Gemini 协作能力的可安装化，提供了很好的参考。
  - 但本仓库的重点不同：`GuDaStudio/skills` 更像通用协作 skill 集合，而 `research-skills` 更强调“单一研究合同 + 单一任务目录 + 单一产物树”的学术工作流。

---

## 🚀 快速开始 (0 → 1)

这是从“还没装”到“开始跑 canonical task”的最短稳定路径。

需要细节时，优先看已经整理好的文档入口：

- [快速开始](docs/zh/quickstart.md)
- [安装指南](docs/zh/guide/install.md)
- [CLI 参考](docs/zh/reference/cli.md)
- [系统架构](docs/zh/architecture.md)

### 1. 先选入口

稳定入口现在有三类：

- `.agent/workflows/*.md` 里的 workflow 命令，例如 `/paper`、`/lit-review`、`/paper-write`、`/code-build`
- 安装 / 升级 CLI：`research-skills`、`rsk`、`rsw`
- Orchestrator CLI：`python3 -m bridges.orchestrator ...`

### 2. 安装或刷新系统

最通用的安装方式是 shell bootstrap，不依赖 Python，只需要 `bash`、`curl`/`wget`、`tar`：

```bash
cd /path/to/your/project
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --project-dir "$PWD" --target all
```

它会安装：

- Codex / Claude Code / Gemini 的 workflow 资产
- 项目集成文件，例如 `.agent/workflows/`、`CLAUDE.md`、`.gemini/`
- shell CLI：`research-skills`、`rsk`、`rsw`，默认落到 `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`

如果机器已经有 Python，且你只想继续使用 Python 分发的升级器 CLI，也可以：

```bash
pipx install research-skills-installer
```

从项目目录刷新已有安装时：

```bash
rsk upgrade --target all --project-dir . --doctor
```

如果你已经跑过上面的 shell bootstrap，后续刷新时重新执行 bootstrap 或 `rsk upgrade --overwrite` 即可。

*Python 边界：shell 版 `rsk check|upgrade|align` 不需要 Python；`--doctor`、`python3 -m bridges.orchestrator ...`、validator 和 tests 仍然需要 `python3`。*

### 3. 先做环境检查

如果机器有 Python，建议在跑大任务前先做稳定预检：

```bash
python3 -m bridges.orchestrator doctor --cwd .
python3 scripts/validate_research_standard.py --strict
```

其中：

- `doctor` 负责检查运行时 CLI、API key、MCP wiring
- validator 负责检查仓库级 contract / schema 一致性

### 4. 先 plan 再 run

先看任务依赖、产物路径和路由：

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

`task-plan` 会展示：

- contract outputs
- 前置任务
- functional owner 与 handoff trace
- runtime plan（`draft` / `review` / `fallback`）

### 5. 运行 canonical research task

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

常用控制项：

- `--focus-output` 与 `--output-budget`：缩小 active outputs，减少辅助文件扩散
- `--research-depth deep` 配合 `--max-rounds`：强制更窄、更有对抗性的证据扩展与修订流程
- `--only-target <id>`：对结构化 Stage-I 任务 `I4`-`I8`，回读现有 artifact，并且只重跑指定 actionable target

示例：只重跑一个 planning step

```bash
python3 -m bridges.orchestrator task-run \
  --task-id I6 \
  --paper-type methods \
  --topic llm-bias \
  --cwd . \
  --only-target S1
```

### 6. 运行严格学术代码流

当代码本身是研究产物，而不是泛工程实现时，用 `code-build`：

```bash
python3 -m bridges.orchestrator code-build \
  --method "Staggered DID" \
  --topic policy-effects \
  --domain econ \
  --focus full \
  --cwd .
```

带上 `--topic` 后，`code-build` 会进入严格 Stage-I 路径：

- `I5` code specification
- `I6` zero-decision planning
- `I7` execution + performance packaging
- `I8` review

并且支持 targeted follow-up：

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

### 7. 需要 slash-command UX 时再用 workflow 命令

如果你的客户端已经挂载了 workflow 入口 markdown，可以直接用这些命令：

| 命令 | 用途 | 示例 |
|------|------|------|
| `/paper` | 论文写作工作流入口（基于对话选择） | `/paper ai-in-education CHI` |
| `/lit-review` | 系统性文献综述 | `/lit-review transformer architecture 2020-2024` |
| `/paper-read` | 深度阅读单篇论文 | `/paper-read https://arxiv.org/abs/2303.08774` |
| `/find-gap` | 识别研究空白（5种 Gap） | `/find-gap LLM in education` |
| `/build-framework` | 构建理论框架与概念图谱 | `/build-framework technology acceptance` |
| `/academic-write` | 学术段落/章节写作辅助 | `/academic-write introduction AI ethics` |
| `/paper-write` | 完整论文（草稿端到端） | `/paper-write ai-in-education empirical CHI` |
| `/synthesize` | 证据综合 / Meta 分析规划 | `/synthesize ai-in-education` |
| `/study-design` | 实证研究设计 | `/study-design ai-in-education` |
| `/ethics-check` | 伦理评估与 IRB 审查材料 | `/ethics-check ai-in-education` |
| `/submission-prep` | 投稿材料打包生成 | `/submission-prep ai-in-education CHI` |
| `/rebuttal` | 审稿意见回复与矩阵生成 | `/rebuttal ai-in-education` |
| `/code-build` | 严格 Stage-I 学术代码流 | `/code-build "Staggered DID" --topic policy-effects --domain econ --focus full` |
| `/proofread` | AI 去痕与终审校对 | `/proofread ai-in-education` |
| `/academic-present` | 学术报告制作 | `/academic-present ai-in-education --format slidev` |

---

## CLI 安装与参数说明

这一节只说明“安装器/升级器 CLI”本身，不展开 orchestrator 的研究执行参数。

### 1. CLI 有哪几种安装方式

#### 方案 A：Shell bootstrap 安装 CLI（推荐）

适用场景：
- 机器上没有 Python
- 你只想快速安装 `research-skills` / `rsk` / `rsw`
- 你希望同时把 workflow 资产也装好

命令：

```bash
cd /path/to/your/project
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- \
  --project-dir "$PWD" \
  --target all
```

效果：
- 安装 shell CLI：`research-skills`、`rsk`、`rsw`
- 安装 `research-paper-workflow` skill 到对应客户端目录
- 安装项目内 `.agent/workflows/`、`CLAUDE.md`、`.gemini/` 等集成文件

默认 CLI 目录：
- `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`

如果装完后命令不可用，通常是因为这个目录不在 `PATH` 中。可在 shell 配置里加入：

```bash
export PATH="$HOME/.local/bin:$PATH"
```

#### 方案 B：通过 `pipx` 安装 Python CLI

适用场景：
- 机器上已经有 Python
- 你想继续使用现有 PyPI 分发方式

命令：

```bash
pipx install research-skills-installer
```

效果：
- 安装 Python 版 `research-skills` / `rsk` / `rsw`
- CLI 本身进入 PATH
- 不会自动把 workflow 资产写入你的项目，仍需手动执行 `rsk upgrade`

#### 方案 C：从本地仓库安装 shell CLI

适用场景：
- 你已经 clone 了这个仓库
- 你希望控制安装目录，或用 `link` 模式维护本地副本

命令：

```bash
./scripts/install_research_skill.sh \
  --target all \
  --project-dir /path/to/project \
  --install-cli \
  --overwrite
```

### 2. Shell bootstrap 参数说明

入口脚本：
- `scripts/bootstrap_research_skill.sh`

常用参数：

| 参数 | 作用 | 默认值 / 说明 |
|------|------|---------------|
| `--repo <owner/repo|git-url>` | 指定上游 GitHub 仓库 | 默认取 `RESEARCH_SKILLS_REPO`，否则 `jxpeng98/research-skills` |
| `--ref <tag-or-branch>` | 指定安装的版本或分支 | 默认自动解析 latest release |
| `--ref-type <tag|branch>` | 指定 `--ref` 是 tag 还是 branch | 默认 `tag` |
| `--target <codex|claude|gemini|all>` | 指定写入哪些客户端目录 | 默认 `all` |
| `--project-dir <path>` | 指定项目集成文件的写入目录 | 默认当前目录 |
| `--install-cli` | 安装 shell CLI | 默认开启 |
| `--no-cli` | 跳过 shell CLI 安装，只装 workflow 资产 | 与 `--install-cli` 相反 |
| `--cli-dir <path>` | 指定 shell CLI 安装目录 | 默认 `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}` |
| `--overwrite` | 覆盖已存在的 skill / CLI / 项目文件 | 默认关闭 |
| `--doctor` | 安装后运行环境预检 | 仅在存在 `python3` 时执行 |
| `--dry-run` | 只打印将要执行的动作 | 不实际下载和写文件 |

示例：

```bash
# 安装指定 tag
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- \
  --repo jxpeng98/research-skills \
  --ref v0.1.0 \
  --ref-type tag \
  --project-dir "$PWD" \
  --target all \
  --overwrite

# 只装 workflow，不装 CLI
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- \
  --project-dir "$PWD" \
  --target claude \
  --no-cli

# 预演安装动作
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- \
  --project-dir "$PWD" \
  --target codex \
  --dry-run
```

### 3. 本地安装脚本参数说明

入口脚本：
- `scripts/install_research_skill.sh`

常用参数：

| 参数 | 作用 | 默认值 / 说明 |
|------|------|---------------|
| `--target <codex|claude|gemini|all>` | 指定写入哪些客户端目录 | 默认 `all` |
| `--mode <copy|link>` | 复制文件或创建软链接 | 默认 `copy` |
| `--project-dir <path>` | 指定项目集成文件写入目录 | 默认当前目录 |
| `--install-cli` | 安装 shell CLI | 默认关闭 |
| `--no-cli` | 跳过 shell CLI 安装 | 默认行为 |
| `--cli-dir <path>` | 指定 shell CLI 安装目录 | 默认 `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}` |
| `--overwrite` | 覆盖已有目标 | 默认关闭 |
| `--doctor` | 安装后运行 `python3 -m bridges.orchestrator doctor` | 仅在存在 `python3` 时执行 |
| `--dry-run` | 只打印将要执行的动作 | 不实际写文件 |

说明：
- 如果你想长期维护一个本地 clone，推荐 `--mode link`
- 如果你只想一次性安装，推荐 `--mode copy`
- `--mode link` 适合本地仓库安装，不适合远程 bootstrap

### 4. `rsk` / `research-skills` CLI 子命令说明

shell CLI 和 Python CLI 共享相同的命令名：
- `research-skills`
- `rsk`
- `rsw`

#### `rsk check`

用途：
- 查看本地已安装 skill 版本
- 查看上游最新 release
- 判断是否有可升级版本

参数：

| 参数 | 作用 |
|------|------|
| `--repo <owner/repo|url>` | 指定上游仓库 |
| `--json` | 输出 JSON，便于脚本或 CI 使用 |
| `--strict-network` | 若上游查询失败则返回失败 |

示例：

```bash
rsk check
rsk check --repo jxpeng98/research-skills
rsk check --json
```

#### `rsk upgrade`

用途：
- 下载上游 release/branch 压缩包
- 调用安装器刷新 skills、项目集成文件，以及 shell CLI

常用参数：

| 参数 | 作用 |
|------|------|
| `--repo <owner/repo|url>` | 指定上游仓库 |
| `--ref <tag-or-branch>` | 指定版本或分支 |
| `--ref-type <tag|branch>` | 指定 ref 类型 |
| `--target <codex|claude|gemini|all>` | 指定安装目标 |
| `--project-dir <path>` | 指定项目路径 |
| `--no-cli` | 升级时不刷新 shell CLI |
| `--cli-dir <path>` | 指定 shell CLI 目录 |
| `--overwrite` | 覆盖已有目标 |
| `--doctor` | 升级后执行 doctor |
| `--dry-run` | 预演升级动作 |

示例：

```bash
rsk upgrade --project-dir . --target all --overwrite
rsk upgrade --repo jxpeng98/research-skills --ref main --ref-type branch --project-dir . --target claude
rsk upgrade --project-dir . --target codex --dry-run
```

#### `rsk align`

用途：
- 打印一个简短说明，告诉你 CLI 装了什么、`upgrade` 会改哪些路径

参数：

| 参数 | 作用 |
|------|------|
| `--repo <owner/repo|url>` | 仅用于在示例输出中替换 repo 提示 |

示例：

```bash
rsk align
rsk align --repo jxpeng98/research-skills
```

### 5. 常用环境变量

| 环境变量 | 作用 |
|----------|------|
| `RESEARCH_SKILLS_REPO` | 默认上游仓库，省去每次传 `--repo` |
| `RESEARCH_SKILLS_BIN_DIR` | shell CLI 默认安装目录 |
| `CODEX_HOME` | Codex skill 安装根目录 |
| `CLAUDE_CODE_HOME` | Claude Code skill 安装根目录 |
| `GEMINI_HOME` | Gemini skill 安装根目录 |
| `GITHUB_TOKEN` / `GH_TOKEN` | 私有仓库或 GitHub API 限流时的认证令牌 |

### 6. 什么时候需要 Python

不需要 Python 的部分：
- shell bootstrap 安装
- shell CLI 的 `check` / `upgrade` / `align`
- 本地安装脚本的 `copy/link` 资产安装

仍然需要 Python 的部分：
- `--doctor`
- `python3 -m bridges.orchestrator ...`
- 仓库内其他 validator / orchestrator / test 命令

---

## 🧬 动态领域挂载 (Dynamic Domains)

**为什么没有针对 Economics、Computer Science 或 Biology 单独做拆分安装包？**

在系统架构上，我们**将“核心执行管线”与“学科专业知识”彻底解耦**。
当你执行在客户端执行安装时，获取的仅仅是纯“骨架”能力（比如如何做系统性综述，如何规划提纲）。 

实际执行时，特定的检查清单（如经济学的平行趋势检验、生物的 IRB 安全条例）均通过 `--domain` 以 **动态按需挂载 (Runtime Injection)** 方式实现。
例如，使用 `/code-build --domain econ` 时，系统会在运行时加载 `skills/domain-profiles/economics.yaml`，应用经济学专属诊断，并屏蔽不相关领域 profile。这种设计保持底层安装轻量，同时避免 Prompt 污染。

---

## 🏗 标准化层与跨模型契约
为了让 Codex、Claude、Gemini 输出可相互继承的中间件，系统使用严苛的“契约”驱动运转。

- **工作流契约**: `standards/research-workflow-contract.yaml` (所有 Task ID，必需前置条件与质量门规范)
- **能力映射路由**: `standards/mcp-agent-capability-map.yaml` (所有 MCP 工具代理，自动 fallback 以及检查清单）
- **落盘规范**: 所有代理人生成的学术内容必须严格落进 `RESEARCH/[topic]/` 对应目录下。

### Skills + Agents 协同流程（ASCII）

```text
用户目标 / Prompt
        |
        v
Skill 路由层（Task ID + paper_type）
        |
        +--------------------------+
        |                          |
        v                          v
MCP 证据采集                  Agent 运行时路由
        |                          |
        +------------+-------------+
                     v
                  Draft 生成
                     |
                     v
                  Review 复核
                     |
         +-----------+-----------+
         |                       |
         v                       v
   Triad 三端审查（可选）   双端/单端自动降级
                     \       /
                      v     v
                 Summarizer 综合
                     |
                     v
           质量门 + 产物落盘输出
              -> RESEARCH/[topic]/...
```
*(详情请参考 [docs/zh/advanced/agent-skill-collaboration.md](docs/zh/advanced/agent-skill-collaboration.md)；旧版镜像路径仍保留在 [guides/advanced/agent-skill-collaboration.md](guides/advanced/agent-skill-collaboration.md))*

---

## 多模型并发审查 (`orchestrator`)

支持通过 Orchestrator 网桥，联动本地不同接口服务执行复合流。
*(需预先在环境变量配置了 `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`)*

```bash
# 先看任务前置、产物和路由
python3 -m bridges.orchestrator task-plan --task-id F3 --paper-type empirical --topic my-topic --cwd .

# 并发分析：三端平行背靠背审查，并由 Claude 做 Summary
python3 -m bridges.orchestrator parallel --prompt "分析数据的可靠性约束" --cwd . --summarizer claude

# 契约执行：强制按照 F3 的要求调度
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic my-topic --cwd .

# 严格 Stage-I 学术代码流
python3 -m bridges.orchestrator code-build --method "Staggered DID" --topic my-topic --domain econ --focus full --cwd .

# 步进交互模式 (Interactive Mode)：在调用任何 Agent 前暂停并提示输入 Y/n 确认
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic my-topic --cwd . -i

# MCP环境严格测试：如果没有相关搜素工具环境则强制阻挡
python3 -m bridges.orchestrator task-run --task-id B1 --paper-type systematic-review --topic my-topic --cwd . --mcp-strict

# 收敛辅助文件，并强化证据深度/修订深度
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic my-topic --cwd . \
  --focus-output manuscript/manuscript.md \
  --research-depth deep \
  --draft-profile deep-research \
  --review-profile strict-review \
  --triad-profile deep-research \
  --triad \
  --max-rounds 4

# 只重开指定 Stage-I target
python3 -m bridges.orchestrator code-build --method "Transformer Fine-Tuning" --topic llm-bias --domain cs --focus full \
  --only-target I5:decision-1 \
  --only-target I8:P1-01 \
  --cwd .
```

`task-run` 的几个关键控制项：

- `--focus-output <path>`：可重复；只激活本次运行需要的 contract output。
- `--output-budget <n>`：限制本次运行最多处理多少个 contract outputs。
- `--research-depth deep`：显式要求证据扩展、反例搜索、边界条件检查与更窄结论。
- `--max-rounds <n>`：提高 review 阻断后的修订轮数。
- `--only-target <id>`：对 Stage-I 结构化产物，回读已有 artifact，并且只重跑指定 actionable target。
- 内置 profiles：`focused-delivery`、`deep-research`、`strict-review`、`rapid-draft`、`default`。

---

## 支持接入的学术数据库映射

| API 来源 | 用途 | 覆盖范围 |
|--------|---------|----------|
| Semantic Scholar | 第一搜索源文献检索 | 200M+ 论文 |
| arXiv | 理工科预印本读取 | 全集 |
| OpenAlex | 文献计量与本体网络 | 250M+ 作品 |
| Crossref | DOI 源数据核对验证 | 140M+ DOIs |

---

## 开发者与贡献者指引

由于该项目为高度结构化的学术框架，禁止直接魔改导致 Schema 失效报错。

### CI 流水线与本地验证
如果你修改了 yaml 合同、修改了路由链路，或者修改了 `.md` 的依赖产物节点，请必须使用以下命令校验通过：

```bash
# 验证框架格式合同 (无 warning 方可合并)
python3 scripts/validate_research_standard.py --strict
# 运行单元测试
python3 -m unittest tests.test_orchestrator_workflows -v

# 验证你在项目里最新跑出来的数据结果结构是否与合同相符
python3 scripts/validate_project_artifacts.py --cwd ./project  --topic <topic> --task-id H1 --strict
```

如果你希望测试传统的底层安装脚本能力，请使用: `scripts/install_research_skill.sh`


### 发版自动化 (Release Automation)
由 CI 接管或手动拉草稿：
```bash
./scripts/release_automation.sh pre --tag v0.1.0 --from-tag v0.1.0-beta.6
./scripts/release_automation.sh post --tag v0.1.0 --create-release
```

---

## 目录结构介绍

```
research-skills/
├── standards/                # 核心合同真源：workflow/capability map
├── research-paper-workflow/  # 各大平台无缝挂载的便携 Skill 技能包
├── .agent/workflows/         # 安装后的 workflow 入口 markdown / slash-command surface
├── bridges/                  # Python Orchestrator 多端路由通信网桥
├── skills/                   # 系统全系学术卡片
│   ├── [...]                 # 对应阶段 A 到 K
│   └── domain-profiles/      # (动态挂载的领域知识图谱 Economics, Bio等)
├── schemas/                  # Validator 数据验证
├── eval/                     # 性能及覆盖率对焦 Test Cases
├── guides/                   # 最佳实践与深潜开发文档
├── scripts/                  # CI 维护器
└── tests/                    # 单元测试验证
```

许可协议: MIT
