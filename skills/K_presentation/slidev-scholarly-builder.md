---
id: slidev-scholarly-builder
stage: K_presentation
description: "Generate a ready-to-run Slidev presentation using slidev-theme-scholarly with academic layouts, BibTeX citations, and theme presets."
inputs:
  - type: SlideDeckSpec
    description: "Slide-by-slide content specification from slide-architect"
  - type: Bibliography
    description: "BibTeX references for inline citations"
    required: false
  - type: FigureSpecs
    description: "Adapted slide-ready figures"
    required: false
outputs:
  - type: SlidevDeck
    artifact: "presentation/slides.md"
  - type: BibTeXFile
    artifact: "presentation/references.bib"
constraints:
  - "Must use slidev-theme-scholarly layouts and components"
  - "Must include working BibTeX citation integration"
  - "Must select an appropriate visual preset"
failure_modes:
  - "Node.js not installed"
  - "BibTeX file has malformed entries"
tools: [filesystem, code-runtime]
tags: [presentation, slidev, scholarly, markdown, citations, web]
domain_aware: false
---

# Slidev Scholarly Builder Skill

Generate a complete, ready-to-run Slidev presentation using `slidev-theme-scholarly` — with academic layouts, inline BibTeX citations, visual presets, and export capabilities.

## Related Task IDs

- `K3` (Slidev presentation generation)

## Output (contract path)

- `RESEARCH/[topic]/presentation/slides.md`
- `RESEARCH/[topic]/presentation/references.bib`

## When to Use

- When Slidev + scholarly is chosen as the output backend
- When you need web-accessible slides with live navigation
- When citations and academic formatting are essential
- When you want code-friendly, version-controlled slides

## Process

### Step 1: Scaffold the Project

```bash
# Option A: CLI scaffolding (recommended)
npx -y --package slidev-theme-scholarly sch init my-talk --template academic

# Option B: Manual setup
mkdir presentation && cd presentation
npm init -y
npm i -D @slidev/cli@latest slidev-theme-scholarly
```

Generated structure:
```
presentation/
├── slides.md            # Main slide deck
├── references.bib       # BibTeX bibliography
├── package.json         # Dependencies
└── public/              # Static assets (images, figures)
    └── figures/
```

### Step 2: Configure Frontmatter

The first slide's YAML frontmatter controls the entire deck:

```yaml
---
theme: scholarly
title: "Your Paper Title: Full Title Goes Here"
info: |
  Brief description of the talk and paper.
authors:
  - name: First Author
    institution: University A
  - name: Second Author
    institution: University B
footerLeft: "Author et al."
footerMiddle: "Conference Name 2026"
footerRight: ""
themeConfig:
  preset: oxford-burgundy    # see preset guide below
  font: modern               # 'modern' | 'traditional' | 'minimal'
  paginationX: r             # pagination position
  paginationY: t
---
```

### Step 3: Select a Visual Preset

| Preset | Color Palette | Best For |
|--------|-------------|---------|
| `oxford-burgundy` | Deep red + cream | Traditional academic; humanities |
| `yale-blue` | Navy + white | Business schools; formal presentations |
| `princeton-orange` | Orange + black | Engineering; tech conferences |
| `nordic-blue` | Light blue + white | Clean, modern; Scandinavian aesthetic |
| `monochrome` | Black + white + gray | Maximum readability; printable |
| `warm-sepia` | Warm brown + cream | Relaxed seminars; humanities |
| `high-contrast` | Black + yellow/white | Accessibility; large venues |

Apply preset via CLI:
```bash
npx sch theme apply oxford-burgundy --font traditional --file slides.md
# or apply preset only
npx sch theme preset apply yale --file slides.md
```

### Step 4: Use Structure Layouts

#### Cover Slide
```markdown
---
layout: cover
---

# Your Talk Title

Subtitle or conference name

<!-- Speaker notes go here -->
```

#### Section Divider
```markdown
---
layout: section
---

# Method

How we studied this
```

#### Agenda / Outline
```markdown
---
layout: agenda
---

# Today's Talk

1. **Motivation** — Why this matters
2. **Method** — How we studied it
3. **Results** — What we found
4. **Implications** — So what?
```

#### End Slide
```markdown
---
layout: end
---

# Thank You

Questions?

Contact: email@university.edu
```

### Step 5: Use Content Layouts

#### Default Content
```markdown
---
layout: default
---

# Remote Work Increases Productivity by 34%

- Controlled for tenure, education, and department
- Effect is robust to multiple specifications
- Strongest for knowledge workers (β = 0.42)
```

#### Bullets with Click-Reveal
```markdown
---
layout: bullets
---

# Three Key Contributions

<v-clicks>

1. **Theoretical**: Extends autonomy theory to remote contexts
2. **Methodological**: Novel instrument for measuring WFH productivity
3. **Practical**: Evidence-based policy for hybrid work design

</v-clicks>
```

#### Two Columns
```markdown
---
layout: two-cols
---

# Comparing Methods

::left::

### Quantitative
- N = 5,000 employees
- Panel data (2019–2023)
- Fixed effects regression

::right::

### Qualitative
- 28 semi-structured interviews
- Thematic analysis
- Member checking
```

#### Figure
```markdown
---
layout: figure
figureCaption: "Figure 1: Productivity gains by remote work intensity (95% CI)"
figureUrl: /figures/main_result.png
---

# Remote Work Has a Non-Linear Effect on Productivity
```

#### Image Left / Right
```markdown
---
layout: image-right
image: /figures/study_design.png
---

# Study Design

- Quasi-experimental design
- Difference-in-differences
- Treatment: COVID-induced remote work policy
- Control: Essential workers (on-site)
```

### Step 6: Use Academic Layouts

#### Methodology
```markdown
---
layout: methodology
---

# Research Design

::methodology::

**Design**: Difference-in-differences
**Sample**: 5,000 employees across 12 firms
**Period**: 2019–2023 (quarterly)
**Treatment**: COVID remote work policy (March 2020)
**Outcome**: Productivity index (output/hours)
```

#### Results
```markdown
---
layout: results
---

# Main Findings

::results::

| Variable | Coefficient | SE | p |
|----------|------------|-----|---|
| Remote ratio | 0.34*** | 0.08 | <.001 |
| Team size (mod.) | -0.05* | 0.02 | .03 |
| Interaction | 0.12** | 0.04 | .008 |

R² = 0.23, N = 5,000
```

#### Comparison
```markdown
---
layout: compare
---

# Our Approach vs. Prior Work

::left::

### Prior Studies
- Cross-sectional designs
- Self-reported productivity
- No causal identification

::right::

### This Study
- Panel data with fixed effects
- Objective productivity measures
- Quasi-experimental variation
```

#### Timeline
```markdown
---
layout: timeline
---

# Research Timeline

- **2019 Q1-Q4**: Baseline data collection
- **2020 Q1**: COVID policy shock (treatment)
- **2020 Q2-2023 Q4**: Post-treatment observation
- **2024 Q1**: Analysis and write-up
```

### Step 7: Use Components

#### Inline Citations
```markdown
Previous work has established this relationship @smith2020.
The effect size is consistent with meta-analytic estimates !@jones2021.
```
- `@citekey` → narrative citation: "Smith (2020)"
- `!@citekey` → parenthetical citation: "(Jones, 2021)"

#### Theorem / Block
```markdown
<Theorem type="theorem" title="Proposition 1">
If autonomy increases with remote work, and autonomy mediates
the relationship between work arrangement and productivity,
then remote work will increase productivity.
</Theorem>

<Block type="info" title="Key Finding">
The effect is driven by knowledge workers (β = 0.42)
rather than routine workers (β = 0.08, n.s.).
</Block>
```

#### Keywords
```markdown
<Keywords :keywords="['remote work', 'productivity', 'autonomy theory', 'quasi-experiment']" />
```

#### Steps
```markdown
<Steps :steps="[
  'Identify treatment and control groups',
  'Verify parallel trends assumption',
  'Estimate difference-in-differences model',
  'Run robustness checks'
]" />
```

#### Highlight
```markdown
The key result is that remote work increases productivity by
<Highlight>34% for knowledge workers</Highlight>.
```

### Step 8: Add References Slide

```markdown
---
layout: references
---

# References

<!-- Bibliography is auto-generated from references.bib -->
<!-- Only cited works (@citekey) appear automatically -->
```

For custom placement within a slide:
```markdown
[[bibliography]]
```

### Step 9: BibTeX Integration

Create `references.bib` alongside `slides.md`:

```bibtex
@article{smith2020,
  author = {Smith, Alice and Jones, Bob},
  title = {Remote Work and Productivity: A Natural Experiment},
  journal = {Journal of Labor Economics},
  year = {2020},
  volume = {38},
  pages = {1--30},
  doi = {10.1234/jle.2020.001}
}
```

> Citations referenced with `@smith2020` or `!@smith2020` in slides will be auto-resolved from this file.

### Step 10: Export and Preview

```bash
# Preview with live reload
npx slidev

# Export to PDF
npx slidev export

# Export to PDF with dark mode
npx slidev export --dark

# Build as SPA for hosting
npx slidev build
```

### Step 11: Useful CLI Commands

```bash
# List available layouts
npx sch layout list

# List available components
npx sch component list

# List available presets
npx sch theme list

# Append academic snippets
npx sch snippet append theorem --file slides.md
npx sch snippet append references --file slides.md

# Apply workflow skeleton
npx sch workflow apply paper --file slides.md

# Check project health
npx sch doctor
```

## Quality Bar

The Slidev deck is **ready** when:

- [ ] Frontmatter configured: theme, authors, footer, preset
- [ ] Every slide uses an appropriate scholarly layout
- [ ] Assertions in titles; evidence in bodies
- [ ] All inline citations resolve against `references.bib`
- [ ] References slide uses `layout: references`
- [ ] Figures in `/public/figures/` and referenced correctly
- [ ] Speaker notes present for content slides (HTML comments)
- [ ] Animations (`v-click`) used only where needed
- [ ] PDF export tested: `npx slidev export`
- [ ] Presentation runs: `npx slidev` with no errors

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Missing `references.bib` | Citations render as raw `@citekey` | Create bib file alongside slides.md |
| Wrong image path | Figure not found | Use `/figures/name.png` (relative to `public/`) |
| Overusing `v-clicks` | Slow, clickety presentation | Only animate for comprehension |
| No speaker notes | Presenter forgets key points | Add `<!-- notes -->` to every content slide |
| Not testing PDF export | PDF looks different from web | Run `npx slidev export` and review |
| Using raw CSS when layout exists | Inconsistent styling | Use scholarly layouts; consult `npx sch layout list` |
