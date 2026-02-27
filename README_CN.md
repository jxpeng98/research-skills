# Academic Deep Research Skills

一套专为学术研究场景设计的 Claude Code 深度研究技能系统。

## 功能特性

- 📚 **系统性文献综述** - 遵循 PRISMA 方法论
- 📖 **论文深度阅读** - 结构化笔记 + BibTeX
- 🧪 **证据综合与 Meta 分析** - 叙述/定性/定量综合（对齐 PRISMA）
- 📝 **完整论文草稿** - 大纲→全文→claim-evidence 校验→图表规划
- 🧩 **研究设计到发表** - 研究设计、伦理/IRB、投稿打包、rebuttal 返修流程
- 🔍 **研究 Gap 识别** - 5类学术空白分析
- 🧠 **理论框架构建** - 概念关系映射
- ✍️ **学术写作辅助** - 符合学术规范
- 🧱 **跨模型标准合同** - 用统一 Task ID 和产物路径对齐 Codex/Claude/Gemini

## 标准化层

统一标准入口（单一真源）：
- `standards/research-workflow-contract.yaml`
- `standards/mcp-agent-capability-map.yaml`（按 Task ID 编排 MCP + agent）
- Task ID：`A1` ... `I3`
- 产物根目录：`RESEARCH/[topic]/`

可移植 Codex skill 包：
- `research-paper-workflow/SKILL.md`

本地一致性校验器：

```bash
python3 scripts/validate_research_standard.py
python3 -m unittest tests.test_orchestrator_workflows -v
```

多端安装脚本：

```bash
./scripts/install_research_skill.sh --target all --project-dir /path/to/project --doctor
```

CI 流水线：
- `.github/workflows/ci.yml`（在 PR/push 上运行 `py_compile`、严格校验与单元测试）

Beta 发布文档：
- `release/v0.1.0-beta.2.md`
- `release/v0.1.0-beta.1.md`
- `release/rollback.md`
- `release/automation.md`
- `release/templates/beta-acceptance-template.md`

如需把警告也视为失败，使用 `--strict`。

发布自动化：

```bash
./scripts/release_automation.sh pre --tag v0.1.0-beta.2
./scripts/release_automation.sh post --tag v0.1.0-beta.2
```

协同分工规则：
- Skill = 流程路由层（`task_id`、输出路径、质量门）
- MCP = 证据/工具层
- Agents = 起草与复核层（主执行/复核/回退由能力映射定义）

协同增强指南：
- `guides/agent-skill-collaboration.md`
- `guides/install-multi-client.md`

## Skills + Agents 协同流程（ASCII）

```text
用户目标 / Prompt
        |
        v
Skill 路由层（Task ID + paper_type）
  - standards/research-workflow-contract.yaml
  - standards/mcp-agent-capability-map.yaml
        |
        +--------------------------+
        |                          |
        v                          v
MCP 证据采集                  Agent 运行时路由
(search/extraction/stats)     (codex / claude / gemini)
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

## 快速开始

### 安装

将此仓库克隆到你的项目中，Claude Code 会自动识别 `.agent/workflows/` 中的命令。

```bash
git clone <repository-url> research-skills
```

安装到 Codex + Claude Code + Gemini：

```bash
cd research-skills
./scripts/install_research_skill.sh --target all --project-dir /path/to/project --doctor
```

安装脚本说明：
- `--target codex|claude|gemini|all` 选择安装目标。
- `--mode copy|link` 控制复制或软链接。
- `--overwrite` 覆盖已有安装。
- `--dry-run` 预览安装动作，不写文件。

### 使用命令

| 命令 | 用途 | 示例 |
|------|------|------|
| `/paper` | 论文写作工作流入口（选择路径） | `/paper ai-in-education CHI` |
| `/lit-review` | 系统性文献综述 | `/lit-review transformer architecture 2020-2024` |
| `/paper-read` | 深度阅读论文 | `/paper-read https://arxiv.org/abs/2303.08774` |
| `/find-gap` | 识别研究空白 | `/find-gap LLM in education` |
| `/build-framework` | 构建理论框架 | `/build-framework technology acceptance` |
| `/academic-write` | 学术写作辅助 | `/academic-write introduction AI ethics` |
| `/paper-write` | 完整论文草稿 | `/paper-write ai-in-education empirical CHI` |
| `/synthesize` | 证据综合 / Meta 分析 | `/synthesize ai-in-education` |
| `/study-design` | 实证研究设计 | `/study-design ai-in-education` |
| `/ethics-check` | 伦理/IRB 文档包 | `/ethics-check ai-in-education` |
| `/submission-prep` | 投稿打包 | `/submission-prep ai-in-education CHI` |
| `/rebuttal` | 返修/回复审稿意见 | `/rebuttal ai-in-education` |
| `/code-build` | 构建研究代码 | `/code-build \"Staggered DID\" --domain econ` |

Task ID 建议：
- 与用户确认 `paper_type + task_id`（例如 `systematic-review + E3`）
- 严格按照 `standards/research-workflow-contract.yaml` 的输出路径落盘

## 三端并发分析（parallel）

并发执行 `codex + claude + gemini`，再由总结端做综合分析：

```bash
# 预检：检查本地 CLI、API Key、MCP 命令绑定
python -m bridges.orchestrator doctor --cwd ./project

python -m bridges.orchestrator parallel \
  --prompt "分析该研究方案的主要风险与改进点" \
  --cwd ./project \
  --summarizer claude

# 可选：按运行自定义人格/风格/权限（非全局）
python -m bridges.orchestrator parallel \
  --prompt "审查该研究方案的证据风险与修复顺序" \
  --cwd ./project \
  --summarizer claude \
  --profile-file ./standards/agent-profiles.example.json \
  --profile strict-review \
  --summarizer-profile strict-review
```

说明：
- `doctor` 会检查 CLI 可用性、关键环境变量、标准文件、外部 MCP 命令绑定。
- 默认尝试三端并发；若某端不可用，会自动退化为双端或单端。
- `--summarizer` 指定并发后负责总结归纳的端（`codex|claude|gemini`）。
- `--profile-file/--profile/--summarizer-profile` 支持按本次运行注入人格、审稿风格和工具权限配置。
- 运行时默认启用非交互执行（`CI=1`、`TERM=dumb`）+ 硬超时，避免并发阶段卡死。

## Agent 编排（task-run）

使用 Task ID + 能力映射做标准化协同：

```bash
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project

# 可选：强制要求所需 MCP 可用
python -m bridges.orchestrator task-run \
  --task-id B1 \
  --paper-type systematic-review \
  --topic ai-in-education \
  --cwd ./project \
  --mcp-strict

# 可选：强制要求所需技能规范文件可用
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project \
  --skills-strict

# 可选：启用三端独立审查（Codex + Claude + Gemini）
python -m bridges.orchestrator task-run \
  --task-id G3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project \
  --triad

# 可选：按阶段覆盖 profile（draft/review/triad 分开控制）
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project \
  --profile-file ./standards/agent-profiles.example.json \
  --profile default \
  --draft-profile rapid-draft \
  --review-profile strict-review \
  --triad-profile strict-review
```

说明：
- `task-run` 会读取 `standards/mcp-agent-capability-map.yaml` 选择主执行/复核/回退 agent。
- `task-run` 已支持 `codex`、`claude`、`gemini` 三端运行时直连执行。
- 若映射到本地不可用 agent，会按能力映射自动回退到可用运行时 agent。
- `task-run` 会自动注入 `task_skill_mapping.required_skills` 到 draft/review 提示词。
- `task-run` 会自动注入 `skill_catalog` 中的 `required_skill_cards`（类别、focus、默认产物、技能规范路径）。
- `task-run --profile-file` + `--draft-profile/--review-profile/--triad-profile` 可对不同阶段单独设定 profile，而不改全局默认。
- `task-run --skills-strict` 会在技能规范文件缺失时阻断执行。
- `task-run --triad` 会增加第三端独立审查，使非代码阶段也能稳定三端协同。
- 外部 MCP 可通过环境变量命令接入，例如：`RESEARCH_MCP_SCHOLARLY_SEARCH_CMD`。

Profile 模板：`standards/agent-profiles.example.json`

## 核心工作流

### 1. 系统性文献综述 `/lit-review`

遵循 PRISMA 2020 方法论：

```
研究问题定义 (PICO/PEO)
       ↓
多学术数据库检索 (Semantic Scholar, arXiv, Google Scholar)
       ↓
标题/摘要筛选 → 全文筛选
       ↓
数据提取 + 质量评估
       ↓
证据综合（可选 Meta 分析） + PRISMA 流程图
```

### 2. 论文深度阅读 `/paper-read`

从论文中提取：
- 研究问题 (RQs)
- 理论框架
- 研究方法 (设计、样本、分析)
- 核心发现
- 贡献与局限性
- Future Work

输出格式：Markdown 笔记 + BibTeX 引用

### 3. 研究 Gap 识别 `/find-gap`

识别以下类型的研究空白：
- **理论 Gap** - 框架不完整或冲突
- **方法论 Gap** - 研究方法存在局限
- **实证 Gap** - 缺乏特定情境证据
- **知识 Gap** - 某主题研究不足
- **人群 Gap** - 特定群体未被研究

### 4. 理论框架构建 `/build-framework`

- 现有理论梳理与对比
- 概念关系映射 (Mermaid 图谱)
- 假设/命题推导

### 5. 学术写作辅助 `/academic-write`

支持论文各章节：
- Introduction (研究背景、问题陈述)
- Literature Review (主题组织、批判性综述)
- Methodology (方法论证明)
- Discussion (发现解释、理论对话)
- Conclusion (贡献总结、局限性)

## 学术证据评级

| 等级 | 证据类型 |
|------|----------|
| **A** | 系统性综述、Meta 分析、RCT |
| **B** | 队列研究、高影响因子期刊论文 |
| **C** | 案例研究、专家意见、会议论文 |
| **D** | 预印本、工作论文 |
| **E** | 轶事、理论推测 |

## 目录结构

```
research-skills/
├── .agent/workflows/     # 用户命令
├── skills/               # 可复用技能模块
├── guides/               # 协同增强指南
├── templates/            # 输出模板
├── scripts/              # 本地校验与工具脚本
├── standards/            # 规范合同（Task ID + 输出路径）
├── research-paper-workflow/  # 可移植 Codex skill 包
├── RESEARCH/             # 研究输出目录
├── CLAUDE.md             # Claude Code 快速参考
└── README.md             # 本文件
```

## 学术数据库支持

- **Semantic Scholar** - 跨领域 (2亿+论文)，API 直接调用
- **arXiv** - 物理/数学/CS/AI，API 直接调用
- **Google Scholar** - 跨领域，通过网页搜索
- **PubMed** - 生物医学，通过网页搜索

## License

MIT
