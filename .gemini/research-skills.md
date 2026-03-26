# Research Skills — Academic Deep Research System

## Project Overview

This is an **Academic Deep Research Skills** system providing systematic tools for the full research lifecycle: literature review, paper analysis, gap identification, study design, evidence synthesis, manuscript writing, compliance checking, and submission preparation.

Canonical standard (cross-model):
- `standards/research-workflow-contract.yaml`
- `research-paper-workflow/references/workflow-contract.md` — **Task ID table** (A1–I8)
- `research-paper-workflow/references/platform-routing.md` — Task ID → workflow mapping
- Artifact root: `RESEARCH/[topic]/`

## Quick Commands

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

## How Task-ID Routing Works

When a user requests a specific task (e.g. "I need to do A1_5 hypothesis generation"), use the `/paper` workflow as the master router:

1. Read `.agent/workflows/paper.md` → it maps **every task ID** (A1–I8) to the correct sub-workflow or skill card
2. Follow the routing. Examples:
   - `A1` → use `question-refiner` skill → output `RESEARCH/[topic]/framing/research_question.md`
   - `A3` → `/build-framework` workflow
   - `B1` → `/lit-review` workflow
   - `F3` → `/paper-write` workflow
   - `H1` → `/submission-prep` workflow
3. For detailed execution guidance on any task ID, read the corresponding **stage playbook**:
   - `research-paper-workflow/references/stage-A-framing.md` (tasks A1–A5)
   - `research-paper-workflow/references/stage-B-literature.md` (tasks B1–B6)
   - `research-paper-workflow/references/stage-C-design.md` (tasks C1–C5)
   - `research-paper-workflow/references/stage-D-ethics.md` (tasks D1–D3)
   - `research-paper-workflow/references/stage-E-synthesis.md` (tasks E1–E5)
   - `research-paper-workflow/references/stage-F-writing.md` (tasks F1–F6)
   - `research-paper-workflow/references/stage-G-compliance.md` (tasks G1–G4)
   - `research-paper-workflow/references/stage-J-proofread.md` (tasks J1–J4)
   - `research-paper-workflow/references/stage-H-submission.md` (tasks H1–H4)
   - `research-paper-workflow/references/stage-I-code.md` (tasks I1–I8)

## Skill Loading Strategy

Skills are organized in two tiers to optimize token usage:

### Default Mode (Token-Efficient)
Use `skills-core.md` for the consolidated skill reference (~8KB). Contains core purpose, process, and output format for each skill.

### Detailed Mode (Full Reference)
Load full skill files from `skills/[stage]/[skill-name].md` only when:
- First encounter with complex edge cases
- Need detailed output format templates
- Error recovery requiring fallback strategies

### Skill Directory Structure
```
skills/
├── A_framing/     (question-refiner, gap-analyzer, hypothesis-generator, theory-mapper, venue-analyzer)
├── B_literature/  (academic-searcher, paper-screener, paper-extractor, citation-formatter, ...)
├── C_design/      (study-designer, ...)
├── D_ethics/      (ethics-irb-helper, ...)
├── E_synthesis/   (evidence-synthesizer, ...)
├── F_writing/     (manuscript-architect, ...)
├── G_compliance/  (reporting-checker, prisma-checker, ...)
├── H_submission/  (submission-packager, rebuttal-assistant, ...)
├── I_code/        (code-specification, code-planning, code-execution, code-review, ...)
├── K_presentation/ (presentation-planner, slide-architect, slidev-scholarly-builder, beamer-builder)
└── Z_cross_cutting/ (citation-formatter, quality-assessor, ...)
```

### Invocation Pattern
When a workflow says "Use the **skill-name** skill":
1. Check `skills-core.md` for the skill's core process
2. If sufficient: execute using core reference
3. If need detail: load `skills/[stage]/[skill-name].md` for full templates

## Output Structure

```
RESEARCH/[topic]/
├── framing/                 # A-stage outputs (RQ, hypothesis, contribution, venue)
├── protocol.md              # Research protocol
├── search_strategy.md       # Database-specific queries
├── search_log.md            # Reproducible search records
├── screening/               # Screening logs + PRISMA flow
├── notes/                   # Individual paper notes
├── extraction_table.md      # Data extraction table
├── quality_table.md         # Quality assessment
├── synthesis_matrix.md      # Theme × Paper matrix
├── synthesis.md             # Final synthesis report
├── manuscript/              # Outline, draft, claims map, figures plan
├── submission/              # Cover letter, checklist, statements
├── revision/                # Rebuttal + response materials
├── analysis/                # Code + data pipelines
├── bibliography.bib         # BibTeX references
└── ...
```

**Path Convention:** `[topic]` should be normalized: lowercase, hyphens for spaces (e.g., "AI Ethics" → `ai-ethics`).

## Multi-Model Collaboration

The `bridges/` directory provides multi-model collaboration via Codex and Gemini CLIs:

```bash
python3 -m bridges.orchestrator doctor --cwd .
python3 -m bridges.orchestrator parallel --prompt "Analyze this study design" --cwd . --summarizer gemini
python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic your-topic --cwd . --triad
```
