---
name: research-paper-workflow
description: Standardized end-to-end workflow for academic paper production across Codex, Claude Code, and Gemini. Use when a user needs to choose a paper type (empirical, qualitative, systematic review, methods, theory), select a workflow stage, and produce consistent artifacts under RESEARCH/[topic]/ with explicit task IDs, quality gates, and submission-ready outputs.
---

# Research Paper Workflow

Run a model-agnostic paper workflow using shared Task IDs and artifact contracts.

This is a **self-contained skill package**. All assets needed for execution — workflows, skill specifications, output templates, standards, and agent roles — are bundled in subdirectories of this package. No external repo access is needed.

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

## Bundled Workflows

Full workflow definitions are included in the `workflows/` subdirectory of this skill package. When a user invokes any command above (e.g. `/paper`, `/lit-review`), read the corresponding file from `workflows/<command-name>.md` for the complete execution instructions.

The `workflows/paper.md` file is the **master router** — it maps every Task ID (A1–K4) to the correct sub-workflow or skill card. Start there for any task-ID-based request.

## Skill Directory Structure

The skill system covers the full research lifecycle across 11 stages:

```
skills/
├── A_framing/       (question-refiner, contribution-crafter, hypothesis-generator, theory-mapper, gap-analyzer, venue-analyzer)
├── B_literature/    (academic-searcher, paper-screener, paper-extractor, citation-snowballer, fulltext-fetcher, citation-formatter, concept-extractor, literature-mapper, reference-manager-bridge)
├── C_design/        (study-designer, rival-hypothesis-designer, robustness-planner, dataset-finder, variable-constructor, data-dictionary-builder, data-management-plan, prereg-writer, variable-operationalizer)
├── D_ethics/        (ethics-irb-helper, statement-generator, deidentification-planner)
├── E_synthesis/     (effect-size-calculator, evidence-synthesizer, quality-assessor, publication-bias-checker, qualitative-coding)
├── F_writing/       (manuscript-architect, analysis-interpreter, effect-size-interpreter, table-generator, figure-specifier, meta-optimizer, discussion-writer)
├── G_compliance/    (prisma-checker, reporting-checker, tone-normalizer)
├── J_proofread/     (ai-fingerprint-scanner, human-voice-rewriter, similarity-checker, final-proofreader)
├── H_submission/    (submission-packager, rebuttal-assistant, peer-review-simulation, fatal-flaw-detector, reviewer-empathy-checker, credit-taxonomy-helper, limitation-auditor)
├── I_code/          (code-builder, data-cleaning-planner, data-merge-planner, code-specification, code-planning, code-execution, code-review, reproducibility-auditor, stats-engine)
├── K_presentation/  (presentation-planner, slide-architect, slidev-scholarly-builder, beamer-builder)
├── Z_cross_cutting/ (academic-context-maintainer, metadata-enricher, model-collaborator, self-critique)
└── domain-profiles/ (economics, finance, psychology, biomedical, education, cs-ai, ...)
```

## Output Structure

```
RESEARCH/[topic]/
├── context/                 # Project-level research state + decision log
├── framing/                 # A-stage outputs (RQ, hypothesis, contribution, venue)
├── protocol.md              # Research protocol
├── search_log.md            # Reproducible search records
├── screening/               # Screening logs + PRISMA flow
├── notes/                   # Individual paper notes
├── extraction_table.md      # Data extraction table
├── synthesis.md             # Final synthesis report
├── manuscript/              # Outline, draft, claims map, figures plan
├── proofread/               # AI detection, humanization, similarity, final proofread
├── submission/              # Cover letter, checklist, statements
├── revision/                # Rebuttal + response materials
├── analysis/                # Code + data pipelines
├── presentation/            # Slide deck spec, slides.md / slides.tex
└── bibliography.bib         # BibTeX references
```

## Required Behavior

- Use the canonical task and output definitions in `references/workflow-contract.md`.
- Keep stage labels and task IDs unchanged across models.
- Do not infer stage order alphabetically when the contract exposes explicit ordering metadata.
- If a requested output is missing prerequisites, create a gap note and ask whether to:
  1. continue with placeholders, or
  2. run the prerequisite task first.
- Keep claims, methods, and evidence aligned (run integrity checks for stage `G`).
- When a workflow references `templates/<name>.md`, load the template from the `templates/` subdirectory of this package.

## Skill Loading Strategy

Three-tier loading for token efficiency. All paths are relative to this skill package directory:

1. **Quick lookup (~3KB):** Use `skills-summary.md` — skill names + one-line descriptions per stage. Use this to identify which skill to invoke.
2. **Default reference (~19KB):** Use `skills-core.md` — consolidated process descriptions, templates, and output formats. Use this when executing a skill.
3. **Full specification:** Load `skills/[stage]/[skill-name].md` — detailed edge cases, error recovery, quality bars, and verbose templates. Use this only when the core reference is insufficient.

## Bundled Assets

This package includes the following subdirectories:

| Directory | Contents |
|-----------|----------|
| `workflows/` | 16 workflow definitions (slash commands) |
| `references/` | Stage playbooks + workflow contract |
| `skills/` | 71 detailed skill spec files across 13 stage directories |
| `skills-summary.md` | Quick-reference skill index (~3KB) |
| `skills-core.md` | Consolidated skill reference (~19KB) |
| `templates/` | 44 output templates for manuscripts, submissions, ethics, etc. |
| `standards/` | Canonical contract YAML + capability map + agent profiles |
| `roles/` | 10 agent role definitions for orchestrator execution |

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
  - `references/stage-I-code.md` (tasks I1–I9)
  - `references/stage-K-presentation.md` (tasks K1–K4)
