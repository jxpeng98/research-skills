---
id: reporting-checker
stage: G_compliance
description: "Validate reporting guideline completeness for target study type (CONSORT, STROBE, COREQ, SRQR, TRIPOD, etc.)."
inputs:
  - type: Manuscript
    description: "Draft manuscript"
  - type: DesignSpec
    description: "Study design for guideline selection"
outputs:
  - type: ReportingChecklist
    artifact: "reporting_checklist.md"
constraints:
  - "Must select appropriate guideline based on study design"
  - "Must reference specific manuscript sections for each item"
  - "Must distinguish must-fix from nice-to-have gaps"
failure_modes:
  - "No standard guideline exists for the study type"
  - "Multiple guidelines applicable with conflicting requirements"
  - "Checklist items marked 'present' but content is actually insufficient"
tools: [filesystem, reporting-guidelines]
tags: [compliance, reporting, CONSORT, STROBE, COREQ, SRQR, TRIPOD, guidelines]
domain_aware: true
---

# Reporting Checker Skill

Check whether a manuscript is complete and aligned with the appropriate reporting guideline, producing a structured "what's missing" action list with specific manuscript locations.

## Related Task IDs

- `G1` (reporting completeness)

## Output (contract path)

- `RESEARCH/[topic]/reporting_checklist.md`

## When to Use

- Before submission (final quality assurance pass)
- Before sharing a preprint / conference camera-ready
- When converting working notes into a paper draft
- After major revisions (verify nothing was dropped)

## Process

### Step 1: Determine Study Design and Select Guideline

Pick the closest match and confirm with the target venue's instructions:

| Study Design | Guideline | Items | Official Reference |
|-------------|-----------|-------|--------------------|
| Systematic review / meta-analysis | PRISMA 2020 | 27 | Use **prisma-checker** skill instead |
| Randomized controlled trial | CONSORT 2010 | 25 | consort-statement.org |
| Observational — cohort | STROBE | 22 | strobe-statement.org |
| Observational — case-control | STROBE | 22 | strobe-statement.org |
| Observational — cross-sectional | STROBE | 22 | strobe-statement.org |
| Qualitative — interview / focus group | COREQ | 32 | Tong et al. 2007 |
| Qualitative — broader (case study, ethnography, process) | SRQR | 21 | O'Brien et al. 2014 |
| Prediction model development/validation | TRIPOD | 22 | tripod-statement.org |
| Mixed methods | GRAMMS | 7 | O'Cathain et al. 2008 |
| Case report / case series | CARE | 13 | care-statement.org |
| Diagnostic accuracy | STARD | 30 | stard-statement.org |

> If the design is mixed, apply the dominant design's checklist and add the secondary guideline as an addendum. Document the selection rationale.

### Step 2: Apply Checklist Item-by-Item

For each guideline item, check the manuscript against the **core requirements** listed below. Mark each as ✅ Present / ⚠️ Partial / ❌ Missing / N/A.

#### CONSORT 2010 — 25 Items (RCTs)

| # | Section | Item | What to check |
|---|---------|------|---------------|
| 1a | Title | Identification as RCT | Title contains "randomised" / "randomized" |
| 1b | Abstract | Structured summary | PICO + key results + conclusions |
| 2a | Introduction | Scientific background | Problem + existing evidence |
| 2b | Introduction | Specific objectives or hypotheses | Clearly stated |
| 3a | Methods | Trial design | Parallel, crossover, factorial, etc. |
| 3b | Methods | Changes to methods after trial start | Deviations documented |
| 4a | Methods | Eligibility criteria | Specific inclusion/exclusion |
| 4b | Methods | Settings and locations | Where data collected |
| 5 | Methods | Interventions | Sufficient detail for replication |
| 6a | Methods | Outcomes | Primary and secondary, fully defined |
| 6b | Methods | Changes to outcomes | After trial commenced |
| 7a | Methods | Sample size | How determined + assumptions |
| 7b | Methods | Interim analyses | If applicable |
| 8a | Methods | Randomization — sequence generation | Method described |
| 8b | Methods | Randomization — type | Block, stratified, etc. |
| 9 | Methods | Allocation concealment | Mechanism described |
| 10 | Methods | Implementation | Who generated, enrolled, assigned |
| 11a | Methods | Blinding | Who was blinded |
| 11b | Methods | Similarity of interventions | If blinded |
| 12a | Methods | Statistical methods | Primary outcome analysis |
| 12b | Methods | Additional analyses | Subgroup, adjusted |
| 13a | Results | Participant flow — diagram | CONSORT flow diagram |
| 13b | Results | Losses and exclusions | With reasons |
| 14a | Results | Dates | Recruitment and follow-up |
| 14b | Results | Why trial ended | Or was stopped |
| 15 | Results | Baseline data | Table of demographics |
| 16 | Results | Numbers analysed | ITT or per-protocol |
| 17a | Results | Outcomes and estimation | Effect sizes + CI |
| 17b | Results | Binary outcomes | Absolute and relative sizes |
| 18 | Results | Ancillary analyses | Subgroups, interactions |
| 19 | Results | Harms | All important adverse events |
| 20 | Discussion | Limitations | Sources of bias, imprecision |
| 21 | Discussion | Generalisability | External validity |
| 22 | Discussion | Interpretation | Consistent with results |
| 23 | Other | Registration | Trial registry number |
| 24 | Other | Protocol | Where accessible |
| 25 | Other | Funding | Source and role |

#### STROBE — 22 Items (Observational Studies)

| # | Section | Item | What to check |
|---|---------|------|---------------|
| 1 | Title/Abstract | Informative title + structured abstract | Study design in title |
| 2 | Introduction | Scientific background + rationale | |
| 3 | Introduction | Objectives + pre-specified hypotheses | |
| 4 | Methods | Study design | Key elements stated early |
| 5 | Methods | Setting | Locations, dates, periods |
| 6 | Methods | Participants | Eligibility, sources, selection, follow-up |
| 7 | Methods | Variables | Outcomes, exposures, confounders clearly defined |
| 8 | Methods | Data sources / measurement | Each variable, comparability |
| 9 | Methods | Bias | Efforts to address potential bias |
| 10 | Methods | Study size | How arrived at, power |
| 11 | Methods | Quantitative variables | Groupings, cutpoints justified |
| 12 | Methods | Statistical methods | Confounders, interactions, missing data, sensitivity |
| 13 | Results | Participants | Numbers at each stage, non-participation reasons |
| 14 | Results | Descriptive data | Demographics, exposures, follow-up time |
| 15 | Results | Outcome data | Number of events or summary measures |
| 16 | Results | Main results | Unadjusted + adjusted estimates with CI |
| 17 | Results | Other analyses | Subgroups, interactions, sensitivity |
| 18 | Discussion | Key results | Summary |
| 19 | Discussion | Limitations | Bias, direction, magnitude |
| 20 | Discussion | Interpretation | Cautious, considering objectives, multiplicity |
| 21 | Discussion | Generalisability | External validity |
| 22 | Other | Funding | Source |

#### COREQ — 32 Items (Qualitative: Interviews & Focus Groups)

| # | Domain | Item | What to check |
|---|--------|------|---------------|
| 1 | Team | Interviewer/facilitator | Who conducted |
| 2 | Team | Credentials | Qualifications |
| 3 | Team | Occupation | At time of study |
| 4 | Team | Gender | Of researcher |
| 5 | Team | Experience/training | In qualitative research |
| 6 | Relationship | Relationship established | Prior to study |
| 7 | Relationship | Participant knowledge of the interviewer | What participants knew about researcher's goals |
| 8 | Relationship | Interviewer characteristics | Bias, assumptions reported |
| 9 | Design | Methodological orientation | Grounded theory, phenomenology, etc. |
| 10 | Design | Sampling | How and why participants selected |
| 11 | Design | Method of approach | How contacted |
| 12 | Design | Sample size | Number of participants |
| 13 | Design | Non-participation | Reasons |
| 14 | Setting | Setting of data collection | Where |
| 15 | Setting | Presence of non-participants | Others present? |
| 16 | Setting | Description of sample | Demographics, relevant data |
| 17 | Collection | Interview guide | Questions, probes, prompts |
| 18 | Collection | Repeat interviews | Were they done? |
| 19 | Collection | Audio/visual recording | Method used |
| 20 | Collection | Field notes | Taken during/after? |
| 21 | Collection | Duration | Of interviews |
| 22 | Collection | Data saturation | Was it discussed? |
| 23 | Collection | Transcripts returned | To participants for comment? |
| 24 | Analysis | Number of data coders | How many |
| 25 | Analysis | Description of coding tree | Provided? |
| 26 | Analysis | Derivation of themes | How identified |
| 27 | Analysis | Software | Which program used |
| 28 | Analysis | Participant checking | Feedback on findings? |
| 29 | Reporting | Quotations presented | Data to support themes? |
| 30 | Reporting | Data and findings consistent | |
| 31 | Reporting | Clarity of major themes | |
| 32 | Reporting | Clarity of minor themes | |

#### SRQR — 21 Items (Broader Qualitative Research)

| # | Section | Item | What to check |
|---|---------|------|---------------|
| 1 | Title | Title | Concise, nature of study |
| 2 | Abstract | Problem, purpose, approach, findings, contribution | |
| 3 | Problem | Problem formulation | Relevance |
| 4 | Purpose | Purpose or research question | |
| 5 | Approach | Qualitative approach and paradigm | Named, justified |
| 6 | Methods | Researcher characteristics/reflexivity | |
| 7 | Methods | Context | Setting described |
| 8 | Methods | Sampling strategy | Documented, justified |
| 9 | Methods | Ethical issues | IRB, consent |
| 10 | Methods | Data collection methods | Described in detail |
| 11 | Methods | Data collection instruments/technologies | |
| 12 | Methods | Units of study | Number, relevant characteristics |
| 13 | Methods | Data processing | Transcription, verification |
| 14 | Methods | Data analysis | Process and procedures |
| 15 | Methods | Techniques to enhance trustworthiness | |
| 16 | Results | Synthesis and interpretation | |
| 17 | Results | Links to empirical data | Quotes, excerpts, evidence |
| 18 | Discussion | Integration with prior work, theory | |
| 19 | Discussion | Limitations | |
| 20 | Other | Conflicts of interest | |
| 21 | Other | Funding | |

#### TRIPOD — 22 Items (Prediction Models)

| # | Section | Item | What to check |
|---|---------|------|---------------|
| 1 | Title | D/V | Development, validation, or both |
| 2 | Abstract | Summary | Key elements |
| 3a | Intro | Background | Context, rationale |
| 3b | Intro | Objectives | Development, validation, or both |
| 4a | Methods | Source of data | Design, setting |
| 4b | Methods | Dates | Study period |
| 5a | Methods | Participants | Eligibility |
| 5b | Methods | Treatment | If applicable |
| 6a | Methods | Outcome | Definition, blinding |
| 6b | Methods | Outcome timing | |
| 7a | Methods | Predictors | Definition, timing |
| 7b | Methods | Predictor assessment | Blinding |
| 8 | Methods | Sample size | Derivation |
| 9 | Methods | Missing data | Handling |
| 10a | Methods | Model development | Modeling approach |
| 10b | Methods | Model specification | Predictor selection |
| 10d | Methods | Validation | Internal, external |
| 11 | Methods | Risk groups | If created |
| 12 | Results | Flow diagram | |
| 13 | Results | Demographics | Key characteristics |
| 14 | Results | Model development | Unadjusted + adjusted |
| 15 | Results | Model performance | Discrimination + calibration |
| 16 | Results | Model updating | If done |
| 17 | Discussion | Limitations | |
| 18 | Discussion | Interpretation | Cautious |
| 19a | Other | Supplementary | Model equation or code |
| 19b | Other | Availability | Data, code |
| 20 | Other | Funding | |
| 21 | Other | Conflicts | |
| 22 | Other | Registration | |

### Step 3: Grade Each Gap

For each ❌ or ⚠️ item, classify:

| Severity | Meaning | Action |
|----------|---------|--------|
| **Must-fix** | Reviewer will raise it; venue may desk-reject | Block submission |
| **Should-fix** | Improves rigor; reduces revision round | Fix before submission |
| **Nice-to-have** | Best practice but not venue-required | Fix if time allows |

### Step 4: Generate Fix Action List

For each gap, specify exactly what content to add and where:

```
Item #7 (STROBE): Variables — ❌ Missing
Location: Methods § 3.2
Action: Add table with outcome variable definition, measurement,
        exposure/predictor operationalization, and confounder list
Priority: Must-fix
```

## Quality Bar

The reporting checklist is **ready** when:

- [ ] Correct guideline selected and justified (with venue confirmation note)
- [ ] Every checklist item mapped to a specific manuscript section or marked N/A with justification
- [ ] All must-fix items have specific action instructions
- [ ] No item marked "present" without verifying the content is sufficient (not just mentioned)

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Marking "present" because the word appears | Section exists but content is insufficient | Check substance, not just existence |
| Skipping N/A justification | Reviewer questions why item omitted | Always document why N/A |
| Using wrong guideline for study type | Misalignment wastes effort | Confirm design → guideline mapping with venue |
| Ignoring guideline extensions | STROBE has cohort/case-control/cross-sectional variants | Use the design-specific extension |

## Minimal Output Format

```markdown
# Reporting Checklist

## Guideline: [CONSORT / STROBE / COREQ / SRQR / TRIPOD]
- Study design: ...
- Venue: ...
- Justification: ...

## Checklist

| # | Item | Status | Manuscript location | Gap / Action | Priority |
|---|------|--------|-------------------|--------------|-----------| 
| 1 | ... | ✅/⚠️/❌/N/A | § X.Y | ... | Must-fix / Should-fix / N/A |

## Action List (prioritized)

### Must-fix
1. ...

### Should-fix
1. ...
```
