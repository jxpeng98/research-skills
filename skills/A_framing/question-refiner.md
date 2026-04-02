---
id: question-refiner
stage: A_framing
description: "Transform vague research topics into structured, answerable research questions using PICO, PEO, or SPIDER framing plus FINER evaluation."
inputs:
  - type: UserQuery
    description: "Raw research topic or area of interest"
outputs:
  - type: RQSet
    artifact: "framing/research_question.md"
constraints:
  - "Must apply PICO, PEO, or SPIDER as fit to the study design"
  - "Must evaluate with FINER criteria"
  - "Must generate inclusion/exclusion criteria"
failure_modes:
  - "Topic too broad to produce actionable RQ"
  - "Missing domain context for framework selection"
tools: [filesystem]
tags: [framing, research-question, PICO, PEO, SPIDER, FINER]
domain_aware: false
---

# Question Refiner Skill

Transform vague research topics into structured, answerable research questions using academic frameworks.

## Purpose

Convert raw research ideas into well-structured research questions that are:
- Specific and focused
- Answerable within scope
- Grounded in appropriate frameworks (PICO/PEO/SPIDER)
- Evaluated by FINER criteria

## Process

### Step 1: Topic Exploration

Ask clarifying questions:

1. **Domain Clarification**
   - What is the broad research area?
   - What discipline(s) does this span?
   - What is the specific phenomenon of interest?

2. **Scope Definition**
   - What aspects are you most interested in?
   - What is the geographic scope?
   - What is the temporal scope?
   - What population/context is relevant?

3. **Output Requirements**
   - What type of output do you need? (Review, empirical study, qualitative study, methods paper, theoretical paper)
   - What is the target venue/audience?
   - What is the expected depth of analysis?

### Step 2: Framework Application

#### For Intervention/Effect Studies - Use PICO

- **P**opulation: Who is being studied?
- **I**ntervention: What intervention or exposure?
- **C**omparison: What is the comparison/control?
- **O**utcome: What outcomes are measured?

Example:
- P: University students
- I: AI tutoring systems
- C: Traditional tutoring
- O: Learning outcomes, engagement

→ RQ: "How do AI tutoring systems compare to traditional tutoring in improving learning outcomes and engagement among university students?"

#### For Non-Intervention Studies - Use PEO

- **P**opulation: Who is being studied?
- **E**xposure: What phenomenon/exposure?
- **O**utcome: What is observed/measured?

Example:
- P: Software developers
- E: Remote work adoption
- O: Productivity, well-being

→ RQ: "How does remote work adoption affect productivity and well-being among software developers?"

#### For Qualitative / Experience / Process Studies - Use SPIDER

- **S**ample: Who or what will be studied?
- **PI** Phenomenon of Interest: What process, experience, practice, or meaning is focal?
- **D**esign: Interviews, ethnography, case study, observation, document analysis, or mixed qualitative design?
- **E**valuation: What kind of understanding is sought (meanings, mechanisms, process explanations, practices)?
- **R**esearch type: Qualitative / mixed-methods

Example:
- S: Platform managers and complementors
- PI: Governance tensions during AI policy rollout
- D: Multi-case interview study
- E: How governance practices are interpreted and negotiated
- R: Qualitative

→ RQ: "How do platform managers and complementors negotiate governance tensions during AI policy rollout?"

### Step 3: Question Refinement

Transform into specific question types:

| Type | Starter | Use When |
|------|---------|----------|
| Descriptive | "What is...?" | Exploring phenomenon |
| Comparative | "How does X compare to Y...?" | Comparing groups/conditions |
| Relational | "What is the relationship between...?" | Examining associations |
| Causal | "Does X cause Y...?" | Testing causation |
| Exploratory | "How do participants experience...?" | Understanding perspectives |
| Process / Mechanism | "How does X unfold...?" | Explaining sequences, turning points, and mechanisms |
| Interpretive | "How do actors make sense of...?" | Understanding meanings, framings, or identity work |

### Step 4: FINER Evaluation

Evaluate the research question:

| Criterion | Question | Assessment |
|-----------|----------|------------|
| **F**easible | Can it be done with available resources? | ✓/✗ |
| **I**nteresting | Is it worth investigating? | ✓/✗ |
| **N**ovel | Does it add new knowledge? | ✓/✗ |
| **E**thical | Can it be done ethically? | ✓/✗ |
| **R**elevant | Does it matter to stakeholders? | ✓/✗ |

### Step 5: Output

Generate structured output:

```markdown
## Research Question

**Main RQ**: [Refined research question]

**Sub-questions**:
1. [Sub-RQ 1]
2. [Sub-RQ 2]
3. [Sub-RQ 3]

## Framework Used
[PICO/PEO breakdown]

## FINER Assessment
| Criterion | Assessment | Notes |
|-----------|------------|-------|
| Feasible | ✓ | ... |
| Interesting | ✓ | ... |
| Novel | ✓ | ... |
| Ethical | ✓ | ... |
| Relevant | ✓ | ... |

## Inclusion Criteria
- Criterion 1
- Criterion 2

## Exclusion Criteria
- Criterion 1
- Criterion 2

## Key Search Terms
- Term 1
- Term 2 (synonym: ...)
```

## Usage

This skill is called by:
- `/lit-review` - At the scoping phase
- `/find-gap` - To clarify research area
- `/build-framework` - To define scope

## Quality Bar

- [ ] 每个 RQ 通过五项 FINER 评估 (Feasible, Interesting, Novel, Ethical, Relevant)
- [ ] PICO/PEO 四要素均已显式填写
- [ ] 至少一个 primary RQ 可直接对应分析计划
- [ ] 每个 RQ 附带 scope boundary — 明确研究不包括什么
- [ ] 术语定义一致且无歧义

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| RQ 过宽 | 无法在一篇论文中回答 | 用 PICO 强制拆分 sub-RQ |
| 缺 boundary | Reviewer 质疑 scope creep | 每个 RQ 附 exclusion list |
| 混淆 RQ 与假设 | RQ 不可检验 | RQ 用疑问句，假设用陈述句 |
| 术语不一致 | 同一概念多种表述 | 建立 glossary 并固定用词 |
| 忽视 feasibility | 提出无法获取数据的问题 | FINER-F 先行评估 |

## When to Use

- 选题还模糊、范围过大或研究问题不可执行时
- 导师反馈研究问题不够聚焦时
- 需要在 PICO/PEO 框架下结构化问题时
- 从课题申请转化为论文时需要收窄 scope
