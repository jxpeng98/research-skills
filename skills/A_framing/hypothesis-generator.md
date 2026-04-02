---
id: hypothesis-generator
stage: A_framing
description: "Translate research questions into hypotheses, propositions, or sensitizing concepts with mechanisms, rival explanations, and boundary conditions."
inputs:
  - type: RQSet
    description: "Refined research questions from question-refiner"
  - type: TheoreticalFramework
    description: "Optional theoretical framework"
    required: false
outputs:
  - type: HypothesisSet
    artifact: "framing/hypothesis.md"
constraints:
  - "Each confirmatory hypothesis must specify direction and expected sign when applicable"
  - "Must include at least one rival explanation or alternative interpretation per key claim"
  - "Must distinguish exploratory from confirmatory hypotheses"
  - "Must link every hypothesis to an operationalizable test"
failure_modes:
  - "Research questions are exploratory but no propositions or sensitizing concepts are produced"
  - "Hypotheses are untestable (no clear DV/IV or evidence form specified)"
  - "Mechanisms are asserted without grounding in theory or prior evidence"
tools: [filesystem]
tags: [framing, hypothesis, propositions, mechanisms]
domain_aware: false
---

# Hypothesis Generator Skill

Translate research questions into testable hypotheses, qualitative working propositions, or sensitizing concepts with mechanisms and boundary conditions.

## Purpose

Translate research questions into hypotheses, propositions, or sensitizing concepts with mechanisms, rival explanations, and boundary conditions.

## Related Task IDs

- `A1_5` (hypothesis generation)

## Output (contract path)

- `RESEARCH/[topic]/framing/hypothesis.md`

## Inputs

- `framing/research_question.md`
- Optional: `theoretical_framework.md`

## When To Use

- After research questions are refined (A1) and before study design (C1)
- When you need to lock in testable predictions, not just questions
- When the theoretical framing implies directional expectations

## Process

### Step 1: Classify the Epistemological Mode

Before generating hypotheses, determine what kind of knowledge claim each RQ supports:

| RQ Type | Output Type | Format |
|---------|-------------|--------|
| Confirmatory / theory-testing | Directional hypothesis (H1a, H1b…) | "X is positively associated with Y" |
| Exploratory / theory-building | Working proposition (P1, P2…) | "We propose that X influences Y through mechanism M" |
| Interpretive / qualitative | Sensitizing concept | "We focus on how actors interpret X in context C" |
| Descriptive | No hypothesis needed | Document the descriptive target instead |

> **Anti-pattern**: Forcing confirmatory hypotheses on exploratory RQs. If the field has no prior basis for direction, use propositions with explicit "we do not predict direction" statements.

### Step 2: Generate Hypotheses Per RQ

For each research question, produce 1–3 hypotheses following this structure:

#### For Quantitative / Confirmatory Hypotheses

```
H[n][letter]: [IV/predictor] is [direction: positively/negatively/nonlinearly]
              associated with [DV/outcome]
              [conditional: when/among/for <moderator/context>].

Mechanism:    [Why — theoretical logic or prior evidence]
Sign:         [Expected: β > 0 / β < 0 / inverted-U / threshold]
Boundary:     [When it may NOT hold — context, population, or time limits]
Test:         [How to operationalize — model family, key coefficient, data source]
```

**Example**:

```
H1a: Remote work adoption is positively associated with individual productivity
     among knowledge workers in technology firms.

Mechanism:    Reduced commute time and interruptions increase deep work hours
              (Bloom et al., 2015; autonomy → intrinsic motivation, SDT).
Sign:         β > 0 for remote_hours on productivity_index.
Boundary:     May not hold for roles requiring physical collaboration
              or for workers with inadequate home infrastructure.
Test:         OLS with firm fixed effects; DV = quarterly_output_index;
              IV = remote_work_ratio; controls = tenure, role_type, team_size.
```

#### For Qualitative / Interpretive Propositions

```
P[n]: We propose that [actors/participants] [process verb: negotiate/construct/
      experience/interpret] [phenomenon] through [mechanism or practice]
      in [context/setting].

Focal process:   [What unfolding, sequence, or practice to trace]
Evidence form:   [Interview episodes / field observations / documents / cases]
Disconfirmation: [What evidence would challenge this proposition]
```

**Example**:

```
P1: We propose that platform managers negotiate governance tensions
    during AI policy rollout through iterative legitimation practices
    that shift between coercive and normative framing.

Focal process:   How managers frame policy changes to complementors over time
Evidence form:   Longitudinal interview sequences + policy document evolution
Disconfirmation: If managers use only one framing mode without adaptation
```

#### For Sensitizing Concepts (Grounded Theory / Ethnography)

```
Sensitizing concept: [concept label]
Orienting question:  [What aspect of experience/practice does this direct attention to?]
Expected evidence:   [Types of episodes, interactions, or artifacts to look for]
```

### Step 3: Add Rival Explanations

For each key hypothesis or proposition, add at least one rival:

| Hypothesis | Rival Explanation | Observable Difference | Test to Distinguish |
|------------|-------------------|----------------------|-------------------|
| H1a: Remote → productivity↑ | Selection: productive workers choose remote | Pre-remote baseline differs | Compare pre/post within-person |
| H1a: Remote → productivity↑ | Hawthorne: novelty effect only | Effect decays over time | Add time-since-adoption interaction |

> **Rule**: If you cannot articulate a rival, the hypothesis may be tautological or unfalsifiable. Revisit the mechanism.

### Step 4: Map Hypotheses to Design Requirements

Create a traceability table linking each hypothesis to what is needed downstream:

| Hypothesis | Required Data | Model / Analytic Procedure | Key Assumption | Links to |
|------------|--------------|---------------------------|----------------|----------|
| H1a | remote_ratio, output_index | OLS + FE | Exogeneity of remote adoption | C1 design, C3 analysis plan |
| P1 | Interview transcripts (3 waves) | Process coding → temporal bracketing | Theoretical saturation | C1 case boundaries, C2 interview guide |

### Step 5: Self-Critique

Before finalizing, evaluate each hypothesis against these criteria:

| Criterion | Question | Pass? |
|-----------|----------|-------|
| **Testable** | Can you specify what result would disconfirm it? | |
| **Grounded** | Does the mechanism cite theory or prior empirical evidence? | |
| **Bounded** | Are scope conditions explicit (who, when, where)? | |
| **Non-trivial** | Would a competent reviewer find the prediction non-obvious? | |
| **Independent** | Do hypotheses test distinct mechanisms (not just rewordings)? | |
| **Operational** | Can you name the specific variables, measures, or evidence forms? | |

## Quality Bar

A hypothesis set is **ready** when:

- [ ] Every confirmatory hypothesis has direction + mechanism + boundary + test
- [ ] Every exploratory proposition has process + evidence form + disconfirmation criteria
- [ ] At least one rival explanation per key claim
- [ ] Traceability table links to C1/C3 design requirements
- [ ] Self-critique checklist shows no failures
- [ ] Hypothesis numbering is stable (H1a, H1b, H2a… not renumbered later)

## Common Pitfalls

| Pitfall | Why It's a Problem | Fix |
|---------|--------------------|-----|
| "X affects Y" without direction | Unfalsifiable — any result confirms | Specify β > 0 or β < 0 |
| Too many hypotheses (>8) | Inflates family-wise error rate; reviewers suspect fishing | Consolidate to core 3–5 |
| Mechanisms without citations | Reviewer rejects as speculation | Ground in theory or cite 2+ priors |
| Missing moderators | Claim appears universal when it's conditional | Add boundary conditions explicitly |
| Restating RQ as hypothesis | No predictive content added | Hypothesis must add direction, mechanism, or scope |

## Minimal Output Format

```markdown
# Hypothesis Set

## Epistemological mode
[Confirmatory / Exploratory / Mixed]

## Hypotheses

### H1a: [statement]
- **Mechanism**: ...
- **Expected sign**: ...
- **Boundary conditions**: ...
- **Operationalization**: IV = ..., DV = ..., model = ...

### H1b: [statement]
...

## Rival Explanations

| Hypothesis | Rival | Observable difference | Distinguishing test |
|---|---|---|---|

## Traceability

| Hypothesis | Data needed | Model | Key assumption | Feeds into |
|---|---|---|---|---|

## Self-Critique Checklist
- [ ] All hypotheses testable/disconfirmable
- [ ] Mechanisms grounded in theory/evidence
- [ ] Scope conditions explicit
- [ ] Rival explanations documented
```

## When to Use

- 研究问题已经结构化（PICO/PEO），需要转化为可检验命题
- 理论框架已确定，需要推导 testable predictions
- Pre-registration 前需要锁定假设方向和备择
- 定性研究需要 propositions 而非假设时
