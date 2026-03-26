---
id: dataset-finder
stage: C_design
description: "Identify feasible primary or secondary datasets, access routes, and coverage limitations for an empirical design."
inputs:
  - type: RQSet
    description: "Research questions defining the target phenomenon and unit of analysis"
  - type: DesignSpec
    description: "Study design constraining sampling frame, timeframe, and measures"
  - type: AnalysisPlan
    description: "Planned estimands and identification strategy"
    required: false
outputs:
  - type: DatasetPlan
    artifact: "design/dataset_plan.md"
constraints:
  - "Must distinguish candidate datasets from the recommended primary dataset"
  - "Must document access constraints, licensing, and expected update cadence"
  - "Must state key coverage gaps that threaten identification or validity"
failure_modes:
  - "No feasible dataset can identify the target construct or treatment timing"
  - "Access restrictions make the preferred dataset non-viable within project constraints"
tools: [filesystem, scholarly-search, metadata-registry]
tags: [design, data, datasets, feasibility, provenance]
domain_aware: true
---

# Dataset Finder Skill

Systematically identify, evaluate, and compare candidate datasets — selecting the best feasible option for the study before implementation begins.

## Related Task IDs

- `C4` (data management and dataset planning)

## Output (contract path)

- `RESEARCH/[topic]/design/dataset_plan.md`

## When to Use

- After study design specifies what data is needed
- When choosing between primary collection and secondary data
- When working with administrative or proprietary data that requires access negotiation
- When combining multiple data sources

## Process

### Step 1: Translate the Design into Data Requirements

Before searching for datasets, specify exactly what data the study needs:

| Requirement | Question to Answer | Example |
|------------|-------------------|---------|
| **Unit of observation** | What entity is one row? | Individual employee / firm-quarter / country-year |
| **Population** | Who/what is included? | Full-time employees in tech firms, 2018–2023 |
| **Time coverage** | What period do you need? | Pre/post intervention; rolling 5-year window |
| **Geographic scope** | Where? | US only / OECD / global |
| **Key variables** | What must be measurable? | Remote work status, productivity, team composition |
| **Treatment identification** | How is treatment/exposure identified? | Policy change date, random assignment, eligibility threshold |
| **Frequency** | How granular? | Monthly, quarterly, annual, one-time cross-section |
| **Sample size requirement** | Minimum viable N? | Power analysis → N ≥ 500 for MDE = 0.2 SD |

### Step 2: Search for Candidate Datasets

#### Where to Search

| Source Category | Examples | Typical Access |
|----------------|---------|---------------|
| **Government/census** | Census Bureau, BLS, Eurostat, OECD.Stat, World Bank Open Data | Open/free; may require registration |
| **Academic repositories** | ICPSR, Harvard Dataverse, UK Data Service, Zenodo | Free for academic use; DUA may required |
| **Panel/survey data** | PSID, NLSY, SOEP, SHARE, HILDA, CFPS | Application + DUA; 2–12 week lead time |
| **Administrative data** | Tax records, hospital records, education systems | Restricted; IRB + data agreement; months of lead time |
| **Commercial/proprietary** | Compustat, CRSP, Refinitiv, LinkedIn, patent databases | Institutional subscription required |
| **Web/API data** | Twitter/X API, GitHub API, OpenStreetMap | Rate limits; terms of service; ethical review |
| **Primary collection** | Surveys (Prolific, MTurk, Qualtrics), interviews, experiments | Full control but costly and time-consuming |

#### How to Evaluate

For each candidate dataset, assess:

| Criterion | What to Check | Rating |
|-----------|--------------|--------|
| **Coverage** | Does it include the target population and time period? | Full / Partial / None |
| **Granularity** | Is the unit of observation fine enough? | Exact match / Needs aggregation / Too coarse |
| **Key variables** | Are treatment, outcome, and control variables available? | All / Most / Missing critical |
| **Identification** | Can you identify the treatment/exposure timing? | Strong / Weak / Not possible |
| **Sample size** | Sufficient for your power requirement? | Sufficient / Marginal / Too small |
| **Quality** | Measurement reliability, response rates, known issues? | High / Moderate / Low |
| **Access** | Time to obtain, cost, restrictions? | Immediate / Weeks / Months |
| **License** | Can results be published? Can data be shared? | Open / Academic only / Restricted |
| **Documentation** | Codebook, data dictionary, technical reports available? | Comprehensive / Basic / Poor |
| **Recency** | Latest available year/wave? | Current / 2–3 years lag / Stale |

### Step 3: Build the Comparison Matrix

```markdown
## Candidate Dataset Comparison

| Criterion | Dataset A | Dataset B | Dataset C (Primary) |
|-----------|-----------|-----------|---------------------|
| **Name** | PSID | Census ACS | Prolific Survey |
| **Coverage** | US households, 1968– | US population, 2005– | Custom sample |
| **Unit** | Individual-year | Person or household | Individual |
| **Key variables** | ✅ Income, employment | ⚠️ No WFH variable pre-2019 | ✅ Custom measures |
| **Treatment ID** | ⚠️ No policy variation | ⚠️ Cross-sectional only | ✅ Randomized |
| **N** | ~9,000 families | ~3.5M per year | 500 (custom) |
| **Access** | DUA, 4 weeks | Open | Immediate |
| **Cost** | Free | Free | $3/participant |
| **Documentation** | Excellent | Excellent | Self-generated |
| **Limitation** | Panel attrition | No longitudinal link | External validity |
```

### Step 4: Assess Data Combination Feasibility

When a single dataset is insufficient:

| Combination Strategy | When to Use | Key Challenges |
|---------------------|------------|----------------|
| **Merge on identifier** | Same entities appear in both datasets | Key matching, privacy, coverage mismatch |
| **Contextual enrichment** | Add area-level statistics to individual data | Ecological fallacy; aggregation level mismatch |
| **Imputation from external** | Missing variable can be imputed from related dataset | Imputation validity; measurement error |
| **Triangulation** | Different datasets address different parts of the RQ | Integration logic must be pre-specified |

### Step 5: Make a Recommendation

| Component | What to Document |
|-----------|-----------------|
| **Primary dataset** | Name, access route, expected timeline |
| **Backup option** | If primary access fails |
| **Combination strategy** | If multiple sources needed |
| **Main risks** | Coverage gaps, access delays, measurement limitations |
| **Mitigation** | What you'll do if risks materialize |

### Step 6: Document Access and Ethics

| Item | What to Record |
|------|---------------|
| **Access procedure** | DUA, application, subscription, IRB requirement |
| **Timeline** | Expected time from application to data receipt |
| **Cost** | Subscription, collection costs, incentive payments |
| **Data storage** | Where data will be held; encryption; retention period |
| **Sharing restrictions** | Can cleaned data be shared? Conditions? |
| **Citation requirement** | Required citation or acknowledgment for the dataset |
| **Terms of use** | Any restrictions on methodology, publication, or derivative data |

## Quality Bar

The dataset plan is **ready** when:

- [ ] Data requirements explicitly derived from study design
- [ ] At least 3 candidate options evaluated
- [ ] Comparison matrix covers: coverage, key variables, identification, access, quality
- [ ] Primary and backup datasets recommended with justification
- [ ] Access procedure, timeline, and cost documented
- [ ] Key coverage gaps identified with impact on identification/validity
- [ ] Data combination strategy specified (if multiple sources)
- [ ] Ethics and licensing requirements documented

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Choosing data first, then RQ | Data convenience drives research questions | Start from RQ → design → data requirements |
| Assuming access is immediate | Many datasets take months to obtain | Start data access process early; have backup plan |
| Ignoring documentation quality | Poor codebooks → measurement errors | Check codebook availability before committing |
| Not checking variable availability by year | Variable added in 2019 but you need 2015 data | Verify variable-year coverage matrix |
| Single dataset with no backup | If access fails, project stalls | Always identify a backup option |
| Not assessing data quality | High missingness, measurement error | Check technical reports, validation studies, prior user publications |

## Minimal Output Format

```markdown
# Dataset Plan

## Data Requirements
- Unit of analysis: [entity]
- Population: [who/what]
- Time period: [when]
- Key variables needed: [list]
- Identification strategy: [what variation is needed]
- Minimum N: [from power analysis]

## Candidate Comparison

| Criterion | Dataset A | Dataset B | Dataset C |
|-----------|-----------|-----------|-----------|

## Recommendation
- **Primary**: [name] — [justification]
- **Backup**: [name] — [when to use]
- **Access**: [procedure, timeline, cost]

## Known Gaps and Risks
| Gap | Impact | Mitigation |
|-----|--------|-----------|
```
