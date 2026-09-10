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
  - "Must state the compared corpus and unavailable sources; never invent similarity percentages"
failure_modes:
  - "Unable to access author's prior publications for self-plagiarism check"
  - "Field-standard terminology flagged as overlap"
tools: [filesystem]
tags: [proofread, similarity, plagiarism, originality, paraphrase]
domain_aware: false
---

# Similarity Checker Skill

Compare the requested text with available sources and report attribution issues and coverage limits.

## Purpose

Identify reuse or close paraphrase through actual source comparison. Keep
quotation, citation and permitted reuse explicit. Do not infer plagiarism from
style, common terminology or an unsupported similarity estimate.

## When to Use

- When the user asks to compare draft passages with supplied or accessible sources
- For formal J3 attribution/overlap checks; J2 rewriting is not an automatic prerequisite

## Related Task IDs

- `J3` (similarity & originality check)

## Outputs (contract paths)

- `RESEARCH/[topic]/proofread/similarity_report.md`

## Inputs

- Requested passage or current manuscript and the source texts available for comparison
- Direct comparisons may be reported in chat without J1/J2 or a project. Formal
  J3 runs retain their artifact and citation-risk requirements. Missing source
  text leaves that comparison unverified; do not claim a whole-corpus clearance.
- If inputs are missing or insufficient, identify the unavailable comparison and
  the source text needed. A formal run records a gap note in
  `RESEARCH/[topic]/context/gap_notes.md`; a direct answer states the gap in chat.

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

Conventional headings, standard terms and common methods descriptions are not
findings by themselves. Report a concern only when the available source comparison
shows unattributed reuse or an actual wording problem. Preserve precise language;
rewriting is not a substitute for appropriate quotation and attribution.

### Step 4: Document Each Flag

For every overlap:

| Field | Content |
|-------|---------|
| **Location** | Section + paragraph |
| **Overlap type** | Verbatim / close paraphrase / structural / boilerplate |
| **Source** | Author's prior paper / cited source / generic boilerplate |
| **Excerpt** | The overlapping text |
| **Suggestion** | Deeper rewrite / add quotation / add citation / acceptable as-is |

### Step 5: Record Coverage and Limits

List compared sources, their locators, unavailable sources and actual findings.
Do not fabricate a percentage or use a numeric threshold to declare originality.
An externally supplied similarity result is evidence only for its recorded tool,
corpus and settings; inspect the matches instead of treating its score as a verdict.
Stop at this comparison; revise text only when the user requested that action.

## Output Contract

- `SimilarityReport`: write `RESEARCH/[topic]/proofread/similarity_report.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when producing, revising, or validating central scholarly claims.
- Follow `references/evidence-ledger-contract.md`: supported claims need source pointers; unsupported central claims become `gap_note` rows and `RESEARCH/[topic]/context/gap_notes.md` entries.
- For final writing, proofread, submission, rebuttal, citation, or presentation-facing outputs, apply `references/citation-risk-policy.md` and write or update `RESEARCH/[topic]/proofread/citation-risk-report.md` when citation risk is material.

## Quality Bar

The similarity report is **ready** when:

- [ ] Self-plagiarism check completed (or explicitly noted author's works not available)
- [ ] Paraphrase quality assessed against available source text; missing comparisons are marked
- [ ] Conventional wording is distinguished from substantiated attribution concerns
- [ ] Every flag has location, type, source, excerpt, and suggestion
- [ ] Compared corpus, source locators and unavailable material documented

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
- Compared corpus and unavailable sources: [list with locators and limits]
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
