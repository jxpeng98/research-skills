---
id: human-voice-rewriter
stage: J_proofread
description: "Improve the clarity and scholarly voice of requested passages while preserving meaning, evidence and required disclosure."
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
  - "Must change only what improves the requested text; correct passages may remain unchanged"
failure_modes:
  - "Meaning drift from overly aggressive rewriting"
  - "Introducing factual errors during paraphrasing"
  - "Creating new AI patterns by using a single rewriting template"
tools: [filesystem]
tags: [proofread, humanization, rewriting, voice, style]
domain_aware: false
---

# Human-Voice Rewriter Skill

Revise scholarly expression without changing the research or claiming to establish authorship.

## Purpose

Resolve concrete expression problems using the smallest faithful edit. Do not
add invented evidence, personal stance, deliberate mistakes or detector-oriented
changes. Keep required AI disclosure; the legacy task/output names are retained.

## When to Use

- When the user asks to improve scholarly expression or author voice
- When actual language issues were identified in J1 or supplied feedback
- Use J4 for grammar-only corrections and source comparison for attribution issues

## Related Task IDs

- `J2` (human-voice rewrite)

## Outputs (contract paths)

- `RESEARCH/[topic]/proofread/humanized_manuscript.md`

## Inputs

- Requested passage or manuscript; any existing J1 findings or supplied feedback
- A direct edit needs only the relevant text and constraints; return it in chat
  without starting J1 or requiring a project. Formal J2 runs retain the declared
  inputs, full manuscript artifact and applicable gates below.
- If a required input is missing or insufficient, write a gap note under `RESEARCH/[topic]/context/gap_notes.md` and ask for the missing artifact instead of inventing content.

## Process

### Step 1: Prioritize Passages

Prioritize actual meaning/readability problems and the user's selected passages.
Verify supplied flags rather than treating their severity as a command to rewrite.
Correct text may remain unchanged.

### Step 2: Make the Smallest Useful Edit

Remove empty qualifiers, clarify supported relationships and simplify awkward
syntax where this improves the requested text. Preserve necessary uncertainty,
technical phrasing, disciplinary conventions and intentional author voice.
Choose strategies by the actual issue, with no minimum variety or length quota.

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

### Step 4: Verify the Affected Passages

Compare the revised passage with its source using the integrity checks above.
Fix any actual drift and recheck that change. Stop when the requested issues are
resolved; do not run a detector-confidence loop. Use an independent reviewer
only when required and actually available through the active Host; follow
`skills/Z_cross_cutting/model-collaborator.md` and report unavailable requirements.

### Step 5: Return the Requested Output

Return the selected passages for a direct edit. For a formal J2 run, integrate
changes into the full manuscript and record them in its change log.

## Output Contract

- `HumanizedManuscript`: write `RESEARCH/[topic]/proofread/humanized_manuscript.md`.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, statistical results, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose or review artifacts.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when producing, revising, or validating central scholarly claims.
- Follow `references/evidence-ledger-contract.md`: supported claims need source pointers; unsupported central claims become `gap_note` rows and `RESEARCH/[topic]/context/gap_notes.md` entries.
- For final writing, proofread, submission, rebuttal, citation, or presentation-facing outputs, apply `references/citation-risk-policy.md` and write or update `RESEARCH/[topic]/proofread/citation-risk-report.md` when citation risk is material.

## Quality Bar

The humanized manuscript is **ready** when:

- [ ] All substantive issues in the requested scope are resolved or reported as gaps
- [ ] Every edit improves a demonstrated issue; no strategy quota applies
- [ ] Scientific accuracy verified for every rewritten passage
- [ ] Meaning, author voice and required disclosure are preserved
- [ ] Formal J2 output includes the full manuscript; direct edits respect requested scope
- [ ] Change log documents every rewrite with original/new side by side

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Using one rewrite template for all passages | Ignores the actual language issue | Choose the smallest edit that helps this passage |
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
