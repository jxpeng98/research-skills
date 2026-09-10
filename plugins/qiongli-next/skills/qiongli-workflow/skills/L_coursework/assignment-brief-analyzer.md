---
id: assignment-brief-analyzer
stage: L_coursework
description: "Parse coursework, capstone, and dissertation assignment briefs into constraints, task type, missing information, and integrity boundaries."
inputs:
  - type: UserQuery
    description: "Assignment prompt, brief, handbook excerpt, rubric text, or user summary"
outputs:
  - type: AssignmentBrief
    artifact: "assignment/brief.md"
  - type: AcademicIntegrityNotes
    artifact: "assignment/academic_integrity_notes.md"
constraints:
  - "Must not invent module rules, rubric criteria, source requirements, AI policy, or institutional rules"
  - "Must distinguish coursework type from academic subject routing"
  - "Must mark timed exams, quizzes, and assessed problem sets as concept-support only"
failure_modes:
  - "Treats a course assignment as a journal manuscript"
  - "Infers missing rubric or AI policy as if supplied by the institution"
  - "Allows drafting before user-supplied personal, empirical, or placement facts exist"
tools: [filesystem]
tags: [coursework, assignment-brief, rubric, learning-outcomes, academic-integrity]
domain_aware: true
---

# Assignment Brief Analyzer Skill

Parse a student-facing assignment brief into auditable constraints before any coursework drafting starts.

## Purpose

Create `RESEARCH/[topic]/assignment/brief.md` and the first `assignment/academic_integrity_notes.md` so downstream coursework or dissertation tasks preserve the task prompt, level, word count, allowed sources, missing rules, and permitted-assistance boundary.

## When to Use

- When the user provides an assignment brief, coursework prompt, module task, rubric, learning outcomes, dissertation handbook, or capstone brief.
- Before drafting coursework prose when the task requirements are not already captured.
- When the user asks what an assignment is asking them to do.

## Inputs

- `UserQuery`: assignment brief, prompt, rubric, handbook excerpt, or user summary.
- Optional current draft or notes supplied by the user.
- If the brief, rubric, or AI policy is missing or insufficient, record the
  exact gap and ask for the source before treating it as an assignment rule.

## Process

1. Extract assignment title, module/program context, level, deadline, word count, citation style, source requirements, and output format.
2. Classify task type as essay, report, case analysis, reflective writing, literature review, research proposal, presentation, portfolio, lab or methods report, capstone project, or dissertation component.
3. Record missing requirements rather than inferring them.
4. Identify academic-integrity constraints, AI-policy status, and requests that must be blocked.
5. Keep subject routing separate from coursework classification.

## Output Contract

Write `RESEARCH/[topic]/assignment/brief.md` with:

- assignment identity
- task type
- supplied requirements
- inferred-but-unconfirmed risks
- missing information
- routing recommendation

Write or update `RESEARCH/[topic]/assignment/academic_integrity_notes.md` with:

- supplied AI policy
- missing AI policy fields
- user-supplied facts
- blocked fabrication or grade-guarantee requests

## Quality Bar

- [ ] The brief separates supplied requirements from missing or inferred information.
- [ ] The coursework type does not override subject routing.
- [ ] AI policy and permitted assistance are visible.
- [ ] Personal, empirical, or placement claims are flagged when user material is missing.
- [ ] Each finding points to supplied brief text; interpretation states what
  that text requires; implication names the planning action without upgrading
  unconfirmed guidance into a rule.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---|---|---|
| Inventing rubric criteria | Creates false compliance | Mark rubric as missing and ask for it |
| Treating case assignment as business research | Over-activates subject routing | Keep type and subject decisions separate |
| Drafting reflection without user facts | Fabricates lived experience | Ask for user-supplied experience first |
