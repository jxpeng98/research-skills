---
id: table-generator
stage: F_writing
version: "0.2.1"
description: "Generate publication-ready tables (descriptive stats, regression, ANOVA) with APA/journal formatting."
inputs:
  - type: StatsReport
    description: "Statistical output requiring tabulation"
  - type: VenueAnalysis
    description: "Table formatting requirements from target venue"
    required: false
outputs:
  - type: FormattedTables
    artifact: "manuscript/tables/"
constraints:
  - "Must follow target venue table style (APA, journal-specific)"
  - "Must include proper notes, significance markers, and source annotations"
failure_modes:
  - "Complex multi-panel tables exceed venue column limits"
  - "Inconsistent decimal precision across tables"
tools: [filesystem]
tags: [writing, tables, formatting, APA, publication-ready]
domain_aware: true
---

# Table Generator Skill

Produce publication-ready formatted tables from statistical output.

## Related Task IDs

- `F5` (tables and figures)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/tables/`

## Procedure

1. **Identify table type** from statistical output:
   - Descriptive statistics / summary table
   - Regression / model comparison table
   - ANOVA / mixed model table
   - Correlation matrix
   - Cross-tabulation / contingency
2. **Apply formatting rules**:
   - Column alignment (numbers right-aligned)
   - Decimal precision (consistent within table)
   - Significance markers with legend
   - Note placement (below table)
   - Horizontal rules per APA/venue style
3. **Generate in multiple formats**:
   - Markdown (for draft review)
   - LaTeX (for typesetting)
   - CSV (for data verification)
4. **Check against venue requirements**:
   - Column count limits
   - Font/size specifications
   - Required elements (title, notes, source)

## Minimal Output Format

```markdown
# Table 1: Descriptive Statistics

| Variable | N | Mean | SD | Min | Max |
|---|---|---|---|---|---|

*Note.* SD = standard deviation.
```
