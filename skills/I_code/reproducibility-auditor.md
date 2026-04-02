---
id: reproducibility-auditor
stage: I_code
description: "Verify magic numbers, random seeds, containerization instructions, and fail-graceful contingencies for reproducibility."
inputs:
  - type: AnalysisCode
    description: "Code and analysis artifacts to audit"
outputs:
  - type: ReproducibilityReport
    artifact: "code/reproducibility_audit.md"
constraints:
  - "Must check all random seeds are set and documented"
  - "Must verify environment specification exists (requirements.txt etc.)"
failure_modes:
  - "Hidden statefulness not caught by static analysis"
  - "Platform-specific dependencies not documented"
tools: [filesystem]
tags: [code, reproducibility, audit, random-seeds, containerization]
domain_aware: false
---

# Reproducibility Auditor Skill

Audit a project for computational reproducibility and document a rerun recipe.

## Purpose

Verify magic numbers, random seeds, containerization instructions, and fail-graceful contingencies for reproducibility.

## Related Task IDs

- `I4` (reproducibility audit)

## Output (contract path)

- `RESEARCH/[topic]/code/reproducibility_audit.md`

## Inputs

- Current project under `RESEARCH/[topic]/`
- Entry commands used to generate results/figures

## Process

- **Environment**: language versions + key dependencies pinned
- **Randomness**: seeds controlled; nondeterminism documented
- **Data provenance**: where data came from; hashes/versions when possible
- **Execution**: one-command rerun path (or a short ordered list)
- **Outputs**: paths are stable and documented

## Required audit format (`code/reproducibility_audit.md`)

```markdown
---
task_id: I4
template_type: reproducibility_audit
topic: <topic>
primary_artifact: code/reproducibility_audit.md
---

# Reproducibility Audit

## Audit Contract Block
```json
{
  "task_id": "I4",
  "topic": "<topic>",
  "audit_artifact": "code/reproducibility_audit.md",
  "reviewed_artifacts": ["code/plan.md", "code/performance_profile.md"],
  "environment_files": ["requirements.txt"],
  "seed_policy_status": "PASS | WARN | BLOCK",
  "rerun_entrypoints": [
    {"command": "..."}
  ],
  "verdict": "PASS | WARN | BLOCK",
  "blocking_gaps": ["..."]
}
```

## Audit Scope
- ...

## Environment Evidence
- ...

## Data Provenance / Immutability
- ...

## Determinism / Seed Control
- ...

## Rerun Recipe
1. ...

## Failure Points / Recovery
- ...

## Audit Verdict
- ...

## Required Remediations
- ...

## Confidence
- 0.xx
```

## Quality Bar

- [ ] Random seed 已固定且 rerun 产出一致
- [ ] 环境可从 lock file 重建
- [ ] 数据路径不含 hardcoded absolute path
- [ ] README 包含完整 setup + run 指令
- [ ] Rerun recipe 存在且已测试通过

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 只固定主 seed | 子过程的随机性未控制 | 每个随机操作都 derive seed |
| 环境未锁定 | pip install 获取最新版可能 break | 使用 lock file |
| 数据未版本化 | 数据更新后结果不同 | 数据快照 + hash 记录 |
| 缺少 rerun 脚本 | 不知道按什么顺序跑 | Makefile / dvc pipeline |
| 忽视 OS 差异 | macOS vs Linux 浮点差异 | 容器化或文档记录 OS requirement |

## When to Use

- 代码完成后需要验证可复现性时
- 投稿前需要 reproducibility evidence 时
- 需要检查 random seed、容器化和 rerun 能力时
- 代码共享/开源前的最终审计时
