---
id: reviewer-empathy-checker
stage: H_submission
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
  - "Must check that every response includes location of changes"
failure_modes:
  - "Cultural tone differences not accounted for"
  - "Over-correction making responses seem obsequious"
  - "Flagging appropriate assertiveness as defensiveness"
tools: [filesystem]
tags: [submission, rebuttal, empathy, tone, reviewer-response]
domain_aware: false
---

# Reviewer Empathy Checker Skill

Quality-check the response letter for tone, completeness, and precision — ensuring every reviewer feels heard, respected, and satisfied.

## Purpose

Neutralize defensiveness in reviewer responses and ensure each concern is addressed with exact precision.

## Related Task IDs

- `H2_5` (reviewer empathy check)

## Output (contract path)

- `RESEARCH/[topic]/revision/reviewer_empathy_check.md`

## When to Use

- After drafting the response letter (from `rebuttal-assistant` H2) but BEFORE submitting
- When you feel emotional about negative reviews (cool-down pass)
- For second/third round revisions where reviewer patience is lower

## Process

### Step 1: Check Every Response for the ACE Structure

Each response must contain all three elements:

| Element | What It Does | Example |
|---------|-------------|---------|
| **A**cknowledge | Show you understood the concern | "We thank the reviewer for highlighting this important concern about..." |
| **C**hange (or justify) | Describe what you did or why you didn't | "We have added a power analysis in § 3.2 (p. 12)" or "We respectfully note that... because [reason + citation]" |
| **E**vidence | Point to the exact location | "See revised manuscript, § 3.2, paragraph 3, page 12" |

**Missing ACE audit**:

| Response | A | C | E | Issue |
|----------|---|---|---|-------|
| R1.1 | ✅ | ✅ | ❌ | Missing location — reviewer can't verify the change |
| R1.3 | ❌ | ✅ | ✅ | Missing acknowledgment — reviewer feels dismissed |
| R2.2 | ✅ | ❌ | N/A | Missing action — reviewer's concern is unaddressed |

### Step 2: Flag Defensive Language Patterns

Scan for language that will antagonize a reviewer:

#### Red Flags (rewrite immediately)

| Pattern | Example | Why It's Problematic | Suggested Rewrite |
|---------|---------|---------------------|-------------------|
| "Obviously..." | "This is obviously correct" | Implies the reviewer is unintelligent | "As described in § X, the rationale is..." |
| "The reviewer misunderstood" | "R1 misunderstood our approach" | Blames the reviewer for your unclear writing | "We apologize for the lack of clarity. Our intention was... We have revised § X to clarify" |
| "We disagree" (bare) | "We disagree with this assessment" | Confrontational without substance | "We appreciate this perspective. We note that [reasoning with citations]. We have added clarification in § X" |
| "As we clearly stated" | "As clearly stated on page 5..." | Condescending; implies reviewer didn't read carefully | "We have revised § X to make this point more prominent" |
| "We already addressed this" | "This was already addressed in § 3" | Dismissive | "Thank you for raising this. We discuss this in § 3 but have now expanded the explanation to make the rationale more explicit" |
| "This suggestion is beyond scope" | Bare refusal without engagement | Reviewer feels unheard | "We agree this is a valuable direction. Due to [constraints], we have added this as a future research direction in § 6" |
| "Not relevant" | "This comment is not relevant to our study" | Insulting | "We appreciate this angle. While our study focuses on [scope], we acknowledge [connection] and have noted this in § X" |

#### Acceptable Assertive Language (don't over-correct)

| Pattern | When Appropriate | Example |
|---------|-----------------|---------|
| "We respectfully note that..." | Polite disagreement with substance | "We respectfully note that this design choice follows [Author (Year)], who demonstrates that..." |
| "Following [Author], we maintain..." | Defending a methodology with citation | "Following Angrist & Pischke (2009), we maintain that IV estimation is appropriate here because..." |
| "After careful consideration, we have retained..." | Explaining why no change was made | "After careful consideration, we retained the original specification because [reason]. We have added justification in § X" |

### Step 3: Check Response Completeness

For every reviewer comment in the decision letter:

| Check | What to Verify | Problem If Missing |
|-------|---------------|-------------------|
| **Addressed?** | Is there a response for this specific comment? | Reviewer feels ignored → hostile in next round |
| **Directly answers the question?** | Does the response address what was actually asked? | Reviewer asks about endogeneity; you discuss sample size |
| **Location specified?** | Does the response point to where changes were made? | Reviewer can't verify → assumes nothing changed |
| **Changed vs. justified?** | Is the action clear (we changed X / we kept X because Y)? | Ambiguous responses → reviewer confusion |
| **New content consistent?** | Do the new additions create contradictions elsewhere? | Added robustness check contradicts interpretation in Discussion |

### Step 4: Check Tone Calibration

The overall tone should be:

| Quality | What It Looks Like | What It Does NOT Look Like |
|---------|-------------------|---------------------------|
| **Grateful** | "We thank the reviewer for this careful reading" | "We are grateful for this opportunity" (obsequious) |
| **Professional** | "We have carefully considered this point" | "Obviously we agree" (dismissive agreement) |
| **Specific** | "We added 3 sentences in § 4.2 addressing [concern]" | "We have taken this into account" (vague) |
| **Confident but humble** | "Following [citation], we believe our approach is appropriate" | "We are right because..." |
| **Honest about limitations** | "We acknowledge that X remains a limitation" | "We have addressed all concerns" (if you haven't) |

### Step 5: Cross-Check Between Responses

| Issue | What to Check |
|-------|--------------|
| **Contradictory responses** | You told R1 you added a robustness check but told R2 it's unnecessary |
| **Double-counting changes** | Same change used to satisfy two different concerns (sometimes valid — flag it) |
| **Escalation risk** | R1 flags a concern in Round 1; your response doesn't fully address it → R1 escalates in Round 2 |
| **Implicit promises** | "We will address this in future work" — is this an appropriate deferral or a dodge? |

## Quality Bar

The empathy check is **ready** when:

- [ ] Every reviewer comment has a response with ACE structure (Acknowledge, Change/Justify, Evidence/Location)
- [ ] Zero red-flag defensive phrases remain
- [ ] No response answers a different question than what was asked
- [ ] Cross-response consistency verified (no contradictions between answers to different reviewers)
- [ ] Tone is grateful + professional + specific + honest
- [ ] Responses that disagree include citations and reasoning (not bare disagreement)

## Minimal Output Format

```markdown
# Reviewer Empathy Check

## Summary
- Total responses checked: [n]
- ACE structure complete: [n/total]
- Defensive language flagged: [n] instances
- Completeness issues: [n]
- Cross-response contradictions: [n]

## ACE Audit

| ID | Acknowledge | Change/Justify | Location | Status |
|----|-------------|---------------|----------|--------|
| R1.1 | ✅ | ✅ | ✅ | ✅ Pass |
| R1.2 | ✅ | ❌ | — | ⚠️ Missing action |

## Defensive Language Flags

| ID | Flagged Text | Issue | Suggested Rewrite |
|----|-------------|-------|-------------------|

## Completeness Issues

| ID | Reviewer Question | Response Problem | Fix |
|----|------------------|------------------|-----|

## Cross-Response Consistency
| Issue | R1 Response | R2 Response | Contradiction? |
|-------|-----------|-----------|---------------|
```
