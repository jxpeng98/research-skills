---
id: effect-size-interpreter
stage: F_writing
description: "Translate raw effect sizes into meaningful magnitude language using benchmarks, contextual comparisons, and practical significance framing."
inputs:
  - type: StatsReport
    description: "Model output with effect size estimates"
  - type: AnalysisPlan
    description: "Planned effect size metrics and benchmarks"
    required: false
outputs:
  - type: EffectSizeInterpretation
    artifact: "manuscript/effect_size_interpretation.md"
constraints:
  - "Must use domain-appropriate benchmarks, not generic Cohen's d thresholds"
  - "Must distinguish statistical from practical significance"
  - "Must contextualize magnitude by comparing to prior effect sizes in the field"
failure_modes:
  - "Using Cohen's 'small/medium/large' without field context"
  - "Ignoring confidence interval width when interpreting magnitude"
  - "Confusing standardized and unstandardized effect sizes"
tools: [filesystem]
tags: [writing, effect-size, interpretation, practical-significance, magnitude]
domain_aware: true
---

# Effect Size Interpreter Skill

Translate statistical effect sizes into language that communicates practical and theoretical importance, not just p-values.

## Purpose

Translate raw effect sizes into meaningful magnitude language using benchmarks, contextual comparisons, and practical significance framing.

## Related Task IDs

- `F3` (result interpretation — effect size component)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/effect_size_interpretation.md`

## When to Use

- After primary analysis is complete
- When writing the Results or Discussion section
- When a reviewer asks "is this effect meaningful?"

## Process

### Step 1: Identify the Effect Size Metric

| Design | Common Metrics | When to Use |
|--------|---------------|-------------|
| Group comparison (2 groups) | Cohen's d, Hedges' g | d for large N; g for small N correction |
| Group comparison (3+ groups) | η², partial η², ω² | η² overestimates; ω² preferred |
| Correlation | r, R² | r for bivariate; R² for explained variance |
| Regression | β (standardized), b (unstandardized) | β for cross-variable comparison; b for interpretable units |
| Binary outcome | OR (odds ratio), RR (risk ratio), NNT | OR for case-control; RR for cohorts; NNT for clinical impact |
| Meta-analysis | SMD (standardized mean difference) | Same as Hedges' g for most purposes |
| Qualitative | N/A — use evidence strength descriptors | See Step 4 |

### Step 2: Contextualize Magnitude

#### Generic Benchmarks (use with CAUTION)

| Metric | Small | Medium | Large | Source |
|--------|-------|--------|-------|--------|
| Cohen's d | 0.2 | 0.5 | 0.8 | Cohen (1988) |
| Hedges' g | 0.2 | 0.5 | 0.8 | Same scale |
| r | 0.10 | 0.30 | 0.50 | Cohen (1988) |
| R² | 0.02 | 0.13 | 0.26 | Cohen (1988) |
| η² | 0.01 | 0.06 | 0.14 | Cohen (1988) |
| OR | 1.5 | 2.5 | 4.3 | Chen et al. (2010) |

> **WARNING**: Cohen himself called these "crude" benchmarks. Reviewers increasingly reject uncontextualized use of "small/medium/large." ALWAYS supplement with field-specific context (Step 2b).

#### Field-Specific Contextualization (preferred)

| Strategy | How to Apply | Example |
|----------|------------|---------|
| **Compare to prior estimates** | Cite 3–5 similar studies' effect sizes | "Our d = 0.35 is consistent with prior estimates ranging from 0.20 to 0.45 (Smith 2020; Jones 2021)" |
| **Convert to concrete units** | Unstandardize if possible | "A 1-SD increase in remote work corresponds to 0.35 SD (or roughly 2.1 points) improvement in productivity" |
| **Benchmark against known effects** | Compare to well-understood effects in the domain | "This effect is comparable to the established relationship between job satisfaction and turnover intention (r = 0.30)" |
| **Calculate practical metrics** | NNT, BESD, probability of superiority | NNT = 10 means treating 10 patients to benefit one |
| **Use percentage change** | For business/policy audiences | "This corresponds to approximately a 7% increase in conversion rate" |

### Step 3: Report with Precision

#### Writing Template

```
The effect of [IV] on [DV] was [direction and adjective] (d = 0.35, 95% CI [0.18, 0.52]).
This estimate is [consistent with / larger than / smaller than] the range of prior
estimates in [domain] (d = 0.20–0.45; citations).
In practical terms, this corresponds to [unstandardized interpretation].
The confidence interval [does / does not] include [threshold of practical significance],
suggesting [practical significance assessment].
```

#### Reporting Checklist

| Element | Check |
|---------|-------|
| Effect size point estimate | ✅ Always report |
| 95% confidence interval | ✅ Always report — more informative than p-value |
| Metric identified | ✅ State whether d, g, r, β, OR, etc. |
| Direction stated | ✅ Positive/negative/null |
| Field comparison | ✅ Compare to 2+ prior studies' effects |
| Practical conversion | ✅ Unstandardize or convert to meaningful units |
| CI interpretation | ✅ Discuss width and what range of effects is plausible |

### Step 4: Special Cases

#### Null Results
Do NOT interpret as "no effect." Instead:
- Report the CI range: "The 95% CI [-0.05, 0.15] is consistent with effects ranging from negligibly negative to small positive"
- State which effect sizes the CI rules out: "We can rule out effects larger than d = 0.15"
- Compute power: "With N = 200, we had 80% power to detect d ≥ 0.28; our CI does not exclude smaller meaningful effects"

#### Very Small Effects
- In large samples, even d = 0.05 can be statistically significant
- Ask: "At what scale does this accumulate?" (e.g., 0.05 SD per student × millions of students)
- Report both statistical and practical thresholds

#### Non-Linear / Interaction Effects
- Report conditional effect sizes: "The effect of X on Y is d = 0.6 for high-Z participants but d = 0.1 for low-Z participants"
- Avoid reporting only the main effect when interactions are significant

## Quality Bar

The effect size interpretation is **ready** when:

- [ ] Every primary result has an effect size with 95% CI
- [ ] No result described only by p-value
- [ ] Field-specific context provided (not just generic Cohen's benchmarks)
- [ ] At least one practical/unstandardized interpretation offered
- [ ] Null results interpreted as precision statements, not as "no effect"
- [ ] CI width discussed (not just whether it excludes zero)

## Minimal Output Format

```markdown
# Effect Size Interpretation

## Primary Results

| Hypothesis | Effect | 95% CI | Metric | Field Benchmark | Practical Interpretation |
|-----------|--------|--------|--------|----------------|------------------------|
| H1a | d = 0.35 | [0.18, 0.52] | Hedges' g | Prior range: 0.20–0.45 | ~2.1 point increase on 30-point scale |
| H1b | β = 0.12 | [0.02, 0.22] | Standardized β | Prior range: 0.05–0.20 | ~3% increase per SD |

## Contextual Comparisons
- Compared to [known benchmark]: ...
- NNT / BESD / practical metric: ...

## Null Results (if any)
- CI rules out effects > [threshold]
- Power sufficient/insufficient for [effect size]

## Interpretation Summary
[2–3 sentence narrative integrating magnitude, precision, and practical importance]
```
