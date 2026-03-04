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

### 1. 安装 CLI（推荐方式）

推荐使用 `pipx` 全局安装 Research Skills 编排工具：

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
| `/code-build` | CCG驱动的研究代码规划与实施 | `/code-build "Staggered DID" --domain econ` |
| `/proofread` | AI 去痕与终审校对 | `/proofread ai-in-education` |

如果你**通过纯命令行调度编排**（执行指定的 Task ID）：
```bash
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic <topic> --cwd . --triad
```

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

# MCP环境严格测试：如果没有相关搜素工具环境则强制阻挡
python -m bridges.orchestrator task-run --task-id B1 --paper-type systematic-review --topic my-topic --cwd . --mcp-strict
```

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
