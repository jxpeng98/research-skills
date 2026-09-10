---
id: ai-fingerprint-scanner
stage: J_proofread
description: "Audit scholarly passages for concrete clarity, repetition and precision problems; does not determine authorship or detector scores."
inputs:
  - type: Manuscript
    description: "Full draft manuscript to scan"
outputs:
  - type: AIDetectionReport
    artifact: "proofread/ai_detection_report.md"
constraints:
  - "Must assess relevant language patterns in context without a finding quota"
  - "Must assign severity (high/medium/low) to each flagged passage"
  - "Must produce actionable rewrite directions"
failure_modes:
  - "False positives on naturally repetitive academic prose"
  - "Field-specific conventions mistaken for AI patterns"
tools: [filesystem]
tags: [proofread, ai-detection, fingerprint, writing-patterns]
domain_aware: false
---

# AI Fingerprint Scanner Skill

Inspect the requested scholarly text for language problems supported by its context.

## Purpose

Identify unclear, repetitive or imprecise language and propose specific corrections.
The legacy name and `AIDetectionReport` type do not imply authorship detection.
Do not estimate detection probabilities, optimize for evasion or require a rewrite
when the text is correct. Keep required AI disclosure and valid disciplinary style.

## When to Use

- When the user requests a language-pattern diagnosis of supplied scholarly text
- For formal J1 language review; ordinary grammar edits can use J4 directly

## Related Task IDs

- `J1` (AI fingerprint scan)

## Outputs (contract paths)

- `RESEARCH/[topic]/proofread/ai_detection_report.md`

## Inputs

- Requested passage or manuscript; reuse prior findings when their source is unchanged
- Direct passage checks can be answered in chat. The artifact and full coverage
  requirements below apply to formal J1 runs; missing project files alone do
  not require an interview or a saved gap note for an answerable text check.
- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.

## Process

### Step 1: Scan for Pattern Categories

Inspect only relevant categories; a pattern is a finding only when it harms
clarity, precision or the requested voice in this passage:

| Pattern | Check |
|---|---|
| Repeated transitions or lists | Do they obscure the relation between claims? |
| Repetitive sentence/paragraph structure | Does it make the argument hard to follow? |
| Empty qualifiers | Can they be removed without losing uncertainty or meaning? |
| Vague wording | Can available evidence support a more precise expression? |
| Unnecessary jargon or generic terms | Which wording is accurate for this audience? |

Do not infer authorship from sentence length, polished prose or common phrases.

### Step 2: Flag Each Passage

For every flagged passage, record:

| Field | What to Capture |
|-------|----------------|
| **Section** | Which manuscript section (Introduction, Methods, etc.) |
| **Paragraph** | Paragraph number within section |
| **Excerpt** | The exact flagged text (15–50 words) |
| **Pattern type** | Which category from Step 1 |
| **Severity** | `high` / `medium` / `low` |
| **Rewrite direction** | One-line correction tied to the actual language problem |

### Step 3: Prioritize and Summarize

- Count passages by severity level
- Identify the sections with highest concentration
- Set priority by the effect on meaning and readability, not pattern counts

### Step 4: Verify and Stop

Check that every finding identifies an actual issue and preserves source meaning.
Report zero findings when appropriate. Stop after the requested audit; do not
start rewriting, another model or J2 merely because a passage was flagged.
Independent review, when required, follows `skills/Z_cross_cutting/model-collaborator.md`.

## Output Contract

- `AIDetectionReport`: write `RESEARCH/[topic]/proofread/ai_detection_report.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when producing, revising, or validating central scholarly claims.
- Follow `references/evidence-ledger-contract.md`: supported claims need source pointers; unsupported central claims become `gap_note` rows and `RESEARCH/[topic]/context/gap_notes.md` entries.
- For final writing, proofread, submission, rebuttal, citation, or presentation-facing outputs, apply `references/citation-risk-policy.md` and write or update `RESEARCH/[topic]/proofread/citation-risk-report.md` when citation risk is material.

## Quality Bar

The language-pattern report is **ready** for its stated scope when:

- [ ] Relevant patterns have been checked in context; no finding quota applies
- [ ] Every flagged passage has section, paragraph, excerpt, pattern, severity, and direction
- [ ] Severity distribution is documented (count per level)
- [ ] Findings are prioritized by demonstrated language impact
- [ ] Formal full-manuscript coverage is documented; unavailable sections remain explicit

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Flagging domain-standard phrasing | "We conducted a regression analysis" is normal | Cross-check against field norms before flagging |
| Missing structural patterns | Only checking word-level, not paragraph-level | Explicitly check paragraph rhythm and template patterns |
| Over-sensitivity to hedging | All academic papers hedge | Only flag hedging that is generic and adds no information |
| Ignoring Methods section | Methods naturally use formulaic language | Preserve precise conventional wording unless a concrete problem is demonstrated |

## Output Template

```markdown
---
task_id: J1
template_type: ai_detection_report
topic: <topic>
primary_artifact: proofread/ai_detection_report.md
---

# Language-Pattern Report (J1)

## Summary
- Total passages flagged: [n]
- High severity: [n] | Medium: [n] | Low: [n]
- Reviewed scope and evidence limits: [sections/passages; unavailable material]

## Flagged Passages

| # | Section | ¶ | Excerpt | Pattern | Severity | Rewrite Direction |
|---|---------|---|---------|---------|----------|-------------------|
| 1 | Introduction | 3 | "Furthermore, it is important to note that..." | Formulaic transition + generic hedging | high | Replace with field-specific connective; drop empty qualifier |

## Review Basis
- Actual reviewed source and locations: [list]
- Review mode: [self-review / actual independent reviewers and limits]

```
