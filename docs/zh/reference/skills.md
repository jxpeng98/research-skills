# Skills 指南

> 本页由 `python3 scripts/generate_skill_docs.py` 基于 `skills/registry.yaml` 自动生成，请不要手工编辑。

这一页是面向使用者的 `skills/` 全景说明。

它主要回答这些问题：

- 当前研究问题应该落在哪个 stage？
- 每个 stage 里到底包含哪些能力？
- 哪些 skill 是 canonical、会被系统自动注入？
- 哪些 markdown 文件只是补充卡片或 Stage-I 镜像目录，不应该和主 skill 混在一起理解？

::: tip Canonical Source
系统自动路由的 canonical skill 列表以 `skills/registry.yaml` 为准；这一页是在它基础上的用户版说明。
中文界面会优先读取其中的 `summary_zh`、`display_name_zh` 和 `when_to_use_zh`。
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
如果你更关心“系统综述 / qualitative paper / methods paper / 审稿回复”这种真实场景怎么选路径，请看 [任务场景](/zh/guide/task-recipes)。

## 先记住几个边界

- 当前 internal skill registry 覆盖的是 `A` 到 `I` 阶段，再加 `K_presentation` 和 `Z_cross_cutting`。
- `J` 类 proofread / polish 入口目前主要在 workflow 层，不是单独的 top-level internal skill stage。
- `skills/` 下面有一部分文件是**补充卡片**，还有一部分是 Stage-I 代码链路的**镜像目录**；它们都很有用，但不等于“独立的 canonical routed skill”。

## Stage 总览

| Stage | 关注点 | Skill 数量 | 使用者最常见的问题 |
|---|---|---:|---|
| `A_framing` | 选题、问题、理论、gap、期刊定位 | 6 | “我的研究问题和贡献到底是什么？” |
| `B_literature` | 检索、筛选、提取、引文、文献地图 | 9 | “文献怎么系统找、系统筛、系统整理？” |
| `C_design` | 研究设计、变量、稳健性、数据可得性 | 9 | “这个研究该怎么设计和 operationalize？” |
| `D_ethics` | IRB、隐私、治理 | 3 | “伦理与数据合规材料要怎么准备？” |
| `E_synthesis` | 证据综合、质量评估、发表偏倚 | 5 | “已有证据要怎么整合和评级？” |
| `F_writing` | 结构、结果解释、表格、图、摘要 | 7 | “如何把分析结果写成论文？” |
| `G_compliance` | PRISMA、报告规范、学术语气 | 3 | “论文是否已经满足提交前规范？” |
| `H_submission` | 投稿包、回复审稿、模拟评审 | 7 | “投稿前后怎么打包和应对审稿？” |
| `I_code` | 学术代码、统计、可复现性 | 10 | “研究代码如何实现、审查、复现？” |
| `K_presentation` | 学术报告、幻灯片规划、Slidev、Beamer | 4 | “怎么把论文变成一个可讲、可答辩的学术报告？” |
| `Z_cross_cutting` | 元数据、多模型协作、自我批判 | 3 | “哪些能力是跨阶段通用的？” |

## 按 Stage 看 Canonical Skills

### A. Framing

当你还在定义研究问题、理论锚点、贡献定位、目标期刊时，用 Stage A。

| Skill | 中文名 | 适用场景 | 产出类型 |
|---|---|---|---|
| `question-refiner` | 研究问题精炼 | 当选题还模糊、范围过大或研究问题不可执行时使用。 | `RQSet` |
| `contribution-crafter` | 贡献提炼 | 当引言需要点明主要贡献，或需要为文章定位和强调创新点时使用。 | `ContributionStatement` |
| `hypothesis-generator` | 假设生成 | 当你需要把研究问题转成可检验假设或 propositions 时使用。 | `HypothesisSet` |
| `theory-mapper` | 理论映射 | 当你需要概念图、理论框架或机制关系图时使用。 | `TheoreticalFramework` |
| `gap-analyzer` | 研究空白分析 | 当你需要从已有文献中证明 novelty 和贡献空间时使用。 | `GapAnalysis` |
| `venue-analyzer` | 期刊匹配分析 | 当研究方向已较清楚，需要判断目标期刊或会议匹配度时使用。 | `VenueAnalysis` |

### B. Literature

当你要构建某个主题的文献基础，尤其是系统综述或可复现检索流程时，用 Stage B。

| Skill | 中文名 | 适用场景 | 产出类型 |
|---|---|---|---|
| `academic-searcher` | 学术检索 | 当你需要可复现的检索式、数据库搜索和 search log 时使用。 | `SearchQueryPlan`, `SearchResults`, `SearchLog` |
| `paper-screener` | 文献筛选 | 当你需要按纳入排除标准筛选文献并留下决策记录时使用。 | `ScreeningDecisionLog`, `PRISMAFlowData` |
| `paper-extractor` | 论文提取 | 当你需要把入选论文转成结构化笔记和 extraction table 时使用。 | `ExtractionTable`, `PaperNotes` |
| `citation-snowballer` | 引文滚雪球 | 当已有种子文献但覆盖还不够时使用。 | `SnowballLog` |
| `fulltext-fetcher` | 全文获取 | 当你已找到候选论文但缺少全文时使用。 | `FullTextStatus` |
| `citation-formatter` | 引文格式化 | 当写作前需要统一 bibliography、citekey 和引文格式时使用。 | `Bibliography` |
| `concept-extractor` | 检索概念提取 | 当检索概念不稳定，需要补 controlled vocabulary 和 Boolean 术语时使用。 | `ConceptMap` |
| `literature-mapper` | 文献地图 | 当你需要重组文献流派、机制簇和开放问题时使用。 | `LiteratureMap` |
| `reference-manager-bridge` | 文献管理器桥接 | 当你需要与 Zotero、Mendeley 或 EndNote 双向交换文献时使用。 | `Bibliography`, `RISExport`, `CSLJSONExport` |

### C. Design

当问题已经较清楚，下一步变成“怎么设计研究、怎么找数据、怎么定义变量”时，用 Stage C。

| Skill | 中文名 | 适用场景 | 产出类型 |
|---|---|---|---|
| `study-designer` | 研究设计 | 当研究问题已清楚，需要搭建设计、测量和分析方案时使用。 | `DesignSpec`, `AnalysisPlan`, `DataManagementPlan`, `Instruments`, `Preregistration` |
| `rival-hypothesis-designer` | 竞争解释设计 | 当你需要主动处理替代解释和 competing theories 时使用。 | `RivalHypotheses` |
| `robustness-planner` | 稳健性规划 | 当经验设计需要预先规定稳健性和敏感性分析时使用。 | `RobustnessPlan` |
| `dataset-finder` | 数据集搜寻 | 当你不确定有哪些可行数据源和获取路径时使用。 | `DatasetPlan` |
| `variable-constructor` | 变量构造 | 当你需要把抽象构念落成可审计变量和编码规则时使用。 | `VariableSpec` |
| `data-dictionary-builder` | 数据字典构建 | 当需要为数据集建立权威的变量定义和编码手册时使用。 | `DataDictionary` |
| `data-management-plan` | 数据管理计划 | 当需要按资助方或机构要求编写 DMP 时使用。 | `DataManagementPlan` |
| `prereg-writer` | 预注册撰写 | 当需要在数据收集前锁定假设、设计和分析计划时使用。 | `Preregistration` |
| `variable-operationalizer` | 变量操作化 | 当需要选择或设计测量工具、从构念到变量做 traceability 时使用。 | `OperationalizationMap` |

### D. Ethics

当研究涉及 IRB、人类受试者、敏感数据或数据治理要求时，用 Stage D。

| Skill | 中文名 | 适用场景 | 产出类型 |
|---|---|---|---|
| `ethics-irb-helper` | 伦理与IRB助手 | 当研究涉及 IRB、人类受试者或敏感数据时使用。 | `EthicsPackage` |
| `statement-generator` | 声明生成 | 当准备投稿并需要编写伦理及数据公开声明时使用。 | `Manuscript` |
| `deidentification-planner` | 去标识化规划 | 当你需要技术层面的隐私保护和去标识化方案时使用。 | `DeidentificationPlan` |

### E. Synthesis

当你已经有了证据材料，现在要做证据整合、质量评估或偏倚检查时，用 Stage E。

| Skill | 中文名 | 适用场景 | 产出类型 |
|---|---|---|---|
| `effect-size-calculator` | 效应量计算 | 当需要将原始统计数据转换为统一的效应量（如 Cohen's d）时使用。 | `EffectSizeTable`, `AnalysisCode` |
| `evidence-synthesizer` | 证据综合 | 当你已有证据材料，需要做叙事综合或 meta-analysis 时使用。 | `EvidenceTable`, `SynthesisMatrix` |
| `quality-assessor` | 质量评估 | 当你需要评估 risk of bias 和证据确定性时使用。 | `QualityTable`, `GRADESummary` |
| `publication-bias-checker` | 发表偏倚检查 | 当你需要判断结果是否受发表偏倚影响时使用。 | `PublicationBiasReport` |
| `qualitative-coding` | 定性数据编码 | 处理原始质性数据、摘要文本，需整理出现象与主题编码时使用。 | `DataDictionary`, `ThematicCodebook` |

### F. Writing

当你的主要问题变成“怎么把分析和证据写成论文文本”时，用 Stage F。

| Skill | 中文名 | 适用场景 | 产出类型 |
|---|---|---|---|
| `manuscript-architect` | 论文架构师 | 当你需要搭建论文整体结构、章节推进和核心论证主线时使用。 | `ManuscriptOutline`, `Manuscript`, `ClaimGraph`, `FiguresTablesPlan` |
| `analysis-interpreter` | 结果分析解释 | 当你需要把 quant 或 qualitative findings 写成有分析深度的结果叙述时使用。 | `ResultInterpretation` |
| `effect-size-interpreter` | 效应量解释 | 当你需要把统计系数翻译成读者能理解的实际意义时使用。 | `EffectInterpretation` |
| `table-generator` | 论文表格生成 | 当你需要把统计结果整理成论文级表格时使用。 | `FormattedTables` |
| `figure-specifier` | 图形规范定义 | 当你需要先定义图的逻辑、编码和可及性要求时使用。 | `FigureSpecs` |
| `meta-optimizer` | 题摘关键词优化 | 当你需要优化标题、摘要和关键词的可发现性时使用。 | `MetaOptimization` |
| `discussion-writer` | 讨论部分起草 | 当结果部分写完，需要解释结果及其理论现实意义时使用。 | `DiscussionDraft`, `StorySpine` |

### G. Compliance

当论文已经成形，需要做规范检查、PRISMA 核对和语气收敛时，用 Stage G。

| Skill | 中文名 | 适用场景 | 产出类型 |
|---|---|---|---|
| `prisma-checker` | PRISMA检查 | 当系统综述或证据综合需要核对 PRISMA 完整性时使用。 | `PRISMAChecklist` |
| `reporting-checker` | 报告规范检查 | 当你需要核查 CONSORT、STROBE、COREQ、SRQR 或 TRIPOD 时使用。 | `ReportingChecklist` |
| `tone-normalizer` | 学术语气归一 | 当文本太松、太满、太绝对或废话过多时使用。 | `ToneNormalization` |

### H. Submission

当稿件接近投稿，或者已经进入审稿往返阶段时，用 Stage H。

| Skill | 中文名 | 适用场景 | 产出类型 |
|---|---|---|---|
| `submission-packager` | 投稿包组装 | 当稿件接近投稿，需要准备 cover letter、声明和补充材料时使用。 | `SubmissionPackage` |
| `rebuttal-assistant` | 审稿回复助手 | 当你需要把审稿意见转成逐点回复矩阵时使用。 | `ResponseToReviewers`, `ResponseLetter` |
| `peer-review-simulation` | 同行评审模拟 | 当你想在投稿前做多 persona 压力测试时使用。 | `PeerReviewSimulation` |
| `fatal-flaw-detector` | 致命缺陷检测 | 当你想先做一轮 desk-reject 风险扫描时使用。 | `FatalFlawAnalysis` |
| `reviewer-empathy-checker` | 审稿沟通校准 | 当回复内容技术上正确，但语气可能过硬或防御性过强时使用。 | `EmpathyCheck` |
| `credit-taxonomy-helper` | CRediT贡献声明 | 当投稿需要 CRediT 作者贡献声明或需理清署名伦理时使用。 | `CRediTStatement` |
| `limitation-auditor` | 研究局限审计 | 完稿前梳理研究的缺陷、数据局限及其应对机制时使用。 | `LimitationSection`, `MitigationStrategy` |

### I. Code

当你做的是学术代码、统计执行、数据流水线和可复现性收口时，用 Stage I。它比通用工程 prompt 更强调“低自由度、强审计”。

当前严格主链是：

1. `code-specification`
2. `code-planning`
3. `code-execution`
4. `code-review`
5. `reproducibility-auditor`

这也是 `code-build --focus full` 想要强化的使用方式。

| Skill | 中文名 | 适用场景 | 产出类型 |
|---|---|---|---|
| `code-builder` | 学术代码构建 | 当你需要把论文方法转成可执行研究代码时使用。 | `AnalysisCode` |
| `data-cleaning-planner` | 数据清洗规划 | 当原始数据清洗需要变成可审计流程时使用。 | `CleaningPlan` |
| `data-merge-planner` | 数据合并规划 | 当多数据源需要安全合并并保留 provenance 时使用。 | `MergePlan` |
| `code-specification` | 代码规范定义 | 当编码前必须先锁定约束、输入输出和验收标准时使用。 | `CodeSpec` |
| `code-planning` | 代码执行规划 | 当你需要零自由裁量的实现计划和并行拆分时使用。 | `CodePlan` |
| `code-execution` | 代码执行 | 当你需要按既定计划实现代码并记录 profiling 与验证证据时使用。 | `PerformanceProfile` |
| `code-review` | 学术代码审查 | 当你需要第二模型审查代码逻辑、统计有效性和方法一致性时使用。 | `CodeReview` |
| `reproducibility-auditor` | 可复现性审计 | 当你需要检查 seed、环境、rerun recipe 和复现证据时使用。 | `ReproducibilityReport` |
| `release-packager` | 发布打包 | 当需要为投稿或归档准备代码/数据的可复现发布包时使用。 | `ReleasePackage` |
| `stats-engine` | 统计引擎 | 当重点是统计建模、诊断和假设检验，而不是一般编码时使用。 | `StatsReport` |

### K. Presentation

当论文已经成形，下一步是把内容转成 seminar、conference talk 或 defense deck 时，用 Stage K。

| Skill | 中文名 | 适用场景 | 产出类型 |
|---|---|---|---|
| `presentation-planner` | 报告规划 | 当你需要从论文出发策划一个学术报告的结构和取舍时使用。 | `PresentationPlan` |
| `slide-architect` | 幻灯片架构 | 当你需要逐页定义幻灯片的内容、布局和演讲备注时使用。 | `SlideDeckSpec` |
| `slidev-scholarly-builder` | Slidev 学术构建 | 当你选择 Slidev + scholarly 主题作为演示后端时使用。 | `SlidevDeck`, `BibTeXFile` |
| `beamer-builder` | Beamer 构建 | 当你选择 LaTeX Beamer 作为演示后端时使用。 | `BeamerDeck`, `BibTeXFile` |

### Z. Cross-Cutting

当问题并不属于某一个论文 stage，而是跨阶段通用时，用 Stage Z。

| Skill | 中文名 | 适用场景 | 产出类型 |
|---|---|---|---|
| `metadata-enricher` | 元数据补全 | 当 DOI、作者、年份或 venue 元数据在不同产物之间不一致时使用。 | `Bibliography` |
| `model-collaborator` | 多模型协作 | 当你需要 Codex、Claude 和 Gemini 分工协作或交叉复核时使用。 | `CollaborationTrace` |
| `self-critique` | 自我批判 | 当你想主动提高 red-teaming 强度、压制浅层推理和过度主张时使用。 | `CritiqueLog` |

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
- `business-management`
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
