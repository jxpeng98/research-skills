---
id: reviewer-empathy-checker
stage: H_submission
version: "0.2.0"
description: "Neutralize defensiveness in reviewer responses and ensure each concern is addressed with exact precision."
inputs:
  - type: ResponseToReviewers
    description: "Draft response to reviewers"
outputs:
  - type: EmpathyCheck
    artifact: "revision/reviewer_empathy_check.md"
constraints:
  - "Must flag defensive language"
  - "Must verify each response directly addresses the reviewer's concern"
failure_modes:
  - "Cultural tone differences not accounted for"
  - "Over-correction making responses seem obsequious"
tools: [filesystem]
tags: [submission, rebuttal, empathy, tone, reviewer-response]
domain_aware: false
---

# Reviewer Empathy Checker Skill

Neutralize defensiveness and ensure every reviewer concern is acknowledged and addressed concretely.

## Related Task IDs

- `H2_5` (reviewer empathy check)

## Output (contract path)

- `RESEARCH/[topic]/revision/reviewer_empathy_check.md`

## Inputs

- Draft response matrix/letter (or raw reviewer comments)
- Target venue tone conventions (if known)

## Procedure

1. For each reviewer point, check the response contains:
   - acknowledgement (show you understood)
   - action taken (or principled justification for not changing)
   - evidence/location (where in the manuscript it was changed)
2. Flag:
   - dismissive phrasing (“obviously”, “we disagree”, “misunderstood”)
   - missing location references
   - responses that answer a different question than asked

## Minimal output format (`revision/reviewer_empathy_check.md`)

```markdown
# Reviewer Empathy Check

| comment_id | risk (tone/completeness/misalignment) | problematic text | suggested rewrite | required action |
|---|---|---|---|---|
```
