# Stage I — Research Code Support (I1–I8)

This stage makes the computational parts reproducible: implementation, pipelines, audits, and cross-model review.

## Canonical outputs (contract paths)

- `I1` → `analysis/` (method implementation)
- `I2` → `analysis/` (reproduction)
- `I3` → `analysis/` (data pipeline)
- `I4` → `code/reproducibility_audit.md`
- `I5` → `code/code_specification.md`
- `I6` → `code/plan.md`
- `I7` → `code/performance_profile.md`, `code/container_config/`, `code/documentation/`, `analysis/`
- `I8` → `code/code_review.md`

## Quality gate focus

- `Q4` (reproducibility baseline) is the primary gate in this stage.
- Semantic gate report: update `quality-gate-report.md` with `q4_reproducibility_baseline`; evidence must anchor data lineage, code entrypoints, commands, environment, outputs, seeds, and rerun limits.

---

## Recommended workflow pattern (CCG-style)

Use a low-freedom sequence for reliability:

1. **Specification (`I5`)**: extract constraints (I/O schema, invariants, edge cases, metrics, tooling)
2. **Planning (`I6`)**: produce a zero-decision plan (tasks, dependencies, checkpoints)
3. **Execution (`I7`)**: implement + profile + document
4. **Independent review (`I8`)**: separate model reviews logic + stats validity + failure cases
5. **Audit (`I4`)**: seeds, versions, determinism, data provenance, rerun instructions

## Academic Analysis Code

Stage I code is academic analysis code, not application architecture. Start from
the estimand, hypothesis, analysis plan, or manuscript-facing table/figure that
the code must support. Only add abstractions that make the research pipeline more
auditable.

Required analysis-code constraints:

- Preserve dataset lineage: raw input, cleaning rules, exclusions, missingness,
  joins, derived variables, and sample construction.
- Treat model diagnostics and robustness checks as first-class outputs, not
  optional plots added after the fact.
- Write manuscript-facing tables, figures, and machine-readable result files to
  predictable paths under `RESEARCH/[topic]/analysis/` or
  `RESEARCH/[topic]/manuscript/`.
- Record seeds, dependency notes, command logs, and rerun instructions.
- Separate finding, interpretation, and implication in analysis reports.
- Prefer scripts, notebooks, Quarto files, or small modules readable by
  researchers over service layers, controllers, framework scaffolding, or
  unnecessary classes.

When exploratory analysis is requested, label outputs as exploratory and record
assumptions. Do not let exploratory code silently become claim-supporting
evidence without a Stage I specification, plan, execution record, and review.

## I8 — Academic Code Review

**Definition of done**
- Review covers method fidelity, inferential validity, leakage risks, and reproducibility evidence
- Findings are severity-ranked and tied to concrete code or artifact evidence
- Blocking academic risks are separated from non-blocking cleanup
- Prefer a dedicated academic code reviewer role when available, rather than folding I8 into implementation ownership

---

## What “done” looks like for code artifacts

### `analysis/`
- Contains runnable scripts/notebooks with clear entrypoints
- Includes a minimal dataset stub or synthetic data generator for verification
- Writes outputs to a predictable location (avoid hidden state)
- Records dataset lineage, model diagnostics, robustness checks, and
  manuscript-facing output paths

### `code/container_config/`
- Optional but recommended when dependencies are fragile
- Minimal `Dockerfile` or environment instructions

### `code/documentation/`
- `README.md` for how to run, reproduce, and interpret outputs

---

## Multi-model collaboration (Codex / Claude)

Use the orchestrator to split roles:

- Codex: implementation + execution
- Claude: narrative documentation + reasoning checks
- Academic code reviewer role (I8): severity-ranked audit of method fidelity and reproducibility claims

When triad is unavailable, fall back to:
- dual-chain (generate → verify), or
- single-agent with explicit self-critique log.
