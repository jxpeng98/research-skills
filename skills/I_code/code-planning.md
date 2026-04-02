---
id: code-planning
stage: I_code
description: "Transform code specifications into parallelizable, zero-decision execution plans."
inputs:
  - type: CodeSpec
    description: "Code specification with constraints"
outputs:
  - type: CodePlan
    artifact: "code/plan.md"
constraints:
  - "Must produce plans that require zero decisions during execution"
  - "Must identify parallelizable tasks"
failure_modes:
  - "Dependencies prevent meaningful parallelization"
  - "Plan too granular for practical execution"
tools: [filesystem]
tags: [code, planning, execution-plan, parallelization]
domain_aware: false
---

# Code Planning Skill

Transform a specification into a zero-decision execution plan that can be parallelized and audited.

## Purpose

Transform code specifications into parallelizable, zero-decision execution plans.

## Related Task IDs

- `I6` (code planning)

## Output (contract path)

- `RESEARCH/[topic]/code/plan.md`

## Inputs (minimum)

- `code/code_specification.md` (or an equivalent spec in the prompt)
- Current repository/project structure under `RESEARCH/[topic]/`

## Process

1. **Decompose** into small tasks (≤ 30–60 min each).
2. **Order** tasks by dependencies and risk (do the riskiest first).
3. **Define checkpoints** with concrete “pass/fail” criteria.
4. **Plan artifacts**: where code, data, plots, and reports will be written.
5. **Plan commands**: exact commands to run for each checkpoint.

## Required plan format (`code/plan.md`)

```markdown
---
task_id: I6
template_type: code_plan
topic: <topic>
primary_artifact: code/plan.md
---

# Execution Plan

## Plan Contract Block
```json
{
  "task_id": "I6",
  "topic": "<topic>",
  "spec_source": "code/code_specification.md",
  "plan_artifact": "code/plan.md",
  "steps": [
    {
      "step_id": "S1",
      "depends_on": [],
      "owner": "<agent-or-role>",
      "command": "<exact command>",
      "outputs": ["..."],
      "checkpoint": "<observable pass/fail rule>",
      "rollback": "<recovery action>"
    }
  ]
}
```

## Scope Lock
- ...

## Assumptions From Spec
- ...

## Step Ledger
1. [ ] `S1` — owner — output — checkpoint — rollback

## Checkpoint Matrix
| Step | Inputs | Output | Pass Condition | Failure Trigger |
| --- | --- | --- | --- | --- |
| ... | ... | ... | ... | ... |

## Exact Run Commands
- ...

## Parallelization / Dependency Map
- ...

## Rollback / Recovery
- ...

## Risks / Blockers
- ...
```

## Quality Bar

- [ ] 每个步骤有明确的输入/输出定义
- [ ] 依赖关系图已标注且无循环依赖
- [ ] 可并行步骤已标识
- [ ] 每个步骤有验收标准（assertion 或 test case）
- [ ] 估算完成时间/复杂度

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 步骤粒度不均 | 有的步骤 5 分钟有的 5 小时 | 拆分大步骤至 1-2 小时粒度 |
| 依赖不明 | 并行执行时出错 | 画依赖 DAG 并标注 critical path |
| 缺少验收标准 | 不知道步骤是否完成 | 每步附带 assertion checklist |
| 过度设计 | 计划比代码还长 | 保持计划简洁，focus on interface |
| 忽略数据 I/O | 计划只关心算法 | 明确数据流 format 和存储位置 |

## When to Use

- 已有 code specification，需要拆分为可并行的执行步骤时
- 复杂分析需要零自由裁量的实现计划时
- 团队协作需要明确模块划分和接口定义时
- 需要估算工作量和识别关键路径时
