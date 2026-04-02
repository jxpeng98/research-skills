---
id: code-execution
stage: I_code
description: "Execute code plans with performance profiling (cProfile/vectorization) and optimization."
inputs:
  - type: CodePlan
    description: "Execution plan to follow"
outputs:
  - type: PerformanceProfile
    artifact: "code/performance_profile.md"
constraints:
  - "Must follow plan exactly without improvisation"
  - "Must capture performance metrics"
failure_modes:
  - "Runtime errors from environment mismatch"
  - "Performance bottlenecks in unoptimized code"
tools: [filesystem, code-runtime]
tags: [code, execution, profiling, performance, optimization]
domain_aware: false
---

# Code Execution Skill

Execute the plan: implement, test, profile, and document research code with reproducible outputs.

## Purpose

Execute code plans with performance profiling (cProfile/vectorization) and optimization.

## Related Task IDs

- `I7` (code execution)

## Outputs (contract paths)

- `RESEARCH/[topic]/analysis/`
- `RESEARCH/[topic]/code/performance_profile.md`
- `RESEARCH/[topic]/code/container_config/` (optional but recommended)
- `RESEARCH/[topic]/code/documentation/`

## Inputs

- `code/plan.md`
- Any required datasets (or a synthetic generator for verification)

## Process

1. **Implement incrementally** with tests after each unit.
2. **Validate** against the spec (I/O, invariants, edge cases).
3. **Add a minimal runnable entrypoint** (script or CLI).
4. **Profile** hot paths (time + memory) and record results.
5. **Document** how to reproduce:
   - dependencies
   - commands
   - expected outputs

## Required performance profile format (`code/performance_profile.md`)

```markdown
---
task_id: I7
template_type: performance_profile
topic: <topic>
primary_artifact: code/performance_profile.md
---

# Performance Profile

## Execution Contract Block
```json
{
  "task_id": "I7",
  "topic": "<topic>",
  "plan_source": "code/plan.md",
  "performance_artifact": "code/performance_profile.md",
  "analysis_outputs": ["analysis/..."],
  "documentation_outputs": ["code/documentation/..."],
  "container_outputs": ["code/container_config/..."],
  "validation_runs": [
    {"step_id": "S1", "evidence": "..."}
  ],
  "profiling_targets": [
    {"component": "...", "command": "..."}
  ]
}
```

## Scope Executed
- ...

## Implementation Ledger
| Step ID | Planned Output | Observed Output | Status | Notes |
| --- | --- | --- | --- | --- |
| S1 | ... | ... | PASS | ... |

## Validation Evidence
| Check | Evidence | Result | Artifact |
| --- | --- | --- | --- |
| ... | ... | ... | ... |

## Artifact Inventory
- `analysis/...`
- `code/documentation/...`
- `code/container_config/...`

## Environment / Containerization
- OS:
- Python/R version:
- Key deps:

## Profiling Results
- Dataset size:
- Command:
| Component | Time | Notes |
|---|---:|---|

## Optimization Actions Taken
1. ...

## Reproduction Commands
1. ...

## Remaining Gaps / Blockers
- ...
```

## Quality Bar

- [ ] 每个计划步骤都已实现且有对应测试
- [ ] cProfile 输出已记录且无明显瓶颈（或已解释原因）
- [ ] 内存占用在合理范围内
- [ ] 边界情况和错误路径有明确处理
- [ ] 代码风格一致且通过 linter

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 跳过 profiling | 不知道瓶颈在哪 | 至少运行一次 cProfile/line_profiler |
| 无 error handling | 遇到缺失数据直接 crash | 添加 try/except + 明确 fallback |
| 未验证中间结果 | 错误沿 pipeline 传播 | 每步添加 shape/summary assertion |
| N+1 查询 | 循环内重复 I/O | 批量化操作 |
| 忽略内存 | 大数据集 OOM | 使用 chunk processing 或 lazy eval |

## When to Use

- 代码计划已制定，需要按步骤实现并验证时
- 需要 cProfile 性能分析和瓶颈优化时
- 需要记录执行证据（log、profiling output）时
- 从 code-planning 输出接力到实际代码编写时
