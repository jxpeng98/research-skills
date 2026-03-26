---
id: credit-taxonomy-helper
stage: H_submission
description: "Generate CRediT (Contributor Roles Taxonomy) author contribution statements with ICMJE cross-check and authorship ethics guidance."
inputs:
  - type: Manuscript
    description: "Finalized manuscript with author list"
  - type: UserQuery
    description: "Author roles and contributions"
outputs:
  - type: CRediTStatement
    artifact: "submission/credit_statement.md"
constraints:
  - "Must use all 14 CRediT roles"
  - "Must ensure every author has at least one role"
  - "Must cross-check against ICMJE authorship criteria"
failure_modes:
  - "Author contributions unknown or disputed"
  - "Gift or ghost authorship not flagged"
  - "Solo author edge case"
tools: [filesystem]
tags: [submission, CRediT, authorship, contributions, taxonomy, ICMJE]
domain_aware: false
---

# CRediT Taxonomy Helper Skill

Generate accurate CRediT author contribution statements, cross-check against ICMJE authorship criteria, and flag common authorship ethics issues.

## Related Task IDs

- `H1` (submission packaging — author contributions component)

## Output (contract path)

- `RESEARCH/[topic]/submission/credit_statement.md`

## When to Use

- During submission packaging (H1)
- When a venue requires CRediT statements (increasingly common)
- When authorship discussions need a structured framework
- When verifying that all listed authors meet authorship criteria

## Process

### Step 1: Understand the 14 CRediT Roles

| # | Role | Definition | Typical Contributors |
|---|------|-----------|---------------------|
| 1 | **Conceptualization** | Ideas; formulation of research goals and aims | PI, senior researchers |
| 2 | **Methodology** | Development or design of methodology; creation of models | Methodologist, PI |
| 3 | **Software** | Programming, software development; implementing algorithms | Developer, data scientist |
| 4 | **Validation** | Verification of replication/reproducibility of results | Co-author, research assistant |
| 5 | **Formal analysis** | Application of statistical, mathematical, or computational techniques | Analyst, data scientist |
| 6 | **Investigation** | Conducting research and investigation; data/evidence collection | Research assistants, PhD students |
| 7 | **Resources** | Provision of study materials, equipment, computing resources | Lab director, institution |
| 8 | **Data curation** | Management activities to annotate, scrub data, maintain data | Data manager, research assistant |
| 9 | **Writing – original draft** | Preparation, creation of the initial draft | Lead author(s) |
| 10 | **Writing – review & editing** | Critical review, commentary, revision | All co-authors (expected) |
| 11 | **Visualization** | Preparation of figures, tables, graphics | Any author with visualization skills |
| 12 | **Supervision** | Oversight and leadership responsibility | PI, senior co-author, advisor |
| 13 | **Project administration** | Management and coordination; planning and execution | PI, project manager |
| 14 | **Funding acquisition** | Securing financial support for the project | PI, co-PI |

**Contribution levels**: Each author-role combination should be marked as:
- **Lead**: primary responsibility for this role
- **Equal**: shared responsibility equally
- **Supporting**: contributed to but not primarily responsible

### Step 2: Map Each Author to Roles

Collect information through author consultation:

| Author | Con | Met | Sof | Val | For | Inv | Res | Dat | W-OD | W-RE | Vis | Sup | Adm | Fun |
|--------|-----|-----|-----|-----|-----|-----|-----|-----|------|------|-----|-----|-----|-----|
| A. Smith | L | L | — | — | S | — | — | — | L | E | — | — | L | L |
| B. Jones | S | S | L | L | L | — | — | L | S | E | L | — | — | — |
| C. Lee | — | — | — | — | — | L | S | S | — | E | — | L | — | S |

### Step 3: Verify Completeness

| Check | What to Verify | Issue if Failing |
|-------|---------------|-----------------|
| Every author has ≥1 role | No author listed without contribution | Ghost authorship risk |
| Every applicable role filled | Key roles (e.g., Formal analysis) have ≥1 author | Incomplete attribution |
| At least one author for W-OD | Someone wrote the initial draft | Accountability gap |
| All authors for W-RE | All co-authors reviewed and edited | ICMJE requirement |
| Conceptualization has clear lead | Intellectual origin is attributed | Credit disputes |

### Step 4: Cross-Check Against ICMJE Criteria

The ICMJE defines authorship as meeting ALL FOUR of:

| ICMJE Criterion | Required? | How CRediT Maps |
|-----------------|-----------|-----------------|
| **Substantial contribution** to conception/design, data acquisition, or analysis/interpretation | ✅ Must meet | Conceptualization, Methodology, Investigation, Formal analysis |
| **Drafting or critically revising** the manuscript for important intellectual content | ✅ Must meet | Writing – original draft, Writing – review & editing |
| **Final approval** of the version to be published | ✅ Must meet | Implicit — must be confirmed separately |
| **Agreement to be accountable** for all aspects of the work | ✅ Must meet | Implicit — must be confirmed separately |

> **Key check**: If an author has only "Resources" or "Funding acquisition" but no other role, they likely do NOT meet ICMJE authorship criteria. They should be moved to Acknowledgments.

#### Authorship Ethics Flags

| Issue | How to Detect | Action |
|-------|-------------- |--------|
| **Gift authorship** | Author has only "Resources" or "Supervision" or "Funding" with no intellectual contribution | Move to Acknowledgments; or document their intellectual contribution |
| **Ghost authorship** | Someone who contributed substantially (e.g., writing, analysis) is not listed | Add as author or acknowledge |
| **Honorary authorship** | Senior person added for prestige/politics | Violates ICMJE — discuss with team |
| **Order disputes** | Multiple authors claim "equal first" or "lead" on key roles | Use "†equal contribution" notation; discuss early |
| **AI authorship** | AI tool listed as co-author | Most journals prohibit — list in AI disclosure instead |

### Step 5: Generate Output in Required Formats

#### Tabular Format (most common)

```markdown
# CRediT Author Contributions

| Author | Roles |
|--------|-------|
| A. Smith | Conceptualization (lead), Methodology (lead), Writing – original draft (lead), Writing – review & editing (equal), Project administration (lead), Funding acquisition (lead), Formal analysis (supporting) |
| B. Jones | Methodology (supporting), Software (lead), Validation (lead), Formal analysis (lead), Data curation (lead), Writing – original draft (supporting), Writing – review & editing (equal), Visualization (lead) |
| C. Lee | Investigation (lead), Resources (supporting), Data curation (supporting), Writing – review & editing (equal), Supervision (lead), Funding acquisition (supporting) |
```

#### Narrative Format (some journals prefer)

```
A. Smith: Conceptualization (lead), Methodology (lead), Writing – original draft (lead), 
Writing – review & editing (equal), Project administration (lead), Funding acquisition (lead).
B. Jones: Methodology (supporting), Software (lead), Validation (lead), Formal analysis (lead),
Data curation (lead), Writing – original draft (supporting), Visualization (lead).
```

#### Matrix Format (for internal use / supplements)

```
| Author  | Con | Met | Sof | Val | For | Inv | Res | Dat | W-OD | W-RE | Vis | Sup | Adm | Fun |
|---------|-----|-----|-----|-----|-----|-----|-----|-----|------|------|-----|-----|-----|-----|
| Smith   |  L  |  L  |  —  |  —  |  S  |  —  |  —  |  —  |  L   |  E   |  —  |  —  |  L  |  L  |
| Jones   |  S  |  S  |  L  |  L  |  L  |  —  |  —  |  L  |  S   |  E   |  L  |  —  |  —  |  —  |
| Lee     |  —  |  —  |  —  |  —  |  —  |  L  |  S  |  S  |  —   |  E   |  —  |  L  |  —  |  S  |

L = Lead, E = Equal, S = Supporting, — = Not applicable
```

## Quality Bar

The CRediT statement is **ready** when:

- [ ] All 14 CRediT roles considered for each author
- [ ] Every author has at least one substantive role
- [ ] Contribution levels (Lead / Equal / Supporting) specified
- [ ] ICMJE cross-check passed (all 4 criteria met per author)
- [ ] No gift, ghost, or honorary authorship issues
- [ ] Format matches venue requirements (tabular or narrative)
- [ ] All authors confirmed their roles (documented externally)

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Everyone gets "Writing – review & editing" but nothing else | Inflates contribution | Ensure W-RE is genuine (tracked changes evidence) |
| Forgetting "Validation" role | Under-credited verification work | Ask who checked results/replication |
| Supervisor listed but didn't meet ICMJE | Gift authorship | Move to Acknowledgments or document intellectual contribution |
| AI tool listed as author | Most journals prohibit | AI use goes in AI disclosure statement, not CRediT |
| Order disputes unresolved | Tension at submission time | Discuss author order BEFORE writing; document agreement |
