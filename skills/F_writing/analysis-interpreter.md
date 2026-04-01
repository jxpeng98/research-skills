---
id: analysis-interpreter
stage: F_writing
description: "Translate quantitative, qualitative, or synthesized findings into analytical narratives that preserve uncertainty, surface mechanisms, and narrow claims to defensible scope conditions."
inputs:
  - type: StatsReport
    description: "Model results, diagnostics, and robustness checks"
    required: false
  - type: EvidenceTable
    description: "Coded qualitative evidence, case summaries, fieldnotes, or synthesis matrices"
    required: false
  - type: AnalysisPlan
    description: "Pre-specified estimands and decision rules"
    required: false
  - type: RobustnessPlan
    description: "Planned robustness checks and threats"
    required: false
outputs:
  - type: ResultInterpretation
    artifact: "manuscript/results_interpretation.md"
constraints:
  - "Must distinguish descriptive findings from causal interpretation"
  - "Must note uncertainty, assumptions, and failed robustness checks"
  - "Must separate observation, interpretation, and implication"
  - "Must surface mechanism candidates, rival explanations, and boundary conditions when evidence permits"
  - "Must keep first-order evidence separate from researcher interpretation"
  - "Must avoid re-litigating the entire methods section"
failure_modes:
  - "Narrative overclaims beyond the estimator and identification strategy"
  - "Null or imprecise findings are reframed as support without justification"
  - "Themes, cases, or quotes are restated without analytic interpretation"
tools: [filesystem]
tags: [writing, results, interpretation, robustness, uncertainty]
domain_aware: true
---

# Analysis Interpreter Skill

Turn statistical output, qualitative evidence, or synthesis results into manuscript-ready interpretation that is analytically deep, honestly uncertain, and defensible under review.

## Purpose

Translate quantitative, qualitative, or synthesized findings into analytical narratives that preserve uncertainty, surface mechanisms, and narrow claims to defensible scope conditions.

## Related Task IDs

- `F3` (full manuscript draft — results interpretation component)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/results_interpretation.md`

## When to Use

- After primary analysis is complete
- When writing the Results or Discussion section
- When a reviewer says "the interpretation is superficial" or "overclaimed"

## Process

### Step 1: Read the Result Pattern

Before interpreting, fully inventory what the analysis produced:

#### For Quantitative Results

| Element | What to Record | Where It Matters |
|---------|---------------|-----------------|
| **Primary estimate** | Coefficient, direction, magnitude | Core finding |
| **Precision** | 95% CI width, standard error | How confident are we? |
| **Statistical significance** | p-value, but NEVER as sole indicator | Gateway, not conclusion |
| **Effect size** | See `effect-size-interpreter` for contextualization | Practical importance |
| **Diagnostics** | Model fit (R², AIC), residual plots, VIF | Does the model hold? |
| **Robustness** | Which sensitivity checks passed/failed | How fragile is the result? |
| **Null findings** | Which hypotheses were NOT supported | Often more interesting than confirmations |
| **Heterogeneity** | Subgroup differences, interaction effects | Boundary conditions |
| **Failed assumptions** | Non-normality, heteroskedasticity, endogeneity | Qualification needed? |

#### For Qualitative Results

| Element | What to Record | Where It Matters |
|---------|---------------|-----------------|
| **Core themes/categories** | Labels, definitions, relationships | Organizing structure |
| **Theme prevalence** | How many cases/participants exhibited each theme | Not frequency counting — but evidence breadth |
| **Focal episodes/quotes** | Specific instances that exemplify the theme | Evidence grounding |
| **Deviant/negative cases** | Cases that contradict the dominant pattern | Credibility + boundary conditions |
| **Process/temporal patterns** | How phenomena unfold over time | For process research |
| **Cross-case variation** | How themes differ across cases/contexts | Transferability |
| **Evidence strength by source** | Interview vs document vs observation convergence | Triangulation |
| **Coding reliability** | Inter-rater agreement, codebook evolution | Dependability signal |

### Step 2: Climb the Interpretive Depth Ladder

For each major finding, move through five levels — stopping at the level your evidence supports:

```
Level 1 — DESCRIPTION:     What happened? What pattern emerged?
                           "We observe that X is positively associated with Y"

Level 2 — MECHANISM:       Why might it have happened? What process connects X → Y?
                           "This is consistent with [theory], which posits that M mediates..."

Level 3 — RIVAL:           What else could explain it? What alternative stories fit the data?
                           "However, this pattern could also reflect [rival], because..."

Level 4 — BOUNDARY:        When or where does the claim narrow or break?
                           "This relationship holds for [context] but may not extend to [other context]"

Level 5 — IMPLICATION:     Why does it matter? For theory, practice, or method?
                           "This finding suggests that [theory] needs revision in [specific way]"
```

> **Rule**: Every results paragraph must reach at least Level 2. If you cannot get past Level 1, the paragraph is a methods-section orphan, not a result.

> **Anti-pattern**: Jumping directly from Level 1 to Level 5 (description → implication) without mechanism or boundary. This produces shallow "interesting finding → companies should…" writing that reviewers reject.

### Step 3: Write at the Right Level for Each Finding Type

#### Reporting Confirmed Hypotheses

```markdown
Template:
"Hypothesis H[n] predicted that [IV] would be [direction] associated with [DV].
Consistent with this prediction, the coefficient was [β = X, 95% CI [L, U], p = Y],
corresponding to [practical interpretation: see effect-size-interpreter].
This result aligns with [theory/prior work], which suggests that [mechanism].
However, we note that [alternative explanation] cannot be fully ruled out given
[design limitation]."
```

**Example**:
```
H1a predicted that remote work adoption would be positively associated with
individual productivity. Consistent with this prediction, the estimated
coefficient was β = 0.31 (95% CI [0.14, 0.48], p < .001), suggesting that
a one-standard-deviation increase in remote work ratio is associated with
approximately a 0.31 SD increase in quarterly output index. This aligns with
autonomy theory (Deci & Ryan, 2000), which posits that reduced interruptions
and increased schedule control enhance deep work. However, selection effects
cannot be fully excluded: higher-performing employees may negotiate remote
arrangements more successfully (see robustness check R1).
```

#### Reporting Null Results

```markdown
Template:
"Contrary to H[n], we did not find a statistically significant association
between [IV] and [DV] (β = X, 95% CI [L, U], p = Y). The confidence interval
is consistent with effects ranging from [L interpretation] to [U interpretation],
meaning we cannot rule out [substantively meaningful effects if CI is wide].
With our sample of N = [n], we had [power]% power to detect effects of d ≥ [MDE],
so effects smaller than this threshold remain plausible."
```

> **Never write**: "There was no effect of X on Y." This conflates **absence of evidence** with **evidence of absence**. The CI tells you what effects are plausible.

#### Reporting Qualitative Findings

```markdown
Template:
"A central finding was the theme of [label] ([n/N] participants / [n/N] cases),
which captures [definition and analytic interpretation, not just label].
This theme was most pronounced among [subgroup/context] and manifested through
[specific practice/process]. A representative instance is [informant/case]:
'[quote]' ([ID], [context]).

However, this pattern was not universal. In [n] cases, we observed [deviant
pattern], which suggests [boundary condition or alternative explanation].
[Informant/case] described: '[counter-quote]' ([ID]).

Theoretically, this resonates with [framework]'s concept of [specific construct],
particularly the idea that [mechanism]. It extends prior understanding by showing
that [novel contribution: process variant / new mechanism / boundary condition]."
```

> **Anti-pattern**: "Theme 1 is X. Participant A said '...' Participant B said '...' Participant C said '...'" — this is quote dumping, not analysis. Every quote must serve an analytic purpose.

#### Reporting Robustness Results

```markdown
Template:
"To assess the sensitivity of our results, we conducted [n] robustness checks
(see Table [n] in the Appendix). The primary estimate remained [stable/changed]
across specifications:
- [Check 1: result] — addresses [threat]
- [Check 2: result] — addresses [threat]
When [most challenging check], the coefficient [changed direction / reduced to
non-significance / remained stable], suggesting [interpretation of fragility or robustness]."
```

### Step 4: Handle Special Interpretation Challenges

| Challenge | How to Handle |
|-----------|---------------|
| **Contradictory results** | Report both; discuss when they diverge and why; do not bury the inconvenient finding |
| **Marginal significance (p ≈ .05)** | Report exact p-value; emphasize CI and effect size; do not spin as "trending" |
| **Unexpected findings** | Label as exploratory; propose mechanism; add to future work; do not treat as confirmed |
| **Suppressor effects** | When adding a control flips a sign — explain the mechanism transparently |
| **Strong theory, weak evidence** | Narrow the claim; discuss possible reasons (measurement, power, context) |
| **Strong evidence, no theory** | Present as empirical regularity; call for theoretical development |
| **Qualitative: single compelling case** | Use as "paradigmatic" or "revelatory" case — but note it is one instance |
| **Qualitative: informant disagreement** | Report the disagreement as a finding — it reveals tensions, not errors |

### Step 5: Flag Limitations at Point of Interpretation

Don't defer all limitations to a separate section. At the point where a claim is made, note:

| Limitation Type | Where to Mention |
|----------------|-----------------|
| Identification / causal caveat | Immediately after causal-sounding claims |
| External validity | When discussing implications for other contexts |
| Measurement validity | When reporting on specific variables |
| Missing data impact | When reporting affected analyses |
| Single-source / common-method | When correlations are discussed |
| Reflexivity (qualitative) | When interpretation could be shaped by researcher positionality |

### Step 6: Write the Discussion Bridge

Connect results to the broader conversation:

```
For each key finding:
1. State the finding (1 sentence — reference the result, don't re-argue)
2. Compare to prior literature (agreements + disagreements)
3. Explain WHY it agrees/disagrees (mechanism, context, method differences)
4. State the theoretical implication (how does this update our understanding?)
5. State one practical implication (if warranted — do not overclaim)
6. Identify the boundary condition (when might this NOT hold?)
```

## Quality Bar

The results interpretation is **ready** when:

- [ ] Every finding reaches at least Level 2 on the interpretive depth ladder
- [ ] Confirmed hypotheses have: estimate + CI + effect size + mechanism + rival caveat
- [ ] Null results are reported as precision statements, not as "no effect"
- [ ] Qualitative themes are analytically interpreted, not just illustrated with quotes
- [ ] Deviant/negative cases are reported and interpreted
- [ ] Robustness results are summarized with threat → check → result structure
- [ ] Limitations are mentioned at the point of interpretation, not all deferred
- [ ] No claim exceeds the identification strategy (correlational design ≠ causal language)

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| "Results show that X causes Y" with observational data | Overclaiming causation | Use "is associated with" / "predicts" |
| Quoting Cohen's benchmarks without field context | Generic interpretation | Use `effect-size-interpreter` for contextualization |
| All limitations in one paragraph at the end | Looks perfunctory | Mention at point of interpretation + collect in section |
| Qualitative: quote dumping without analysis | Weak analytical contribution | Every quote must serve an analytic purpose (explain the quote after presenting it) |
| Hiding null results in supplementary | Reviewer will find them | Report null results in the main text |
| "Future research should…" without foundation | Vague; reviewer sees it as filler | Each "future" direction should emerge from a specific limitation of this study |

## Minimal Output Format

```markdown
# Results Interpretation

## Finding 1: [H1a / Theme 1 / Primary Result]

### Description (Level 1)
[What the data show — estimate, CI, pattern]

### Mechanism (Level 2)
[Why this pattern — theory, prior evidence]

### Rival Explanations (Level 3)
[Alternative interpretations — with evidence for/against]

### Boundary Conditions (Level 4)
[When this may not hold — moderators, context, time]

### Implications (Level 5)
[Theoretical + practical, calibrated to evidence strength]

## Finding 2: ...

## Null / Unexpected Results
[Report with precision framing and power context]

## Robustness Summary
| Check | Threat Addressed | Result | Primary Conclusion Changes? |
|-------|-----------------|--------|----------------------------|

## Cross-Finding Synthesis
[How findings relate to each other — do they tell a coherent story?]
```
