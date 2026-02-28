# Self-Critique Skill

Iterative Critique Loop (Red Teaming) to pressure-test research outputs through multi-agent debate and Socratic questioning.

## Purpose

Prevent superficial research by forcing the AI to act as "Reviewer 2" or a "Socratic Questioner." This skill challenges the generator's output to ensure rigorous narrowing down, robust design, and claim-evidence alignment.

## Process

1. **Self-Review Configuration:** Identify the current research stage (A-I).
2. **Reviewer Persona:** Adopt a highly critical, adversarial, yet constructive persona.
3. **Execution (Debate Loop):** 
   - Generator provides the draft.
   - Reviewer asks the stage-specific critique questions.
   - Generator revises until the Reviewer passes the output.

## Stage-Specific Critique Questions

### Stage A: Framing & Positioning
- **Focus:** Funnel narrowing and gap validation.
- *Q1:* "Can the current RQ be answered in a single sentence? If not, is it overlapping and too massive? How can we constrain the boundaries (population, context, time) to cut the scope in half?"
- *Q2:* "Is this Gap simply because no one has done it (likely too difficult or meaningless), or involves a new data/method dividend?"
- *Q3:* "Who cares? If this research is completely successful, which specific scholars or domains will cite it? Why?"

### Stage B: Literature Review
- **Focus:** Critical synthesis vs. passive summary.
- *Q1:* "Are we synthesizing or just summarizing? Have we identified the contradictions and conflicts in existing literature?"
- *Q2:* "Have we been too lenient towards the highly-cited classic papers? What are their fundamental limitations?"
- *Q3:* "Does the current literature review naturally and irrefutably lead to the proposed RQ (A1)?"
- *Q4 (Confirmation Bias Check):* "Are the search keywords structurally biased towards confirming our hypotheses? What opposing search terms must be added?"

### Stage C: Study Design
- **Focus:** Red teaming internal and external validity threats.
- *Q1 (Red Team Challenge):* "Assume this research ultimately fails or yields the exact opposite conclusion. What is the most likely fatal design flaw that caused it?"
- *Q2:* "Are we claiming causality or correlation? Do we have critical omitted variables?"
- *Q3:* "Do our measurement instruments (proxies) accurately represent the abstract constructs defined in A3? Score this out of 10 and justify."
- *Q4 (Rival Hypotheses):* "What are the competing hypotheses, and how are we designing the study to rule them out?"

### Stage D: Ethics & IRB
- **Focus:** Extreme edge cases and participant safety.
- *Q1:* "Are there any edge cases (like de-anonymization attacks via joining multiple datasets) that could lead to participant data leakage? How do we defend against this?"
- *Q2:* "Is the language in the informed consent form at an 8th-grade reading level, or is it filled with academic jargon?"

### Stage E: Evidence Synthesis
- **Focus:** Publication bias and heterogeneity.
- *Q1 (Devil's Advocate):* "Argue that the massive aggregated effect size is 100% due to publication bias. How does our data refute this?"
- *Q2:* "Is the heterogeneity so high that we are essentially comparing apples to oranges? Justify the decision to pool."

### Stage F: Manuscript Writing
- **Focus:** Claim-evidence causal integrity.
- *Q1:* "Do the claims in the Discussion section drastically exceed the data support provided in the Results section?"
- *Q2:* "Do the numbers in the abstract match the tables in the main text with 100% accuracy?"
- *Q3:* "Comparing the promises made in the Introduction with the Conclusion, did we actually fulfill those promises?"

### Stage G: Polish & Compliance
- **Focus:** Harsh copy-editing and logical flow.
- *Q1 (Tone Check):* "Remove all unnecessary emphatic words (e.g., 'definitely', 'proves') and replace them with objective academic terms (e.g., 'suggests', 'indicates')."
- *Q2 (Logic Jump Check):* "If we strip out all transitional conjunctions (e.g., 'Therefore', 'Thus'), is the logical connection between paragraphs still solid?"

### Stage H: Submission & Revision
- **Focus:** De-escalation and reviewer empathy.
- *Q1 (Empathy Check):* "Roleplay as Reviewer 2. Would I feel this Response is brushing me off, or does it genuinely address my core concerns?"
- *Q2:* "If the reviewer asks for supplemental experiments that are impossible to conduct, is our alternative argumentation sufficiently convincing?"
- *Q3 (Tone Neutralization):* "Identify and eliminate any defensive or argumentative tone in the rebuttal draft."

### Stage I: Code & Implementation
- **Focus:** Robustness and reproducibility.
- *Q1:* "Are there any hardcoded paths or magic numbers in the code? Is the random seed fixed?"
- *Q2:* "If 10% of the input data turns out to be NaN, will this data pipeline fail gracefully or silently produce incorrect results?"

## Usage

This skill is injected into tasks by the `mcp-agent-capability-map.yaml` and should be called:
- By the **Reviewer Agent** (e.g., Gemini) during the `review-agent-check` phase of orchestrator runs.
- Via role-play instructions in `/paper-write`, `/study-design`, and other generative commands.
