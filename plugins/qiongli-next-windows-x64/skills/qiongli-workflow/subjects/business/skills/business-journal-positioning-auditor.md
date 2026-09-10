---
id: business-journal-positioning-auditor
stage: C_design
description: "Audit business and management papers for theory contribution, construct clarity, setting fit, and doctoral-level journal positioning."
inputs:
  - type: DesignSpec
    description: "Research question, phenomenon, theory, constructs, data source, and method"
  - type: LiteratureMap
    description: "Target conversation, boundary conditions, and competing explanations"
  - type: Manuscript
    description: "Draft introduction, theory, methods, findings, and discussion when available"
outputs:
  - type: BusinessJournalPositioningAudit
    artifact: "analysis/business_journal_positioning_audit.md"
constraints:
  - "Must distinguish business phenomenon, theory contribution, empirical setting, and managerial implication"
  - "Must evaluate doctoral-level journal contribution rather than course-assignment completeness"
  - "Must surface construct, sampling, identification, or qualitative transparency gaps before recommending submission"
failure_modes:
  - "Paper describes an interesting business setting but lacks theory contribution"
  - "Constructs are named but not grounded in a management literature stream"
  - "Evidence does not support the level of analysis or mechanism claimed"
tools: [filesystem]
tags: [business, management, strategy, organization, journal-positioning]
domain_aware: true
---

# Business Journal Positioning Auditor Skill

Audit whether a business, management, strategy, organization, marketing, or operations manuscript is positioned for a doctoral-level journal contribution.

## Purpose

Prevent a business paper from reading like a descriptive case, consulting report, or class project when the target is a publishable academic journal manuscript.

## When to Use

- Before finalizing the introduction, theory section, or research design.
- When reviewer risk centers on weak theory contribution, construct drift, thin setting justification, or unclear level of analysis.
- When a project started from undergraduate or master's research needs to be raised to doctoral-level journal standards.

## Inputs

- `DesignSpec`: phenomenon, theory, constructs, unit of analysis, sample, data source, method, and target venue.
- `LiteratureMap`: focal business literature stream, adjacent streams, core constructs, and unresolved debate.
- `Manuscript`: introduction, theory, methods, findings, and discussion when available.
- `DomainProfile`: load `skills/domain-profiles/business-management.yaml` when available.

If any input is missing, write a blocked-check note under `RESEARCH/[topic]/context/gap_notes.md` and do not invent the missing contribution logic.

## Process

1. Classify the manuscript as theory-building, theory-testing, mixed-method, qualitative, quantitative, review, or method paper.
2. State the target business literature conversation and the claimed theory contribution in one sentence each.
3. Check whether constructs, level of analysis, boundary conditions, and mechanism match the evidence.
4. Audit whether the method and transparency appendix meet doctoral-level journal expectations for the target venue.
5. Identify the strongest rival framing that a reviewer could use to reject the paper.
6. Produce a pass / revise / blocked verdict with specific revisions.

## Output Contract

Write `RESEARCH/[topic]/analysis/business_journal_positioning_audit.md`:

```markdown
# Business Journal Positioning Audit

## Target Conversation
- Journal family:
- Literature stream:
- Unresolved debate:

## Theory Contribution
| Claim | Current evidence | Contribution risk | Required revision |
|---|---|---|---|

## Construct And Setting Fit
| Construct | Level of analysis | Evidence source | Validity risk |
|---|---|---|---|

## Method Transparency
| Requirement | Current status | Gap |
|---|---|---|

## Reviewer Rejection Risk
- Most likely desk-reject reason:
- Strongest rival framing:
- Required positioning change:

## Verdict
- Decision: pass / revise / blocked
- Required changes:
```

## Quality Bar

- The audit names the business theory contribution, not only a practical implication.
- Constructs, setting, sample, and level of analysis are aligned.
- Qualitative work documents sampling, coding, negative cases, and evidence chain; quantitative work documents construct validity, model fit, robustness, and inference.
- The final verdict holds undergraduate-and-above projects to doctoral-level journal contribution standards.
