---
id: data-management-plan
stage: C_design
description: "Generate FAIR-compliant data management plans specifying storage, backup, retention, sharing, and archival strategy."
inputs:
  - type: DesignSpec
    description: "Study design with data types and volumes"
  - type: EthicsPackage
    description: "Ethics requirements constraining data handling"
    required: false
outputs:
  - type: DataManagementPlan
    artifact: "data_management_plan.md"
constraints:
  - "Must comply with FAIR principles (Findable, Accessible, Interoperable, Reusable)"
  - "Must specify retention periods and destruction procedures"
  - "Must identify repository for data deposit"
failure_modes:
  - "Institutional DMP requirements unknown"
  - "Data too sensitive for open deposit"
tools: [filesystem]
tags: [design, data-management, FAIR, storage, archival]
domain_aware: false
---

# Data Management Plan Skill

Generate a complete Data Management Plan (DMP) following FAIR principles and funder requirements.

## Purpose

Generate FAIR-compliant data management plans specifying storage, backup, retention, sharing, and archival strategy.

## Related Task IDs

- `C3` (data management planning)

## Output (contract path)

- `RESEARCH/[topic]/data_management_plan.md`

## Inputs

- **Study design** (`study_design.md`) — data types, expected volume, sensitivity level
- **Ethics requirements** (optional) — constraints from IRB/GDPR

## Process

1. **Data Description**
   - Enumerate all data types (survey, experimental, administrative, secondary)
   - Estimate volume and format per type
   - Classify sensitivity level (public / restricted / confidential)

2. **Collection & Organization**
   - Specify file naming conventions
   - Define folder structure
   - Document version control procedures

3. **Storage & Security**
   - Primary and backup storage locations
   - Encryption and access control measures
   - Compliance with institutional IT policies

4. **Preservation & Sharing**
   - Repository selection (Zenodo, Dryad, ICPSR, domain-specific)
   - License selection (CC-BY, CC0, restricted)
   - Embargo period justification if applicable
   - Persistent identifier (DOI) assignment

5. **Retention & Destruction**
   - Minimum retention period (typically 5-10 years post-publication)
   - Destruction protocol for sensitive data
   - Documentation of what is retained vs. destroyed

6. **Responsibilities & Resources**
   - Data steward assignment
   - Estimated storage costs
   - Training requirements

## Minimal Output Format

```markdown
# Data Management Plan

## 1. Data Description
| Data Type | Format | Volume | Sensitivity |
|---|---|---|---|

## 2. Collection & Organization
...

## 3. Storage & Security
...

## 4. Preservation & Sharing
| Repository | License | Embargo | DOI |
|---|---|---|---|

## 5. Retention & Destruction
...

## 6. Responsibilities
...
```

## Quality Bar

- [ ] 覆盖 FAIR 四原则（Findable, Accessible, Interoperable, Reusable）
- [ ] 存储、备份、保留和归档策略均已说明
- [ ] 数据共享计划包含 embargo/access levels
- [ ] 隐私保护措施与 ethics package 一致
- [ ] DMP 格式符合资助方模板要求

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 模板化填写 | 不具体到项目 | 每个字段填入 project-specific 内容 |
| 忽略成本 | 存储/归档需要预算 | 在 DMP 中注明成本估算 |
| 未考虑数据销毁 | 保留期限过后如何处理不清 | 明确 retention period + destruction policy |
| 访问权限不明 | 团队成员权限未定义 | 建立 role-based access matrix |
| 与 IRB 不一致 | DMP 承诺共享但 IRB 限制 | 交叉核查两份文档 |

## When to Use

- 基金申请需要提交 DMP 时
- 机构 IRB/ethics 要求数据治理文档时
- 协作研究需要明确数据存储和共享规则时
- FAIR 原则合规要求时
