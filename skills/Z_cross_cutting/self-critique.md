---
id: self-critique
stage: Z_cross_cutting
version: "0.1.0"
description: "Iterative red teaming and Socratic questioning to continuously critique and refine AI-generated outputs."
inputs:
  - type: AnyArtifact
    description: "Any output requiring quality assurance"
outputs:
  - type: CritiqueLog
    artifact: "review/self_critique_log.md"
constraints:
  - "Must apply structured questioning protocol"
  - "Must iterate until no new issues found or max rounds reached"
failure_modes:
  - "Circular critique without convergence"
  - "Inability to identify own systematic biases"
tools: [filesystem]
tags: [cross-cutting, critique, red-team, quality-assurance, Socratic]
domain_aware: false
---

# Self-Critique Skill

Iterative Critique Loop (Red Teaming) to pressure-test research outputs through multi-agent debate and Socratic questioning.

## Purpose

Prevent superficial research by forcing the AI to act as "Reviewer 2" or a "Socratic Questioner." This skill challenges the generator's output to ensure rigorous narrowing down, robust design, and claim-evidence alignment.

## Process

1. **Self-Review Configuration:** Identify the current research stage (A-I).
2. **Reviewer Persona:** Adopt a highly critical, adversarial, yet constructive persona.
3. **Execution (Debate Loop):** 
   - **Draft Generation:** Generator provides the primary draft based on provided context and skills.
   - **Dynamic Literature Critique:** The Reviewer first analyzes the provided MCP Evidence (literature abstracts, metadata, full texts) and formulates 2-3 highly specific questions based on the controversies or limitations identified in those exact reference texts.
   - **Stage-Specific Critique:** The Reviewer then appends the rigorous, stage-specific questions (listed below).
   - **Iterative Revision:** The Reviewer passes or blocks the draft. If blocked, the Generator revises targeted fixes until the Reviewer passes the output or reaches the maximum allowed rounds.

## Stage-Specific Critique Questions

### Stage A: Framing & Positioning
- **Focus:** Funnel narrowing and gap validation.
- *Q1:* "Can the current RQ be answered in a single sentence? If not, is it overlapping and too massive? How can we constrain the boundaries (population, context, time) to cut the scope in half?"
- *Q2:* "Is this Gap simply because no one has done it (likely too difficult or meaningless), or involves a new data/method/theoretical dividend?"
- *Q3:* "Who cares? If this research is completely successful, which specific scholars or domains will cite it? Why?"
- *Q4:* "Does the framing rely on buzzwords without clear definitions? Which core concepts are assumed but actually contested in the literature?"
- *Q5:* "Are the proposed boundaries (e.g., geographic, temporal) justified theoretically, or merely chosen for convenience? What bias does this introduce?"

### Stage B: Literature Review
- **Focus:** Critical synthesis vs. passive summary.
- *Q1:* "Are we synthesizing or just summarizing? Have we identified the contradictions and conflicts in existing literature, rather than just agreeing with them?"
- *Q2:* "Have we been too lenient towards the highly-cited classic papers? What are their fundamental, unacknowledged limitations?"
- *Q3:* "Does the current literature review naturally and irrefutably logically lead to the proposed RQ (A1) as the only logical next step?"
- *Q4 (Confirmation Bias Check):* "Are the search keywords structurally biased towards confirming our hypotheses? What opposing search terms (null hypothesis literature) must be added?"
- *Q5:* "Are we overly reliant on WEIRD (Western, Educated, Industrialized, Rich, Democratic) samples masquerading as universal findings?"

### Stage C: Study Design
- **Focus:** Red teaming internal and external validity threats.
- *Q1 (Red Team Challenge):* "Assume this research ultimately fails or yields the exact opposite conclusion. What is the most likely fatal design/methodological flaw that caused it?"
- *Q2:* "Are we claiming causality or correlation? Do we have critical omitted confounding variables that account for the observed effect?"
- *Q3:* "Do our measurement instruments (proxies) accurately represent the abstract constructs defined in A3? Score this out of 10 and justify."
- *Q4 (Rival Hypotheses):* "What are the top 3 competing hypotheses, and exactly which variables/methods rule them out?"
- *Q5:* "Is the sample size justified by a rigorous power analysis, or based on 'rule of thumb'? What is the Minimum Detectable Effect?"

### Stage D: Ethics & IRB
- **Focus:** Extreme edge cases and participant safety.
- *Q1:* "Are there any edge cases (like de-anonymization attacks via joining multiple datasets over time) that could lead to participant data leakage? How do we defend against this mathematically or structurally?"
- *Q2:* "Is the language in the informed consent form at an 8th-grade reading level, or is it filled with academic jargon?"
- *Q3:* "Does the research involve vulnerable populations indirectly? Even if not the primary target, could they be disproportionately affected?"
- *Q4:* "What is the potential dual-use nature of these findings? Can this methodology be weaponized?"

### Stage E: Evidence Synthesis
- **Focus:** Publication bias and heterogeneity.
- *Q1 (Devil's Advocate):* "Argue that the massive aggregated effect size is 100% due to publication bias, file-drawer effect, and p-hacking. How does our data formally refute this?"
- *Q2:* "Is the heterogeneity between studies so high that we are essentially comparing apples to oranges? Justify the decision to pool mathematically (e.g., $I^2$ threshold)."
- *Q3:* "How sensitive is the overall conclusion to the removal of the specific single largest or most extreme study? (Leave-one-out sensitivity)."
- *Q4:* "Are we trusting the reported standard errors of primary studies blindly, or detecting reporting anomalies?"

### Stage F: Manuscript Writing
- **Focus:** Claim-evidence causal integrity.
- *Q1:* "Do the claims in the Discussion section drastically exceed the mathematical data support provided in the Results section?"
- *Q2:* "Does each major analytical paragraph move beyond description to explain at least one of: mechanism, tension, alternative explanation, boundary condition, or implication?"
- *Q3:* "Where are we merely restating results, themes, or citations instead of interpreting why the pattern matters?"
- *Q4:* "Are alternative explanations, contradictory evidence, and null cases confronted explicitly rather than buried in vague caveats?"
- *Q5:* "Comparing the promises made in the Introduction with the Conclusion, did we actually fulfill those promises without moving the goalposts?"
- *Q6:* "Is the limitations section honest and specific about boundary conditions, or just boilerplate text apologizing for basic boundaries?"

### Stage G: Polish & Compliance
- **Focus:** Harsh copy-editing and logical flow.
- *Q1 (Tone Check):* "Remove all unnecessary emphatic words (e.g., 'definitely', 'proves') and replace them with objective academic terms (e.g., 'suggests', 'indicates')."
- *Q2 (Logic Jump Check):* "If we strip out all transitional conjunctions (e.g., 'Therefore', 'Thus'), is the logical connection between paragraphs still solid?"
- *Q3:* "Are the active and passive voices mixed arbitrarily? Where the researchers acted, did they use active voice to take accountability?"
- *Q4:* "Is the manuscript bloated? Is there any paragraph that does not directly serve to establish the gap, methods, results, or interpretation?"

### Stage H: Submission & Revision
- **Focus:** De-escalation and reviewer empathy.
- *Q1 (Empathy Check):* "Roleplay as Reviewer 2. Would I feel this Response is brushing me off, or does it genuinely address my core concerns?"
- *Q2:* "If the reviewer asks for supplemental experiments that are impossible to conduct, is our alternative argumentation sufficiently convincing and gracious?"
- *Q3 (Tone Neutralization):* "Identify and eliminate any defensive, snarky, or argumentative tone in the rebuttal draft."
- *Q4:* "Did the revisions requested introduce contradictory statements in different parts of the manuscript (e.g., fixing methods but forgetting to update the abstract)?"

### Stage I: Code & Implementation
- **Focus:** Robustness and reproducibility.
- *Q1:* "Are there any hardcoded paths or magic numbers in the code? Is the random seed fixed globally for ALL stochastic operations?"
- *Q2:* "If 10% of the input data turns out to be NaN, will this data pipeline fail gracefully or silently produce incorrect aggregated results?"
- *Q3:* "Are the computational environment dependencies explicitly pinned (e.g., requirements.txt, Dockerfile) to prevent 'works on my machine' syndrome?"
- *Q4:* "Is there an unacknowledged O(N^2) or worse operation that will cause the code to hang if the dataset size scales 10x?"

## Usage

This skill is injected into tasks by the `mcp-agent-capability-map.yaml` and should be called:
- By the **Reviewer Agent** (e.g., Gemini) during the `review-agent-check` phase of orchestrator runs.
- Via role-play instructions in `/paper-write`, `/study-design`, and other generative commands.
