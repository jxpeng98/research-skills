"""Stage-specific critique questions extracted from skills/self-critique.md.

Provides structured critique questions for each research workflow stage (A–I).
Used by the orchestrator to inject targeted review prompts during task-run
revision loops.
"""
from __future__ import annotations

# Task ID prefix → stage key mapping
_TASK_STAGE_MAP: dict[str, str] = {
    "A": "framing",
    "B": "literature",
    "C": "study_design",
    "D": "ethics",
    "E": "synthesis",
    "F": "manuscript",
    "G": "polishing",
    "H": "submission",
    "I": "code",
}

CRITIQUE_QUESTIONS: dict[str, list[str]] = {
    "framing": [
        "Can the current RQ be answered in a single sentence? If not, is it overlapping and too massive? How can we constrain the boundaries (population, context, time) to cut the scope in half?",
        "Is this Gap simply because no one has done it (likely too difficult or meaningless), or involves a new data/method/theoretical dividend?",
        "Who cares? If this research is completely successful, which specific scholars or domains will cite it? Why?",
        "Does the framing rely on buzzwords without clear definitions? Which core concepts are assumed but actually contested in the literature?",
        "Are the proposed boundaries (e.g., geographic, temporal) justified theoretically, or merely chosen for convenience? What bias does this introduce?",
    ],
    "literature": [
        "Are we synthesizing or just summarizing? Have we identified the contradictions and conflicts in existing literature, rather than just agreeing with them?",
        "Have we been too lenient towards the highly-cited classic papers? What are their fundamental, unacknowledged limitations?",
        "Does the current literature review naturally and irrefutably logically lead to the proposed RQ (A1) as the only logical next step?",
        "Are the search keywords structurally biased towards confirming our hypotheses? What opposing search terms (null hypothesis literature) must be added?",
        "Are we overly reliant on WEIRD (Western, Educated, Industrialized, Rich, Democratic) samples masquerading as universal findings?",
    ],
    "study_design": [
        "Assume this research ultimately fails or yields the exact opposite conclusion. What is the most likely fatal design/methodological flaw that caused it?",
        "Are we claiming causality or correlation? Do we have critical omitted confounding variables that account for the observed effect?",
        "Do our measurement instruments (proxies) accurately represent the abstract constructs defined in A3? Score this out of 10 and justify.",
        "What are the top 3 competing hypotheses, and exactly which variables/methods rule them out?",
        "Is the sample size justified by a rigorous power analysis, or based on 'rule of thumb'? What is the Minimum Detectable Effect?",
    ],
    "ethics": [
        "Are there any edge cases (like de-anonymization attacks via joining multiple datasets over time) that could lead to participant data leakage? How do we defend against this mathematically or structurally?",
        "Is the language in the informed consent form at an 8th-grade reading level, or is it filled with academic jargon?",
        "Does the research involve vulnerable populations indirectly? Even if not the primary target, could they be disproportionately affected?",
        "What is the potential dual-use nature of these findings? Can this methodology be weaponized?",
    ],
    "synthesis": [
        "Argue that the massive aggregated effect size is 100% due to publication bias, file-drawer effect, and p-hacking. How does our data formally refute this?",
        "Is the heterogeneity between studies so high that we are essentially comparing apples to oranges? Justify the decision to pool mathematically (e.g., $I^2$ threshold).",
        "How sensitive is the overall conclusion to the removal of the specific single largest or most extreme study? (Leave-one-out sensitivity).",
        "Are we trusting the reported standard errors of primary studies blindly, or detecting reporting anomalies?",
    ],
    "manuscript": [
        "Do the claims in the Discussion section drastically exceed the mathematical data support provided in the Results section?",
        "Do the numbers in the abstract specifically match the tables in the main text with 100% accuracy?",
        "Comparing the promises made in the Introduction with the Conclusion, did we actually fulfill those promises without moving the goalposts?",
        "Is the limitations section honest and self-critical, or just boilerplate text apologizing for basic boundaries?",
        "Could an independent team replicate this study using ONLY the methods text provided, without emailing the authors?",
    ],
    "polishing": [
        "Remove all unnecessary emphatic words (e.g., 'definitely', 'proves') and replace them with objective academic terms (e.g., 'suggests', 'indicates').",
        "If we strip out all transitional conjunctions (e.g., 'Therefore', 'Thus'), is the logical connection between paragraphs still solid?",
        "Are the active and passive voices mixed arbitrarily? Where the researchers acted, did they use active voice to take accountability?",
        "Is the manuscript bloated? Is there any paragraph that does not directly serve to establish the gap, methods, results, or interpretation?",
    ],
    "submission": [
        "Roleplay as Reviewer 2. Would I feel this Response is brushing me off, or does it genuinely address my core concerns?",
        "If the reviewer asks for supplemental experiments that are impossible to conduct, is our alternative argumentation sufficiently convincing and gracious?",
        "Identify and eliminate any defensive, snarky, or argumentative tone in the rebuttal draft.",
        "Did the revisions requested introduce contradictory statements in different parts of the manuscript (e.g., fixing methods but forgetting to update the abstract)?",
    ],
    "code": [
        "Are there any hardcoded paths or magic numbers in the code? Is the random seed fixed globally for ALL stochastic operations?",
        "If 10% of the input data turns out to be NaN, will this data pipeline fail gracefully or silently produce incorrect aggregated results?",
        "Are the computational environment dependencies explicitly pinned (e.g., requirements.txt, Dockerfile) to prevent 'works on my machine' syndrome?",
        "Is there an unacknowledged O(N^2) or worse operation that will cause the code to hang if the dataset size scales 10x?",
    ],
}


def get_stage_for_task(task_id: str) -> str | None:
    """Map a canonical task ID (e.g. 'F3', 'A1') to its stage key."""
    prefix = task_id.strip().upper()[:1]
    return _TASK_STAGE_MAP.get(prefix)


def get_critique_questions(task_id: str) -> list[str]:
    """Return critique questions for the given task's stage.

    Returns an empty list if the task ID maps to an unknown stage.
    """
    stage = get_stage_for_task(task_id)
    if not stage:
        return []
    return list(CRITIQUE_QUESTIONS.get(stage, []))
