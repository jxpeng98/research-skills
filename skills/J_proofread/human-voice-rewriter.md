---
id: human-voice-rewriter
stage: J_proofread
description: "Rewrite AI-flagged passages to sound authentically human-authored while preserving scientific accuracy."
inputs:
  - type: Manuscript
    description: "Current manuscript text"
  - type: AIDetectionReport
    description: "Flagged passages from ai-fingerprint-scanner"
outputs:
  - type: HumanizedManuscript
    artifact: "proofread/humanized_manuscript.md"
constraints:
  - "Must not alter statistical values, citations, or technical terminology"
  - "Must preserve claim strength and evidence links"
  - "Must vary rewriting strategy across passages"
failure_modes:
  - "Meaning drift from overly aggressive rewriting"
  - "Introducing factual errors during paraphrasing"
  - "Creating new AI patterns by using a single rewriting template"
tools: [filesystem]
tags: [proofread, humanization, rewriting, voice, style]
domain_aware: false
---

# Human-Voice Rewriter Skill

Rewrite AI-flagged passages so they sound authentically human-authored, while rigorously preserving scientific accuracy.

## Purpose

Transform passages identified by the AI fingerprint scanner (J1) into text that reads as if written by a human researcher — with varied rhythm, field-specific idiom, and natural imperfection — without distorting evidence or weakening claims.

## When to Use

- After J1 (AI fingerprint scan) identifies high/medium severity passages
- As the core rewriting step in the `/proofread` workflow (J2)
- When reviewers or editors flag text as potentially AI-generated

## Related Task IDs

- `J2` (human-voice rewrite)

## Outputs (contract paths)

- `RESEARCH/[topic]/proofread/humanized_manuscript.md`

## Inputs

- Full manuscript
- `proofread/ai_detection_report.md` with flagged passages

## Process

### Step 1: Prioritize Passages

Work through flagged passages in order:
1. **High severity** — rewrite required
2. **Medium severity** — rewrite recommended
3. **Low severity** — consider but may leave if natural in context

### Step 2: Apply Diverse Rewriting Strategies

Vary your approach across passages — do NOT apply the same template to every rewrite:

| Strategy | How | When to Use |
|----------|-----|-------------|
| **Sentence length variation** | Mix short declarative (5–10 words) with longer complex sentences (25–35 words) | Uniform-length passages |
| **Field-specific connectives** | Replace generic transitions with discipline terms ("This identifies a tension between…", "Decomposing the effect…") | Formulaic transitions |
| **Author voice injection** | Add personal stance markers: "We argue", "Our reading of the evidence suggests" | Generic hedging |
| **Concrete specificity** | Replace abstract generalization with concrete examples or numbers | Vague elaboration |
| **Structure breaking** | Use parenthetical asides, rhetorical questions, or mid-paragraph pivots | Template paragraph structures |
| **Deliberate imperfection** | Minor stylistic asymmetry — not every parallel is perfectly balanced | Over-polished constructions |

### Step 3: Verify Scientific Accuracy

For EVERY rewritten passage, check:

| Integrity Check | ✓/✗ |
|----------------|-----|
| Statistical values unchanged | |
| Citations remain correctly attributed | |
| Technical terminology preserved | |
| Claim strength not weakened or inflated | |
| Causal language matches original intent | |
| No evidence omitted or added | |

### Step 4: Multi-Agent Verification (recommended)

When using multi-agent mode:
- **Agent 1 (Drafter)**: Performs the rewrite
- **Agent 2 (Reviewer)**: Re-scans rewritten text for residual AI patterns
- **Agent 3 (Auditor)**: Diff-checks original vs. rewrite for meaning preservation
- Loop until reviewer AI-detection confidence ≥ 85%

### Step 5: Produce Full Revised Manuscript

Output the complete manuscript with all rewrites integrated, not just the individual passages.

## Quality Bar

The humanized manuscript is **ready** when:

- [ ] All high-severity passages from J1 are rewritten
- [ ] At least 3 different rewriting strategies used across the document
- [ ] Scientific accuracy verified for every rewritten passage
- [ ] No new AI patterns introduced by uniform rewriting
- [ ] Full manuscript output (not fragments)
- [ ] Change log documents every rewrite with original/new side by side

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Using one rewrite template for all passages | Creates new detectable patterns | Deliberately vary strategy per passage |
| Over-casualizing academic prose | Sounds unprofessional | Keep formality level appropriate to venue |
| Changing "significant" to a synonym without context | May alter statistical meaning | Distinguish statistical "significant" from colloquial use |
| Rewriting Methods section too aggressively | Methods should be precise and reproducible | Lighter touch in Methods — focus on transitions, not procedures |
| Not verifying evidence preservation | Meaning drift goes undetected | Mandatory diff-check per passage |

## Output Template

```markdown
---
task_id: J2
template_type: humanized_manuscript
topic: <topic>
primary_artifact: proofread/humanized_manuscript.md
---

# Humanized Manuscript

[Full revised manuscript text with all J2 rewrites integrated]

---

## J2 Change Log

| # | Section | Original Excerpt | Rewritten Excerpt | Strategy Used | Accuracy Check |
|---|---------|-----------------|-------------------|---------------|---------------|
| 1 | Introduction ¶3 | "Furthermore, it is important to note that…" | "This tension surfaces most clearly when…" | Field-specific connective + concrete specificity | ✓ all 6 checks passed |
```
