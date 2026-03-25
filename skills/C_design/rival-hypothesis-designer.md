---
id: rival-hypothesis-designer
stage: C_design
description: "Construct competing theories and rival explanations to strengthen design by pre-empting reviewer objections."
inputs:
  - type: DesignSpec
    description: "Study design and identification strategy"
  - type: HypothesisSet
    description: "Primary hypotheses or propositions"
outputs:
  - type: RivalHypotheses
    artifact: "design/rival_hypotheses.md"
constraints:
  - "Must produce at least 3 plausible rival explanations for the main effect"
  - "Must specify observable implications that distinguish the focal from each rival"
  - "Must link to design controls or empirical tests"
failure_modes:
  - "Rivals are straw men (no one would actually propose them)"
  - "Rivals cannot be distinguished from the focal hypothesis with available data"
  - "Only statistical threats listed; no substantive theoretical alternatives"
tools: [filesystem]
tags: [design, rival-hypotheses, threats-to-validity, competing-theories]
domain_aware: false
---

# Rival Hypothesis Designer Skill

Proactively construct the alternative explanations reviewers will raise — and design tests to address them.

## Related Task IDs

- `C1_5` (rival hypothesis design)

## Output (contract path)

- `RESEARCH/[topic]/design/rival_hypotheses.md`

## When to Use

- After hypotheses (A1_5) and study design (C1) are drafted
- Before locking the analysis plan (C3) — rivals may require additional data or controls
- When the identification strategy has known weaknesses
- For qualitative research: before data collection, to plan disconfirming-case search

## Process

### Step 1: Identify Rival Sources

Rivals come from four sources — check all four:

| Source | What to Ask | Example |
|--------|-------------|---------|
| **Alternative theories** | What other theoretical framework predicts the same pattern? | SDT vs expectancy theory both predict motivation effects |
| **Confounders** | What unmeasured variable produces a spurious association? | Firm size drives both digital adoption and performance |
| **Reverse causality** | Could the DV cause the IV instead? | Productive workers choose remote work (not reverse) |
| **Methodological artifacts** | Could the pattern be an artifact of measurement, timing, or selection? | Common-method bias in single-source surveys |

For qualitative research, add:
| Source | What to Ask | Example |
|--------|-------------|---------|
| **Alternative framings** | What other interpretive lens would organize the same data differently? | Power lens vs sensemaking lens on the same governance episodes |
| **Informant bias** | Do participants have reasons to present a particular narrative? | Managers emphasize their rationality; employees emphasize constraints |

### Step 2: Construct Each Rival in Detail

For each rival, specify all four elements:

```
Rival R[n]: [name]

Mechanism:       Why this rival would produce the same observed pattern
Observable       What data pattern would look different IF this rival
Implication:     is the true explanation (not the focal hypothesis)
Design Control:  How the study design already addresses this
                 (control variable, fixed effect, matching, case selection)
Empirical Test:  What additional test could distinguish focal from rival
                 (interaction term, instrumental variable, disconfirming case)
```

**Example — Quantitative**:

```
Rival R1: Selection (productive workers self-select into remote work)

Mechanism:       High-performers negotiate remote arrangements, creating
                 selection bias in cross-sectional estimates
Observable       Pre-remote productivity should differ between treatment
Implication:     and control groups (if selection is driving the result)
Design Control:  Individual fixed effects absorb time-invariant ability
Empirical Test:  Event study around remote policy change (exogenous shock);
                 test for parallel pre-trends
```

**Example — Qualitative**:

```
Rival R1: Retrospective rationalization (managers construct coherent
          narratives post-hoc rather than reporting actual process)

Mechanism:       Interview accounts reflect sensemaking norms rather than
                 real-time governance practices
Observable       Real-time observations would show messier, less coherent
Implication:     governance practices than interviews suggest
Design Control:  Triangulate with meeting minutes and contemporaneous documents
Empirical Test:  Compare interview accounts with archival records of the
                 same events; code for consistency/discrepancy
```

### Step 3: Build the Rival Comparison Matrix

| ID | Rival | Mechanism | Observable Difference | Design Control | Empirical Test | Status |
|----|-------|-----------|----------------------|----------------|----------------|--------|
| R1 | Selection | Self-selection into treatment | Pre-trends differ | FE | Event study | Planned |
| R2 | Reverse causality | DV drives IV | Temporal sequence reversed | Lagged IV | Granger test | Planned |
| R3 | Omitted variable (firm size) | Spurious correlation via firm size | Effect vanishes when controlling | Control variable | Oster (2019) δ | Planned |
| R4 | Hawthorne effect | Novelty, not treatment | Effect decays | Time interaction | Cohort-by-time analysis | Nice-to-have |

### Step 4: Assess Rival Quality

Evaluate each rival against these criteria:

| Criterion | Question | Good Rival | Bad Rival |
|-----------|----------|-----------|-----------|
| **Plausible** | Would a reasonable reviewer propose this? | "Selection effects are common in observational studies" | "Aliens influenced the data" |
| **Distinguishable** | Can you design a test to tell them apart? | Pre-trend test distinguishes selection from treatment | Both predict identical data patterns |
| **Non-trivial** | Does it challenge the core claim, not a peripheral detail? | Challenges the identification strategy | Questions a control variable's coding |
| **Grounded** | Is there theory or evidence behind the rival? | Cites prior work showing this rival matters | Pure speculation |

> **Anti-pattern**: Listing only statistical threats (heteroskedasticity, non-normality) as "rivals." These are estimation concerns, not rival explanations. Rivals must offer a **different substantive explanation** for the pattern.

### Step 5: Feed Downstream

- Each rival with a planned test → add to `C3` analysis plan and `C3_5` robustness plan
- Each rival with a design control → verify the control is in `C1` study design
- Each rival without a test → acknowledge explicitly as a limitation in `F3` discussion

## Quality Bar

The rival hypothesis set is **ready** when:

- [ ] At least 3 substantive rivals (not just statistical estimation concerns)
- [ ] Each rival has all 4 elements: mechanism, observable implication, design control, empirical test
- [ ] At least 2 rivals are empirically distinguishable with available data
- [ ] Rivals span different sources (not all "omitted variable" variants)
- [ ] Traceability downstream: each test appears in C3 or C3_5; unresolvable rivals appear in planned limitations

## Minimal Output Format

```markdown
# Rival Hypotheses

## Focal Claim
[Primary hypothesis or proposition being defended]

## Rival Comparison Matrix

| ID | Rival | Mechanism | Observable Difference | Design Control | Empirical Test | Status |
|----|-------|-----------|----------------------|----------------|----------------|--------|

## Detailed Rival Specifications

### R1: [Name]
- **Mechanism**: ...
- **Observable implication**: ...
- **Design control**: ...
- **Empirical test**: ...

### R2: [Name]
...

## Assessment
- Rivals addressed by design: R1, R3
- Rivals testable empirically: R2, R4
- Rivals acknowledged as limitations: R5

## Downstream Links
- Analysis plan additions: [tests to add to C3]
- Robustness plan additions: [checks to add to C3_5]
- Planned limitations: [rivals that cannot be resolved]
```
