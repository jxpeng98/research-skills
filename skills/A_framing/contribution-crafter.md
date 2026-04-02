---
id: contribution-crafter
stage: A_framing
description: "Draft a compelling contribution statement highlighting theoretical, methodological, and empirical novelty."
inputs:
  - type: RQSet
    description: "Research questions and PICO boundaries"
  - type: GapAnalysis
    description: "Identified literature gaps"
outputs:
  - type: ContributionStatement
    artifact: "framing/contribution_statement.md"
constraints:
  - "Must explicitly state 'This paper makes three primary contributions...'"
  - "Must contrast with existing literature baseline"
  - "Must articulate the 'so what?' (implications)"
failure_modes:
  - "Overclaiming causality when design is observational"
  - "Confusing a new context with a new theoretical contribution"
tools: [filesystem]
tags: [contribution, novelty, framing, positioning]
domain_aware: true
---

# Contribution Crafter Skill

Draft a compelling, structured contribution statement that clearly articulates the theoretical, methodological, and empirical novelty of the research to editors and reviewers.

## Purpose

Draft a compelling contribution statement highlighting theoretical, methodological, and empirical novelty.

## Related Task IDs

- `A2` (contribution-novelty)

## Output (contract path)

- `RESEARCH/[topic]/framing/contribution_statement.md`

## When to Use

- When framing the pitch for a manuscript introduction
- When positioning the paper for a specific high-impact venue
- When editors or reviewers question the novelty of the research
- When transitioning from a research proposal to a manuscript

## Process

### Step 1: Baseline Extraction
- Read the `GapAnalysis` to understand what is currently missing in the literature.
- Read the `RQSet` to understand what the paper specifically addresses.
- Identify the "Baseline": What is the current state-of-the-art assumption or consensus?

### Step 2: Categorization of Novelty
Classify the contribution into one or more areas:
- **Theoretical**: Refining, challenging, or extending existing theory.
- **Methodological**: Introducing a new estimator, measurement tool, or identification strategy.
- **Empirical**: Providing the first causal evidence, utilizing a novel dataset, or resolving mixed findings.

### Step 3: Drafting the Statement
Write a 3-5 bullet point summary. For each point, use the structure:
1. **The Status Quo**: What previous literature did.
2. **The Pivot**: What this study does differently.
3. **The Yield**: What we learn that we didn't know before (the "so what").

### Step 4: Narrative Integration
Draft a cohesive 2-3 paragraph "Contributions" section suitable for the end of a manuscript Introduction. Ensure it explicitly begins with a strong signposting sentence (e.g., "This study makes three primary contributions to the literature...").

## Quality Bar

- [ ] Clear distinction between what is known and what is new
- [ ] Explicitly answers the "so what?" question for each claimed contribution
- [ ] Avoids generic claims (e.g., "this is the first study to do X in country Y")
- [ ] Aligns precisely with the boundaries set in the `RQSet` (no overclaiming)
- [ ] Formatted suitably for direct inclusion in a manuscript introduction

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Incrementalism | "We do X but with newer data" | Focus on *why* the new data changes the theoretical understanding |
| Overclaiming | Claiming causal contribution from correlational data | Downgrade verbs (e.g., "demonstrates" → "suggests") |
| Missing Baseline | Stating what is done without context | Always contrast against the specific status quo |
| Conflating Task with Contribution | "We collected a dataset" is not a contribution | "We open a new line of inquiry by leveraging a novel dataset to show..." |
