---
id: proposal-writer
stage: F_writing
description: "Draft research proposals, opening reports, prospectuses, and study-plan documents from framing and design artifacts."
inputs:
  - type: RQSet
    description: "Research questions and objectives"
  - type: GapAnalysis
    description: "Literature gap and novelty rationale"
  - type: TheoreticalFramework
    description: "Theory, constructs, and conceptual model"
  - type: ContributionStatement
    description: "Expected contribution and significance"
  - type: DesignSpec
    description: "Study design and feasibility constraints"
  - type: AnalysisPlan
    description: "Planned analytic strategy"
  - type: DataManagementPlan
    description: "Data governance and reproducibility plan"
  - type: VenueAnalysis
    description: "Target program, venue, department, or funder expectations"
    required: false
outputs:
  - type: ResearchProposal
    artifact: "proposal/research_proposal.md"
constraints:
  - "Must separate planned work from completed findings"
  - "Must mark missing evidence, citations, access permissions, or institutional requirements"
  - "Must distinguish research proposal/opening report content from preregistration and manuscript drafting"
failure_modes:
  - "Proposal overclaims results before data collection or analysis"
  - "Opening report lacks a feasible timeline, risk plan, or method justification"
  - "Narrative invents citations, data access, sample sizes, or institutional rules"
tools: [filesystem]
tags: [writing, proposal, opening-report, prospectus, study-plan, drafting]
domain_aware: true
---

# Proposal Writer Skill

Draft research proposals, opening reports, prospectuses, and study-plan documents from existing framing, literature, and design artifacts.

## Purpose

Create an approval-ready research plan that explains what will be studied, why it matters, how it will be executed, and what risks remain. This is not a preregistration, not a systematic review protocol, and not a manuscript draft.

## When to Use

- Before a thesis, dissertation, grant, capstone, or course project needs a proposal, prospectus, research plan, or opening report.
- When a user asks for `research proposal`, `opening report`, `开题报告`, `研究计划书`, or `prospectus`.
- After A-stage framing and at least a draft C-stage design exist, but before results-focused manuscript drafting.
- When committee, supervisor, department, venue, or funder approval depends on feasibility, contribution, methods, and schedule clarity.

## Inputs

- `RQSet`: `RESEARCH/[topic]/framing/research_question.md`
- `GapAnalysis`: `RESEARCH/[topic]/gap_analysis.md`
- `TheoreticalFramework`: `RESEARCH/[topic]/theoretical_framework.md`
- `ContributionStatement`: `RESEARCH/[topic]/framing/contribution_statement.md`
- `DesignSpec`: `RESEARCH/[topic]/study_design.md`
- `AnalysisPlan`: `RESEARCH/[topic]/analysis_plan.md`
- `DataManagementPlan`: `RESEARCH/[topic]/data_management_plan.md`
- Optional `VenueAnalysis`: `RESEARCH/[topic]/framing/venue_analysis.md`

If inputs are missing or insufficient, write explicit gap notes in `RESEARCH/[topic]/context/gap_notes.md` and keep the proposal section marked as unresolved. Do not invent citations, data access, sample sizes, statistical results, ethics approval, supervisor requirements, or institutional rules.

## Process

### Step 1: Classify the Proposal Context

Select the output emphasis before drafting:

| Context | Main reviewer question | Emphasis |
|---|---|---|
| Thesis / dissertation opening report | Is this feasible and worth approving? | RQ, theory, method, timeline, risks |
| Grant / fellowship proposal | Why fund this now? | Significance, novelty, deliverables, feasibility |
| Course / capstone proposal | Can this be completed on schedule? | Scope, method, milestones, available data |
| Journal registered report / prospectus | Is the logic confirmatory enough? | Design logic, hypotheses, analysis plan |

Use the user's language. For Chinese `开题报告`, preserve the canonical artifact path but write headings and prose in Chinese when requested.

### Step 2: Build the Proposal Argument

Write the proposal as a plan, not as completed research:

1. Establish the problem and why it matters.
2. Identify the literature gap and why existing work cannot answer the RQ.
3. State the RQ/objectives/hypotheses or propositions.
4. Explain the theoretical framework and expected contribution.
5. Justify design choices against feasible alternatives.
6. Specify data, instruments, analysis, ethics, and data management.
7. Show feasibility through timeline, milestones, risks, and fallback options.

Keep finding, interpretation, and implication distinct:
- `finding`: expected or prior evidence only; label planned findings as anticipated.
- `interpretation`: what the study will be able to infer if evidence supports the plan.
- `implication`: what the contribution would mean, bounded by the proposed design.

### Step 3: Draft From Template

Use `templates/research-proposal-template.md` and write to:

`RESEARCH/[topic]/proposal/research_proposal.md`

Required sections:
- project information
- background and problem statement
- research gap and literature positioning
- theoretical framework
- research questions, objectives, hypotheses, or propositions
- expected contribution and significance
- study design and methods
- analysis plan
- ethics, data management, and reproducibility
- feasibility, timeline, and milestones
- risks, limitations, and fallback plan
- references and unresolved evidence gaps

### Step 4: Approval Readiness Check

Before finalizing, verify:

| Check | Pass condition |
|---|---|
| Alignment | RQ, theory, method, data, and contribution describe the same study |
| Feasibility | Data access, sampling, timeline, tools, and ethics are realistic |
| Evidence support | Literature claims have citations or marked gaps |
| Method specificity | Design and analysis are concrete enough for review |
| Proposal voice | Planned work is not written as completed findings |
| Risk plan | Major threats have mitigations or fallback options |

## Output Contract

- `ResearchProposal`: write `RESEARCH/[topic]/proposal/research_proposal.md`.
- Use `templates/research-proposal-template.md`.
- Separate planned finding, interpretation, and implication claims.
- Do not invent citations, data, sample sizes, statistical results, ethics approval, supervisor comments, reviewer comments, or institutional requirements.
- Apply `references/academic-output-rubric.md` before finalizing scholarly prose.

## Quality Bar

- [ ] The proposal states a clear research problem, RQ, and expected contribution.
- [ ] The gap is supported by literature or marked as an evidence gap.
- [ ] The design, data, analysis, ethics, and data management sections are mutually consistent.
- [ ] The timeline and feasibility plan identify dependencies and fallback options.
- [ ] All missing inputs are visible as gap notes rather than silently filled.
- [ ] Language matches the requested context: research proposal, prospectus, opening report, or 开题报告.

## Common Pitfalls

| Pitfall | Problem | Fix |
|---|---|---|
| Treating proposal as a manuscript | Implies findings already exist | Use planned-work language and label expected contributions |
| Treating proposal as preregistration | Over-focuses on decision rules and omits rationale | Keep preregistration details summarized; link to `preregistration.md` if present |
| Vague feasibility section | Committee cannot judge completion risk | Add timeline, access dependencies, and fallback plan |
| Invented institutional rules | Creates compliance risk | Mark requirement as missing and ask for the program/funder rule |
| Citation-light gap statement | Novelty claim is unsupported | Add citations or list the gap under unresolved evidence |
