---
id: code-review
stage: I_code
description: "Secondary model reviews code logic, security, statistical validity, and domain-specific correctness."
inputs:
  - type: AnalysisCode
    description: "Code to review"
  - type: DomainProfile
    description: "Domain checklist for review"
    required: false
outputs:
  - type: CodeReview
    artifact: "code/code_review.md"
constraints:
  - "Must check statistical correctness against domain checklist"
  - "Must verify random seed handling and reproducibility"
failure_modes:
  - "Reviewer lacks domain expertise for statistical validation"
  - "False positives on acceptable coding patterns"
tools: [filesystem]
tags: [code, review, security, statistical-validity, domain-checklist]
domain_aware: true
---

# Code Review Skill

Independent review of research code for correctness, reproducibility, and statistical validity.

## Purpose

Secondary model reviews code logic, security, statistical validity, and domain-specific correctness.

## Related Task IDs

- `I8` (code review)

## Output (contract path)

- `RESEARCH/[topic]/code/code_review.md`

## Domain Integration

When `--domain` is specified, load `skills/domain-profiles/[domain].yaml` and apply:
- Domain-specific **common_pitfalls** as mandatory review items
- Domain-specific **stats_diagnostics** as validation checkpoints
- Domain-specific **method_templates[*].required_diagnostics**, **failure_modes**, and **minimum_report_fields** for each detected method

## Inputs

- `AnalysisCode`: Code to review
- `DomainProfile`: Domain checklist for review
- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.
- Treat literature, data, citations, and project files as evidence sources; keep unsupported assumptions visibly marked.

## Process

### Gate-Aware Review

Review code and analysis outputs against `standards/quality-gate-contract.yaml`. For domain methods, load `skills/domain-profiles/[domain].yaml` and compare the implementation to `method_templates[*].required_diagnostics`, `failure_modes`, and `minimum_report_fields`; block when reported estimates omit required fields or diagnostics.

### Correctness
- Does the implementation match the method/spec (I5)?
- Are edge cases handled explicitly?
- Are there silent failure modes (NaNs, empty slices, wrong joins)?
- Do loops/vectorizations match mathematical definitions exactly?

### Statistical validity
- Are estimands and standard errors computed correctly?
- Are assumptions checked (or at least flagged)?
- Are multiple comparisons / leakage risks addressed (if relevant)?
- Is the effect size reported alongside p-values?

### Reproducibility
- Fixed seeds where appropriate
- Deterministic pipelines documented (versions, configs)
- Clear rerun instructions in `code/documentation/`
- Container config (Docker/Singularity) if applicable

### Security / data safety (as applicable)
- No secrets in code
- Safe file I/O paths
- Privacy constraints respected (D3)

## Domain Profile Review Contract

Do not maintain local domain review checklists in this skill. For each detected method, load `skills/domain-profiles/[domain].yaml`, match against `method_templates[*]`, and review implementation, outputs, and reports against the template's `required_diagnostics`, `required_artifacts`, `failure_modes`, and `minimum_report_fields`. If the method is absent from the profile, record an insufficient-input or unsupported-method finding rather than inventing domain rules.

## Required review format (`code/code_review.md`)

```markdown
---
task_id: I8
template_type: code_review
topic: <topic>
primary_artifact: code/code_review.md
---

# Code Review

## Review Contract Block
```json
{
  "task_id": "I8",
  "topic": "<topic>",
  "review_target": "<code / notebook / pipeline>",
  "spec_source": "code/code_specification.md",
  "plan_source": "code/plan.md",
  "review_artifact": "code/code_review.md",
  "verdict": "PASS | BLOCK",
  "blocking_findings": [
    {"finding_id": "P1-01"}
  ],
  "review_coverage": [
    "method_fidelity",
    "statistical_validity",
    "reproducibility"
  ]
}
```

## Verdict
- ...

## Scope Reviewed
- ...

## Findings Table
| Finding ID | Severity | Area | Evidence | Required Action |
| --- | --- | --- | --- | --- |
| P1-01 | P1 | statistical_validity | ... | ... |

## Blocking Findings
1. ...

## Non-Blocking Findings
- ...

## Domain Checklist Status
- [ ] Item 1 (from domain profile)
- [ ] Item 2
- ...

## Reproducibility / Statistical Validity
- ...

## Required Fix Order
1. ...

## Residual Risks
- ...

## Confidence
- 0.xx
```

## Output Contract

- `CodeReview`: write `RESEARCH/[topic]/code/code_review.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

## Quality Bar

- [ ] 逻辑正确性已逐函数审查
- [ ] 统计方法与 analysis plan 一致
- [ ] 无数据泄漏（训练/测试/时间序列前瞻）
- [ ] 代码风格和命名一致
- [ ] Review 报告包含具体行号引用

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 只查格式 | 忽视逻辑和统计错误 | 按 logic → statistics → style 优先级 |
| 不理解方法 | 审查者不熟悉 estimator | 审查前阅读 analysis plan + 方法文献 |
| 无 action item | 评论模糊 | 每条评论是 must-fix / should-fix / nice-to-have |
| 忽视数据泄漏 | 训练集信息泄漏到测试 | 专门检查 data split boundary |
| 审查不独立 | 看了作者解释再审 | 先盲审代码，再看文档 |

## When to Use

- 分析代码完成后需要第二模型/人审查时
- 需要检查统计有效性和方法论一致性时
- 投稿前需要独立核验代码逻辑时
- 需要安全性审查（数据泄漏、路径注入）时
