---
id: fatal-flaw-detector
stage: H_submission
description: "Constructive desk-reject analysis identifying critical flaws that would prevent publication."
inputs:
  - type: Manuscript
    description: "Draft manuscript"
outputs:
  - type: FatalFlawAnalysis
    artifact: "revision/fatal_flaw_analysis.md"
constraints:
  - "Must categorize flaws by severity (fatal/major/minor)"
  - "Must propose specific remediation for each flaw"
  - "Must test from the reviewer's perspective, not the author's"
failure_modes:
  - "False positive on methodological choices that are defensible"
  - "Missing domain-specific fatal flaws"
  - "Focusing on writing issues while missing structural problems"
tools: [filesystem]
tags: [submission, fatal-flaw, desk-reject, quality-gate, pre-submission]
domain_aware: true
---

# Fatal Flaw Detector Skill

Pre-submission desk-reject analysis: identify flaws likely to cause immediate rejection and propose mitigations before a reviewer sees them.

## Related Task IDs

- `H4` (fatal flaw analysis)

## Output (contract path)

- `RESEARCH/[topic]/revision/fatal_flaw_analysis.md`

## When to Use

- Before final submission (last quality gate)
- After major revisions (verify new flaws weren't introduced)
- When an author is unsure whether a paper is "ready"

## Process

### Step 1: Run the Fatal Flaw Checklist

Check each category systematically. A single "fatal" finding means the paper should NOT be submitted until it is fixed.

#### Category 1: Scope & Positioning

| Flaw | What to Check | Fatal? |
|------|--------------|--------|
| Out of scope | Does the RQ/topic match the venue's published scope? | Yes — desk reject |
| No clear contribution | Can you state what's new in 1 sentence by end of page 2? | Yes — "so what?" rejection |
| Contribution overclaimed | Does the paper claim "first" or "prove" without support? | Major |
| Not positioned against latest work | Missing citations from last 2–3 years in the field? | Major |
| Wrong paper type for venue | Submitting a methods paper to a theory journal? | Yes — desk reject |

#### Category 2: Method & Design

| Flaw | What to Check | Fatal? |
|------|--------------|--------|
| RQ-method mismatch | Does the method answer the stated RQ? (e.g., correlational method for causal claim) | Yes |
| Identification strategy absent | For causal claims: where is the exogenous variation? | Yes (for quant causal papers) |
| Sample too small, no power justification | Is there a power analysis or sample size rationale? | Major |
| Data quality undocumented | Missing data, selection into sample, measurement validity? | Major |
| Validity threats unaddressed | Known threats mentioned but not mitigated? | Major to Fatal |
| Ethics/IRB not mentioned | Human subjects without ethics clearance? | Yes — desk reject at many venues |

#### Category 3: Results & Analysis

| Flaw | What to Check | Fatal? |
|------|--------------|--------|
| Claims exceed evidence | Discussion claims stronger than Results support? | Major to Fatal |
| P-hacking signals | Many analyses, only significant ones reported? | Major |
| No effect sizes | Only p-values reported? | Major (increasingly required) |
| No robustness checks | Single specification, no sensitivity analysis? | Major |
| Contradictory results not discussed | Null or opposite findings hidden? | Fatal — reviewers will find them |

#### Category 4: Writing & Structure

| Flaw | What to Check | Fatal? |
|------|--------------|--------|
| Abstract doesn't match results | Abstract says "significant" but results are marginal? | Major |
| Introduction >5 pages | Rambling intro that doesn't reach the RQ quickly? | Major |
| Missing sections | No limitation section? No data availability? | Major |
| Figures/tables not self-explanatory | Can you understand the table without reading the text? | Minor |
| Inconsistent terminology | Same concept called different names? | Minor |

#### Category 5: Formatting & Compliance

| Flaw | What to Check | Fatal? |
|------|--------------|--------|
| Over word/page limit | Exceeds by >10%? | Yes — administrative desk reject |
| Author info in blind submission | Names in headers, tracked changes metadata, or self-citations identifying authors? | Yes — desk reject |
| Wrong reference format | APA when venue requires Vancouver? | Minor but sloppy → bad first impression |
| Reporting checklist not included | Venue requires CONSORT/STROBE/PRISMA? | Major to Fatal |
| Supplementary materials missing | Referenced in text but not included? | Major |

#### Category 6: Reproducibility

| Flaw | What to Check | Fatal? |
|------|--------------|--------|
| No data availability statement | Required by venue? | Major |
| Code not shared | Claims computational contribution but no code? | Major |
| Results not reproducible from methods | Could another researcher replicate from what's written? | Fatal (in principle; hard to catch pre-submission) |

### Step 2: Simulate Reviewer Objections

For each potential flaw, think like a hostile-but-fair reviewer:

```
If I were reviewing this paper and skeptical of the contribution, what would I attack first?
```

Common reviewer attack patterns:
1. "Why should I care?" → Gap not established or contribution unclear
2. "This has been done before" → Literature review incomplete
3. "This doesn't prove what you claim" → Method-claim mismatch
4. "But what about X?" → Omitted variable / rival explanation
5. "The sample is too [small/biased/specific]" → External validity
6. "I'm not convinced by the data" → Measurement, missing data, outliers

### Step 3: Classify and Prioritize

| Severity | Definition | Action |
|----------|-----------|--------|
| **Fatal** | Would cause desk reject or guaranteed rejection | Must fix before submission |
| **Major** | Likely to cause "reject" recommendation from ≥1 reviewer | Should fix before submission |
| **Minor** | Would be mentioned in review but not fatal | Fix if time allows |

### Step 4: Propose Specific Remediation

For each flaw, specify a concrete fix:

```
Flaw:        Claims exceed evidence (Major)
Location:    Discussion § 5.2, paragraph 3
Evidence:    "Our results prove that remote work increases productivity"
             but the design is observational (no causal identification)
Fix:         Replace "prove" with "our results are consistent with";
             add "we cannot rule out [specific alternative]" caveat
Effort:      Low (wording change)
```

## Quality Bar

The fatal flaw analysis is **ready** when:

- [ ] All 6 categories checked systematically
- [ ] Every finding classified as Fatal / Major / Minor
- [ ] Fatal and major findings have specific remediation with location
- [ ] At least 3 potential reviewer objections simulated
- [ ] Overall recommendation: Submit / Fix-then-submit / Do-not-submit

## Minimal Output Format

```markdown
# Fatal Flaw Analysis

## Overall Recommendation: [Submit / Fix-then-submit / Do-not-submit]

## Findings

| # | Category | Flaw | Severity | Location | Fix | Effort |
|---|----------|------|----------|----------|-----|--------|
| 1 | Method | RQ-method mismatch | Fatal | § 3.1 | ... | High |
| 2 | Results | No effect sizes | Major | § 4 | ... | Medium |

## Simulated Reviewer Objections

| Objection | Likely Reviewer [1/2/3]| Pre-emptive Defense |
|-----------|----------------------|---------------------|

## Summary
- Fatal flaws: [n] — [list]
- Major flaws: [n] — [list]
- Minor flaws: [n]
```
