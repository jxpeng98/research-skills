---
id: rebuttal-assistant
stage: H_submission
description: "Generate point-by-point response matrix and tracked revision plan from reviewer comments."
inputs:
  - type: ReviewComments
    description: "Reviewer feedback and editor decision letter"
  - type: Manuscript
    description: "Original submitted manuscript"
outputs:
  - type: ResponseToReviewers
    artifact: "revision/response_matrix.md"
  - type: ResponseLetter
    artifact: "revision/response_letter.md"
constraints:
  - "Must address every reviewer point without exception"
  - "Must tag each response with action taken (revised/clarified/rebutted)"
failure_modes:
  - "Ambiguous reviewer comments requiring interpretation"
  - "Conflicting reviewer demands"
tools: [filesystem]
tags: [submission, rebuttal, revision, reviewer-response, point-by-point]
domain_aware: false
---

# Rebuttal Assistant Skill

Turn reviewer comments into a structured revision plan and a professional response package (response matrix + letter), including handling conflicting reviews.

## When to Use

- After receiving reviews (major/minor revision, reject-and-resubmit)
- When you need a systematic way to track changes and responses

## Inputs (Ask / Collect)

- Reviewer comments (copy/paste or files)
- Editor letter (if any)
- Current manuscript (or at least section outlines)
- What you *can* change (data collection possible? new analyses possible? timeline constraints?)

## Process

1. Parse comments into atomic items (one request per row)
2. Classify: major / minor / clarification / additional analysis / writing / citation
3. Decide action: accept / partially accept / respectfully disagree (with evidence)
4. Plan edits and assign locations (section + paragraph)
5. Draft response matrix, then response letter (editor-first framing)
6. Ensure responses are polite, specific, and verifiable (what changed + where)

## Outputs

- Response matrix → `RESEARCH/[topic]/revision/response_matrix.md` (use `templates/rebuttal-response-matrix.md`)
- Response letter → `RESEARCH/[topic]/revision/response_letter.md` (use `templates/rebuttal-letter.md`)
- Revision plan (optional) → `RESEARCH/[topic]/revision/revision_plan.md`

