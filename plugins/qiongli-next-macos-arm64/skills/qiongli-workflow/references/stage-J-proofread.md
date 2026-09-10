# Stage J — Scholarly Proofreading (J1–J4)

Choose the requested language or source-comparison task. A supplied passage can
be checked directly; a formal Stage J run retains its canonical outputs below.
Task IDs and legacy artifact names remain stable. They do not establish AI
authorship, plagiarism clearance or detector performance.

## Canonical outputs (contract paths)

- `J1` → `proofread/ai_detection_report.md`
- `J2` → `proofread/humanized_manuscript.md`
- `J3` → `proofread/similarity_report.md`, `proofread/citation-risk-report.md`
- `J4` → `proofread/proofread_checklist.md`

## Scope, integrity and stopping

Preserve meaning, numbers, citations, technical terms, claim strength and required
AI disclosure. Correct expression without adding author experiences, findings,
examples or intentional mistakes. Flag unsupported claims for the appropriate
review instead of silently strengthening or weakening them during proofreading.

Use one agent for ordinary work. Check the requested material once, fix actual
issues and verify affected passages; unchanged correct text needs no rewrite.
No detector percentage, self-confidence threshold, style quota or vote count is
an acceptance criterion. Never optimize text to evade authorship/integrity checks.
When independent review is explicitly required, use
`skills/Z_cross_cutting/model-collaborator.md`; unavailable independent reviewers
leave that requirement unresolved. Formal review counts and approval gates remain.

For direct edits, return the passage or findings in chat. For formal work, record
source scope, changes and missing evidence in the required artifacts. Registered
project writes use preview/approval/CAS. Stop after the selected task; optional
later J tasks are not automatically scheduled.

## J1 — Language-Pattern Audit

Use `skills/J_proofread/ai-fingerprint-scanner.md` to identify repetition,
empty qualifiers, unclear structure or imprecise wording that harms this text.
Patterns are contextual prompts for inspection, not evidence of who wrote it.
Keep valid disciplinary phrasing. Report actual locations, excerpts, impact and
suggested corrections; a clean report may have zero findings. State the reviewed
scope and do not claim a full-manuscript scan from an excerpt.

## J2 — Scholarly Voice Revision

Use `skills/J_proofread/human-voice-rewriter.md` for requested expression changes.
Reuse supplied passages and any available J1 findings. Select the smallest edit
that resolves a demonstrated issue, verify the changed text against the source,
and retain correctly expressed passages. A formal J2 artifact integrates changes
into the manuscript; a bounded edit returns only the requested material.
Use `references/scholarly-voice.md` for natural English/Chinese expression and
paragraph continuity. Language-only changes preserve the existing claim IDs,
evidence ledger and graph; ambiguous reasoning is flagged, not rewritten as fact.

## J3 — Source-Overlap and Attribution Check

Use `skills/J_proofread/similarity-checker.md` to compare available sources with
the draft. Record both source and manuscript locators, overlap type and whether
quotation, attribution or a faithful paraphrase is needed. Standard terminology
and conventional methods wording alone do not establish misconduct. Missing
source text means the comparison is unavailable, not clean. Report the checked
corpus and gaps; do not invent percentages or infer a whole-corpus clearance.
Apply `references/citation-risk-policy.md` for the formal citation-risk output.

## J4 — Final Proofread

Use `skills/J_proofread/final-proofreader.md` for grammar, consistency and the
requested style. Verify acronyms, figures, tables, equations and citations only
against available materials. Retain the original voice and valid style choices;
record inaccessible targets and venue rules as unverified. A formal checklist
contains corrections, style decisions, reviewed scope and remaining items.
