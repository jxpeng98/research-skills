---
id: variable-operationalizer
stage: C_design
description: "Map abstract constructs to concrete, measurable variables with validity/reliability justification."
inputs:
  - type: TheoreticalFramework
    description: "Constructs requiring operationalization"
  - type: DesignSpec
    description: "Study design context"
outputs:
  - type: OperationalizationMap
    artifact: "design/operationalization.md"
constraints:
  - "Each construct must map to at least one measurable variable"
  - "Must justify measurement validity (content, construct, criterion)"
  - "Must cite established scales when available"
failure_modes:
  - "No validated instrument exists for construct"
  - "Construct too abstract for single operationalization"
tools: [filesystem, scholarly-search]
tags: [design, operationalization, constructs, measurement, validity]
domain_aware: true
---

# Variable Operationalizer Skill

Bridge from abstract theoretical constructs to concrete, measurable variables — ensuring that what you measure actually captures what you theorize.

## Purpose

Map abstract constructs to concrete, measurable variables with validity/reliability justification.

## Related Task IDs

- `C1` (study design — operationalization step)

## Output (contract path)

- `RESEARCH/[topic]/design/operationalization.md`

## When to Use

- After theoretical framework is established
- When translating hypotheses into testable predictions
- When selecting or adapting measurement instruments
- When a reviewer challenges "how did you measure X?"

## Process

### Step 1: List All Constructs from the Theoretical Framework

For each construct, document:

| Field | What to Capture | Example |
|-------|----------------|---------|
| **Construct name** | Label from theory | "Self-efficacy" |
| **Theoretical definition** | How the theory defines it | "Belief in one's capability to execute behaviors necessary to produce specific outcomes" (Bandura, 1977) |
| **Dimensionality** | Unidimensional or multi-dimensional | Multi-dimensional: magnitude, strength, generality |
| **Role in model** | IV, DV, mediator, moderator, control | Mediator (between training → performance) |
| **Level of analysis** | Individual, team, organization, country | Individual |

### Step 2: Search for Validated Instruments

For each construct, conduct a measurement literature search:

| Search Strategy | Where to Look |
|----------------|---------------|
| **Seminal papers** | Original theory paper often proposes measures |
| **Psychometric validation studies** | Search "[construct] + scale + validation" |
| **Systematic reviews of measures** | Search "[construct] + measurement + systematic review" |
| **Meta-analyses** | Often report which measures are most commonly used |
| **Domain-specific databases** | PsycTESTS, PROMIS, HaPI, COSMIN database |

**Instrument Evaluation Criteria**:

| Criterion | What to Check | Minimum Threshold |
|-----------|--------------|-------------------|
| **Content validity** | Do items cover the construct domain? | Expert panel validation or systematic development |
| **Construct validity** | Factor structure matches theory? | CFA with acceptable fit (CFI ≥ .90, RMSEA ≤ .08) |
| **Criterion validity** | Correlates with known outcomes? | Significant correlations with expected criteria |
| **Internal consistency** | Items measure same construct? | Cronbach's α ≥ .70 (exploratory) or ≥ .80 (established) |
| **Test-retest reliability** | Stable over time? | ICC ≥ .70 for stable constructs |
| **Sensitivity to change** | Detects meaningful change? | Important for intervention studies |
| **Cross-cultural validation** | Validated in your population? | Translation + validation study in target language/culture |
| **Length and burden** | Feasible for participants? | Consider survey fatigue, completion time |

### Step 3: Map Construct → Variable → Instrument → Item

Create the full traceability chain:

```markdown
## Construct: Self-Efficacy

### Theoretical Definition
Bandura (1977): "Belief in one's capability to execute behaviors
necessary to produce specific outcomes"

### Operationalization Decision

| Option | Instrument | Items | Reliability | Pros | Cons |
|--------|-----------|-------|-------------|------|------|
| Option A | General Self-Efficacy Scale (GSE-10) | 10 items, 4-pt Likert | α = .87 (meta) | Widely validated; short; 33 languages | General, not domain-specific |
| Option B | New General Self-Efficacy Scale (NGSE) | 8 items, 5-pt Likert | α = .86 | Better discrimination | Less widely used |
| Option C | Custom domain-specific | TBD | TBD | Tailored to context | No prior validation |

### Selected: GSE-10 (Schwarzer & Jerusalem, 1995)
**Justification**: Most widely validated; available in target language;
acceptable reliability; used in 5 prior studies in our domain.

### Variable Specification
| Variable | Items | Response | Scoring | Expected Range |
|----------|-------|----------|---------|---------------|
| `gse_total` | Mean of 10 items | 1–4 Likert | Mean score | 1.0–4.0 |
| `gse_item_1` ... `gse_item_10` | Individual items | 1–4 | Direct | 1–4 |
```

### Step 4: Handle Constructs Without Validated Instruments

When no validated instrument exists:

| Strategy | When to Use | What to Document |
|----------|------------|-----------------|
| **Adapt existing scale** | Close instrument exists in adjacent domain | Original instrument, adaptations made, pilot results |
| **Develop new items** | Novel construct | Development methodology (DeVellis, 2017), expert review, cognitive interviews |
| **Use proxy measures** | Construct can't be directly measured | Why proxy is acceptable, correlation evidence, limitations |
| **Behavioral indicators** | Construct manifests in observable behavior | Operationally define the behavior, inter-rater reliability |
| **Archival/administrative** | Data exists in records | How record captures construct, validity argument |

> **Flag**: Any construct operationalized with an un-validated instrument must be clearly noted as a measurement limitation.

### Step 5: Address Measurement Validity Threats

| Threat | What It Means | How to Mitigate |
|--------|-------------- |-----------------|
| **Common method bias** | All measures from same source/method | Multi-source data, temporal separation, marker variable test |
| **Social desirability** | Respondents give "correct" answers | Anonymity assurance, indirect measures, SDS scale |
| **Recall bias** | Retrospective self-report is unreliable | Limit recall period, use behavioral anchors, diary methods |
| **Response set** | Acquiescence, extreme responding | Mix item direction, use reverse-coded items, check response patterns |
| **Translation inequivalence** | Translated items don't carry same meaning | Back-translation, committee method, cognitive debriefing |
| **Floor/ceiling effects** | Measure can't discriminate at extremes | Check distributions; consider adaptive items (IRT) |

### Step 6: Document Adaptation Decisions

When adapting an existing instrument:

```markdown
## Adaptation Log: [Instrument Name]

### Original
- Author: [citation]
- Language: [original language]
- Population: [original validation population]

### Adaptations Made
| Item # | Original | Adapted | Reason |
|--------|----------|---------|--------|
| 5 | "At my workplace..." | "In my research lab..." | Context-specific wording |
| 8 | [removed] | — | Not relevant to population |

### Validation Steps Taken
1. Expert panel review (n = 3 domain experts)
2. Cognitive interviewing (n = 5 pilot participants)
3. Pilot test (n = 30): α = .84, CFA fit: CFI = .93, RMSEA = .06
```

## Quality Bar

Operationalization is **ready** when:

- [ ] Every construct has a defined operationalization with traceability
- [ ] Validated instruments preferred; adaptations documented with justification
- [ ] For each instrument: reliability evidence (α or ICC) and validity evidence (content, construct, or criterion)
- [ ] Un-validated measures are flagged as limitations
- [ ] Common measurement threats identified with mitigations
- [ ] Adaptation/translation procedures documented if applicable
- [ ] Cross-reference with `data-dictionary-builder` is consistent

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Using a scale without checking validation in your population | May not be valid cross-culturally | Check for validation studies in target culture/language |
| Mixing response scales across instruments | Confuses participants | Standardize where possible; at minimum, document differences |
| Only reporting α for reliability | α is not sufficient for all decisions | Report appropriate reliability (α for internal consistency, ICC for inter-rater, test-retest for stability) |
| No justification for measure selection | Reviewer asks "why this scale?" | Compare options in a decision table |
| Proxy measure without acknowledging limitations | Overstating measurement precision | Explicitly label proxies and discuss attenuation |
| Single-item measures for complex constructs | Low reliability, construct underrepresentation | Use multi-item scales unless strong justification |

## Minimal Output Format

```markdown
# Operationalization Map

## Construct → Variable Mapping

| Construct | Role | Instrument | Items | Reliability | Validity | Variable |
|-----------|------|-----------|-------|-------------|----------|----------|
| Self-efficacy | Mediator | GSE-10 | 10, 4-pt Likert | α = .87 | Construct (CFA) | gse_total |
| Job satisfaction | DV | MSQ-20 | 20, 5-pt Likert | α = .91 | Criterion | jsq_total |
| Team size | Moderator | HR records | — | — | Face | team_size |

## Measurement Decisions

### [Construct 1]
- Options considered: [table]
- Selected: [instrument], Justification: [reason]
- Adaptations: [if any]

## Measurement Limitations
- [Construct X] uses unvalidated proxy measure
- Common method bias: all self-report data
```
