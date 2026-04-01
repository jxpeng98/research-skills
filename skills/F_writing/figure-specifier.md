---
id: figure-specifier
stage: F_writing
description: "Specify publication-quality figures with exact data mapping, visual encoding, and accessibility compliance."
inputs:
  - type: StatsReport
    description: "Statistical results requiring visualization"
  - type: VenueAnalysis
    description: "Figure formatting requirements"
    required: false
outputs:
  - type: FigureSpecs
    artifact: "manuscript/figures/"
constraints:
  - "Must specify exact data → visual encoding mapping"
  - "Must include accessibility considerations (colorblind-safe palettes, alt-text)"
  - "Must generate reproducible specification for each figure"
  - "Every figure must have a clear analytical purpose — no decorative charts"
failure_modes:
  - "High-dimensional data resists 2D visualization"
  - "Venue resolution/size requirements conflict with readability"
  - "Figure duplicates information already in a table"
tools: [filesystem, code-runtime]
tags: [writing, figures, visualization, accessibility, publication-ready]
domain_aware: true
---

# Figure Specifier Skill

Design publication-quality figures that communicate findings clearly, meet venue standards, and survive black-and-white printing and accessibility checks.

## Purpose

Specify publication-quality figures with exact data mapping, visual encoding, and accessibility compliance.

## Related Task IDs

- `F5` (tables and figures)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/figures/`

## When to Use

- After primary analysis is complete
- When results are better shown visually than in tables
- When the paper needs a conceptual/theoretical framework diagram
- When venue requires specific figure formats

## Process

### Step 1: Decide What Deserves a Figure

| Content | Figure? | Best Figure Type | Alternative |
|---------|---------|-----------------|-------------|
| Overall effect + CI | ✅ | Coefficient plot / forest plot | Table if few effects |
| Distribution shape | ✅ | Histogram / violin / density | Table with descriptive stats |
| Relationship between 2 continuous vars | ✅ | Scatter with trend line | Table of correlations |
| Trend over time | ✅ | Line chart | Table with time periods |
| Group comparison | ⚠️ Sometimes | Bar chart with CI / dot plot | Table if <3 groups |
| Process / framework | ✅ | Flow diagram / conceptual model | Text if very simple |
| PRISMA flow | ✅ Required | Flow diagram | N/A |
| Geographic pattern | ✅ | Map / choropleth | Table by region |
| Model comparison | ❌ Usually table | — | Multi-model regression table |
| Interaction effects | ✅ | Marginal effects plot | Table is hard to interpret |
| Qualitative process model | ✅ | Process diagram / Mermaid | In-text narrative |

> **Rule**: A figure must communicate something a table cannot. If the figure just re-presents tabular numbers as bars, use the table instead.

### Step 2: Select the Right Figure Type

#### For Quantitative Results

| Analytical Purpose | Figure Type | When to Use | When NOT to Use |
|-------------------|-------------|------------|-----------------|
| **Show an effect** | Coefficient plot | Multiple predictors; model comparison | Single predictor |
| **Compare effects across studies** | Forest plot | Meta-analysis | Non-meta contexts |
| **Show a relationship** | Scatter plot + fit line | Two continuous variables | Categorical predictor |
| **Show a distribution** | Histogram / violin / density / box plot | Understanding variable shape | If only mean + SD needed |
| **Show change over time** | Line chart | Longitudinal / time series | Cross-sectional data |
| **Show group differences** | Dot plot with CI / bar chart with error bars | 2–6 groups | >6 groups (use table) |
| **Show interaction effects** | Marginal effects / interaction plot | When interaction is significant | Main effects only |
| **Show geographic patterns** | Choropleth / map | Spatial data | Non-spatial data |
| **Show model diagnostics** | Residual plot / QQ plot | Supplementary | Main paper (usually) |

#### For Qualitative / Conceptual

| Purpose | Figure Type | Tool |
|---------|-------------|------|
| Theoretical framework | Concept map / box-and-arrow | Mermaid diagram |
| Process model | Timeline / phased diagram | Mermaid flowchart |
| Data structure (Gioia method) | 1st order → 2nd order → aggregate | Mermaid or structured table |
| PRISMA flow | 4-phase flow diagram | Mermaid or template |
| CONSORT flow | Enrollment → allocation → follow-up → analysis | Mermaid or template |
| Case comparison | Multi-case matrix visualization | Table + highlights |

### Step 3: Specify Visual Encoding

For each figure, document:

| Encoding Channel | Specification | Notes |
|-----------------|---------------|-------|
| **X-axis** | Variable, scale (linear/log), range, label | Include units |
| **Y-axis** | Variable, scale, range, label | Include units |
| **Color** | Variable mapped to color, palette name | Must be colorblind-safe |
| **Shape** | Variable mapped to shape (if any) | Max 4–5 shapes |
| **Size** | Variable mapped to size (if any) | Use sparingly |
| **Facets** | Grouping variable for small multiples | Limit to 2×3 or similar |
| **Error representation** | 95% CI, SE, SD — specify which | Label clearly |
| **Reference lines** | Null effect (0), thresholds, benchmarks | Always label |

#### Colorblind-Safe Palettes

| Palette | Good For | Colors |
|---------|----------|--------|
| **viridis** | Sequential (continuous data) | Yellow → blue → purple |
| **cividis** | Sequential (CVD-optimized) | Yellow → blue |
| **RColorBrewer Set2** | Categorical (up to 8 groups) | Distinct, muted colors |
| **okabe-ito** | Categorical (up to 8 groups) | Designed for color vision deficiency |
| **Grey-scale safe** | When journal charges for color | Use patterns/shapes instead |

> **Test**: Always simulate how the figure looks in greyscale and with deuteranopia simulation before finalizing.

### Step 4: Set Dimensions and Resolution

| Venue Type | Typical Requirements |
|-----------|---------------------|
| Journal (print) | 300 DPI minimum; single column = 3.3" width; double column = 6.9" |
| Journal (online only) | 150–300 DPI; more flexible sizing |
| Conference (ACM/IEEE) | Often specific template sizes; check LaTeX template |
| Slide / presentation | 150 DPI sufficient; 16:9 or 4:3 aspect ratio |

**File format priority**: PDF (vector) > EPS > SVG > PNG (raster, 300+ DPI) > JPEG (avoid for data figures)

### Step 5: Draft Captions

Every caption must make the figure self-explanatory:

```
Figure [n]. [Descriptive title that states the finding or relationship shown].
[One sentence explaining what the reader should observe.]
[Abbreviation definitions. N = sample size. Error bars represent 95% CIs.]
[Data source note if applicable.]
```

**Example**:
```
Figure 2. Marginal effect of remote work ratio on productivity index,
moderated by team size. The positive effect of remote work diminishes
for teams larger than 8 members (interaction: β = −0.02, p = .04).
Shaded regions represent 95% confidence intervals. N = 342.
```

> **Anti-pattern**: "Figure 1. Results." — this tells the reader nothing. The caption should contain the finding.

### Step 6: Draft Alt-Text for Accessibility

```
Alt-text: [Figure type] showing [what variables/relationship].
[Key pattern: direction, magnitude, notable features].
[Key data point if applicable]. [Sample size].
```

**Example**:
```
Alt-text: Scatter plot showing the relationship between remote work ratio
(x-axis, 0 to 1) and productivity index (y-axis, 1 to 7). The trend line
shows a positive linear relationship (β = 0.31). Points are colored by
team size (blue: small teams <5; orange: large teams >8). Most data points
cluster between 0.2–0.6 on the x-axis. N = 342.
```

### Step 7: Generate Reproducible Code Specification

For each figure, specify the code that produces it:

```python
# Figure 1: [Title]
# Data: analysis_results.csv (columns: remote_ratio, productivity_index, team_size)
# Package: matplotlib + seaborn (Python) or ggplot2 (R)

import matplotlib.pyplot as plt
import seaborn as sns

fig, ax = plt.subplots(figsize=(6.9, 4))  # double-column width
sns.scatterplot(data=df, x='remote_ratio', y='productivity_index',
                hue='team_size_cat', palette='okabe_ito', ax=ax)
# ... regression line, CI shading, labels
plt.savefig('figures/fig1_remote_productivity.pdf', dpi=300, bbox_inches='tight')
```

## Quality Bar

Figures are **ready** when:

- [ ] Each figure communicates something a table cannot
- [ ] Visual encoding is fully specified (x, y, color, size, error bars documented)
- [ ] Colorblind-safe palette used and tested
- [ ] Caption is self-explanatory (reader can understand without body text)
- [ ] Alt-text drafted for accessibility
- [ ] Dimensions and resolution meet venue requirements
- [ ] Code/specification for reproduction is documented
- [ ] No figure duplicates information already in a table
- [ ] Every figure is referenced in the manuscript text

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| 3D charts | Distort perception; nearly always worse than 2D | Use 2D |
| Pie charts | Hard to compare; no error representation | Use bar/dot plot |
| Rainbow colormaps (jet) | Not colorblind-safe; perceptually non-uniform | Use viridis/cividis |
| Bar chart starting at non-zero | Exaggerates differences | Start at 0 (or use dot plot) |
| Too many categories in legend | Unreadable | Max 5–6 categories; group others into "Other" |
| Missing error bars | Reader cannot assess uncertainty | Always show CI or SE |
| Caption says only "Results" | Uninformative | Caption must state the finding |
| JPEG for data figures | Lossy compression creates artifacts | Use PDF or PNG at 300 DPI |

## Minimal Output Format

```markdown
# Figure Specifications

## Figure 1: [Descriptive title]

- **Type**: [scatter / forest / coefficient / line / flow diagram]
- **Analytical purpose**: [what this figure communicates]
- **Data**: x=[variable], y=[variable], color=[variable], facet=[variable]
- **Palette**: [viridis / okabe-ito / grey-safe]
- **Dimensions**: [width]" × [height]" @ [DPI] DPI
- **Format**: [PDF / PNG]

**Caption**: "Figure 1. [Self-explanatory caption with findings]"

**Alt-text**: "[Accessible description]"

**Code**: [language] using [library]
```[code block]```
```
