---
id: slide-architect
stage: K_presentation
description: "Design individual slide content specs using assertion-evidence format, with layout mapping to Slidev/Beamer/PPTX backends."
inputs:
  - type: PresentationPlan
    description: "Story arc, slide blueprint, and audience calibration"
  - type: Manuscript
    description: "Source manuscript for content extraction"
  - type: FigureSpecs
    description: "Publication figures to adapt for slides"
    required: false
outputs:
  - type: SlideDeckSpec
    artifact: "presentation/slide_deck_spec.md"
constraints:
  - "Each slide must have exactly one takeaway message"
  - "Must map every slide type to all three backends"
  - "Must adapt paper figures for slide resolution and viewing distance"
failure_modes:
  - "Slides contain too much text (paper-on-slides)"
  - "Figures not adapted for projection"
tools: [filesystem]
tags: [presentation, slides, design, assertion-evidence, layout-mapping]
domain_aware: false
---

# Slide Architect Skill

Design the content and visual structure of every slide — specifying layout, message, evidence, and speaker notes — in a backend-agnostic format that can be rendered by Slidev, Beamer, or PPTX.

## Related Task IDs

- `K2` (slide design)

## Output (contract path)

- `RESEARCH/[topic]/presentation/slide_deck_spec.md`

## When to Use

- After `presentation-planner` has produced a slide blueprint
- When designing slide content before choosing or generating the final format
- When adapting a presentation across different backends

## Process

### Step 1: Understand the Assertion-Evidence Framework

**Traditional slide** (BAD):
```
Title: "Results of Regression Analysis"
• β₁ = 0.34 (p < .05)
• β₂ = -0.12 (p > .05)
• R² = 0.23
• Model was significant at 5% level
```

**Assertion-Evidence slide** (GOOD):
```
Title (assertion): "Remote work increases productivity by 34%"
Body (evidence): [Figure showing the relationship with CI bands]
```

> **Rule**: The slide title IS the takeaway. The body IS the evidence. If you can't state the takeaway in one sentence, the slide is doing too much.

### Step 2: Map Slide Types to Layouts

| Slide Type | Purpose | Title Convention | Body Convention |
|-----------|---------|-----------------|-----------------|
| **Cover / Title** | Opening slide | Talk title (full) | Authors, affiliation, venue, date |
| **Outline / Agenda** | Roadmap | "Roadmap" or "Today's Talk" | 3–5 numbered items |
| **Section Divider** | Signal transition | Section name | Optional subtitle |
| **Content** | General information | Assertion sentence | 3–5 bullets OR figure |
| **Figure** | Present a visual | Assertion about what figure shows | Full-width figure + minimal annotation |
| **Table** | Present data summary | Assertion about what table shows | Simplified table (≤5 rows, ≤6 cols) |
| **Methodology** | Explain approach | "How We Studied This" | Diagram or step-by-step visual |
| **Results** | Present findings | Finding statement | Figure + key numbers |
| **Comparison** | Contrast two things | "X vs Y" or "[Finding] in Context" | Side-by-side columns |
| **Quote** | Highlight a citation | Key quote text (large) | Source attribution |
| **Timeline** | Show progression | "Research Timeline" | Timeline visual |
| **Acknowledgments** | Credits | "Acknowledgments" | Names, funding, logos |
| **References** | Bibliography | "References" or "Selected References" | Citation list or auto-generated |
| **Appendix** | Backup content | "Appendix: [Topic]" | Detailed content |

### Step 3: Layout Backend Mapping

| Slide Type | Slidev Scholarly | LaTeX Beamer | PPTX |
|-----------|-----------------|--------------|------|
| Cover | `layout: cover` | `\begin{frame}\titlepage\end{frame}` | Title Slide layout |
| Outline | `layout: agenda` | `\begin{frame}{Outline}\tableofcontents\end{frame}` | Title + Content |
| Section | `layout: section` | `\section{Title}` (auto-generated) | Section Header |
| Content | `layout: default` or `layout: bullets` | `\begin{frame}{Title}\begin{itemize}...\end{itemize}\end{frame}` | Title + Content |
| Figure | `layout: figure` | `\begin{frame}{Title}\includegraphics[width=\textwidth]{fig}\end{frame}` | Blank + centered image |
| Table | `layout: default` | `\begin{frame}{Title}\begin{table}...\end{table}\end{frame}` | Title + Content |
| Methodology | `layout: methodology` | Custom `columns` frame | Two Content |
| Results | `layout: results` | Custom frame with figure | Title + Content |
| Comparison | `layout: compare` | `\begin{columns}...\end{columns}` | Comparison layout |
| Two-column | `layout: two-cols` | `\begin{columns}...\end{columns}` | Two Content |
| Image + text | `layout: image-left` / `image-right` | `columns` + `\includegraphics` | Content + Image |
| Quote | `layout: quote` | `\begin{frame}\begin{quote}...\end{quote}\end{frame}` | Blank + centered text |
| Timeline | `layout: timeline` | TikZ timeline | SmartArt |
| Acknowledgments | `layout: acknowledgments` | plain frame | Blank |
| References | `layout: references` | `\begin{frame}[allowframebreaks]\printbibliography\end{frame}` | Title + Content |
| End | `layout: end` | Custom "Thank You" frame | Blank |

### Step 4: Design Individual Slides

For each slide in the blueprint, create a specification:

```markdown
## Slide [#]: [Assertion Title]

- **Type**: [content / figure / methodology / results / ...]
- **Layout**: [backend-specific layout name]
- **Assertion** (title): "[One-sentence takeaway]"
- **Evidence** (body):
  - [Figure/table/bullets that PROVE the assertion]
  - Source: [manuscript section/figure reference]
- **Annotations**: [Arrows, highlights, or labels to add to the visual]
- **Speaker Notes**: [What to SAY — not what's on the slide]
- **Transition**: [How this connects to the next slide]
- **Time**: [seconds]
```

### Step 5: Adapt Paper Figures for Slides

Paper figures are optimized for 300 DPI print at journal column width. Slide figures need different treatment:

| Aspect | Paper Figure | Slide Figure |
|--------|-------------|-------------|
| **Resolution** | 300 DPI, small size | 150 DPI, full-screen (1920×1080 or 4:3) |
| **Text size** | 8–10 pt is fine | ≥20 pt minimum for readability |
| **Line thickness** | 0.5–1 pt | 2–3 pt for visibility |
| **Color** | Can rely on subtle differences | High-contrast; account for projector washout |
| **Complexity** | Multi-panel, dense | Simplify — one panel per slide; progressive reveal |
| **Annotations** | Minimal | Add arrows, highlights, callout boxes |
| **Background** | White default | Match slide background (may need transparent) |

**Adaptation decision tree**:
```
Can the paper figure be read from 5 meters away?
├── Yes → Use as-is (maybe increase text size)
└── No → Simplify:
    ├── Can you split into multiple slides? → One panel per slide
    ├── Can you zoom into the key region? → Crop + annotate
    └── Must you redesign? → Rebuild with slide-optimized specs
```

### Step 6: Design Speaker Notes

| Component | What to Include | What NOT to Include |
|-----------|----------------|---------------------|
| **Opening line** | How to start talking about this slide | Verbatim reading of slide text |
| **Key point** | What the audience should remember | Every detail from the paper |
| **Transition** | How to bridge to the next slide | "And now, the next slide..." |
| **Backup detail** | Extra info if someone asks | Full methodology explanation |
| **Timing cue** | "If past 8:00, skip this" | — |

### Step 7: Plan Animations and Progressive Reveal

| Technique | When to Use | Slidev | Beamer |
|-----------|------------|--------|--------|
| **Click-reveal bullets** | Complex list; avoid cognitive overload | `v-click` | `\pause` or `\onslide` |
| **Figure build-up** | Layer-by-layer diagram explanation | `v-clicks` on components | `\only<n>{...}` overlays |
| **Highlight** | Draw attention to specific data point | `<Highlight>` component | `\textbf{}` or `\alert{}` |
| **Fade** | De-emphasize already-discussed content | CSS opacity | `\onslide<n->{...}` with `\color{gray}` |
| **None** | Simple, self-explanatory content | Default | Default |

> **Rule of thumb**: Use progressive reveal ONLY when it aids comprehension. Don't animate just because you can.

## Quality Bar

The slide deck spec is **ready** when:

- [ ] Every slide has exactly one assertion (takeaway message)
- [ ] Evidence supports (not restates) the assertion
- [ ] Layout mapped to all target backends
- [ ] Paper figures adapted for projection (text ≥20pt, high contrast)
- [ ] Speaker notes written for every content slide
- [ ] Animation/reveal used purposefully, not decoratively
- [ ] Total content fills 80–85% of allotted time (buffer for pauses)
- [ ] Appendix slides cross-referenced to anticipated questions

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Descriptive titles | "Results" — says nothing | Assertion title: "Remote work increases productivity by 34%" |
| Walls of text | Audience reads instead of listening | Max 20 words per slide; use figures as evidence |
| Unreadable figures | Text too small; colors wash out on projector | Adapt per Step 5; test on projector |
| No speaker notes | Presentation quality depends on memory | Write notes for every slide; practice with them |
| Over-animated | Distracting transitions dominate content | Animate only for comprehension, not decoration |
| Copy-paste from paper | Paragraphs on slides | Rewrite as assertion + visual evidence |

## Minimal Output Format

```markdown
# Slide Deck Specification

## Metadata
- Talk title: [title]
- Talk type: [conference / seminar / job talk]
- Duration: [minutes]
- Backend: [slidev-scholarly / beamer / pptx]
- Total slides: [n] content + [n] appendix

## Slides

### Slide 1: [Talk Title]
- Type: cover
- Layout: cover (Slidev) / titlepage (Beamer)
- Content: Authors, affiliation, venue, date
- Time: 0:15

### Slide 2: [Assertion]
- Type: content
- Layout: default
- Assertion: "[one sentence]"
- Evidence: [figure / bullets / table]
- Speaker Notes: "[what to say]"
- Time: 1:00

## Appendix Slides
### A1: [Topic]
- Anticipates: "[likely question]"
```
