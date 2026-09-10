# Stage K — Academic Presentation (K1-K4)

Use Stage K when a paper or research project needs a conference talk, seminar deck, defense deck, or compact presentation artifact.

## Stage Inputs

- `RESEARCH/[topic]/context/research_state.md`
- `RESEARCH/[topic]/manuscript/manuscript.md` or a stable outline
- `RESEARCH/[topic]/manuscript/claims_evidence_map.md` or `RESEARCH/[topic]/evidence/claim-evidence-ledger.csv`
- Target audience, time budget, and backend preference: `slidev`, `beamer`, or `pptx`

If inputs are missing, write `RESEARCH/[topic]/context/gap_notes.md` and mark the deck as a planning draft.

## K1 — Presentation Planning

Use `presentation-planner`.

Outputs:

- `RESEARCH/[topic]/presentation/presentation_plan.md`

Required content:

- Audience and venue context
- Talk thesis in one sentence
- Time budget by section
- Must-show evidence and must-cut material
- Risk register for likely audience objections

Quality gate:

- The plan must not mirror the manuscript section-by-section. It must define a talk-specific narrative arc.

## K2 — Slide Architecture

Use `slide-architect`.

Outputs:

- `RESEARCH/[topic]/presentation/slide_deck_spec.md`

Required content:

- Slide-by-slide assertion headings
- Evidence payload for each claim slide
- Visual treatment for tables, figures, quotes, equations, or diagrams
- Backend-neutral layout notes

Quality gate:

- Every evidence slide must map to a source artifact or a gap note.

## K3 — Slidev Build

Use `slidev-scholarly-builder`.

Outputs:

- `RESEARCH/[topic]/presentation/slidev/`
- `RESEARCH/[topic]/presentation/slides.bib`

Required content:

- `slides.md` with academic citation support
- Theme preset and export notes
- Speaker notes for technical or evidence-heavy slides

Quality gate:

- The generated deck must be runnable with Slidev and must keep citations, equations, and figures inspectable.

## K4 — Beamer Build

Use `beamer-builder`.

Outputs:

- `RESEARCH/[topic]/presentation/beamer/`
- `RESEARCH/[topic]/presentation/slides.bib`

Required content:

- `slides.tex` with BibLaTeX citation support
- Theme choice, math support, and overlay plan
- Build notes for PDF export

Quality gate:

- The generated Beamer deck must compile after bibliography files are present, or clearly list missing build prerequisites.

## Stage Handoff

At the end of K-stage work, write or update `RESEARCH/[topic]/context/stage_handoff.md` with:

- Completed presentation artifacts
- Decisions about backend, audience, and time budget
- Unresolved evidence gaps
- Risks for live presentation or Q&A
- Recommended next presentation revision task
