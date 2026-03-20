# Task Recipes

Use this page when you know your real-world goal, but you do not yet know which stages, Task IDs, or skills to lean on.

This page is intentionally task-first:

- start from the job you need done
- map it to stages and Task IDs
- understand which skills are typically involved
- choose the smallest route that still gives you defensible outputs

If you want the full stage-by-stage map, use [Skills Guide](/reference/skills).
If you want paper-type defaults such as "systematic review" or "methods paper," use [Examples](/examples/).

## How To Use This Page

For each scenario below, read it in this order:

1. **When to use**
2. **Minimal route**
3. **Deeper route**
4. **Typical skills**
5. **Typical outputs**

Do not assume you must run every stage.
The best workflow is usually the narrowest route that still satisfies your paper type and evidence needs.

## 1. I Need To Turn A Broad Topic Into A Researchable Question

### When to use

Use this when the topic is still fuzzy, the contribution is not yet clear, or the venue target is still moving.

### Minimal route

- `A1`: refine the question and define scope
- `A4`: identify the strongest gap

### Deeper route

- `A1`
- `A1_5`: generate hypotheses or propositions
- `A2` / `A3`: build theory and positioning
- `A5`: check venue fit

### Typical skills

- `question-refiner`
- `hypothesis-generator`
- `theory-mapper`
- `gap-analyzer`
- `venue-analyzer`

### Typical outputs

- refined research question set
- contribution framing
- theory map
- prioritized gap memo
- venue-fit constraints

### Good first command

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id A1 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

## 2. I Need A Defensible Related-Work Or Systematic Review Base

### When to use

Use this when the main bottleneck is corpus quality, screening discipline, extraction consistency, or PRISMA-style transparency.

### Minimal route

- `B1_5`: concept and keyword expansion
- `B2`: focused paper reading / extraction
- `B3`: literature mapping

### Deeper route

- `B1`: full search
- `B1_5`
- `B2`
- `B3`
- `B4` / `B5`: citation expansion and/or synthesis support
- `G1`: PRISMA check before submission

### Typical skills

- `academic-searcher`
- `concept-extractor`
- `paper-screener`
- `paper-extractor`
- `citation-snowballer`
- `literature-mapper`
- `prisma-checker`

### Typical outputs

- search log
- screening log
- extraction table
- literature map
- PRISMA-ready counts and compliance memo

### Good first command

```bash
python3 -m bridges.orchestrator task-run \
  --task-id B2 \
  --paper-type systematic-review \
  --topic ai-in-education \
  --cwd . \
  --research-depth deep
```

## 3. I Need To Design An Empirical Study Before Writing Or Coding

### When to use

Use this when your question is already stable, but the design, variables, robustness plan, or dataset path is still weak.

### Minimal route

- `C1`: main design
- `C2` or `C3`: variables / data feasibility

### Deeper route

- `C1`
- `C1_5` / `C2`: rival explanations and variable logic
- `C3`: dataset feasibility
- `C3_5` / `C4`: robustness and data management
- `C5`: prereg-style handoff

### Typical skills

- `study-designer`
- `rival-hypothesis-designer`
- `dataset-finder`
- `variable-constructor`
- `robustness-planner`

### Typical outputs

- design spec
- analysis plan
- variable specification
- dataset plan
- robustness plan

### Good first command

```bash
python3 -m bridges.orchestrator task-run \
  --task-id C1 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .
```

## 4. I Need To Turn Results Into A Manuscript

### When to use

Use this when the analysis exists, but the problem is story structure, tables, figures, interpretation, or abstract/title quality.

### Minimal route

- `F1`: outline or manuscript architecture
- `F2`: paragraph- or section-level writing

### Deeper route

- `F1`
- `F2`
- `F3`: full manuscript drafting
- `F4`: tables/figures/results interpretation support
- `F5` / `F6`: abstract, title, keyword, and final polishing support

### Typical skills

- `manuscript-architect`
- `analysis-interpreter`
- `effect-size-interpreter`
- `table-generator`
- `figure-specifier`
- `meta-optimizer`

### Typical outputs

- manuscript outline
- section drafts
- result narratives with uncertainty preserved
- paper-ready tables and figure specifications
- optimized title/abstract/keywords

### Good first command

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --focus-output manuscript/manuscript.md \
  --output-budget 2
```

## 5. I Need Academic Code, Not Generic Product Engineering

### When to use

Use this when the work is a methods paper, empirical pipeline, reproducibility package, or statistics-heavy implementation.

### Minimal route

- `I5`: specification
- `I6`: zero-decision plan

This is the right choice when you are still locking constraints before writing code.

### Deeper route

- `I5`
- `I6`
- `I7`: implementation and profiling
- `I8`: code/statistical review
- `I4`: reproducibility audit

### Typical skills

- `code-specification`
- `code-planning`
- `code-execution`
- `code-review`
- `reproducibility-auditor`
- `stats-engine`

### Typical outputs

- `code/code_specification.md`
- `code/plan.md`
- `code/performance_profile.md`
- `code/code_review.md`
- `code/reproducibility_audit.md`

### Good first command

```bash
python3 -m bridges.orchestrator code-build \
  --method "Staggered DID" \
  --topic policy-effects \
  --domain economics \
  --focus full \
  --paper-type methods \
  --cwd .
```

### If outputs are too broad

Use:

- `--only-target` for selective rerun
- `--research-depth deep` when reasoning is too shallow
- `--focus-output` and `--output-budget` when artifact spread is too wide

## 6. I Need To Stress-Test, Rebut, Or Package For Submission

### When to use

Use this when the manuscript is near completion, already under review, or needs a pre-submission harsh check.

### Minimal route

- `H1`: submission package
- `H2`: rebuttal support

### Deeper route

- `G1` / `G2`: reporting and PRISMA checks
- `G4`: tone cleanup
- `H1`
- `H2`
- `H3`: peer-review simulation
- `H4`: fatal-flaw scan

### Typical skills

- `submission-packager`
- `rebuttal-assistant`
- `peer-review-simulation`
- `fatal-flaw-detector`
- `reviewer-empathy-checker`
- `reporting-checker`

### Typical outputs

- cover letter and submission bundle
- point-by-point rebuttal matrix
- simulated reviewer report
- fatal-flaw memo
- response-tone adjustment log

### Good first command

```bash
python3 -m bridges.orchestrator task-run \
  --task-id H3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

## 7. I Only Need A Narrow Follow-Up, Not A Full Rerun

### When to use

Use this when you already have Stage-I artifacts and only need to rerun a few plan steps or fix a small set of review findings.

### Typical route

- `task-run --only-target <target>`
- `code-build --only-target I6:S1`
- `code-build --only-target I8:P1-01`

### Good examples

```bash
python3 -m bridges.orchestrator task-run \
  --task-id I6 \
  --paper-type methods \
  --topic policy-effects \
  --cwd . \
  --only-target S1
```

```bash
python3 -m bridges.orchestrator code-build \
  --method "Staggered DID" \
  --topic policy-effects \
  --domain economics \
  --focus full \
  --only-target I8:P1-01 \
  --cwd .
```

## Which Page Should You Use Next?

- Need the global map of stages and skills: [Skills Guide](/reference/skills)
- Need paper-type defaults: [Paper Type Playbooks](/examples/paper-type-playbooks)
- Need exact command flags: [CLI Reference](/reference/cli)
- Need collaboration/runtime details: [Agent + Skill Collaboration](/advanced/agent-skill-collaboration)
- Need installation or upgrade help: [Guide Home](/guide/)
