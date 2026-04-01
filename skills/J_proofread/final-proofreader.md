---
id: final-proofreader
stage: J_proofread
description: "Language-level polish covering grammar, tense, acronyms, cross-references, and venue-specific formatting."
inputs:
  - type: HumanizedManuscript
    description: "Manuscript after human-voice rewriting"
  - type: SimilarityReport
    description: "Similarity check output from J3"
outputs:
  - type: ProofreadChecklist
    artifact: "proofread/proofread_checklist.md"
constraints:
  - "Must not alter substantive content"
  - "Must respect venue-specific style guide"
  - "Must verify all cross-references resolve"
failure_modes:
  - "Venue style guide not available"
  - "Inconsistent author preferences for style choices"
tools: [filesystem]
tags: [proofread, grammar, formatting, consistency, final-polish]
domain_aware: false
---

# Final Proofreader Skill

The last editorial pass before submission — correcting grammar, enforcing consistency, and verifying cross-references.

## Purpose

Perform language-level polish that catches errors no other stage addresses: grammar mistakes, tense inconsistencies, undefined acronyms, broken cross-references, and formatting deviations from the venue style guide.

## When to Use

- As the final step before submission (J4)
- After J2 (human-voice rewrite) and J3 (similarity check) are complete
- When switching from one venue's style to another

## Related Task IDs

- `J4` (final proofread)

## Outputs (contract paths)

- `RESEARCH/[topic]/proofread/proofread_checklist.md`

## Inputs

- `proofread/humanized_manuscript.md` (or current manuscript)
- `proofread/similarity_report.md` (for context on any remaining issues)
- Venue style guide (if available)

## Process

### Step 1: Grammar and Syntax Pass

Check for:

| Issue | What to Check |
|-------|--------------|
| **Subject-verb agreement** | Especially in long sentences with intervening clauses |
| **Dangling modifiers** | "Using regression analysis, the data showed…" → the data didn't use regression |
| **Comma splices** | Two independent clauses joined by comma alone |
| **Run-on sentences** | Sentences > 40 words that lose clarity |
| **Parallel structure** | Lists and comparisons must maintain consistent grammar |
| **Article usage** | a/an/the — especially tricky for non-native speakers |

### Step 2: Tense Consistency

| Section | Expected Tense | Common Errors |
|---------|---------------|---------------|
| **Abstract** | Past (results), present (conclusions) | Mixing tenses within a paragraph |
| **Introduction** | Present (established facts), past (prior studies) | Using future tense for completed work |
| **Methods** | Past tense throughout | Switching to present mid-paragraph |
| **Results** | Past tense for findings | Using present for what was found |
| **Discussion** | Present (interpretations), past (findings recap) | "We found … which shows" inconsistency |

### Step 3: Acronym and Abbreviation Check

| Rule | Check |
|------|-------|
| First occurrence defined | Every acronym spelled out at first use |
| Consistent use after definition | Never reverting to full form without reason |
| Not defined in title | Titles should avoid acronyms (most style guides) |
| Abstract is self-contained | Acronyms re-defined in abstract if used |
| Not too many | If > 10 acronyms, consider reducing |

### Step 4: Cross-Reference Verification

| Reference Type | What to Verify |
|---------------|---------------|
| **Figure references** | "Figure 1" → Figure 1 exists and is correctly numbered |
| **Table references** | "Table 3" → Table 3 exists and content matches description |
| **Equation references** | "Equation (2)" → Equation 2 exists |
| **Section references** | "As discussed in Section 3" → Section 3 covers that topic |
| **Citation references** | Every (AuthorYear) has a bibliography entry |
| **Bibliography completeness** | Every bibliography entry is cited in text |

### Step 5: Venue-Specific Formatting

If venue style guide is available, check:

| Element | What to Verify |
|---------|---------------|
| **Spelling convention** | US vs. UK — match venue requirements |
| **Number formatting** | Spell out 1–9, numerals for 10+ (APA); or follow venue rules |
| **Decimal separator** | Period vs. comma |
| **p-value formatting** | p < .05 vs. p < 0.05; italic p or not |
| **Heading hierarchy** | Match venue's heading level system |
| **Reference style** | APA, Chicago, Vancouver, numbered — match venue |
| **Figure/table placement** | End of document vs. inline |

### Step 6: Final Checklist

Run through the complete checklist and log all corrections.

## Quality Bar

The proofread checklist is **ready** when:

- [ ] Grammar and syntax pass complete — all errors corrected
- [ ] Tense consistency verified per section
- [ ] All acronyms defined on first use and used consistently
- [ ] All cross-references verified (figures, tables, equations, sections, citations)
- [ ] Bibliography and in-text citations are 1:1 matched
- [ ] Venue-specific formatting applied (or noted as "no venue guide available")
- [ ] Corrections log documents every change made

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Changing content during proofread | This stage is language-only | Flag content issues for author but do not modify claims |
| Missing orphaned citations | A reference in the bibliography but never cited | Search bibliography entries against manuscript text |
| Inconsistent p-value formatting | p < .001 in one place, p<0.001 in another | Pick one format and enforce throughout |
| Overcorrecting author voice | Removing stylistic choices that are intentional | Ask author about intentional style before correcting |
| Skipping supplementary materials | Supplements also need proofing | Include supplements in the proofread pass |

## Output Template

```markdown
---
task_id: J4
template_type: proofread_checklist
topic: <topic>
primary_artifact: proofread/proofread_checklist.md
---

# Final Proofread Checklist

## Style Decisions Adopted
- Spelling: [US / UK]
- Number format: [APA / venue-specific]
- p-value format: [italic p < .05]
- Reference style: [APA 7th / Chicago / other]

## Corrections Log

| # | Section | Location | Issue | Correction |
|---|---------|----------|-------|------------|
| 1 | Methods ¶4 | Sentence 2 | Subject-verb disagreement | "The data show" → "The data show" (correct as-is) |

## Acronym Register

| Acronym | Full Form | First Use | Consistent? |
|---------|-----------|-----------|-------------|
| OLS | Ordinary Least Squares | Methods ¶1 | ✓ |

## Cross-Reference Audit

| Type | Total | Verified | Issues |
|------|-------|----------|--------|
| Figures | [n] | [n] | [list] |
| Tables | [n] | [n] | [list] |
| Citations | [n] | [n] | [list] |

## Remaining Items for Author Review
- [List any items that require author judgment]
```
