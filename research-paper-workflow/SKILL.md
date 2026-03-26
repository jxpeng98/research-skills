---
name: research-paper-workflow
description: Standardized end-to-end workflow for academic paper production across Codex, Claude Code, and Gemini. Use when a user needs to choose a paper type (empirical, qualitative, systematic review, methods, theory), select a workflow stage, and produce consistent artifacts under RESEARCH/[topic]/ with explicit task IDs, quality gates, and submission-ready outputs.
---

# Research Paper Workflow

Run a model-agnostic paper workflow using shared Task IDs and artifact contracts.

Treat this package as the portable entry skill for all clients. The canonical internal contract and routing layers live in `standards/` plus the referenced stage playbooks.

## Quick Start

1. Ask for `paper_type`: `empirical`, `qualitative`, `systematic-review`, `methods`, or `theory`.
2. Ask for `task_id` from the contract (for example `F3` or `G1`).
3. Execute the task and write outputs to `RESEARCH/[topic]/` using the exact file paths.
4. Apply quality gates before submission tasks (`H1`, `H2`).

## Available Workflow Commands

These commands map to the same behavior across Codex, Claude Code, and Gemini:

```
/paper [topic] [venue]                # Master router — choose paper type + task ID
/lit-review [topic] [year range]     # Systematic literature review (PRISMA)
/paper-read [URL or DOI]             # Deep paper analysis
/find-gap [research area]            # Identify research gaps
/build-framework [theory/concept]    # Build theoretical framework
/academic-write [section] [topic]    # Academic writing assistance
/synthesize [topic] [outcome_id]     # Evidence synthesis / meta-analysis
/paper-write [topic] [type] [venue]  # Full manuscript drafting
/study-design [topic]                # Empirical study design
/ethics-check [topic]                # Ethics / IRB pack
/submission-prep [topic] [venue]     # Submission package
/rebuttal [topic]                    # Rebuttal / response to reviewers
/code-build [method] --domain ...    # Build academic research code
/proofread [topic]                   # AI de-trace / final proofreading
/academic-present [topic]            # Academic presentation preparation
```

## Skill Directory Structure

The skill system covers the full research lifecycle across 11 stages:

```
skills/
├── A_framing/       (question-refiner, hypothesis-generator, theory-mapper, gap-analyzer, venue-analyzer)
├── B_literature/    (academic-searcher, paper-screener, paper-extractor, citation-snowballer, fulltext-fetcher, citation-formatter, concept-extractor, literature-mapper, reference-manager-bridge)
├── C_design/        (study-designer, rival-hypothesis-designer, robustness-planner, dataset-finder, variable-constructor)
├── D_ethics/        (ethics-irb-helper, deidentification-planner)
├── E_synthesis/     (evidence-synthesizer, quality-assessor, publication-bias-checker)
├── F_writing/       (manuscript-architect, analysis-interpreter, effect-size-interpreter, table-generator, figure-specifier, meta-optimizer)
├── G_compliance/    (prisma-checker, reporting-checker, tone-normalizer)
├── H_submission/    (submission-packager, rebuttal-assistant, peer-review-simulation, fatal-flaw-detector, reviewer-empathy-checker)
├── I_code/          (code-builder, data-cleaning-planner, data-merge-planner, code-specification, code-planning, code-execution, code-review, reproducibility-auditor, stats-engine)
├── K_presentation/  (presentation-planner, slide-architect, slidev-scholarly-builder, beamer-builder)
├── Z_cross_cutting/ (metadata-enricher, model-collaborator, self-critique)
└── domain-profiles/ (economics, finance, psychology, biomedical, education, cs-ai, ...)
```

## Output Structure

```
RESEARCH/[topic]/
├── framing/                 # A-stage outputs (RQ, hypothesis, contribution, venue)
├── protocol.md              # Research protocol
├── search_log.md            # Reproducible search records
├── screening/               # Screening logs + PRISMA flow
├── notes/                   # Individual paper notes
├── extraction_table.md      # Data extraction table
├── synthesis.md             # Final synthesis report
├── manuscript/              # Outline, draft, claims map, figures plan
├── submission/              # Cover letter, checklist, statements
├── revision/                # Rebuttal + response materials
├── analysis/                # Code + data pipelines
├── presentation/            # Slide deck spec, slides.md / slides.tex
└── bibliography.bib         # BibTeX references
```

## Required Behavior

- Use the canonical task and output definitions in `references/workflow-contract.md`.
- Keep stage labels and task IDs unchanged across models.
- Treat this package as an entry surface, not as a replacement for repo-internal skill specs or capability-map routing.
- Do not infer stage order alphabetically when the contract exposes explicit ordering metadata.
- If a requested output is missing prerequisites, create a gap note and ask whether to:
  1. continue with placeholders, or
  2. run the prerequisite task first.
- Keep claims, methods, and evidence aligned (run integrity checks for stage `G`).

## Skill Loading Strategy

Two-tier loading for token efficiency:

1. **Default:** Use `skills-core.md` for consolidated skill reference (~10KB)
2. **Detail:** Load full `skills/[stage]/[skill-name].md` only when edge cases, error recovery, or verbose templates are needed

## References

- Task model + outputs: `references/workflow-contract.md`
- Platform routing map: `references/platform-routing.md`
- Coverage matrix: `references/coverage-matrix.md`
- Stage playbooks:
  - `references/stage-A-framing.md` (tasks A1–A5)
  - `references/stage-B-literature.md` (tasks B1–B6)
  - `references/stage-C-design.md` (tasks C1–C5)
  - `references/stage-D-ethics.md` (tasks D1–D3)
  - `references/stage-E-synthesis.md` (tasks E1–E5)
  - `references/stage-F-writing.md` (tasks F1–F6)
  - `references/stage-G-compliance.md` (tasks G1–G4)
  - `references/stage-J-proofread.md` (tasks J1–J4)
  - `references/stage-H-submission.md` (tasks H1–H4)
  - `references/stage-I-code.md` (tasks I1–I8)
  - `references/stage-K-presentation.md` (tasks K1–K4)
