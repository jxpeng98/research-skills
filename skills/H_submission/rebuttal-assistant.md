---
id: rebuttal-assistant
stage: H_submission
description: "Convert reviewer comments into an actionable revision plan, point-by-point response matrix, and formal response letter."
inputs:
  - type: ReviewerComments
    description: "Editor decision letter and reviewer reports"
  - type: Manuscript
    description: "Original submitted manuscript"
outputs:
  - type: RevisionPlan
    artifact: "revision/revision_plan.md"
  - type: ResponseLetter
    artifact: "revision/response_letter.md"
constraints:
  - "Must address every single reviewer point — no silent ignoring"
  - "Must distinguish criticism from suggestion from optional comment"
  - "Must maintain respectful, constructive tone even for hostile reviews"
failure_modes:
  - "Dismissive responses that antagonize reviewers"
  - "Over-promising changes that cannot be delivered"
  - "Missing implicit concerns behind explicit questions"
tools: [filesystem]
tags: [submission, rebuttal, revision, reviewer-response, R&R]
domain_aware: false
---

# Rebuttal Assistant Skill

Convert reviewer feedback into a structured revision plan and persuasive response letter that maximizes the chance of acceptance.

## Related Task IDs

- `H2` (rebuttal / response)

## Output (contract path)

- `RESEARCH/[topic]/revision/revision_plan.md`
- `RESEARCH/[topic]/revision/response_letter.md`

## When to Use

- After receiving a Revise & Resubmit (R&R) decision
- After a conditional accept with minor/major revisions
- When preparing the response matrix before making changes

## Process

### Step 1: Parse and Classify Every Comment

Read the entire decision letter and all reviewer reports. For each point:

| Field | What to Record |
|-------|---------------|
| ID | R[reviewer#].[comment#] (e.g., R1.3 = Reviewer 1, Point 3) |
| Verbatim | Exact quote of the comment |
| Type | Criticism / Suggestion / Question / Praise / Meta-comment |
| Severity | Must-address / Should-address / Optional |
| Domain | Method / Theory / Writing / Other |
| Implicit concern | What is the reviewer REALLY worried about? |

#### Classification Heuristics

| Reviewer Language | Likely Type | Likely Severity |
|------------------|-------------|-----------------|
| "The authors must/should..." | Criticism | Must-address |
| "I would suggest..." | Suggestion | Should-address |
| "I wonder if/whether..." | Question | Must-address (often disguised criticism) |
| "It would be interesting to..." | Suggestion | Optional (unless editor echoes it) |
| "I don't understand..." | Criticism | Must-address — clarity failure |
| "The paper is well-written" | Praise | N/A — but acknowledge |
| "This is not convincing" | Criticism | Must-address — evidence/logic gap |

> **Critical**: "I wonder..." and "It might be interesting..." are often the reviewer's way of flagging a real problem politely. Always check the implicit concern.

### Step 2: Identify the Editor's Priority Signal

The editor's letter often signals what matters most:

| Editor Language | Priority Signal |
|----------------|----------------|
| "Please pay particular attention to R2's concerns about..." | Highest priority |
| "All reviewer comments must be addressed" | Standard — no shortcuts |
| "I share reviewer 1's concern about..." | Editor agrees → critical |
| "You may consider..." | Optional for the editor (but still address) |
| "Conditional on major/minor revisions" | Must-address = mandatory; rest = strong expectation |

### Step 3: Build the Response Matrix

Create a comprehensive tracking table:

| ID | Verbatim | Type | Severity | Implicit | Response Action | Manuscript Change | Status |
|----|----------|------|----------|----------|----------------|-------------------|--------|
| R1.1 | "The sample size seems small" | Criticism | Must | Worried about statistical power | Add power analysis; discuss in limitations | § Methods 3.2: add power analysis paragraph | ☐ |
| R1.2 | "Have you considered variable X?" | Question | Should | Suspects omitted variable bias | Run additional robustness check with X | § Results 4.3: add robustness table | ☐ |
| R2.1 | "The writing is unclear in §3" | Criticism | Must | Lost in methods description | Restructure methods into subsections | § Methods 3: reorganize | ☐ |

### Step 4: Plan Responses by Strategy

For each point, choose a response strategy:

| Strategy | When to Use | Response Template |
|----------|------------|-------------------|
| **Accept + Fix** | Comment is correct and fixable | "We thank Reviewer [n] for this insightful comment. We have [specific change]. See [page, section]." |
| **Accept + Partially Fix** | Comment has merit but full fix is impractical | "We agree this is important. We have [what we did]. We could not [what we couldn't] because [reason]. We added this as a limitation." |
| **Clarify** | The paper already addresses this, but unclearly | "We appreciate this question. Our intention was [clarification]. We have revised [section] to make this clearer." |
| **Respectfully Disagree** | Comment is based on a misunderstanding or different but defensible methodological choice | "We understand this concern. We respectfully note that [reasoning with citations]. We have added a brief justification in [section]." |
| **Defer** | Comment would require a completely new study | "We agree this is a valuable direction for future research. We have added this to our Future Work section (p. X)." |

> **NEVER use**: "The reviewer is wrong" / "We disagree" without detailed justification and citations. Even when the reviewer is wrong, maintain diplomatic respect.

### Step 5: Write the Response Letter

#### Response Letter Structure

```markdown
# Response to Reviewers

Dear [Editor name],

Thank you for the opportunity to revise our manuscript "[Title]" (MS-[number]).
We appreciate the constructive feedback from the reviewers and the editorial guidance.
Below we provide point-by-point responses to each comment. All changes in the
revised manuscript are highlighted in [blue text / track changes].

## Summary of Major Changes
1. [Major change 1 with rationale]
2. [Major change 2 with rationale]
3. ...

---

## Response to Reviewer 1

### R1.1: [Brief paraphrase or quote]
> [Full verbatim quote in blockquote]

**Response**: [Your response]

**Changes made**: [Exact location of changes — page, section, paragraph]

---

### R1.2: ...

---

## Response to Reviewer 2

### R2.1: ...

---

We believe these revisions substantially strengthen the manuscript and
address all reviewer concerns. We look forward to your decision.

Sincerely,
[Authors]
```

#### Response Letter Quality Rules

| Rule | Why |
|------|-----|
| Quote every comment verbatim | Shows you haven't missed or mischaracterized anything |
| Reference exact page/section for every change | Makes the editor's job easy |
| Thank reviewers before disagreeing | Establishes good faith |
| No reviewer comment left unaddressed | Unaddressed = "the authors ignored my concern" |
| Distinguish R&R round (R1, R2, R3) | Track which version each comment refers to |
| Keep responses concise but complete | Reviewers re-review; long responses signal insecurity |

## Quality Bar

The response package is **ready** when:

- [ ] Every reviewer point has an ID, classification, and response action
- [ ] Every must-address item has a specific manuscript change (with page/section)
- [ ] Response letter quotes each comment verbatim
- [ ] Disagreements are backed by reasoning and citations
- [ ] Summary of major changes appears at the top of the response letter
- [ ] Highlighted changes or track changes prepared in the manuscript
- [ ] Total number of points: [n] — verified against reviewer reports

## Minimal Output Format

```markdown
# Revision Plan

## Decision: [Major Revision / Minor Revision / Conditional Accept]
## Editor Priority: [summarize editor's key concern]

## Response Matrix

| ID | Type | Severity | Action | Manuscript Location | Status |
|----|------|----------|--------|--------------------|---------| 
| R1.1 | Criticism | Must | Accept + Fix | § 3.2 | ☐ |

## Response Letter
[Full response letter with verbatim quotes and point-by-point responses]
```
