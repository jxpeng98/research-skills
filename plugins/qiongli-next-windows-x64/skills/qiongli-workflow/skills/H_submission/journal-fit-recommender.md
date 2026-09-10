---
id: journal-fit-recommender
stage: H_submission
description: "Recommend journals from an existing manuscript using venue profile, contribution, methods, evidence, and reviewer-risk fit."
inputs:
  - type: Manuscript
    description: "Current manuscript draft or structured manuscript sections"
  - type: ClaimGraph
    description: "Claim-evidence map for manuscript claims"
  - type: VenueAnalysis
    description: "Venue profile evidence, venue assumptions, or prior venue analysis"
outputs:
  - type: JournalFitRecommendation
    artifact: "submission/journal_fit_recommendation.md"
constraints:
  - "Must be manuscript-first, not target-first"
  - "Must block best-journal claims when manuscript evidence is missing"
  - "Must classify venues as primary, stretch, safe, fallback, or do_not_submit"
tools: [filesystem]
tags: [submission, journal-selection, venue-fit, manuscript-review]
domain_aware: true
---

# Journal Fit Recommender

Recommend journals for an existing manuscript. This is a manuscript-first H5
skill: read the draft, contribution, methods or evidence design, limitations,
claim-evidence map, and venue profiles before ranking venues.

## Purpose

Identify realistic submission targets from the manuscript that exists now,
while blocking unsupported best-journal claims when evidence, methods, claim
support, or venue-profile information is missing.

## When to Use

- The user has an existing manuscript and asks which journal fits best.
- Stage H needs `H5` reverse journal-fit recommendation before submission.
- A target venue is uncertain and the manuscript should drive venue selection.

Do not use this as an early target-first venue scan. For early framing before a
manuscript exists, use `A5` venue analysis instead.

## Inputs

- `RESEARCH/[topic]/manuscript/manuscript.md` or structured manuscript
  sections.
- `RESEARCH/[topic]/framing/research_question.md`.
- `RESEARCH/[topic]/framing/contribution_statement.md`.
- Methods, data, or evidence design summary.
- `RESEARCH/[topic]/manuscript/claims_evidence_map.md` or
  `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv`.
- Limitations audit or fatal flaw report when available.
- Venue profiles or prior venue evidence.

If a required input is missing or insufficient, write a gap note under
`RESEARCH/[topic]/context/gap_notes.md` and block any best-journal claim
instead of inventing fit evidence.

## Process

1. Confirm manuscript readiness.
   Check that the manuscript has a clear research question, contribution,
   methods or evidence design, and claim-evidence support.
2. Build candidate venue evidence.
   Use local venue profiles, subject-specific venue profiles, and any prior
   venue analysis. If the catalog is too thin, mark coverage limits.
3. Score fit dimensions.
   Assess scope fit, contribution fit, method/evidence fit, article type fit,
   audience fit, reporting/data-policy fit, reviewer risk, desk-reject risk,
   and required revisions.
4. Classify each venue.
   Use only `primary`, `stretch`, `safe`, `fallback`, or `do_not_submit`.
5. Check overreach.
   If a high-status venue is attractive but the manuscript evidence does not
   meet its threshold, classify it as `stretch` or `do_not_submit`, not
   `primary`.
6. State next revisions.
   For every viable venue, list the concrete revisions needed before
   submission.

## Output Contract

- `JournalFitRecommendation`: write `RESEARCH/[topic]/submission/journal_fit_recommendation.md`.
- Also write `RESEARCH/[topic]/submission/journal_fit_recommendation.json` when
  machine-readable output is requested.
- Separate finding, interpretation, and implication in the final artifact.
- Do not invent citations, data, sample sizes, venue policies, acceptance
  probabilities, or reviewer comments.
- Apply `references/academic-output-rubric.md` before finalizing scholarly
  prose or review artifacts.

Use this table in the markdown report:

| Venue | Class | Scope fit | Contribution fit | Method/evidence fit | Reviewer risk | Desk-reject risk | Required revision |
|---|---|---|---|---|---|---|---|

Use these classes exactly:

- `primary`
- `stretch`
- `safe`
- `fallback`
- `do_not_submit`

When evidence is incomplete, produce a blocked report with missing inputs and
next collection steps. Do not name a single best journal.

### Evidence Ledger and Source Integrity

- Update `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv` when producing,
  revising, or validating central scholarly claims.
- Follow `references/evidence-ledger-contract.md`: supported claims need source
  pointers; unsupported central claims become `gap_note` rows and
  `RESEARCH/[topic]/context/gap_notes.md` entries.
- For final writing, proofread, submission, rebuttal, citation, or
  presentation-facing outputs, apply `references/citation-risk-policy.md` and
  write or update `RESEARCH/[topic]/proofread/citation-risk-report.md` when
  citation risk is material.

## Quality Bar

- [ ] Manuscript, contribution, methods/evidence design, and claim-evidence map
  were inspected.
- [ ] At least three candidate venues were assessed when the venue catalog
  permits.
- [ ] Every venue has one of the five allowed classes.
- [ ] The report explains why any higher-status journal is not primary when
  the manuscript does not support it.
- [ ] Missing evidence blocks best-journal claims instead of producing a false
  ranking.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---|---|---|
| Ranking by prestige | Ignores scope, evidence, and manuscript maturity | Start from manuscript fit and reviewer risk |
| Target-first shortcut | Reuses A5 venue analysis without reading the draft | Read manuscript and claim map first |
| Unsupported best-journal claim | Names one venue despite missing methods or evidence | Return a blocked H5 report |
| No do-not-submit class | Leaves poor fits looking viable | Use `do_not_submit` for venues outside scope or evidence threshold |
| Generic revision advice | Does not tell the author what must change | Tie revisions to venue policy, reviewer risk, or claim support |
