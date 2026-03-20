# Skills 指南

这一页是面向使用者的 `skills/` 全景说明。

它主要回答这些问题：

- 当前研究问题应该落在哪个 stage？
- 每个 stage 里到底包含哪些能力？
- 哪些 skill 是 canonical、会被系统自动注入？
- 哪些 markdown 文件只是补充卡片或 Stage-I 镜像目录，不应该和主 skill 混在一起理解？

::: tip Canonical Source
系统自动路由的 canonical skill 列表以 `skills/registry.yaml` 为准；这一页是在它基础上的用户版说明。
:::

## 使用者应该怎样理解 `skills/`

- **workflow 命令**，例如 `/paper`、`/lit-review`、`/code-build`，是用户入口。
- **Task ID**，例如 `B2`、`F3`、`I6`，是 contract 层的标准工作单元。
- **skill** 是 orchestrator 在后台通过 `required_skills` 和 `required_skill_cards` 注入的可复用执行规格。

所以，大多数使用者并不需要手工挑选 `skills/*.md` 去逐个执行。
你通常只需要选择：

1. 一个 workflow 入口，或
2. 一个 Task ID（通过 `task-plan` / `task-run`）。

然后系统会自动决定应该加载哪些 skill。

如果你需要看精确命令参数，去 [CLI 参考](/zh/reference/cli)。
如果你需要理解运行时 Agent 与 Skill 如何协同，去 [Agent + Skill 协同](/zh/advanced/agent-skill-collaboration)。
如果你要修改系统本身，去 [扩展 Research Skills](/zh/advanced/extend-research-skills)。
如果你更关心“系统综述 / methods paper / 审稿回复”这种真实场景怎么选路径，请看 [任务场景](/zh/guide/task-recipes)。

## 先记住几个边界

- 当前 internal skill registry 覆盖的是 `A` 到 `I` 阶段，再加 `Z_cross_cutting`。
- `J` 类 proofread / polish 入口目前主要在 workflow 层，不是单独的 top-level internal skill stage。
- `skills/` 下面有一部分文件是**补充卡片**，还有一部分是 Stage-I 代码链路的**镜像目录**；它们都很有用，但不等于“独立的 canonical routed skill”。

## Stage 总览

| Stage | 关注点 | 使用者最常见的问题 |
|---|---|---|
| `A_framing` | 选题、问题、理论、gap、期刊定位 | “我的研究问题和贡献到底是什么？” |
| `B_literature` | 检索、筛选、提取、引文、文献地图 | “文献怎么系统找、系统筛、系统整理？” |
| `C_design` | 研究设计、变量、稳健性、数据可得性 | “这个研究该怎么设计和 operationalize？” |
| `D_ethics` | IRB、隐私、治理 | “伦理与数据合规材料要怎么准备？” |
| `E_synthesis` | 证据综合、质量评估、发表偏倚 | “已有证据要怎么整合和评级？” |
| `F_writing` | 结构、结果解释、表格、图、摘要 | “如何把分析结果写成论文？” |
| `G_compliance` | PRISMA、报告规范、学术语气 | “论文是否已经满足提交前规范？” |
| `H_submission` | 投稿包、回复审稿、模拟评审 | “投稿前后怎么打包和应对审稿？” |
| `I_code` | 学术代码、统计、可复现性 | “研究代码如何实现、审查、复现？” |
| `Z_cross_cutting` | 元数据、多模型协作、自我批判 | “哪些能力是跨阶段通用的？” |

## 按 Stage 看 Canonical Skills

### A. Framing

当你还在定义研究问题、理论锚点、贡献定位、目标期刊时，用 Stage A。

| Skill | 适用场景 | 典型结果 |
|---|---|---|
| `question-refiner` | 选题还很模糊、太大、不可执行 | 结构化研究问题、范围、检索词 |
| `hypothesis-generator` | 需要把问题变成可检验假设或 proposition | 假设集合、机制、边界条件 |
| `theory-mapper` | 需要概念图、理论框架或 Mermaid 图 | 理论框架图、概念关系图 |
| `gap-analyzer` | 需要从已有文献里证明 novelty | 优先级 gap 分析 |
| `venue-analyzer` | 方向基本清楚，想判断期刊/会议是否匹配 | venue fit 说明、格式约束、审稿偏好 |

### B. Literature

当你要构建某个主题的文献基础，尤其是系统综述或可复现检索流程时，用 Stage B。

| Skill | 适用场景 | 典型结果 |
|---|---|---|
| `academic-searcher` | 需要可复现检索式和数据库搜索 | query plan、search results、search log |
| `paper-screener` | 需要做纳入/排除决策 | screening log、PRISMA 计数 |
| `paper-extractor` | 需要把入选论文整理成结构化笔记 | extraction table、paper notes |
| `citation-snowballer` | 已有种子文献，但覆盖还不够 | forward/backward snowball log |
| `fulltext-fetcher` | 找到论文了但还缺全文 | full-text status 与获取记录 |
| `citation-formatter` | 写作前需要统一引文格式 | bibliography、citekeys、BibTeX |
| `concept-extractor` | 检索概念还不稳定，需要补 controlled vocabulary | concept map、Boolean 术语集 |
| `literature-mapper` | 需要把文献流派、机制、问题重新分组 | literature map、主题簇结构 |
| `reference-manager-bridge` | 需要和 Zotero / Mendeley / EndNote 交换文献 | RIS / CSLJSON / bibliography 同步结果 |

### C. Design

当问题已经较清楚，下一步变成“怎么设计研究、怎么找数据、怎么定义变量”时，用 Stage C。

| Skill | 适用场景 | 典型结果 |
|---|---|---|
| `study-designer` | 需要完整研究设计骨架 | design spec、analysis plan、instrument、prereg handoff |
| `rival-hypothesis-designer` | 需要主动处理竞争解释和 alternative theory | rival explanation matrix |
| `robustness-planner` | 需要预先规定稳健性与敏感性分析 | robustness / sensitivity plan |
| `dataset-finder` | 不确定有哪些可行数据源 | dataset feasibility / access plan |
| `variable-constructor` | 需要把概念转成可审计变量 | variable spec、coding rules |

### D. Ethics

当研究涉及 IRB、人类受试者、敏感数据或数据治理要求时，用 Stage D。

| Skill | 适用场景 | 典型结果 |
|---|---|---|
| `ethics-irb-helper` | 需要正式伦理材料 | IRB 材料、consent、招募文本、治理说明 |
| `deidentification-planner` | 需要技术层面的隐私保护方案 | 去标识化 / 隐私控制计划 |

### E. Synthesis

当你已经有了证据材料，现在要做证据整合、质量评估或偏倚检查时，用 Stage E。

| Skill | 适用场景 | 典型结果 |
|---|---|---|
| `evidence-synthesizer` | 需要 narrative synthesis 或 meta-analysis | 综合叙述或 pooled evidence 结果 |
| `quality-assessor` | 需要做 risk-of-bias / certainty judgment | 质量评估与偏倚评级 |
| `publication-bias-checker` | 需要检查发表偏倚 | publication bias report |

### F. Writing

当你的主要问题变成“怎么把分析和证据写成论文文本”时，用 Stage F。

| Skill | 适用场景 | 典型结果 |
|---|---|---|
| `manuscript-architect` | 需要论文整体结构或分节草稿方案 | outline、section plan、draft spine |
| `analysis-interpreter` | 需要把统计输出写成有限度的结果叙述 | bounded results narrative |
| `effect-size-interpreter` | 系数需要转成读者能理解的实际意义 | effect size 的实际含义说明 |
| `table-generator` | 需要把结果整理成论文级表格 | publication-ready tables |
| `figure-specifier` | 需要先把图的逻辑定义清楚再出图 | figure spec、plotting guidance |
| `meta-optimizer` | 需要优化标题、摘要和关键词 | title / abstract / keywords 优化稿 |

### G. Compliance

当论文已经成形，需要做规范检查、PRISMA 核对和语气收敛时，用 Stage G。

| Skill | 适用场景 | 典型结果 |
|---|---|---|
| `prisma-checker` | 系统综述需要 PRISMA 检查 | PRISMA completeness report |
| `reporting-checker` | 需要做 CONSORT / STROBE / COREQ / TRIPOD 等检查 | reporting checklist coverage report |
| `tone-normalizer` | 文字太松、太满、太绝对，或废话偏多 | 学术语气归一化日志 |

### H. Submission

当稿件接近投稿，或者已经进入审稿往返阶段时，用 Stage H。

| Skill | 适用场景 | 典型结果 |
|---|---|---|
| `submission-packager` | 需要组装 cover letter、disclosure、supplement | submission package |
| `rebuttal-assistant` | 需要把审稿意见转成逐点回应矩阵 | point-by-point rebuttal matrix |
| `peer-review-simulation` | 想在投稿前做多 persona 模拟评审 | multi-persona review memo |
| `fatal-flaw-detector` | 想先做一轮 desk-reject 风险扫描 | fatal-flaw analysis |
| `reviewer-empathy-checker` | 技术回应对了，但语气可能太硬 | reviewer-response tone check |

### I. Code

当你做的是学术代码、统计执行、数据流水线和可复现性收口时，用 Stage I。它比通用工程 prompt 更强调“低自由度、强审计”。

当前严格主链是：

1. `code-specification`
2. `code-planning`
3. `code-execution`
4. `code-review`
5. `reproducibility-auditor`

这也是 `code-build --focus full` 想要强化的使用方式。

| Skill | 适用场景 | 典型结果 |
|---|---|---|
| `code-builder` | 需要把论文方法转成可执行研究代码 | 面向学术方法的实现包 |
| `data-cleaning-planner` | 原始数据清洗需要可审计规则 | cleaning rules 与 transformation plan |
| `data-merge-planner` | 多数据源需要安全合并 | merge strategy 与 provenance controls |
| `code-specification` | 编码前必须先锁定约束与边界 | `code/code_specification.md` |
| `code-planning` | 需要 zero-decision 的执行计划 | `code/plan.md` |
| `code-execution` | 需要记录实现、profiling 和验证证据 | `code/performance_profile.md` |
| `code-review` | 需要第二模型做逻辑/统计有效性复核 | `code/code_review.md` |
| `reproducibility-auditor` | 需要检查 seed、环境、rerun recipe | `code/reproducibility_audit.md` |
| `stats-engine` | 重点是统计建模与诊断，而非一般编码 | `analysis/stats_report.md` |

### Z. Cross-Cutting

当问题并不属于某一个论文 stage，而是跨阶段通用时，用 Stage Z。

| Skill | 适用场景 | 典型结果 |
|---|---|---|
| `metadata-enricher` | 各种笔记、引文、结果中的元数据不一致 | DOI / 作者 / venue / 年份标准化结果 |
| `model-collaborator` | 需要 Codex / Claude / Gemini 交叉执行与互审 | 多模型执行与复核方案 |
| `self-critique` | 想强行增加 red-teaming 与 revision 压力 | critique questions 与修订提示 |

## 补充卡片与镜像目录

`skills/` 下的每个 markdown 文件都值得参考，但它们不全都是 primary routed skills。

### 补充型卡片

下面这些卡片很有价值，但当前不都属于 registry 里的一级 routed skills：

| 文件 | 作用 |
|---|---|
| `skills/C_design/data-dictionary-builder.md` | 生成结构化 data dictionary |
| `skills/C_design/data-management-plan.md` | 生成 FAIR 风格数据管理计划 |
| `skills/C_design/prereg-writer.md` | 生成预注册材料 |
| `skills/C_design/variable-operationalizer.md` | 把抽象构念映射为可测量变量 |
| `skills/H_submission/credit-taxonomy-helper.md` | 生成 CRediT 作者贡献说明 |
| `skills/I_code/release-packager.md` | 为 Zenodo / GitHub / Dataverse 整理可发布复现包 |

### Stage-I 镜像目录

下面这些目录主要是为了让 Stage-I 代码链路的 prompt 与执行位置靠得更近：

- `skills/I_code/build/`
- `skills/I_code/planning/`
- `skills/I_code/run/`
- `skills/I_code/qa/`

除非你正在修改 Stage-I 代码链路本身，否则阅读时优先以 `skills/I_code/*.md` 顶层 canonical 文件为主。

### Cross-Cutting 别名

`skills/Z_cross_cutting/tone-normalizer.md` 是一个跨阶段别名；真正的 canonical tone normalization 行为仍以 `skills/G_compliance/tone-normalizer.md` 为主。

## Domain Profiles

底层 skill 系统默认保持通用，学科差异通过 `skills/domain-profiles/*.yaml` 在运行时注入。

当前仓库自带的 profile 包括：

- `biomedical`
- `cs-ai`
- `ecology-environmental`
- `economics`
- `education`
- `epidemiology`
- `finance`
- `political-science`
- `psychology`

适合使用 domain profile 的情况：

- 默认 framing / design 逻辑太泛
- Stage-I 代码链需要领域专属诊断
- 不同学科的 reporting / venue 规范差异明显

例如，Stage-I 代码链可以通过 `--domain` 加载更贴近学科的方法检查规则。

## 接下来该看哪一页？

- 想看命令和参数：去 [CLI 参考](/zh/reference/cli)
- 想理解层次边界：去 [规范约定](/zh/conventions)
- 想理解运行时协同：去 [Agent + Skill 协同](/zh/advanced/agent-skill-collaboration)
- 想新增或改写 skill：去 [扩展 Research Skills](/zh/advanced/extend-research-skills)
