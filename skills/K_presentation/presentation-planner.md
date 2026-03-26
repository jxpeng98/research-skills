---
id: presentation-planner
stage: K_presentation
description: "Design the story arc, content budget, and audience calibration for an academic presentation."
inputs:
  - type: Manuscript
    description: "Finalized or near-final manuscript"
  - type: UserQuery
    description: "Talk type, time limit, audience description, venue details"
outputs:
  - type: PresentationPlan
    artifact: "presentation/presentation_plan.md"
constraints:
  - "Must match slide count to allocated time"
  - "Must explicitly mark content as 'show', 'cut', or 'appendix'"
  - "Must adapt depth to audience expertise level"
failure_modes:
  - "Trying to present the entire paper instead of telling a story"
  - "No time budget leading to rushed delivery"
tools: [filesystem]
tags: [presentation, planning, story-arc, audience, time-management]
domain_aware: false
---

# Presentation Planner Skill

Design the story arc, content budget, and audience strategy for an academic talk — deciding what to show, what to cut, and how to structure the narrative for maximum impact within a time constraint.

## Related Task IDs

- `K1` (presentation planning)

## Output (contract path)

- `RESEARCH/[topic]/presentation/presentation_plan.md`

## When to Use

- After manuscript is substantially complete (or results are available)
- When preparing for a conference talk, seminar, job talk, or poster session
- When adapting a single paper for different audiences or time slots
- Before starting any slide creation

## Process

### Step 1: Classify the Talk Type

| Talk Type | Duration | Slides | Depth | Primary Goal |
|-----------|----------|--------|-------|-------------|
| **Lightning talk** | 3–5 min | 5–8 | Hook only | Spark curiosity; drive audience to paper/poster |
| **Conference talk** | 15–20 min | 15–20 | Core RQ + key result | Communicate contribution; build credibility |
| **Extended talk** | 30 min | 20–25 | Full argument | Full narrative with discussion |
| **Seminar** | 45–60 min | 30–40 | Deep dive | Teach + persuade; expect interruptions |
| **Job talk** | 45–60 min | 35–45 | Deep + broad | Demonstrate research identity + future agenda |
| **Poster pitch** | 2–3 min | Poster | Selective | Attract visitors; start conversation |
| **3-Minute Thesis** | 3 min | 1 | Lay summary | Communicate impact to non-specialists |

**Time-per-slide rule**: Plan for **60–90 seconds per content slide**. Title, outline, and transition slides take ~15 seconds each.

### Step 2: Map Manuscript Sections to Slide Inventory

For each manuscript section, decide what gets presented:

| Manuscript Section | Slides | Decision | Rationale |
|-------------------|--------|----------|-----------|
| Abstract | 0 | Cut | The talk IS the abstract |
| Introduction (motivation) | 2–3 | **Show** | Hook + establish importance |
| Literature review | 1–2 | **Show** (condensed) | Position your work; not exhaustive review |
| Theory / Framework | 1–2 | **Show** if novel | Skip if audience knows the theory |
| Research questions / Hypotheses | 1 | **Show** | Clear statement of what you tested |
| Design / Method | 2–4 | **Show** | Enough for credibility; details in appendix |
| Results | 3–6 | **Show** (key findings) | One figure/table per result; cut minor results |
| Discussion | 1–2 | **Show** (implications) | Don't re-state results; focus on "so what?" |
| Limitations | 0–1 | **Appendix** or brief mention | Anticipate questions but don't defend preemptively |
| Conclusion / Future work | 1 | **Show** | End with your contribution and vision |
| References | 0–1 | **Appendix** | Only if convention requires; use inline citations |

> **The #1 mistake in academic presentations**: Trying to present the paper page-by-page. A talk is NOT a paper — it's a story about the paper.

### Step 3: Design the Story Arc

Choose the narrative structure that best fits your content:

#### Arc A: Three-Act Structure (Most Common)

```
Act 1: SETUP (20% of time)
├── Hook: surprising fact, compelling question, or real-world scenario
├── Context: what we know and why it matters
└── Gap: what we DON'T know (your contribution)

Act 2: INVESTIGATION (60% of time)
├── Approach: how you tackled the problem
├── Evidence: key results (figures-first, not tables)
└── Interpretation: what the results mean

Act 3: RESOLUTION (20% of time)
├── Answer: the clear "so what?"
├── Implications: practical + theoretical
└── Future: where this leads
```

#### Arc B: Claim-First / Pyramid (For Experienced Audiences)

```
CLAIM: State your finding upfront (slide 2)
├── Evidence 1: strongest support
├── Evidence 2: complementary support
├── Evidence 3: robustness / boundary conditions
└── IMPLICATIONS: what this changes
```

#### Arc C: Mystery / Puzzle (For Engaging Seminars)

```
PUZZLE: Present a paradox or anomaly
├── Clue 1: what existing work says
├── Clue 2: where it breaks down
├── YOUR SOLUTION: your theoretical/empirical insight
├── EVIDENCE: show it works
└── RESOLUTION: what we now understand
```

### Step 4: Calibrate for Audience

| Audience Type | Jargon Level | Method Detail | Theory Depth | Implication Focus |
|--------------|-------------|---------------|-------------|-------------------|
| **Domain specialists** (same field) | High | High | Can assume shared knowledge | Theoretical/methodological |
| **Broad disciplinary** (same discipline) | Medium | Medium | Explain briefly | Both theoretical + practical |
| **Interdisciplinary** (mixed fields) | Low | Low (focus on intuition) | Explain fully | Practical / real-world |
| **General public / policy** | Minimal | Minimal | Skip | Impact / action / "why it matters" |
| **PhD committee / defense** | High | Very high | Deep | Demonstrate mastery |

**Adaptation checklist**:
- [ ] Every acronym defined on first use?
- [ ] Method described at appropriate abstraction level?
- [ ] Results shown with visual before numbers?
- [ ] Implications connected to audience's interests?

### Step 5: Create the Slide Blueprint

```markdown
## Slide Blueprint

| # | Type | Title | Key Message | Content | Time |
|---|------|-------|-------------|---------|------|
| 1 | Title | [Talk Title] | — | Authors, affiliation, venue | 0:15 |
| 2 | Outline | Roadmap | 3 parts | — | 0:15 |
| 3 | Content | Motivation | Why this matters | Hook + real-world example | 1:30 |
| 4 | Content | The Gap | What we don't know | 2–3 bullets + key citation | 1:00 |
| 5 | Content | Research Question | Clear RQ | Bold centered text | 0:30 |
| 6 | Section | Method | — | Section divider | 0:15 |
| 7 | Figure | Design Overview | Our approach | Diagram of study design | 1:30 |
| 8 | Content | Data & Sample | Credible sample | Key descriptives | 1:00 |
| 9 | Section | Results | — | Section divider | 0:15 |
| 10 | Figure | Main Result | [Key finding] | Central figure | 1:30 |
| 11 | Figure | Secondary Result | [Supporting finding] | Table or figure | 1:00 |
| 12 | Content | Robustness | Results are robust | Brief summary table | 1:00 |
| 13 | Section | Discussion | — | Section divider | 0:15 |
| 14 | Content | Implications | So what? | 3 bullets: theoretical, practical, future | 1:30 |
| 15 | Content | Thank You + RQ Recap | — | Contact info, QR to paper | 0:30 |

**Total time**: ~12:30 (leaves buffer for pauses and transitions in a 15-min slot)
```

### Step 6: Plan Appendix Slides

Prepare "backup" slides you CAN show if asked, but won't present by default:

| Appendix Slide | Anticipates Question |
|---------------|---------------------|
| Detailed sample demographics | "Tell me more about your sample" |
| Full regression table | "What were the exact coefficients?" |
| Robustness check details | "What if you used a different method?" |
| Sensitivity analysis | "How sensitive are results to assumptions?" |
| Limitations in detail | "What are the limitations?" |
| Additional figures | "Did you look at [subgroup]?" |

### Step 7: Choose Output Backend

| Backend | Best For | Prerequisites | Output |
|---------|---------|---------------|--------|
| **Slidev + scholarly** | Tech-savvy audience; web accessibility; code-friendly | Node.js, npm | Web + PDF |
| **LaTeX Beamer** | Venues expecting .tex; math-heavy talks; LaTeX users | LaTeX distribution | PDF |
| **PowerPoint (PPTX)** | Collaborators need editable format; institutional template | PowerPoint or python-pptx | .pptx |

> After choosing a backend, proceed to the corresponding builder skill: `slidev-scholarly-builder`, `beamer-builder`, or construct manually for PPTX.

## Quality Bar

The presentation plan is **ready** when:

- [ ] Talk type classified with time budget
- [ ] Story arc selected and justified
- [ ] Slide inventory maps every manuscript section to show/cut/appendix
- [ ] Slide blueprint lists every slide with type, title, message, and time
- [ ] Audience calibration applied (jargon, depth, implications)
- [ ] Total slide time is ≤ 85% of allotted time (buffer for pauses/transitions)
- [ ] Appendix slides prepared for anticipated questions
- [ ] Output backend selected

## Common Pitfalls

| Pitfall | Problem | Fix |
|---------|---------|-----|
| Paper-on-slides | Reading bullets verbatim; 40+ text-heavy slides | Start from story arc, not manuscript |
| No time budget | Running over time; rushing the end | Assign time per slide; rehearse |
| Too much method | Audience loses thread in technical details | Method should enable trust, not teach replication |
| Burying the lead | Key result appears on slide 25 of 30 | Use claim-first or show result within first 40% |
| No hook | Audience disengages in first 2 minutes | Start with a question, surprise, or real-world scenario |
| Ignoring audience | Specialists get bored; generalists get lost | Calibrate jargon, depth, and examples |
| No appendix slides | Caught off-guard by Q&A | Prepare 5–10 backup slides |
