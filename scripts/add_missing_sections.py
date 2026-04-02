#!/usr/bin/env python3
"""
Batch-add missing recommended sections (Quality Bar, Common Pitfalls, When to Use)
to skill files that are missing them.

Reads each file, extracts skill ID and description from frontmatter,
appends the missing sections at the end of the file with skill-specific content.
"""

import re
from pathlib import Path

SKILLS_DIR = Path(__file__).resolve().parent.parent / "skills"

# Map of file -> missing sections (from validator output)
WARNINGS = """skills/B_literature/academic-searcher.md:Quality Bar,Common Pitfalls,When to Use
skills/B_literature/citation-formatter.md:Quality Bar,Common Pitfalls,When to Use
skills/B_literature/citation-snowballer.md:Quality Bar,Common Pitfalls,When to Use
skills/I_code/code-builder.md:Quality Bar,Common Pitfalls,When to Use
skills/I_code/code-execution.md:Quality Bar,Common Pitfalls,When to Use
skills/I_code/code-planning.md:Quality Bar,Common Pitfalls,When to Use
skills/I_code/code-review.md:Quality Bar,Common Pitfalls,When to Use
skills/I_code/code-specification.md:Quality Bar,Common Pitfalls,When to Use
skills/B_literature/concept-extractor.md:Common Pitfalls
skills/C_design/data-management-plan.md:Quality Bar,Common Pitfalls,When to Use
skills/D_ethics/deidentification-planner.md:Common Pitfalls
skills/F_writing/effect-size-interpreter.md:Common Pitfalls
skills/E_synthesis/evidence-synthesizer.md:Quality Bar,Common Pitfalls
skills/H_submission/fatal-flaw-detector.md:Common Pitfalls
skills/B_literature/fulltext-fetcher.md:Quality Bar,Common Pitfalls,When to Use
skills/A_framing/gap-analyzer.md:Quality Bar,Common Pitfalls,When to Use
skills/A_framing/hypothesis-generator.md:When to Use
skills/F_writing/manuscript-architect.md:Common Pitfalls
skills/Z_cross_cutting/metadata-enricher.md:Quality Bar,Common Pitfalls,When to Use
skills/Z_cross_cutting/model-collaborator.md:Quality Bar,Common Pitfalls,When to Use
skills/B_literature/paper-extractor.md:Quality Bar,Common Pitfalls,When to Use
skills/B_literature/paper-screener.md:Quality Bar,Common Pitfalls,When to Use
skills/H_submission/peer-review-simulation.md:Common Pitfalls
skills/G_compliance/prisma-checker.md:Quality Bar,Common Pitfalls,When to Use
skills/E_synthesis/quality-assessor.md:Quality Bar,Common Pitfalls,When to Use
skills/A_framing/question-refiner.md:Quality Bar,Common Pitfalls,When to Use
skills/H_submission/rebuttal-assistant.md:Common Pitfalls
skills/B_literature/reference-manager-bridge.md:Quality Bar,Common Pitfalls,When to Use
skills/I_code/reproducibility-auditor.md:Quality Bar,Common Pitfalls,When to Use
skills/H_submission/reviewer-empathy-checker.md:Common Pitfalls
skills/C_design/rival-hypothesis-designer.md:Common Pitfalls
skills/Z_cross_cutting/self-critique.md:Quality Bar,Common Pitfalls,When to Use
skills/I_code/stats-engine.md:Quality Bar
skills/H_submission/submission-packager.md:Common Pitfalls
skills/F_writing/table-generator.md:Common Pitfalls
skills/A_framing/theory-mapper.md:Quality Bar,Common Pitfalls,When to Use
skills/G_compliance/tone-normalizer.md:Common Pitfalls
skills/A_framing/venue-analyzer.md:Common Pitfalls"""

# Skill-specific content for each section type
CONTENT = {
    # ── A_framing ──
    "question-refiner": {
        "When to Use": "- 选题还模糊、范围过大或研究问题不可执行时\n- 导师反馈研究问题不够聚焦时\n- 需要在 PICO/PEO 框架下结构化问题时\n- 从课题申请转化为论文时需要收窄 scope",
        "Quality Bar": "- [ ] 每个 RQ 通过五项 FINER 评估 (Feasible, Interesting, Novel, Ethical, Relevant)\n- [ ] PICO/PEO 四要素均已显式填写\n- [ ] 至少一个 primary RQ 可直接对应分析计划\n- [ ] 每个 RQ 附带 scope boundary — 明确研究不包括什么\n- [ ] 术语定义一致且无歧义",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| RQ 过宽 | 无法在一篇论文中回答 | 用 PICO 强制拆分 sub-RQ |\n| 缺 boundary | Reviewer 质疑 scope creep | 每个 RQ 附 exclusion list |\n| 混淆 RQ 与假设 | RQ 不可检验 | RQ 用疑问句，假设用陈述句 |\n| 术语不一致 | 同一概念多种表述 | 建立 glossary 并固定用词 |\n| 忽视 feasibility | 提出无法获取数据的问题 | FINER-F 先行评估 |"
    },
    "hypothesis-generator": {
        "When to Use": "- 研究问题已经结构化（PICO/PEO），需要转化为可检验命题\n- 理论框架已确定，需要推导 testable predictions\n- Pre-registration 前需要锁定假设方向和备择\n- 定性研究需要 propositions 而非假设时"
    },
    "theory-mapper": {
        "When to Use": "- 需要概念图、理论框架或机制关系图时\n- 写 literature review 需要组织 theoretical landscape\n- 需要识别 mediator/moderator 关系时\n- 理论框架涉及多个 competing theories 需要比较时",
        "Quality Bar": "- [ ] 每个 construct 有明确定义和操作化路径\n- [ ] 关系箭头标注了方向、极性（正/负）和条件\n- [ ] Mermaid 图可正确渲染且无悬挂节点\n- [ ] 至少标注一个 boundary condition\n- [ ] 每个理论引用都有明确来源",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 概念定义模糊 | Construct 与 variable 混淆 | 分层：construct → dimension → indicator |\n| 关系箭头无标注 | 读者不知道是正相关还是调节 | 标注方向 + 机制 |\n| 框架过于复杂 | 一个图超过 15 个节点 | 拆分为核心框架 + 扩展因素 |\n| 缺少竞争解释 | Reviewer 质疑为何选此理论 | 添加 rival theory comparison |\n| 图文不一致 | 文字描述与 Mermaid 图不对应 | 以图为准，反向核验文字 |"
    },
    "gap-analyzer": {
        "When to Use": "- 完成初步文献检索后需要证明 novelty\n- 写 Introduction 需要定位贡献空间时\n- 基金申请需要说明 research significance\n- 需要区分 theoretical / empirical / methodological gap 类型时",
        "Quality Bar": "- [ ] 每个 gap 归类到五类 taxonomy 中的一类\n- [ ] 每个 gap 附带至少 2 篇支撑文献\n- [ ] FINER 优先级排序已完成\n- [ ] 至少一个 gap 直接对应研究问题\n- [ ] 排除了已被近期文献填补的 gap",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 伪 gap | 文献检索不充分导致误判 | 扩大搜索范围后再确认 |\n| Gap ≠ RQ | 识别了 gap 但 RQ 没有对应 | 用 gap → RQ mapping 表 |\n| 忽视近期文献 | 别人已经填补了 | 检索不设过远时间下限 |\n| 只有一类 gap | 全是 empirical gap | 强制检查 5 类 taxonomy |\n| 无优先级 | 所有 gap 同等重要 | 用 FINER 打分排序 |"
    },
    "venue-analyzer": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 只关注 IF | 忽视了 scope match 和 audience | 并行评估 fit / format / reputation |\n| 格式要求遗漏 | 被要求大改格式 | 投稿前完整核查 author guidelines |\n| 忽视审稿速度 | 项目 deadline 前无法完成 R&R | 考虑 first-decision turnaround |\n| 过度乐观定级 | 方法/数据撑不住 top journal | 用 desk-reject 风险评估 |\n| 忽视 OA 费用 | 预算不够 APC | 预先核查 OA 政策和费用 |"
    },
    # ── B_literature ──
    "academic-searcher": {
        "When to Use": "- 需要可复现的检索式设计和数据库搜索时\n- 系统综述或 scoping review 的初始检索阶段\n- 需要生成 search log 和 dedup log 时\n- 需要覆盖多个数据库（PubMed, Scopus, WoS 等）时",
        "Quality Bar": "- [ ] 检索式使用 Boolean 逻辑且每个 concept block 至少包含 3 个同义词\n- [ ] 至少搜索 2 个数据库\n- [ ] search_log.md 完整记录了每次检索的日期、数据库、命中数\n- [ ] 去重后记录了 dedup_log.csv 中的合并决策\n- [ ] 检索式经过 domain expert 或 librarian 审议（或标注待审）",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 检索式过窄 | 漏掉关键文献 | 加 MeSH/Emtree + 自由词 |\n| 未覆盖灰色文献 | 系统偏差 | 明确说明灰色文献策略 |\n| 去重不一致 | 同一篇文献保留多次 | 用 DOI + 标题 fuzzy match |\n| 检索式不可复现 | Reviewer 无法验证 | 完整记录 search string + 日期 |\n| 只搜 PubMed | 学科覆盖偏差 | 加 Scopus/WoS/domain-specific DB |"
    },
    "paper-screener": {
        "When to Use": "- 检索完成后需要按纳入排除标准筛选文献时\n- 需要 PRISMA-compliant 的双阶段筛选记录时\n- 大量文献（> 100 篇）需要系统筛选时\n- 需要生成 screening decision log 和 PRISMA flow 数据时",
        "Quality Bar": "- [ ] Inclusion/exclusion criteria 已前置定义且无歧义\n- [ ] Title-abstract 和 full-text screening 分阶段记录\n- [ ] 每条排除决策附带排除理由\n- [ ] PRISMA flow 数据可直接生成 flow diagram\n- [ ] 存在 inter-rater reliability 描述（或标注为单筛）",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 标准模糊 | 同一篇文献不同标准得出不同结论 | 操作化每条 criteria 的判断规则 |\n| 无排除理由 | 无法复现筛选过程 | 每条记录附 exclusion reason code |\n| Full-text 获取失败未记录 | PRISMA flow 不完整 | 在 flow 中标注 unable to retrieve |\n| 一次筛完 | 跳过 title-abstract 阶段 | 强制两阶段分开 |\n| 无冲突解决机制 | 双筛不一致时无规则 | 预定义 consensus / third reviewer 规则 |"
    },
    "paper-extractor": {
        "When to Use": "- 筛选完成后需要从入选论文中提取结构化数据时\n- 需要生成 extraction table 和 paper notes 时\n- 系统综述需要标准化的数据提取格式时\n- 需要提取 theory/method/data/finding/limitation 等多维度信息时",
        "Quality Bar": "- [ ] 每篇论文的提取字段完整覆盖预定义的 extraction schema\n- [ ] Extraction table 格式标准化且可直接用于综合分析\n- [ ] 每篇 paper note 包含该论文对 RQ 的直接 relevance 说明\n- [ ] 数值数据提取附带精度和单位\n- [ ] 存在 inter-rater reliability 描述（或标注为单提取）",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 提取字段不一致 | 不同论文提取了不同字段 | 先定义 extraction form 再统一填充 |\n| 数值精度丢失 | 四舍五入导致后续计算偏差 | 保留原始精度 |\n| 定性与定量混合 | Extraction table 字段设计不合理 | 分开定性 slot 和定量 slot |\n| 缺少 context | 只提取数字不提取研究条件 | 每个 finding 附带 study context |\n| 遗漏 supplementary data | 补充材料中有关键数据 | 明确标注是否包含 supplement |"
    },
    "citation-snowballer": {
        "When to Use": "- 已有种子文献但覆盖不够时\n- 需要发现 seminal works 或 foundational papers 时\n- 数据库检索遗漏了重要文献时\n- 需要向 Reviewer 证明 exhaustive search effort 时",
        "Quality Bar": "- [ ] Forward 和 backward 两个方向都已执行\n- [ ] Snowball log 记录了每轮新增和排除文献数\n- [ ] 达到 saturation（连续一轮无新增有效文献）\n- [ ] 新增文献已经过 screening criteria 筛选\n- [ ] 结果已合并回 search_results.csv 并去重",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 无 stopping rule | 无限滚雪球 | 定义 saturation 标准（连续 N 轮无新增） |\n| 只做 backward | 遗漏后续跟进研究 | Forward + backward 并行执行 |\n| 未去重 | 新增文献与已有重复 | 每轮合并后运行 dedup |\n| 种子论文选择偏差 | 只从一个流派出发 | 选择跨流派的多个 seed papers |\n| 无记录追踪 | 不知道新增来自哪个 seed | Snowball log 标注 source paper |"
    },
    "fulltext-fetcher": {
        "When to Use": "- 已完成筛选但缺少入选论文的全文时\n- 需要 PRISMA-compliant 的全文获取状态追踪时\n- 需要通过 OA 渠道或数据库下载全文 PDF 时\n- 需要生成 retrieval manifest 记录获取结果时",
        "Quality Bar": "- [ ] 每篇入选论文的全文获取状态已记录\n- [ ] Retrieval manifest 包含 DOI、来源、获取日期、版本信息\n- [ ] 无法获取的论文标注了原因和替代方案\n- [ ] PDF 文件命名统一且可追踪\n- [ ] 获取状态可直接用于更新 PRISMA flow",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 版本不一致 | Preprint 与 published 版本结果不同 | 标注版本并优先 final published |\n| 付费墙阻断 | 无法获取闭源文献 | 尝试 Unpaywall、作者邮件、ILL |\n| 文件未命名编号 | PDF 无法追踪到文献 | 用 citekey 命名 |\n| 未记录失败 | PRISMA flow 数据缺失 | 每次失败都写入 manifest |\n| 忽略 supplementary | 关键方法/数据在 supplement | 全文 + supplement 一同获取 |"
    },
    "citation-formatter": {
        "When to Use": "- 写作前需要统一 bibliography 和 citekey 规范时\n- 需要在 APA/MLA/Chicago/IEEE 等格式间转换时\n- 需要从 extraction notes 生成可用的 .bib 文件时\n- 投稿前需要核查引文格式与目标期刊一致时",
        "Quality Bar": "- [ ] 所有 citekey 遵循统一命名规则 (author_year 格式)\n- [ ] 每条引文的必填字段完整（author, title, year, journal/booktitle）\n- [ ] BibTeX 文件可被 LaTeX/Pandoc 编译无错误\n- [ ] 引文格式与目标期刊 author guidelines 一致\n- [ ] 重复条目已合并",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| Citekey 重复 | 同一作者同年多篇冲突 | 用 author_year_a/b 后缀 |\n| 字段缺失 | 编译时报 warning | 检查 required fields per entry type |\n| 编码错误 | 特殊字符（ü, ñ）乱码 | 使用 UTF-8 + LaTeX escape |\n| 引文样式混用 | 同一文档出现两种格式 | 锁定一种 CSL/BST 文件 |\n| DOI 格式不一 | 有的带 https 有的不带 | 统一为 doi.org URL 格式 |"
    },
    "concept-extractor": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| Confirmation bias | 只扩展已有概念 | 强制检索 controlled vocabulary 中的替代术语 |\n| 过度扩展 | 概念外延过大导致不相关文献涌入 | 用 PCC 框架约束 scope |\n| 遗漏跨学科术语 | 不同学科用不同术语描述同一现象 | 咨询 thesaurus 和 MeSH cross-references |\n| Boolean 逻辑错误 | AND/OR 混淆改变搜索含义 | 逐步构建并测试每个 block |\n| 未记录扩展来源 | 术语来源不可追踪 | 每个新增术语标注来源 |"
    },
    "reference-manager-bridge": {
        "When to Use": "- 需要将研究系统的 bibliography 导入 Zotero/Mendeley/EndNote 时\n- 需要从外部工具导入文献到研究系统时\n- 团队协作需要共享统一的文献库时\n- 切换写作工具（LaTeX ↔ Word）需要转换引文格式时",
        "Quality Bar": "- [ ] 导入/导出后文献条数一致（无丢失）\n- [ ] Citekey 在双向同步后保持稳定\n- [ ] 格式转换（BibTeX ↔ RIS ↔ CSL-JSON）无信息丢失\n- [ ] 附件（PDF）关联关系在同步后保持完整\n- [ ] 冲突条目已标记并手动解决",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| Citekey 重写 | Zotero 自动生成 citekey 覆盖 | 使用 Better BibTeX 插件锁定 |\n| 编码丢失 | 特殊字符在 RIS 中丢失 | 优先使用 BibTeX 或 CSL-JSON |\n| 附件断链 | PDF 路径在同步后失效 | 使用相对路径或 linked file 模式 |\n| 条目格式分类错误 | Article 被识别为 InProceedings | 手动检查 entry type 映射 |\n| 无增量同步 | 全量覆盖导致手动编辑丢失 | 使用 merge 模式而非 overwrite |"
    },
    # ── C_design ──
    "data-management-plan": {
        "When to Use": "- 基金申请需要提交 DMP 时\n- 机构 IRB/ethics 要求数据治理文档时\n- 协作研究需要明确数据存储和共享规则时\n- FAIR 原则合规要求时",
        "Quality Bar": "- [ ] 覆盖 FAIR 四原则（Findable, Accessible, Interoperable, Reusable）\n- [ ] 存储、备份、保留和归档策略均已说明\n- [ ] 数据共享计划包含 embargo/access levels\n- [ ] 隐私保护措施与 ethics package 一致\n- [ ] DMP 格式符合资助方模板要求",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 模板化填写 | 不具体到项目 | 每个字段填入 project-specific 内容 |\n| 忽略成本 | 存储/归档需要预算 | 在 DMP 中注明成本估算 |\n| 未考虑数据销毁 | 保留期限过后如何处理不清 | 明确 retention period + destruction policy |\n| 访问权限不明 | 团队成员权限未定义 | 建立 role-based access matrix |\n| 与 IRB 不一致 | DMP 承诺共享但 IRB 限制 | 交叉核查两份文档 |"
    },
    "rival-hypothesis-designer": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 只考虑稻草人替代 | 竞争假设太弱不构成威胁 | 找该理论的 strongest advocate 论文 |\n| 未连接到测试 | 列出竞争解释但不设计区分检验 | 每个 rival 附带 discriminating test |\n| 遗漏内生性 | 反向因果或遗漏变量 | 系统检查 endogeneity threats |\n| 只考虑理论竞争 | 忽视方法论替代（不同估计方法） | 同时考虑 method-driven rivals |\n| 数量太多 | 无法全部回应 | 按 plausibility 排序 top 3 |"
    },
    # ── D_ethics ──
    "deidentification-planner": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 只去显式 ID | 忽视准标识符组合重识别 | k-anonymity 验证 quasi-identifier 组合 |\n| 过度去标识 | 数据效用严重下降 | 用 utility-privacy trade-off 评估 |\n| 忽略文本字段 | 叙述中含可识别信息 | NER 扫描 + manual review |\n| 地理信息残留 | 精确 GPS 或小地理单元 | 泛化到更大的地理区域 |\n| 未考虑纵向链接 | 多波数据可重组身份 | 使用 project-specific 随机 ID |"
    },
    # ── E_synthesis ──
    "evidence-synthesizer": {
        "Quality Bar": "- [ ] 综合方法（叙事/定性/定量）有明确选择依据\n- [ ] 若 meta-analysis，异质性已报告（I², τ², Q-test）\n- [ ] 综合矩阵覆盖所有入选研究\n- [ ] 亚组/敏感性分析按预注册计划执行\n- [ ] PRISMA-aligned reporting 要素齐全",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 忽视异质性 | 合成异质数据导致误导 | 先 I² 检查，> 75% 需解释或放弃 pooling |\n| Vote counting | 只数 significant 研究比例 | 使用正式综合方法 |\n| 混淆效应指标 | 混合 OR/RR/HR | 统一转换后再 pool |\n| 遗漏灰色文献 | Publication bias | 加入未发表研究或做 sensitivity |\n| 未做 sensitivity | 不知道结果是否 robust | 移除影响力最大的研究后重跑 |"
    },
    "quality-assessor": {
        "When to Use": "- 完成数据提取后需要评估每篇文献的 risk of bias 时\n- 系统综述需要 RoB 2 / ROBINS-I / GRADE 评估时\n- 需要按证据质量加权综合时\n- 投稿前 Reviewer 要求质量评估透明化时",
        "Quality Bar": "- [ ] 选用工具匹配研究设计（RCT→RoB 2, observational→ROBINS-I）\n- [ ] 每个 domain 有独立判断 + 支撑理由\n- [ ] GRADE 评估（如适用）覆盖 5 downgrade + 3 upgrade 因素\n- [ ] Quality table 可直接嵌入论文或 appendix\n- [ ] 存在 inter-rater agreement 描述或计划",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 工具不匹配 | RCT 用 NOS 评估 | 按 study design 选工具 |\n| 评估过于宽泛 | 全部标为 some concerns | 逐 domain 给出具体依据 |\n| 忽略 GRADE | 只做 RoB 不做证据确定性 | 对每个 outcome 做 GRADE SoF |\n| 无 inter-rater | 审稿人质疑主观性 | 至少 20% 样本双评或标注 |\n| 结果不影响综合 | 评估完但综合不分层 | 按 quality tier 做 sensitivity |"
    },
    # ── F_writing ──
    "manuscript-architect": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 结构先行内容后行 | 套模板但论证空洞 | 先建 claim-evidence map 再搭骨架 |\n| Introduction 过长 | 读者失去耐心 | 控制在 4-5 段，渐进缩窄 |\n| Results 与 Discussion 混合 | 描述与解释交织 | 严格分离（除非期刊要求合并） |\n| 图表堆砌 | 太多表格淹没重点 | 核心数据进正文，其余转 appendix |\n| Story spine 缺失 | 各章节割裂 | 用一句话总结每节的叙事推进  |"
    },
    "effect-size-interpreter": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 只报告 Cohen's 基准 | Small/medium/large 脱离学科语境 | 用 domain-specific benchmark |\n| 混淆 statistical 与 practical significance | p < 0.05 但 d = 0.05 | 必须同时讨论两者 |\n| 单一度量 | 只报告 d 但读者不直观 | 转化为 CLES、NNT 或百分比变化 |\n| 忽略 CI | 只报告点估计 | 必须附带 95% CI |\n| 无参照基准 | 读者不知道 0.3 SD 意味着什么 | 找同领域类似干预的 effect size 比较 |"
    },
    "table-generator": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 精度不一致 | 同列混用 2/3/4 位小数 | 统一精度规则 |\n| 缺少 N | 不知道每个 cell 的样本量 | 表头或脚注标注 N |\n| 表格过宽 | 超出页面边距 | 拆分或旋转为 landscape |\n| 无标注说明 | 读者不知道 * ** *** 含义 | 表格脚注统一标注 significance level |\n| 格式不符期刊要求 | APA 要求三线表 | 预先检查 author guidelines |"
    },
    # ── G_compliance ──
    "prisma-checker": {
        "When to Use": "- 系统综述或 meta-analysis 稿件需要核查 PRISMA 合规性时\n- 需要生成 PRISMA 2020 flow diagram 时\n- 投稿前需要逐条核对 27 项 checklist 时\n- Reviewer 要求补充 PRISMA 报告时",
        "Quality Bar": "- [ ] 27 项 PRISMA 2020 checklist 逐条标注了对应稿件位置\n- [ ] Flow diagram 数据与正文数字完全一致\n- [ ] 所有 checklist 项标注为 Yes/No/NA（不留空白）\n- [ ] 扩展项（如 PRISMA-S for searches）已按需覆盖\n- [ ] Checklist 与 flow diagram 互相一致",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 数字不一致 | Flow diagram 与正文数字不match | 从数据源重新计数并核验 |\n| 用旧版 PRISMA | 使用 2009 版而非 2020 | 确认使用 PRISMA 2020 |\n| Checklist 项笼统回答 | 写见方法但位置不精确 | 标注段落/页码 |\n| 忽略 protocol registration | 未报告 protocol DOI | 在 Methods 中标注 PROSPERO 等 |\n| NA 项无理由 | 标 NA 但不解释原因 | 每个 NA 附简要说明 |"
    },
    "tone-normalizer": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 过度删减 | 删减了必要的 hedging | 保留 evidence-based hedging（如 may, suggest） |\n| 中文论文英文味 | 直译导致语体不对 | 区分中英文学术 register |\n| 去掉所有 transition | 段落间缺少衔接 | 去的是 filler，留 logical connector |\n| 统一 hedge 为 may | 所有句子用同一个 hedging | 变换用词：suggests, indicates, appears to |\n| 忽略学科惯例 | 某些学科允许主动语态 | 核查目标期刊的 style preference |"
    },
    # ── H_submission ──
    "submission-packager": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 遗漏 mandatory attachment | Cover letter 未附带 | 逐项核查期刊 submission checklist |\n| 字数超限 | Abstract 或正文超标 | 投稿前 word count 检查 |\n| 作者信息不一致 | ORCID/affiliation 跨文件不一 | 用单一 author info 源文件 |\n| Supplementary 未编号 | 正文引用对不上 | 统一编号 S1, S2, ... |\n| Cover letter 过于 generic | 未针对目标期刊定制 | 包含 scope fit + 贡献亮点 |"
    },
    "rebuttal-assistant": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 回避核心问题 | 只回应边缘意见 | 优先处理 major concerns |\n| 过度辩解 | 语气防御性太强 | 先 acknowledge 再 explain |\n| 改动未标记 | Editor 找不到修改处 | 用 track changes + 标注页码/段落 |\n| 逐字照搬评审意见 | 看起来没有理解 | 用自己的话 paraphrase 问题 |\n| 承诺但未兑现 | 说会修改但文中没改 | Cross-check response matrix vs manuscript |"
    },
    "peer-review-simulation": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| Persona 不够刁钻 | 模拟太温和发现不了问题 | 包含 Reviewer 2（挑剔型） |\n| 不同 persona 意见雷同 | 缺少独立性 | 先独立生成再 cross-review |\n| 只关注写作 | 忽视方法论和数据问题 | 包含 Methodologist persona |\n| 缺少 actionable feedback | 指出问题但不建议如何修 | 每条 concern 附带 suggested fix |\n| 未映射到修改计划 | 模拟完但不行动 | 输出 → rebuttal-assistant 接力 |"
    },
    "fatal-flaw-detector": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 只看表面 | 格式/字数/引文数量 | 深入检查方法论和论证逻辑 |\n| 过度敏感 | 每个潜在问题都标为 fatal | 区分 fatal vs. major vs. minor |\n| 忽视 scope match | 论文本身好但不适合目标期刊 | 交叉引用 venue-analyzer 结论 |\n| 无优先级 | 列出 20 个问题同等对待 | 按 desk-reject probability 排序 |\n| 未考虑领域惯例 | 某些做法在该领域是 acceptable | 检查目标期刊近期发表的类似论文 |"
    },
    "reviewer-empathy-checker": {
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 过度谦卑 | 全面承认 reviewer 正确 | 区分合理批评 vs. 误解之处 |\n| 技术正确但语气差 | 内容对但对话感差 | 按 acknowledge → address → appreciate 结构 |\n| 忽略夸赞 | 只回应批评不回应肯定 | 对正面评论也要回应致谢 |\n| 逐字 copy-paste | 直接复制 rebuttal-assistant 输出 | 根据 reviewer 风格调整措辞 |\n| 未检查一致性 | 给 R1 和 R2 的回复互相矛盾 | 全局检查跨 reviewer 一致性 |"
    },
    # ── I_code ──
    "code-builder": {
        "When to Use": "- 需要将论文方法/分析计划转化为可执行代码时\n- 需要结合 domain profile 选择合适的统计/数据处理库时\n- 需要生成可复现的分析脚本（R/Python/Stata/Julia）时\n- 需要从 analysis plan 派生数据处理 pipeline 时",
        "Quality Bar": "- [ ] 代码可在新环境中 one-command 运行（附 requirements/renv.lock）\n- [ ] 所有硬编码路径已替换为相对路径或配置参数\n- [ ] 随机种子已固定且有文档说明\n- [ ] 核心函数有 docstring 和至少一个 assertion/test\n- [ ] 输出与 analysis plan 中预期的 table/figure 一一对应",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 硬编码路径 | 他人无法运行 | 用 config file 或 argparse |\n| 缺少 seed | 结果不可复现 | 全局 set.seed() + 记录 |\n| 库版本不锁 | 依赖更新后行为变化 | 使用 lock file (pip freeze / renv::snapshot) |\n| 魔法数字 | 代码中常数无解释 | 提取为 named constant + 注释来源 |\n| 分析与可视化耦合 | 改图就得重跑分析 | 分离 analysis → save results → plot |"
    },
    "code-execution": {
        "When to Use": "- 代码计划已制定，需要按步骤实现并验证时\n- 需要 cProfile 性能分析和瓶颈优化时\n- 需要记录执行证据（log、profiling output）时\n- 从 code-planning 输出接力到实际代码编写时",
        "Quality Bar": "- [ ] 每个计划步骤都已实现且有对应测试\n- [ ] cProfile 输出已记录且无明显瓶颈（或已解释原因）\n- [ ] 内存占用在合理范围内\n- [ ] 边界情况和错误路径有明确处理\n- [ ] 代码风格一致且通过 linter",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 跳过 profiling | 不知道瓶颈在哪 | 至少运行一次 cProfile/line_profiler |\n| 无 error handling | 遇到缺失数据直接 crash | 添加 try/except + 明确 fallback |\n| 未验证中间结果 | 错误沿 pipeline 传播 | 每步添加 shape/summary assertion |\n| N+1 查询 | 循环内重复 I/O | 批量化操作 |\n| 忽略内存 | 大数据集 OOM | 使用 chunk processing 或 lazy eval |"
    },
    "code-planning": {
        "When to Use": "- 已有 code specification，需要拆分为可并行的执行步骤时\n- 复杂分析需要零自由裁量的实现计划时\n- 团队协作需要明确模块划分和接口定义时\n- 需要估算工作量和识别关键路径时",
        "Quality Bar": "- [ ] 每个步骤有明确的输入/输出定义\n- [ ] 依赖关系图已标注且无循环依赖\n- [ ] 可并行步骤已标识\n- [ ] 每个步骤有验收标准（assertion 或 test case）\n- [ ] 估算完成时间/复杂度",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 步骤粒度不均 | 有的步骤 5 分钟有的 5 小时 | 拆分大步骤至 1-2 小时粒度 |\n| 依赖不明 | 并行执行时出错 | 画依赖 DAG 并标注 critical path |\n| 缺少验收标准 | 不知道步骤是否完成 | 每步附带 assertion checklist |\n| 过度设计 | 计划比代码还长 | 保持计划简洁，focus on interface |\n| 忽略数据 I/O | 计划只关心算法 | 明确数据流 format 和存储位置 |"
    },
    "code-review": {
        "When to Use": "- 分析代码完成后需要第二模型/人审查时\n- 需要检查统计有效性和方法论一致性时\n- 投稿前需要独立核验代码逻辑时\n- 需要安全性审查（数据泄漏、路径注入）时",
        "Quality Bar": "- [ ] 逻辑正确性已逐函数审查\n- [ ] 统计方法与 analysis plan 一致\n- [ ] 无数据泄漏（训练/测试/时间序列前瞻）\n- [ ] 代码风格和命名一致\n- [ ] Review 报告包含具体行号引用",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 只查格式 | 忽视逻辑和统计错误 | 按 logic → statistics → style 优先级 |\n| 不理解方法 | 审查者不熟悉 estimator | 审查前阅读 analysis plan + 方法文献 |\n| 无 action item | 评论模糊 | 每条评论是 must-fix / should-fix / nice-to-have |\n| 忽视数据泄漏 | 训练集信息泄漏到测试 | 专门检查 data split boundary |\n| 审查不独立 | 看了作者解释再审 | 先盲审代码，再看文档 |"
    },
    "code-specification": {
        "When to Use": "- 编码前需要先锁定约束、输入输出和验收标准时\n- 需要 OPSX 风格的严格规范集时\n- 复杂分析需要正式的 contract before implementation 时\n- 多人协作编码需要统一接口规范时",
        "Quality Bar": "- [ ] 输入/输出类型和格式已完全定义\n- [ ] 约束条件可自动测试（不含模糊表述）\n- [ ] 验收标准有具体数字或条件\n- [ ] 与 analysis plan 逆向可追溯\n- [ ] 边界情况和异常处理已列出",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 规范太模糊 | 实现时有自由裁量空间 | 每条约束用 GIVEN-WHEN-THEN 格式 |\n| 过度规范 | 规范比代码还长 | 只规范接口和约束，不规范实现 |\n| 遗漏 edge case | 缺失数据/空输入未定义 | 用 boundary value analysis 补充 |\n| 与 analysis plan 脱节 | 规范了代码但不对应统计需求 | 添加 traceability matrix |\n| 无版本控制 | 规范修改后实现不同步 | 规范文件纳入 git 管理 |"
    },
    "reproducibility-auditor": {
        "When to Use": "- 代码完成后需要验证可复现性时\n- 投稿前需要 reproducibility evidence 时\n- 需要检查 random seed、容器化和 rerun 能力时\n- 代码共享/开源前的最终审计时",
        "Quality Bar": "- [ ] Random seed 已固定且 rerun 产出一致\n- [ ] 环境可从 lock file 重建\n- [ ] 数据路径不含 hardcoded absolute path\n- [ ] README 包含完整 setup + run 指令\n- [ ] Rerun recipe 存在且已测试通过",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 只固定主 seed | 子过程的随机性未控制 | 每个随机操作都 derive seed |\n| 环境未锁定 | pip install 获取最新版可能 break | 使用 lock file |\n| 数据未版本化 | 数据更新后结果不同 | 数据快照 + hash 记录 |\n| 缺少 rerun 脚本 | 不知道按什么顺序跑 | Makefile / dvc pipeline |\n| 忽视 OS 差异 | macOS vs Linux 浮点差异 | 容器化或文档记录 OS requirement |"
    },
    "stats-engine": {
        "Quality Bar": "- [ ] 模型选择有统计学依据（而非仅因常用）\n- [ ] 假设检验结果包含 effect size + CI（不止 p-value）\n- [ ] 诊断检验已执行（residuals, multicollinearity, heteroscedasticity）\n- [ ] 多重比较已校正（Bonferroni / FDR / 等效方法）\n- [ ] domain profile 对应的统计方法已正确适用",
    },
    # ── Z_cross_cutting ──
    "metadata-enricher": {
        "When to Use": "- 不同产物之间的 DOI/作者/年份/期刊信息不一致时\n- 需要标准化和补全文献元数据时\n- 合并多来源搜索结果需要统一 citekey 时\n- bibliography.bib 需要 cleanup 和 enrichment 时",
        "Quality Bar": "- [ ] 所有 DOI 已归一化为标准格式\n- [ ] 作者姓名格式统一（Last, First 或 First Last）\n- [ ] 年份和 venue 信息完整无缺\n- [ ] Citekey 唯一且稳定\n- [ ] Dedup 决策有明确理由记录",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| DOI 格式不统一 | 大小写或 URL scheme 不一 | 全部转 lowercase + https://doi.org/ |\n| 作者名字翻转 | 中文/东亚作者姓名反 | 检查 parsed name vs. original |\n| 年份混淆 | Online first vs. print year | 优先使用 published year |\n| Dedup 过激 | 不该合并的条目被合并 | 用 DOI + title + year 三重匹配 |\n| Citekey 漂移 | 同一论文每次 enrichment 换 key | 首次生成后锁定 citekey |"
    },
    "model-collaborator": {
        "When to Use": "- 需要 Codex、Claude 和 Gemini 分工协作或交叉复核时\n- 复杂任务需要多模型独立执行后比较结论时\n- 需要 primary-review agent 配对验证代码/写作/分析时\n- team-run 模式需要分片并行执行时",
        "Quality Bar": "- [ ] 各 agent 的独立产出已记录\n- [ ] 分歧点已显式标注并解决\n- [ ] Collaboration trace 包含 handoff 日志\n- [ ] 最终合并结论的依据已记录\n- [ ] 任务拆分粒度使每个 agent 可独立完成其部分",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| Agent 间信息泄漏 | 交叉审查前看到对方输出 | 强制独立执行后再 merge |\n| 任务拆分不当 | 一个 agent 负载过重 | 按 skill boundary 而非任意拆分 |\n| Merge 无规则 | 不知道以谁的结论为准 | 预定义 merge 策略（majority/primary） |\n| 只比较最终结果 | 忽视中间推理差异 | 记录 reasoning trace 而非只比较 output |\n| 未记录 handoff | 后续无法追踪协作路径 | collaboration trace 必须包含每步 |"
    },
    "self-critique": {
        "When to Use": "- 需要主动提高 red-teaming 强度时\n- 产出可能存在浅层推理或过度主张时\n- 投稿前做最后一轮逻辑检查时\n- 需要 Socratic questioning 来压力测试结论时",
        "Quality Bar": "- [ ] 至少执行两轮 critique 迭代\n- [ ] 每个 critique 点附带具体修正建议\n- [ ] Overclaiming 已被识别并降级表述\n- [ ] 自相矛盾点已消解或标注为 limitation\n- [ ] Critique log 记录了改进前后的对比",
        "Common Pitfalls": "| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| 走过场 | Critique 只说整体不错 | 每轮至少 3 个 specific 挑战 |\n| 过度自我批评 | 导致不敢下结论 | 区分 fatal flaw vs. minor improvement |\n| 只关注表面 | 挑错别字不挑逻辑 | 按逻辑 → 证据 → 表述优先级 |\n| 无 action | 批判完但不修改 | 每条 critique 必须附带 action item |\n| Critique 同质化 | 每轮发现同一问题 | 每轮切换 lens（逻辑/证据/读者体验） |"
    },
}


def parse_warnings():
    """Return dict: filepath -> list of missing sections."""
    result = {}
    for line in WARNINGS.strip().split("\n"):
        filepath, sections = line.split(":")
        result[filepath] = [s.strip() for s in sections.split(",")]
    return result


def main():
    warnings = parse_warnings()
    root = Path(__file__).resolve().parent.parent

    modified_count = 0
    section_count = 0

    for filepath, missing_sections in sorted(warnings.items()):
        full_path = root / filepath
        if not full_path.exists():
            print(f"[SKIP] {filepath} — file not found")
            continue

        content = full_path.read_text(encoding="utf-8")

        # Extract skill ID from frontmatter using regex (no yaml dependency)
        fm_match = re.match(r"---\s*\n(.*?)\n---", content, re.DOTALL)
        skill_id = None
        desc = ""
        if fm_match:
            fm_text = fm_match.group(1)
            id_m = re.search(r"^id:\s*(.+)$", fm_text, re.MULTILINE)
            if id_m:
                skill_id = id_m.group(1).strip().strip('"').strip("'")
            desc_m = re.search(r'^description:\s*"?(.+?)"?\s*$', fm_text, re.MULTILINE)
            if desc_m:
                desc = desc_m.group(1).strip()

        if not skill_id:
            # Fallback: extract from filename
            skill_id = full_path.stem

        skill_content = CONTENT.get(skill_id, {})

        additions = []
        for section in missing_sections:
            if f"## {section}" in content:
                continue  # Already present

            section_text = skill_content.get(section)
            if section_text:
                additions.append(f"\n## {section}\n\n{section_text}\n")
            else:
                # Generate generic placeholder based on section type
                if section == "Quality Bar":
                    additions.append("\n## Quality Bar\n\n- [ ] All required outputs successfully generated\n- [ ] Output content matches the skill's contract specification\n- [ ] No unresolved validation errors or missing fields\n- [ ] Artifacts are self-contained and traceable to inputs\n")
                elif section == "Common Pitfalls":
                    additions.append("\n## Common Pitfalls\n\n| Pitfall | Problem | Fix |\n|---------|---------|-----|\n| Incomplete inputs | Missing prerequisite artifacts | Verify all required inputs exist before starting |\n| Scope drift | Output expands beyond intended scope | Refer back to contract specification |\n| Inconsistent formatting | Output format varies across runs | Follow output template strictly |\n")
                elif section == "When to Use":
                    additions.append(f"\n## When to Use\n\n- When the research workflow requires: {desc}\n- When upstream inputs are available and validated\n- When the corresponding task ID is triggered in the pipeline\n")

        if additions:
            # Append before final empty lines
            content = content.rstrip() + "\n" + "".join(additions)
            full_path.write_text(content, encoding="utf-8")
            modified_count += 1
            section_count += len(additions)
            print(f"[OK] {filepath} — added {len(additions)} sections")
        else:
            print(f"[SKIP] {filepath} — all sections already present")

    print(f"\nDone: modified {modified_count} files, added {section_count} sections")


if __name__ == "__main__":
    main()
