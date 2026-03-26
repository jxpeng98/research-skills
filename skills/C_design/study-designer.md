---
id: study-designer
stage: C_design
description: "Design empirical, qualitative, or mixed-methods studies including sampling strategy, instruments, procedures, analysis plans, and protocol lock-in."
inputs:
  - type: RQSet
    description: "Research questions and hypotheses"
  - type: TheoreticalFramework
    description: "Theoretical framework for operationalization"
    required: false
outputs:
  - type: DesignSpec
    artifact: "study_design.md"
  - type: AnalysisPlan
    artifact: "analysis_plan.md"
  - type: DataManagementPlan
    artifact: "data_management_plan.md"
  - type: Instruments
    artifact: "instruments/"
  - type: Preregistration
    artifact: "preregistration.md"
constraints:
  - "Must justify design choice against alternatives"
  - "Must specify design-appropriate sampling adequacy logic (power/MDE for quant; saturation/theoretical sampling for qual)"
  - "Must specify missingness handling or evidence-gap handling appropriate to the design"
failure_modes:
  - "Sampling adequacy logic is missing or mismatched to the design"
  - "Design-method mismatch with research questions"
tools: [filesystem, metadata-registry]
tags: [design, sampling, measures, analysis-plan, preregistration]
domain_aware: true
---

# Study Designer Skill

Design an empirical, qualitative, or mixed-methods study from a research question and constraints, producing a protocol-style design document, analysis plan, instruments skeleton, and optional preregistration draft.

## When to Use

- After `/find-gap` and `/build-framework`, when you want to run an empirical, qualitative, or mixed-methods study.
- When you have a topic + tentative RQs/hypotheses but no concrete methodology yet.
- When you want to write a fully qualitative paper and need the design, coding logic, and evidence plan to be explicit before drafting.

## Granularity Boundary

`study-designer` owns the design package as a whole:
- design choice rationale
- construct-to-measure mapping
- sampling and procedure logic
- analysis summary handoff
- governance and preregistration pointers

Do not split these into separate top-level skills unless a new artifact path and direct pipeline dependency are required.

## Inputs (Ask / Collect)

1. **Research question(s)** and whether the goal is *causal*, *descriptive*, *predictive*, *explanatory*, or *interpretive*
2. **Domain constraints**: data access, population access, timeline, budget, tools
3. **Unit of analysis**: individuals / teams / orgs / posts / countries / etc.
4. **Ethics constraints**: minors, sensitive data, protected classes, high-risk contexts
5. **Target venue** (optional): affects reporting norms and word limits

## Process

### Step 1: Choose Study Type (Fit-to-Question)

Pick the simplest design that can answer the RQ credibly. Use this decision matrix:

#### Design Selection Matrix

| Research Goal | Best Design | When Feasible? | Threats to Address | Alternative if Not Feasible |
|--------------|-------------|----------------|-------------------|-----------------------------|
| **Causal effect of intervention** | Randomized controlled trial (RCT) | Randomization possible; ethical | Attrition, compliance, spillover | Quasi-experimental |
| **Causal effect, no randomization** | Quasi-experimental (DID, RDD, IV, PSM) | Natural variation / policy shock | Parallel trends, instrument validity | Observational with strong controls |
| **Causal effect over time** | Panel / longitudinal | Repeated observations available | Time-varying confounders, attrition | Repeated cross-section |
| **Association / prevalence** | Cross-sectional survey | Large accessible population | Common method bias, endogeneity | Secondary data analysis |
| **Prediction / classification** | Predictive modeling / ML | Large dataset; clear outcome | Overfitting, generalization | Traditional regression |
| **Process / mechanism / meaning** | Qualitative (interviews, ethnography, case study) | Access to informants/sites; depth > breadth | Researcher bias, limited generalizability | Mixed methods |
| **How and why of a phenomenon** | Case study (single or multiple) | Rich, bounded case(s) available | Case selection bias, thick description needed | Comparative case analysis |
| **Theory building from data** | Grounded theory | Phenomenon poorly theorized | Premature closure, procedural fidelity | Thematic analysis with abductive reasoning |
| **Lived experience / meaning-making** | Phenomenology | Access to people who experienced the phenomenon | Bracketing, epoché validity | Narrative inquiry |
| **Both measurement and explanation** | Mixed methods (sequential / concurrent) | Resources for both phases | Integration quality, paradigm coherence | Choose dominant strand and add secondary |

Document why this design is the best tradeoff given constraints. Include explicit reasons why alternative designs were not chosen.

#### Design Justification Template

```markdown
## Design Rationale

**Selected design**: [Design type]

**Why this design fits the RQ**:
[1–2 sentences linking the design capabilities to the RQ requirements]

**Why alternatives were rejected**:
- [Alternative 1]: rejected because [specific reason — access, ethics, feasibility]
- [Alternative 2]: rejected because [specific reason]

**Key tradeoffs accepted**:
- [Tradeoff 1: e.g., "lower internal validity in exchange for higher ecological validity"]
- [Tradeoff 2]
```

### Step 2: Define Constructs, Variables, and Operationalization

#### For Quantitative Designs

| Element | What to Specify | Example |
|---------|----------------|---------|
| **Independent variable(s)** | Definition, measurement, scale, source | Remote work ratio: % of WFH days per month (self-report, continuous) |
| **Dependent variable(s)** | Definition, measurement, scale, source | Productivity index: quarterly output / hours, from HR system |
| **Mediators** | Theory-derived mechanism variables | Autonomy perception: 5-item SDT scale (Deci & Ryan, α = .87) |
| **Moderators** | Boundary condition variables | Team size: count from HR records |
| **Controls** | Confounders to include with justification | Tenure, education, industry — justified by theory and prior evidence |
| **Instruments** | Validated scales, surveys, measures | Use validated instruments; cite source + reliability; document any adaptations |

> **Anti-pattern**: "We control for demographics" — each control must be theoretically justified, not included as a blanket.

#### For Qualitative Designs

| Element | What to Specify | Example |
|---------|----------------|---------|
| **Sensitizing concepts** | Initial theoretical lenses guiding attention | Institutional logic: regulative, normative, cultural-cognitive |
| **Case/site definition** | What constitutes a "case" and its boundaries | Case = platform governance episode; bounded by policy announcement → resolution |
| **Data sources** | Interviews, documents, observations, artifacts | Semi-structured interviews (60 min); board meeting minutes; platform changelogs |
| **Evidence forms** | What counts as evidence for a claim | Informant statements, document excerpts, observer notes, artifact analysis |
| **Coding approach** | Inductive / deductive / hybrid; codebook evolution rules | Hybrid: start with sensitizing concepts, allow emergent codes; stabilize codebook after cycle 2 |

### Step 3: Sampling & Recruitment Plan

#### Quantitative Sampling

| Sampling Strategy | When to Use | How to Determine N |
|------------------|------------|-------------------|
| **Random sampling** | Generalizable population inference | Power analysis: specify α, β, MDE, expected σ |
| **Stratified sampling** | When subgroups are important | Power per stratum |
| **Convenience sampling** | When access is limited | Report limitations; avoid causal language |
| **Cluster sampling** | When randomizing at group level | Adjust for ICC; inflate N |

**Power Analysis Requirements**:

```
Minimum reporting:
- Significance level (α): typically 0.05
- Power (1 − β): typically 0.80 or 0.90
- Minimum detectable effect (MDE): justify from prior literature
- Expected variance: from pilot data or prior studies
- Final sample size: N = [calculated value]
- Tool used: G*Power / pwr (R) / statsmodels (Python)
```

#### Qualitative Sampling

| Strategy | When to Use | Sample Size Guidance |
|----------|------------|---------------------|
| **Purposive** | Selecting information-rich cases | 6–25 participants for interviews; 3–10 for case study |
| **Maximum variation** | Seeking diverse perspectives | Vary on key dimensions; document selection rationale |
| **Theoretical** | Grounded theory — let emerging theory guide | Continue until theoretical saturation; document saturation evidence |
| **Critical case** | Testing a theory in its strongest/weakest case | 1–3 cases; justify why these cases are critical |
| **Snowball** | Hard-to-reach populations | Document recruitment chain; note bias toward connected individuals |

> **Saturation evidence**: Document when no new themes/codes emerge; show code frequency stabilization; report final codebook size.

### Step 4: Data Collection Plan

| Design Type | Key Collection Decisions |
|-------------|------------------------|
| **Survey** | Platform (Qualtrics/Prolific/MTurk), attention checks, response quality, expected response rate, incentive |
| **Interview** | Semi-structured guide, recording consent, transcription method (verbatim), memoing plan, member checking |
| **Experiment** | Randomization mechanism, manipulation check, blinding, debrief, exclusion criteria |
| **Case study** | Data source triangulation plan (interviews + documents + observation), archive access, case database |
| **Secondary data** | Data dictionary, variable mapping, missing data, merge strategy, vintage/vintage effects |
| **Observation** | Protocol, field note structure, observer role (participant/non-participant), duration |

For each instrument, plan:
1. **Pilot testing**: With how many participants? What will you adjust?
2. **Ethical procedures**: Consent process, recording permissions, data security
3. **Quality assurance**: Inter-rater training (for coding), attention checks (for surveys), manipulation checks (for experiments)

### Step 5: Analysis Plan (Pre-specify)

Create an analysis plan that is executable and auditable:

#### For Quantitative Analysis

| Element | What to Specify |
|---------|----------------|
| **Primary model** | Estimator (OLS, logit, DID, SEM, etc.); DV ~ IV + controls |
| **Assumptions** | Linearity, normality, homoskedasticity, independence; how each is tested |
| **Missing data** | Listwise deletion / multiple imputation / FIML; justify choice |
| **Multiple testing** | Bonferroni, Holm, BH-FDR; threshold p-value |
| **Subgroup analysis** | Pre-specified subgroups; justification; interaction terms |
| **Robustness checks** | Link to `robustness-planner` (C3_5) |
| **Software** | Stata / R / Python; packages and versions |

#### For Qualitative Analysis

| Element | What to Specify |
|---------|----------------|
| **Analytic approach** | Thematic analysis / grounded theory / content analysis / narrative analysis / process tracing |
| **Coding strategy** | Inductive / deductive / hybrid; first-cycle + second-cycle codes (Saldaña, 2016) |
| **Codebook management** | Initial codes → evolution rules → stabilization criteria |
| **Memoing** | When and how; reflexive memos, theoretical memos, methodological memos |
| **Cross-case comparison** | Variable-oriented / case-oriented / mixed |
| **Negative case analysis** | How deviant cases will be sought and handled |
| **Saturation documentation** | How saturation is assessed and reported |
| **Software** | NVivo / Atlas.ti / MAXQDA / manual |

### Step 6: Validity / Rigor Plan

#### Quantitative Validity

| Validity Type | Threats | Mitigation |
|--------------|---------|------------|
| **Internal** | Confounders, selection, reverse causality | IV, matching, fixed effects; see `rival-hypothesis-designer` |
| **External** | Sample specificity | Discuss generalization boundaries |
| **Construct** | Measurement error, single-item | Validated scales, multiple indicators |
| **Statistical conclusion** | Low power, violated assumptions | Power analysis, robustness checks |
| **Common method bias** | Single-source data | Temporal separation, marker variable |

#### Qualitative Rigor

| Criterion (Lincoln & Guba, 1985) | Technique | How to Document |
|----------------------------------|-----------|-----------------|
| **Credibility** | Prolonged engagement, triangulation, member checking, peer debriefing | Log engagement duration; triangulation table; member-check responses |
| **Transferability** | Thick description | Provide enough context for readers to judge applicability |
| **Dependability** | Audit trail | Decision log; codebook evolution; memo trail |
| **Confirmability** | Reflexivity statement | Researcher positionality; how bias was managed |
| **Authenticity** | Fair representation | Multiple voices; negative cases; range of perspectives |

### Step 7: Preregistration (Optional but Recommended)

| Registration Platform | When to Use | What to Include |
|----------------------|------------|-----------------|
| **OSF Registries** | General social science | RQ, hypotheses, design, analysis plan, power |
| **AsPredicted** | Quick 9-question format | Data collection status, hypothesis, methods |
| **PROSPERO** | Systematic reviews | Protocol for search, inclusion, analysis |
| **ClinicalTrials.gov** | Clinical/health trials | Full trial protocol |
| **AEA RCT Registry** | Economics experiments | Design, outcomes, sample size |

Document the registration date and any post-registration deviations with justification.

## Outputs (Create/Update)

- Study design doc → `RESEARCH/[topic]/study_design.md` (use `templates/study-design.md`)
- Analysis plan → `RESEARCH/[topic]/analysis_plan.md` (use `templates/analysis-plan.md`)
- Data management plan → `RESEARCH/[topic]/data_management_plan.md` (use `templates/data-management-plan.md`)
- Instruments (optional):
  - Survey → `RESEARCH/[topic]/instruments/survey.md` (use `templates/survey-instrument.md`)
  - Interview guide → `RESEARCH/[topic]/instruments/interview_guide.md` (use `templates/interview-guide.md`)
- Prereg draft (optional) → `RESEARCH/[topic]/preregistration.md` (use `templates/preregistration-template.md`)

## Quality Bar

The study design is **ready** when:

- [ ] Design type selected with explicit justification and alternatives rejected with reasons
- [ ] All constructs operationalized with measurement source and validity evidence
- [ ] Sampling strategy specified with adequacy rationale (power or saturation plan)
- [ ] Data collection instruments identified or designed with pilot plan
- [ ] Analysis plan is executable (estimator, covariates, missing data, robustness)
- [ ] Validity threats identified with matching mitigations
- [ ] For qualitative: coding strategy, saturation plan, and rigor procedures documented
- [ ] Ethical review requirements identified (feeds `ethics-irb-helper`)
- [ ] Preregistration decision made and documented (registered or justified why not)

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| No design alternatives considered | Reviewer asks "why not X?" | Justify design choice against at least 2 alternatives |
| Power analysis missing or post-hoc | Undermines any null findings | Pre-specify; use prior effect sizes |
| "Convenience sample" without limitation | Readers question generalizability | Acknowledge; avoid causal claims |
| Qualitative sample without rationale | "We interviewed 12 people" — why 12? | Document sampling strategy + saturation evidence |
| Controls not theoretically justified | "Kitchen sink" approach | Each control needs a rationale |
| Missing data plan absent | Reviewer flags > 10% missing | Pre-specify handling before analysis |
| No pilot testing | Survey/interview flaws discovered too late | Always pilot with 3–5 participants |

## Notes

- This skill is not legal advice; for ethics/IRB packaging, use **ethics-irb-helper**.
- For implementation code, pair with **code-builder** (and **model-collaborator** when helpful).
- For downstream variable or dataset specifics, pair with **variable-constructor** and **dataset-finder**.
