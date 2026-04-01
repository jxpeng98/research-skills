---
id: table-generator
stage: F_writing
description: "Design and format publication-quality tables following venue standards and APA / journal-specific formatting."
inputs:
  - type: StatsReport
    description: "Statistical output (regressions, descriptive stats, etc.)"
  - type: VenueAnalysis
    description: "Venue table formatting requirements"
outputs:
  - type: Tables
    artifact: "manuscript/tables/"
constraints:
  - "Must be self-explanatory (readable without body text)"
  - "Must follow venue-specific formatting rules"
  - "Must include appropriate precision (not more decimal places than measurement warrants)"
failure_modes:
  - "Table duplicates information already in the text"
  - "Too many decimal places (false precision)"
  - "Missing notes explaining abbreviations or significance markers"
tools: [filesystem]
tags: [writing, tables, formatting, APA, statistics, presentation]
domain_aware: true
---

# Table Generator Skill

Design tables that communicate findings clearly and meet publication standards — tables should tell the story on their own.

## Purpose

Design and format publication-quality tables following venue standards and APA / journal-specific formatting.

## Related Task IDs

- `F5` (figures and tables)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/tables/`

## When to Use

- After primary analysis is complete
- When results are better presented as tables than inline text
- When venue requires specific table formats

## Process

### Step 1: Decide What Deserves a Table

| Content | Table? | Why |
|---------|--------|-----|
| Descriptive statistics (means, SDs, correlations) | ✅ Almost always | Readers expect it; reviewers use it to verify analyses |
| Main regression / model results | ✅ Yes | Core contribution — must be prominent and clear |
| Robustness checks (3+ specifications) | ✅ Yes | Comparing specifications is easier in tabular form |
| Simple comparison (2 numbers) | ❌ Use text | Tables for 2 numbers waste space |
| Qualitative theme summary | ✅ Often useful | Theme / evidence / frequency / exemplar quote table |
| Variable definitions | ✅ Yes, if ≥5 variables | Standard practice for transparency |

> **Rule**: Every table must present information that cannot be efficiently communicated in running text. If a table has ≤3 rows and ≤3 columns, consider using text instead.

### Step 2: Choose Table Type

| Table Type | Common Use | Key Elements |
|-----------|-----------|-------------|
| **Descriptive statistics** | Sample characteristics | N, Mean, SD, Min, Max; separate panels for subgroups |
| **Correlation matrix** | Variable relationships | Lower triangle; diagonal = 1 or SD; significance stars |
| **Regression results** | Main findings | Models in columns; variables in rows; SE or CI below coefficients |
| **Multi-model comparison** | Robustness | Columns = specifications; rows = key variables; R² at bottom |
| **Qualitative themes** | Theme evidence | Theme, definition, frequency/prevalence, exemplar quote |
| **Variable operationalization** | Transparency | Variable name, measure, source, scale, reliability |
| **Summary of included studies** | Systematic review | Author/year, sample, design, key findings, quality rating |
| **GRADE evidence table** | Evidence quality | Outcome, studies (k), certainty, effect size, interpretation |

### Step 3: Structure the Table

#### APA 7th Table Structure (default unless venue specifies otherwise)

```
Table [n]

[Title: Descriptive, specific — what the table shows]

[Column headers]
[Column spanners if needed]
[Body rows — groups separated by horizontal rules]
[Total / summary row if applicable]

Note. General note explaining abbreviations.
* p < .05. ** p < .01. *** p < .001.
[Specific notes marked with superscript letters: ᵃ ᵇ ᶜ]
```

#### Formatting Rules

| Rule | APA 7th | Medical / Science | Economics |
|------|---------|------------------|-----------|
| Lines | Top, header-bottom, table-bottom only (no vertical lines) | Similar (varies by venue) | Minimal rules |
| Significance | Stars (*,**,***) | Stars or bold | Stars + report exact p sometimes |
| SE / CI | In parentheses below coefficient | In parentheses or brackets | In parentheses below coefficient |
| Precision | 2 decimal places for correlations; 2–3 for coefficients | Varies | 3+ common |
| N | Report per column if different | Report per cell if applicable | Report per column |

### Step 4: Construct Each Table

#### Descriptive Statistics Table Example

```markdown
Table 1

Descriptive Statistics and Correlations (N = 342)

| Variable            | M     | SD    | 1      | 2      | 3      | 4      |
|---------------------|-------|-------|--------|--------|--------|--------|
| 1. Productivity     | 4.23  | 0.89  | —      |        |        |        |
| 2. Remote work (%)  | 0.45  | 0.31  | .28**  | —      |        |        |
| 3. Team size        | 6.72  | 3.14  | .12*   | −.08   | —      |        |
| 4. Tenure (years)   | 5.40  | 4.21  | .15**  | .22**  | .03    | —      |

Note. * p < .05. ** p < .01.
```

#### Regression Results Table Example

```markdown
Table 2

OLS Regression Results: Effect of Remote Work on Productivity

|                          | Model 1   | Model 2   | Model 3   |
|--------------------------|-----------|-----------|-----------|
| Remote work ratio        | 0.35***   | 0.28**    | 0.31**    |
|                          | (0.08)    | (0.09)    | (0.10)    |
| Team size                |           | 0.04      | 0.03      |
|                          |           | (0.03)    | (0.03)    |
| Tenure                   |           | 0.06*     | 0.05*     |
|                          |           | (0.02)    | (0.02)    |
| Remote × Team size       |           |           | −0.02     |
|                          |           |           | (0.01)    |
| Firm FE                  | No        | No        | Yes       |
| Observations             | 342       | 342       | 342       |
| R²                       | .08       | .12       | .18       |

Note. Standard errors in parentheses. Model 3 includes firm fixed effects.
* p < .05. ** p < .01. *** p < .001.
```

#### Qualitative Themes Table Example

```markdown
Table 3

Themes in Platform Governance Tensions (N = 28 interviews)

| Theme | Definition | Prevalence | Exemplar Quote |
|-------|-----------|------------|----------------|
| Coercive framing | Managers invoke contractual authority | 21/28 (75%) | "We have to enforce this; it's in the ToS" (M03) |
| Normative framing | Managers appeal to shared values | 18/28 (64%) | "We're all building this together" (M11) |
| Frame switching | Managers alternate between coercive and normative | 14/28 (50%) | "First I explain why, then if needed I point to the rules" (M07) |
```

### Step 5: Quality Check Each Table

| Check | What to Verify |
|-------|---------------|
| **Self-explanatory** | Can you understand the table without reading the text? |
| **Title is descriptive** | Title states what the table shows, not just "Results" |
| **All abbreviations defined** | In the Note below the table |
| **Significance markers consistent** | Same convention throughout the paper |
| **N reported** | Overall and per-column/cell if different |
| **No duplicate info** | Table doesn't repeat what's in the text (reference it) |
| **Precision appropriate** | Not more decimal places than measurement instrument supports |
| **Venue format compliance** | Matches required style (APA, Vancouver, house style) |

## Quality Bar

Tables are **ready** when:

- [ ] Each table is self-explanatory with descriptive title and adequate notes
- [ ] All significance markers, abbreviations, and units defined
- [ ] Precision matches measurement instrument (not artificially high)
- [ ] No table duplicates information available in running text
- [ ] Format matches venue requirements
- [ ] Every table is referenced in the manuscript text

## Minimal Output Format

```markdown
# Table Specifications

## Table 1: [Title]
- **Type**: Descriptive statistics / Regression / Themes / ...
- **Data source**: [which analysis output]
- **Key formatting**: [APA 7th / venue-specific]

[Table content in markdown]

Note. [General note + significance markers]
```
