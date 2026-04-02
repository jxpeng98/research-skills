---
id: discussion-writer
stage: F_writing
description: "Drafts the Discussion section using a structured story spine, separating factual findings from theoretical implications."
inputs:
  - type: ResultsSummary
    description: "Summarized findings and statistical/thematic results"
  - type: ContributionStatement
    description: "Original contribution mapping from the introduction"
outputs:
  - type: DiscussionDraft
    artifact: "manuscript/discussion.md"
  - type: StorySpine
    artifact: "manuscript/discussion_story_spine.md"
constraints:
  - "Must avoid restating results in detail; focus on interpretation"
  - "Must link findings back to the existing literature and theoretical framework"
failure_modes:
  - "Simply repeating the results section without interpretation"
  - "Overclaiming the implications beyond what the evidence supports"
tools: [filesystem]
tags: [writing, discussion, implications, story-spine, drafting]
domain_aware: true
---

# Discussion Writer Skill

Generates the Discussion section using a structured story spine, separating factual findings from theoretical implications.

## Purpose

To draft a compelling and structurally sound Discussion section that interprets the study's findings, situates them within the broader literature, and articulates the theoretical and practical implications without overclaiming.

## When to Use

Use after the Results section has been drafted and the primary findings are finalized. Provide the core contribution statement and literature framing to ensure alignment.

## Expected Inputs

- `RESEARCH/[topic]/manuscript/manuscript.md`
- `RESEARCH/[topic]/framing/research_question.md`
- `RESEARCH/[topic]/framing/contribution_statement.md`
- `RESEARCH/[topic]/literature/literature_map.md`

## Process

### Step 1: Build the Story Spine
Map the principal findings to the core narrative using a structured spine:
1. **The Core Answer:** A direct, concise answer to the main research question based on the findings.
2. **The Contextualization:** How these findings compare, contrast, or add nuance to the existing literature.
3. **The 'So What' (Theoretical Implications):** How the findings change our understanding of the phenomenon or theoretical model.
4. **The 'Now What' (Practical Implications):** Actionable recommendations for practitioners or policymakers.

### Step 2: Draft the Opening Paragraph
- Start strong by restating the primary aim of the study.
- Provide the clearest, most direct answer to the research question without repeating statistical details or raw data.
- State the "bottom line" conclusion upfront.

### Step 3: Draft the Interpretation and Comparison
- Dedicate paragraphs to each major finding or theme.
- For each, provide interpretation: *Why did this happen? What is the underlying mechanism?*
- Compare explicitly with prior literature: *Does this align with Author X (Year) or contradict Author Y (Year)? Why?*

### Step 4: Draft the Implications
- Separate theoretical implications from practical/managerial implications.
- Ensure all claims are strictly tied to the empirical evidence presented in the Results section.

### Step 5: Integration and Tone Check
- Review the draft to ensure a balanced, scholarly tone. Avoid hyperbolic language (e.g., "proves," "solves").
- Output the discussion draft to `manuscript/discussion.md`.
- Output the structural mapping to `manuscript/discussion_story_spine.md`.

## Quality Bar

- [ ] Does not unnecessarily repeat raw results or P-values
- [ ] Explicitly answers the primary research question stated in the Introduction
- [ ] Connects findings meaningfully to at least 2-3 key papers from the literature review
- [ ] Theoretical and practical implications are logically derived from the findings

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Restating Results | Reads like a second results section | Focus on *interpretation* and *meaning*, omit raw data |
| Overclaiming | Claiming causality or broad generalizability not supported by design | Use cautious language ("suggests", "indicates") |
| Ignoring Contradictions | Failing to discuss why results differ from established norms | Directly address unexpected findings and propose potential reasons |
