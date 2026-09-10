---
description: Build academic research code from paper descriptions or methodology names
---

# Code Build Workflow

Builds academic research code using the repo's strict Stage-I workflow rather than generic product-engineering scaffolds.

Canonical Task IDs (from the globally installed `qiongli-workflow` skill):
- `I1` method implementation
- `I2` reproduction
- `I3` data pipeline
- `I4` reproducibility audit
- `I5` code specification
- `I6` code planning
- `I7` execution + performance packaging
- `I8` code review

## Usage

```bash
/code-build [method_name] --topic [topic] --domain [domain] --focus [focus] --tier [standard|advanced]
```

## Options

- `method_name`: The specific methodology (e.g., "Difference-in-Differences", "DSGE")
- `--topic`: research topic slug used to route outputs into `RESEARCH/[topic]/...`
- `--domain`: `finance`, `econ`, `metrics`, `cs`, `psychology`, etc. (runtime-injected profile; auto if omitted)
- `--focus`: maps the request to a Stage-I task
  - `implementation` -> `I1`
  - `reproduction` -> `I2`
  - `data_pipeline` -> `I3`
  - `reproducibility_audit` -> `I4`
  - `code_specification` -> `I5`
  - `code_planning` -> `I6`
  - `execution_performance` -> `I7`
  - `code_review` -> `I8`
  - `full` -> strict sequence `I5 -> I6 -> I7 -> I8`
- `--tier`:
  - `standard`: Use standard libraries (Tier 1)
  - `advanced`: Implement from scratch/equations (Tier 2)
- `--paper`: URL or Path to paper PDF (optional)
- `--triad`: when the strict flow reaches `I8`, add a third independent audit
- `--only-target`: rerun only selected actionable targets from an existing structured Stage-I artifact
  - single-stage example: `--focus code_planning --only-target S1`
  - `full` example: `--focus full --only-target I5:decision-1 --only-target I8:P1-01`

## Full Workflow Steps (`--focus full`)

1. **Specification (`I5`)**:
   - lock the method, I/O contract, seeds, diagnostics, acceptance tests, and forbidden shortcuts
   - no implementation freedom before the spec is explicit

2. **Planning (`I6`)**:
   - turn the spec into a zero-decision execution plan
   - define checkpoints, validation steps, dependency order, and rollback points

3. **Execution (`I7`)**:
   - implement exactly against the approved plan
   - package scripts/notebooks, performance notes, and reproducibility instructions
   - do not invent app scaffolding or generic engineering layers that are not required by the research method

4. **Review (`I8`)**:
   - verify method fidelity, statistical validity, reproducibility, and contract path compliance
   - prioritize blockers over style or refactor suggestions

5. **Optional Audit (`I4`)**:
   - run a targeted reproducibility audit when you need environment, seed, and rerun guarantees reviewed separately

Infer the requested focus when it is omitted. A named project or `--topic`
selects context and destination, not the full sequence. For a narrow code change,
reuse the existing specification and analysis decisions, edit only the affected
code and run its meaningful checks. Do not create new specification, plan,
packaging or review artifacts unless the selected formal task requires them.
Use the full sequence for an explicit full implementation/reproduction request
or `--focus full`. A bounded correction must not be claimed as a completed full
Stage-I run. Native 2.x executes in the active Host without a legacy controller.

## Academic Boundary Review Trigger (MVP)

For `I5` and `I6`, use `boundary-interviewer` before writing the specification or plan when a code decision could change scientific interpretation, reproducibility, claim strength, or claim-evidence traceability.

Use the boundary pass for:

- analysis-plan assumptions that implementation must preserve
- data provenance, preprocessing, missingness, split, or leakage decisions
- random seed, stochastic operation, dependency, or environment choices that affect reproducibility
- synthetic or fixture data limits that affect validation
- output artifacts that feed manuscript claims, figures, tables, or evidence ledgers
- non-goals that prevent the code from implying a broader method claim than the paper supports

The boundary pass must ask one academic question at a time and must record which code decision could change the research claim or evidence threshold.

## Execution Rules

- Treat this as academic research code, not frontend/backend product delivery.
- Keep outputs aligned to `RESEARCH/[topic]/...` and the contract-defined `code/` artifacts.
- Use the runtime-injected domain profile as a constraint source, not a suggestion.
- Prefer reproducibility, diagnostics, and validation evidence over helper-file sprawl.
- Treat Academic Analysis Code as the default style: start from the estimand,
  hypothesis, analysis plan, or manuscript-facing output, then build only the
  scripts/notebooks/modules required to produce auditable evidence.
- Preserve dataset lineage, model diagnostics, robustness checks, manuscript-facing
  tables/figures, seeds, dependency notes, command logs, and rerun instructions.
- Avoid service layers, controllers, framework scaffolding, and unnecessary
  classes unless the research method genuinely requires a reusable library.
- `I4`, `I5`, `I6`, `I7`, and `I8` should emit YAML frontmatter plus JSON contract blocks and fixed section headings, so audit/spec/plan/execution/review artifacts are auditable and machine-scannable.
- The orchestrator will parse those machine-readable blocks after each Stage-I run and surface actionable targets for later reruns or audits.
- `--only-target` reuses those parsed actionable targets and loads the existing `RESEARCH/[topic]/code/*.md` artifact before rerunning the selected stage.
- For `full`, earlier stage outputs are authoritative; later stages should not silently reopen settled decisions.

## Examples

### Full Strict Econometrics Flow
```bash
/code-build "Staggered DID" --topic policy-effects --domain econ --focus full
```
> Runs `I5 -> I6 -> I7 -> I8` for an econometrics implementation and keeps outputs inside `RESEARCH/policy-effects/`.

### Advanced Structural Specification
```bash
/code-build "Dynamic Discrete Choice" --topic entry-exit-model --domain econ --tier advanced --focus code_specification
```
> Produces the strict code specification for a Tier-2 structural method before any implementation starts.

### Data Pipeline Build
```bash
/code-build "Panel Merge Pipeline" --topic cross-country-growth --domain econ --focus data_pipeline
```
> Routes directly to `I3` for data cleaning/merge planning tied to the research topic.

### Final Academic Code Review
```bash
/code-build "Transformer Fine-Tuning" --topic llm-bias --domain cs --focus code_review --triad
```
> Runs `I8` with an optional third independent audit for the final review pass.

### Targeted Follow-Up
```bash
/code-build "Transformer Fine-Tuning" --topic llm-bias --domain cs --focus full --only-target I5:decision-1 --only-target I8:P1-01
```
> Reopens only the selected spec/review targets and skips unselected Stage-I phases.
