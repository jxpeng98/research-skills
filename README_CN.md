# Academic Deep Research Skills

一套专为 Claude Code 设计的深化学术研究技能系统。提供从系统性文献综述、数据提取、理论框架构建到论文起草与评审的全套流水线。

<div align="center">
  <a href="#-快速开始-0--1">🚀 快速开始</a> | 
  <a href="guides/advanced/cli-reference_CN.md">💻 CLI 命令大全</a> | 
  <a href="guides/advanced/agent-skill-collaboration_CN.md">🤝 代理人协同指南</a> | 
  <a href="guides/advanced/extend-research-skills_CN.md">🛠️ 如何二次开发/贡献</a> | 
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
- 🚀 **CCG 强约束代码引擎** - 需求/规划/执行/Review 四步严格拆分的可靠研究代码实施
- 🤖 **多模型（Multi-Model）协同** - 混合调度 Codex、Claude、Gemini 跨阶段作业
- ⚡ **Token 深度优化** - 采用分层结构，指令 Token 开销降低 ~90%

---

## 🚀 快速开始 (0 → 1)

这是熟悉并在你的项目中运行该系统最快的方式。

### 1. 安装 Skills（推荐方式）

更通用的安装方式是直接使用 shell bootstrap，不依赖 Python，只需要 `bash` 和 `curl`/`wget`、`tar`：

```bash
cd /path/to/your/project
curl -fsSL https://raw.githubusercontent.com/jxpeng98/research-skills/main/scripts/bootstrap_research_skill.sh | bash -s -- --project-dir "$PWD" --target all
```

它会同时安装 workflow 资产，以及 shell CLI：`research-skills`、`rsk`、`rsw`，默认落到 `${RESEARCH_SKILLS_BIN_DIR:-~/.local/bin}`。

如果你的机器已经有 Python，仍然可以用 `pipx` 安装升级器 CLI：

```bash
pipx install research-skills-installer
```

安装完成后，你可以使用 `research-skills` 命令，或短别名 `rsk` 和 `rsw`。

### 2. 初始化你的项目环境
运行 upgrade 指令，会将最新的技能包、工作流文件安全地拷贝到当前项目以及本地代理工具（如 `~/.claude/`）的组件目录中。

```bash
cd /path/to/your/project
rsk upgrade --target all --project-dir . --doctor
```

*提示：你可以随时使用 `rsk check` 来检测上游是否有新版本更新。*

如果你已经使用了上面的 shell bootstrap，那么这一步已经完成；后续需要刷新安装内容时，重新执行并加上 `--overwrite` 即可。

*注意：shell 版 `rsk check|upgrade|align` 不需要 Python；`--doctor` 和 orchestrator 相关命令仍然需要 `python3`。*

### 3. 开始跑工作流
在项目目录打开终端，Claude Code 会自动读取挂载的 `RESEARCH/` 相关命令集。

如果你正在使用 **Claude Code**，请直接输入以下快捷指令：

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

如果你**通过纯命令行调度编排**（执行指定的 Task ID）：
```bash
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic <topic> --cwd . --triad
```

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
  --ref v0.1.0-beta.6 \
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
例如，使用 `/code-build --domain economics` 时，系统在运行时只读取 `skills/domain-profiles/economics.yaml`，完全屏蔽不相关的领域代码库与Prompt。这种设计保持了底层的极致精简并杜绝了 Prompt 污染。

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
*(详情请参考 [guides/advanced/agent-skill-collaboration.md](guides/advanced/agent-skill-collaboration.md))*

---

## 多模型并发审查 (`orchestrator`)

支持通过 Orchestrator 网桥，联动本地不同接口服务执行复合流。
*(需预先在环境变量配置了 `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GOOGLE_API_KEY`)*

```bash
# 并发分析：三端平行背靠背审查，并由 Claude 做 Summary
python -m bridges.orchestrator parallel --prompt "分析数据的可靠性约束" --cwd . --summarizer claude

# 契约执行：强制按照 F3 的要求调度
python -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic my-topic --cwd .

# 步进交互模式 (Interactive Mode)：在调用任何 Agent 前暂停并提示输入 Y/n 确认
python -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic my-topic --cwd . -i

# MCP环境严格测试：如果没有相关搜素工具环境则强制阻挡
python -m bridges.orchestrator task-run --task-id B1 --paper-type systematic-review --topic my-topic --cwd . --mcp-strict

# 收敛辅助文件，并强化证据深度/修订深度
python -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic my-topic --cwd . \
  --focus-output manuscript/manuscript.md \
  --research-depth deep \
  --draft-profile deep-research \
  --review-profile strict-review \
  --triad-profile deep-research \
  --triad \
  --max-rounds 4
```

`task-run` 的几个关键控制项：

- `--focus-output <path>`：可重复；只激活本次运行需要的 contract output。
- `--output-budget <n>`：限制本次运行最多处理多少个 contract outputs。
- `--research-depth deep`：显式要求证据扩展、反例搜索、边界条件检查与更窄结论。
- `--max-rounds <n>`：提高 review 阻断后的修订轮数。
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
./scripts/release_automation.sh pre --tag v0.1.0-beta.2
./scripts/release_automation.sh post --tag v0.1.0-beta.2
```

---

## 目录结构介绍

```
research-skills/
├── standards/                # 核心合同真源：workflow/capability map
├── research-paper-workflow/  # 各大平台无缝挂载的便携 Skill 技能包
├── .agent/workflows/         # Claude Code 挂载的斜杠 / 指令文件
├── bridges/                  # Python Orchestrator 多端路由通信网桥
├── skills/                   # 系统全系学术卡片
│   ├── [...]                 # 对应阶段 A 到 J
│   └── domain-profiles/      # (动态挂载的领域知识图谱 Economics, Bio等)
├── schemas/                  # Validator 数据验证
├── eval/                     # 性能及覆盖率对焦 Test Cases
├── guides/                   # 最佳实践与深潜开发文档
├── scripts/                  # CI 维护器
└── tests/                    # 单元测试验证
```

许可协议: MIT
