---
id: code-specification
stage: I_code
description: "Generate strict constraint sets and requirement specifications before coding, following OPSX-style methodology."
inputs:
  - type: UserQuery
    description: "High-level method or feature description"
outputs:
  - type: CodeSpec
    artifact: "code/code_specification.md"
constraints:
  - "Must produce machine-parseable constraints"
  - "Must identify all external dependencies"
failure_modes:
  - "Ambiguous requirements leading to specification gaps"
  - "Over-specification constraining implementation flexibility"
tools: [filesystem]
tags: [code, specification, requirements, OPSX, constraints]
domain_aware: false
---

# Code Specification Skill

Generate a strict, testable constraint set before writing research code.

## Purpose

Generate strict constraint sets and requirement specifications before coding, following OPSX-style methodology.

## Related Task IDs

- `I5` (code specification)

## Output (contract path)

- `RESEARCH/[topic]/code/code_specification.md`

## Inputs (ask for)

- What needs to be implemented (method/pipeline/reproduction target)
- Expected inputs/outputs (file formats, schemas, sample data)
- Constraints: runtime, memory, dependency limits, hardware (CPU/GPU)
- Required validation: unit tests, benchmarks, replication targets

## Process

1. **Define success criteria**: what counts as “correct” (and what doesn’t).
2. **Specify I/O contracts**:
   - input files/columns/shapes
   - output artifacts (paths + formats)
3. **List invariants**: properties that must always hold (e.g., probability sums to 1).
4. **Enumerate edge cases**: missing data, empty groups, small-k meta-analysis, non-convergence.
5. **Allowed dependencies**: language + libraries + version constraints.
6. **Determinism rules**: seeds, randomness sources, hardware nondeterminism notes.
7. **Validation plan**:
   - synthetic data checks
   - unit tests for core functions
   - regression tests for known examples

## Required spec format (`code/code_specification.md`)

```markdown
---
task_id: I5
template_type: code_specification
topic: <topic>
primary_artifact: code/code_specification.md
---

# Code Specification

## Spec Contract Block
```json
{
  "task_id": "I5",
  "topic": "<topic>",
  "method_or_pipeline": "<name>",
  "primary_artifact": "code/code_specification.md",
  "inputs": [
    {"path": "...", "schema": "..."}
  ],
  "outputs": [
    {"path": "...", "format": "..."}
  ],
  "dependencies": {
    "python": ["package>=version"]
  },
  "seeds_policy": {
    "global_seed": "...",
    "nondeterminism_notes": "..."
  },
  "acceptance_tests": [
    {"name": "...", "metric": "...", "pass_condition": "..."}
  ],
  "blocked_decisions": ["..."]
}
```

## Goal
- ...

## Non-Goals
- ...

## Inputs (Schema)
- ...

## Outputs (Paths)
- ...

## Functional Requirements
1. ...

## Non-Functional Requirements
- Performance:
- Determinism:
- Logging:

## Edge Cases And Failure Modes
- ...

## Validation Matrix
| Check | Metric / Observable | Pass Condition | Artifact |
| --- | --- | --- | --- |
| ... | ... | ... | ... |

## Disallowed Shortcuts
- ...

## Blocked Decisions / Escalations
- ...
```

## Quality Bar

- [ ] 输入/输出类型和格式已完全定义
- [ ] 约束条件可自动测试（不含模糊表述）
- [ ] 验收标准有具体数字或条件
- [ ] 与 analysis plan 逆向可追溯
- [ ] 边界情况和异常处理已列出

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 规范太模糊 | 实现时有自由裁量空间 | 每条约束用 GIVEN-WHEN-THEN 格式 |
| 过度规范 | 规范比代码还长 | 只规范接口和约束，不规范实现 |
| 遗漏 edge case | 缺失数据/空输入未定义 | 用 boundary value analysis 补充 |
| 与 analysis plan 脱节 | 规范了代码但不对应统计需求 | 添加 traceability matrix |
| 无版本控制 | 规范修改后实现不同步 | 规范文件纳入 git 管理 |

## When to Use

- 编码前需要先锁定约束、输入输出和验收标准时
- 需要 OPSX 风格的严格规范集时
- 复杂分析需要正式的 contract before implementation 时
- 多人协作编码需要统一接口规范时
