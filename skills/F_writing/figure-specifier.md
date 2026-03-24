---
id: figure-specifier
stage: F_writing
version: "0.2.2"
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
  - "Must generate reproducible code for each figure"
failure_modes:
  - "High-dimensional data resists 2D visualization"
  - "Venue resolution/size requirements conflict with readability"
tools: [filesystem, code-runtime]
tags: [writing, figures, visualization, accessibility, publication-ready]
domain_aware: true
---

# Figure Specifier Skill

Design and specify publication-quality figures with reproducible code.

## Related Task IDs

- `F5` (tables and figures)

## Output (contract path)

- `RESEARCH/[topic]/manuscript/figures/`

## Procedure

1. **Determine figure type** from data and message:
   - Scatter / line / bar / box / violin
   - Forest plot (meta-analysis)
   - Coefficient plot
   - Flow diagram (PRISMA, CONSORT)
   - Conceptual/framework diagram (Mermaid)
2. **Specify visual encoding**:
   - x/y axis: variable, scale, range
   - Color: variable, palette (colorblind-safe)
   - Size/shape: variable mapping
   - Facets: grouping variable
3. **Generate code** in target language:
   - Python: matplotlib/seaborn/plotly
   - R: ggplot2
   - Include exact figure dimensions for venue
4. **Accessibility check**:
   - Colorblind simulation (viridis/cividis palettes)
   - Alt-text draft for each figure
   - Sufficient contrast ratios
5. **Caption drafting**:
   - Descriptive caption following venue style
   - Source note if applicable

## Minimal Output Format

```markdown
# Figure 1: [Title]

**Type**: scatter plot
**Data**: x=variable_a, y=variable_b, color=group
**Palette**: viridis (colorblind-safe)
**Dimensions**: 6" × 4" @ 300 DPI
**Caption**: "Figure 1. ..."
**Alt-text**: "Scatter plot showing..."

## Code (Python)
\```python
import matplotlib.pyplot as plt
...
\```
```
