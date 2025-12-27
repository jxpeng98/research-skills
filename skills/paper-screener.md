# Paper Screener Skill

Apply systematic screening criteria to filter papers for inclusion in a literature review.

## Purpose

Implement a two-stage screening process:
1. Title/Abstract screening
2. Full-text screening

Following PRISMA guidelines for transparent, reproducible screening.

## Process

### Stage 1: Title/Abstract Screening

For each paper, evaluate against inclusion/exclusion criteria based on title and abstract only.

#### Screening Checklist

```markdown
Paper: [Title]

## Inclusion Criteria Check
- [ ] Criterion 1: [Description] → Met/Not Met/Unclear
- [ ] Criterion 2: [Description] → Met/Not Met/Unclear
- [ ] Criterion 3: [Description] → Met/Not Met/Unclear

## Exclusion Criteria Check
- [ ] Exclusion 1: [Description] → Yes/No/Unclear
- [ ] Exclusion 2: [Description] → Yes/No/Unclear

## Decision
- [ ] INCLUDE - Proceed to full-text
- [ ] EXCLUDE - Reason: [...]
- [ ] UNCERTAIN - Need full-text to decide
```

#### Common Inclusion Criteria

| Criterion | Description |
|-----------|-------------|
| Topic Relevance | Addresses the research question |
| Population | Correct population/context |
| Study Type | Matches required study types |
| Language | Published in target language |
| Date Range | Within specified time period |
| Peer Review | Published in peer-reviewed venue |

#### Common Exclusion Criteria

| Criterion | Description |
|-----------|-------------|
| Off-topic | Does not address research question |
| Wrong population | Different population/context |
| Study type | Reviews, editorials, opinion pieces |
| Duplicate | Same study reported elsewhere |
| No access | Full text unavailable |
| Language | Not in accessible language |

### Stage 2: Full-Text Screening

For papers passing Stage 1, retrieve and review full text.

#### Full-Text Screening Checklist

```markdown
Paper: [Title]
Full-text source: [URL/DOI]

## Detailed Criteria Assessment

### Inclusion Criteria
1. [Criterion 1]: 
   - Evidence from text: "[quote]"
   - Assessment: Met/Not Met
   
2. [Criterion 2]:
   - Evidence from text: "[quote]"
   - Assessment: Met/Not Met

### Exclusion Criteria
1. [Exclusion 1]:
   - Evidence from text: "[quote]"
   - Assessment: Applies/Does not apply

## Final Decision
- [ ] INCLUDE - All criteria met
- [ ] EXCLUDE - Reason: [specific criterion failed]

## Notes
[Any relevant observations]
```

### Output: Screening Log

```markdown
# Screening Log: [Review Topic]

## Screening Criteria

### Inclusion Criteria
1. [IC1]
2. [IC2]
3. [IC3]

### Exclusion Criteria
1. [EC1]
2. [EC2]

## Stage 1: Title/Abstract Screening

| ID | Title | Year | Include | Exclude Reason |
|----|-------|------|---------|----------------|
| 1 | [Title] | 2023 | ✓ | |
| 2 | [Title] | 2022 | ✗ | Off-topic |
| 3 | [Title] | 2023 | ? | Need full-text |

**Summary:**
- Total screened: N
- Included: X
- Excluded: Y
- Uncertain: Z

## Stage 2: Full-Text Screening

| ID | Title | Year | Include | Exclude Reason |
|----|-------|------|---------|----------------|
| 1 | [Title] | 2023 | ✓ | |
| 3 | [Title] | 2023 | ✗ | Wrong population |

**Summary:**
- Total assessed: N
- Included: X
- Excluded: Y (with reasons)

## Exclusion Reasons Summary

| Reason | Count |
|--------|-------|
| Off-topic | X |
| Wrong population | Y |
| Wrong study type | Z |
| Duplicate | W |
```

### PRISMA Flow Diagram Data

After screening, generate data for PRISMA flowchart:

```markdown
## PRISMA Flow Data

Records identified: [N]
- Semantic Scholar: [X]
- arXiv: [Y]
- Other sources: [Z]

Records after deduplication: [N1]
Records screened: [N1]
Records excluded (title/abstract): [N2]
Reports sought for retrieval: [N3]
Reports not retrieved: [N4]
Reports assessed for eligibility: [N5]
Reports excluded (full-text): [N6]
  - Reason 1: [count]
  - Reason 2: [count]
Studies included in review: [N7]
```

## Usage

This skill is called by:
- `/lit-review` - During screening phase
