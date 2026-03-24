---
id: reproducibility-auditor
stage: I_code
version: "0.2.0"
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

## Related Task IDs

- `I4` (reproducibility audit)

## Output (contract path)

- `RESEARCH/[topic]/code/reproducibility_audit.md`

## Inputs

- Current project under `RESEARCH/[topic]/`
- Entry commands used to generate results/figures

## Audit checklist

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
