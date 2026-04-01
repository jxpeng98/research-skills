---
id: meta-optimizer
stage: F_writing
description: "Optimize title, abstract, keywords, and metadata for discoverability, impact, and venue compliance."
inputs:
  - type: Manuscript
    description: "Finalized manuscript draft"
  - type: VenueAnalysis
    description: "Venue requirements"
outputs:
  - type: MetadataOptimization
    artifact: "manuscript/meta_optimization.md"
constraints:
  - "Must preserve accuracy — do not overclaim in abstract"
  - "Must comply with venue-specific abstract format and keyword policies"
  - "Must optimize for search discoverability (database indexing, Google Scholar)"
failure_modes:
  - "Title is catchy but does not reflect actual scope"
  - "Abstract omits key results to save words"
  - "Keywords overlap with title words (wastes indexing opportunity)"
tools: [filesystem]
tags: [writing, title, abstract, keywords, SEO, metadata, discoverability]
domain_aware: true
---

# Meta Optimizer Skill

Optimize the title, abstract, keywords, and submission metadata for maximum discoverability while maintaining scientific accuracy.

## Purpose

Optimize title, abstract, keywords, and metadata for discoverability, impact, and venue compliance.

## Related Task IDs

- `F6` (abstract, title, keyword optimization)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/meta_optimization.md`

## When to Use

- After manuscript draft is complete (F2/F3/F4)
- Before submission (as part of submission packaging H1)
- After revision (recheck that changes are reflected in abstract)

## Process

### Step 1: Optimize the Title

A good academic title must balance informativeness and discoverability:

#### Title Formulas by Paper Type

| Paper Type | Formula | Example |
|-----------|---------|---------|
| Empirical (effect) | [DV]: [Evidence type] of [IV] in [Context] | "Employee Productivity: A Natural Experiment of Remote Work Adoption in Tech Firms" |
| Theoretical | [Theory/Framework]: [Novel contribution] | "Beyond Transaction Cost: A Relational View of Platform Governance" |
| Review / Meta-analysis | [Topic]: A [Systematic Review / Meta-Analysis] of [Scope] | "AI in Healthcare: A Systematic Review and Meta-Analysis of Diagnostic Accuracy" |
| Methods | [Method name]: A [Novel/Improved] Approach to [Problem] | "DeepCausal: A Deep Learning Approach to Heterogeneous Treatment Effect Estimation" |
| Qualitative | [Process/Experience]: [Design] of [Context] | "Navigating Algorithmic Governance: A Multi-Case Study of Platform Policy Rollout" |

#### Title Quality Checklist

| Criterion | Check | Why It Matters |
|-----------|-------|---------------|
| **Accurate** | Does the title match the actual findings? | Overclaiming in title → rejection |
| **Specific** | Are the key variables/concepts named? | Vague titles get fewer clicks and citations |
| **Searchable** | Does it contain terms people would search for? | Google Scholar and database indexing |
| **Concise** | Under 15 words (or venue maximum)? | Long titles are truncated in search results |
| **No abbreviations** | Spell out acronyms (unless universally known: AI, DNA) | Ensures search match |
| **No question form** (usually) | Statement or phrase, not a question | Conventions vary by field; questions common in some but not most venues |

### Step 2: Optimize the Abstract

#### Structured Abstract (when required)

| Section | Function | Target Length | Key Content |
|---------|----------|--------------|-------------|
| **Background** | Why this matters | 1–2 sentences | Gap/problem statement |
| **Purpose/Objective** | What you did | 1 sentence | RQ or aim |
| **Methods** | How you did it | 2–3 sentences | Design, sample, key measures/procedures |
| **Results** | What you found | 2–4 sentences | Key quantitative results with effect sizes, or core qualitative findings |
| **Conclusions** | So what? | 1–2 sentences | Implications (not speculation) |

#### Narrative Abstract (when structured is not required)

Use the same flow but without explicit section headers. Check that every element above is present.

#### Abstract Quality Checks

| Check | What to Verify |
|-------|---------------|
| **Standalone** | Can someone understand the paper's contribution from abstract alone? |
| **No undefined jargon** | All terms are defined or self-explanatory |
| **Results reported** | Specific findings included, not just "results are discussed" |
| **No claims beyond data** | Abstract claims ≤ what the results section supports |
| **No references** | Abstracts typically should not cite other papers |
| **Word limit** | Within venue requirement (typically 150–300 words) |
| **Keywords echoed** | Key terms from the title appear in the abstract |
| **First sentence impact** | Opens with the problem or significance, not "This paper..." |

> **Anti-pattern**: "This paper examines..." as the first sentence. Instead: "[Topic/Problem] is [significance statement]. We [action verb]..."

### Step 3: Select Keywords

#### Keyword Strategy

| Principle | Explanation | Example |
|-----------|-------------|---------|
| **Don't repeat title** | Keywords supplement, not duplicate | If title says "remote work", don't list "remote work" as keyword |
| **Use broader + narrower** | One broader term for discoverability, one narrower for precision | "flexible work arrangements" (broad) + "telecommuting" (narrow) |
| **Include method** | Helps method-seekers find you | "difference-in-differences", "grounded theory", "meta-analysis" |
| **Include domain** | If interdisciplinary, signal your field | "information systems", "organizational behavior" |
| **Check controlled vocabulary** | MeSH, JEL, ACM CCS terms improve indexing | JEL: M15, O33 |
| **5–7 keywords typical** | More dilutes; fewer misses opportunities | Check venue requirement |

### Step 4: Optimize Additional Metadata

| Element | What to Check |
|---------|---------------|
| **Running title** | Short version (≤50 characters typically) that captures the essence |
| **Author order** | Confirm CRediT alignment; check venue policy on equal contribution notation |
| **Corresponding author** | Designate; ensure institutional email is current |
| **ORCID** | All authors should link ORCID IDs |
| **Data availability** | Statement matches actual data sharing plan (D1) |
| **AI disclosure** | Matches actual AI tool usage |
| **Conflict of interest** | All authors declared |
| **Funding** | Grant numbers and funders listed |
| **Acknowledgments** | People who helped but don't meet authorship criteria |

## Quality Bar

The meta optimization is **ready** when:

- [ ] Title passes all 6 quality checks
- [ ] Abstract contains all required elements and is within word limit
- [ ] Keywords supplement (not duplicate) the title
- [ ] At least one controlled vocabulary term included in keywords
- [ ] All submission metadata fields populated
- [ ] Abstract accurately reflects the final Results section (not an earlier draft)
- [ ] No overclaiming: abstract claims ≤ what data shows

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Abstract written first, never updated | Reflects an earlier version of the paper | Rewrite abstract LAST, after results are finalized |
| "This paper studies..." opening | Weak and generic | Lead with the problem, gap, or finding |
| Keywords = title words | Wasted indexing opportunity | Use complementary terms |
| Overclaiming in abstract | "We prove that..." when results are correlational | Match claim strength to method |
| Ignoring venue word count | Administrative desk reject | Count words carefully; abstract ≠ introduction |

## Minimal Output Format

```markdown
# Meta Optimization

## Title
- **Proposed**: "[optimized title]"
- **Alternative**: "[backup title]"
- **Checklist**: ✅ Accurate / ✅ Specific / ✅ Searchable / ✅ Concise / ✅ No abbreviations

## Abstract
[Full optimized abstract — structured or narrative per venue]
- Word count: [n] / [limit]

## Keywords
1. [keyword] — rationale: [why chosen]
2. ...
3. ...

## Submission Metadata
| Field | Value |
|-------|-------|
| Running title | ... |
| Corresponding author | ... |
| Data availability | ... |
| AI disclosure | ... |
| COI | ... |
| Funding | ... |
```
