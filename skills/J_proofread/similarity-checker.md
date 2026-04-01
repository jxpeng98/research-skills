---
id: similarity-checker
stage: J_proofread
description: "Identify text overlap including self-plagiarism, close paraphrasing, and boilerplate passages."
inputs:
  - type: HumanizedManuscript
    description: "Manuscript after human-voice rewriting"
outputs:
  - type: SimilarityReport
    artifact: "proofread/similarity_report.md"
constraints:
  - "Must check self-plagiarism, close paraphrase, and boilerplate"
  - "Must provide rewrite or attribution suggestions for each flag"
  - "Must estimate overall similarity level"
failure_modes:
  - "Unable to access author's prior publications for self-plagiarism check"
  - "Field-standard terminology flagged as overlap"
tools: [filesystem]
tags: [proofread, similarity, plagiarism, originality, paraphrase]
domain_aware: false
---

# Similarity Checker Skill

Identify text overlap with known sources, prior publications, and boilerplate passages — ensuring originality before submission.

## Purpose

Detect three types of overlap that can trigger similarity tools (Turnitin, iThenticate) or reviewer concerns: self-plagiarism from the author's prior work, insufficient paraphrasing of cited sources, and generic boilerplate that appears across many papers.

## When to Use

- After J2 (human-voice rewrite) to ensure rewrites are original
- Before submission to any venue that uses plagiarism detection
- As step J3 in the `/proofread` workflow

## Related Task IDs

- `J3` (similarity & originality check)

## Outputs (contract paths)

- `RESEARCH/[topic]/proofread/similarity_report.md`

## Inputs

- `proofread/humanized_manuscript.md` (or current manuscript draft)

## Process

### Step 1: Self-Plagiarism Check

If the author's prior publications are available:

| Check | What to Look For |
|-------|-----------------|
| **Verbatim reuse** | Sentences or paragraphs copied from prior papers |
| **Close paraphrase of own work** | Same structure with minor word swaps |
| **Methods recycling** | Identical methods descriptions across papers |
| **Introduction/lit review recycling** | Same framing used in multiple papers |

> **Note**: Reusing methods descriptions is common and sometimes acceptable — but it should be disclosed or properly cited (e.g., "Following [AuthorYear], we…").

### Step 2: Cross-Reference Paraphrase Quality

For passages that cite sources:

| Quality Level | Description | Action |
|--------------|-------------|--------|
| **Adequate** | Own words, own structure, proper citation | No action needed |
| **Close paraphrase** | Same sentence structure, synonyms swapped | Flag for deeper rewrite |
| **Patchwriting** | Phrases lifted with minimal changes | Flag as high priority |
| **Verbatim without quotes** | Direct copy without quotation marks | Must quote or deeply rewrite |

### Step 3: Boilerplate Detection

Flag generic phrases that appear in many academic papers:

- "This study contributes to the literature by..."
- "The remainder of this paper is organized as follows"
- "Further research is needed to..."
- "This study has several limitations"
- "The findings have important implications for theory and practice"

These are not plagiarism but may inflate similarity scores unnecessarily.

### Step 4: Document Each Flag

For every overlap:

| Field | Content |
|-------|---------|
| **Location** | Section + paragraph |
| **Overlap type** | Verbatim / close paraphrase / structural / boilerplate |
| **Source** | Author's prior paper / cited source / generic boilerplate |
| **Excerpt** | The overlapping text |
| **Suggestion** | Deeper rewrite / add quotation / add citation / acceptable as-is |

### Step 5: Estimate Overall Similarity

Provide a rough estimate based on flagged passages:

| Level | Estimated Overlap | Risk |
|-------|------------------|------|
| **Low** | < 10% | Acceptable for most venues |
| **Moderate** | 10–20% | Review flagged passages; rewrite close paraphrases |
| **High** | > 20% | Significant rewriting needed before submission |

## Quality Bar

The similarity report is **ready** when:

- [ ] Self-plagiarism check completed (or explicitly noted author's works not available)
- [ ] Paraphrase quality assessed for all passages citing sources
- [ ] Boilerplate passages identified
- [ ] Every flag has location, type, source, excerpt, and suggestion
- [ ] Overall similarity estimate documented

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Flagging standard terminology as overlap | "Ordinary least squares regression" is universal | Exclude standard technical terms from overlap counts |
| Missing self-plagiarism in Methods | Most tools catch this; authors often don't notice | Explicitly compare Methods sections across papers |
| Ignoring reference list overlap | Bibliography text can inflate similarity scores | Note but de-prioritize reference list overlap |
| Not distinguishing types of overlap | Treating all overlap equally | Severity differs: verbatim > close paraphrase > boilerplate |

## Output Template

```markdown
---
task_id: J3
template_type: similarity_report
topic: <topic>
primary_artifact: proofread/similarity_report.md
---

# Similarity & Originality Report

## Overall Assessment
- Estimated overlap: [low / moderate / high]
- Total flags: [n]
- By type: verbatim [n] | close paraphrase [n] | structural [n] | boilerplate [n]

## Flagged Passages

| # | Section | Type | Source | Excerpt | Suggestion |
|---|---------|------|--------|---------|------------|
| 1 | Methods ¶2 | close paraphrase | Smith et al. (2023) | "We employed a difference-in-differences design..." | Rewrite with study-specific details |

## Boilerplate Passages
| # | Section | Phrase | Suggested Alternative |
|---|---------|--------|----------------------|

## Self-Plagiarism Check
- Author prior works reviewed: [yes/no, list if yes]
- Flags: [list or "none detected"]
```
