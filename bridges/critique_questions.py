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
        "Is this gap simply because no one has done it (likely too difficult or meaningless), or does it involve a new data/method dividend?",
        "Who cares? If this research is completely successful, which specific scholars or domains will cite it? Why?",
    ],
    "literature": [
        "Are we synthesizing or just summarizing? Have we identified the contradictions and conflicts in existing literature?",
        "Have we been too lenient towards the highly-cited classic papers? What are their fundamental limitations?",
        "Does the current literature review naturally and irrefutably lead to the proposed RQ?",
        "Are the search keywords structurally biased towards confirming our hypotheses? What opposing search terms must be added?",
    ],
    "study_design": [
        "Assume this research ultimately fails or yields the exact opposite conclusion. What is the most likely fatal design flaw that caused it?",
        "Are we claiming causality or correlation? Do we have critical omitted variables?",
        "Do our measurement instruments (proxies) accurately represent the abstract constructs? Score out of 10 and justify.",
        "What are the competing hypotheses, and how are we designing the study to rule them out?",
    ],
    "ethics": [
        "Are there any edge cases (like de-anonymization attacks via joining multiple datasets) that could lead to participant data leakage? How do we defend against this?",
        "Is the language in the informed consent form at an 8th-grade reading level, or is it filled with academic jargon?",
    ],
    "synthesis": [
        "Argue that the massive aggregated effect size is 100% due to publication bias. How does our data refute this?",
        "Is the heterogeneity so high that we are essentially comparing apples to oranges? Justify the decision to pool.",
    ],
    "manuscript": [
        "Do the claims in the Discussion section drastically exceed the data support provided in the Results section?",
        "Do the numbers in the abstract match the tables in the main text with 100% accuracy?",
        "Comparing the promises made in the Introduction with the Conclusion, did we actually fulfill those promises?",
    ],
    "polishing": [
        "Remove all unnecessary emphatic words (e.g., 'definitely', 'proves') and replace them with objective academic terms (e.g., 'suggests', 'indicates').",
        "If we strip out all transitional conjunctions (e.g., 'Therefore', 'Thus'), is the logical connection between paragraphs still solid?",
    ],
    "submission": [
        "Roleplay as Reviewer 2. Would I feel this Response is brushing me off, or does it genuinely address my core concerns?",
        "If the reviewer asks for supplemental experiments that are impossible to conduct, is our alternative argumentation sufficiently convincing?",
        "Identify and eliminate any defensive or argumentative tone in the rebuttal draft.",
    ],
    "code": [
        "Are there any hardcoded paths or magic numbers in the code? Is the random seed fixed?",
        "If 10% of the input data turns out to be NaN, will this data pipeline fail gracefully or silently produce incorrect results?",
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
