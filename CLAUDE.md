# CLAUDE.md — Academic Deep Research Skills

## Project Overview

This is an **Academic Deep Research Skills** system: a contract-driven academic workflow covering the full lifecycle from literature review through manuscript production, compliance checking, submission, and presentation.

Canonical standard (cross-model):
- `standards/research-workflow-contract.yaml` — Task IDs, required outputs, quality gates
- `standards/mcp-agent-capability-map.yaml` — MCP tool mapping & primary/fallback agents
- `research-paper-workflow/references/workflow-contract.md` — **Task ID table** (A1–K4)
- `research-paper-workflow/references/platform-routing.md` — Task ID → workflow mapping
- Artifact root: `RESEARCH/[topic]/`

Local validation:
```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest tests.test_orchestrator_workflows -v
python3 -m bridges.orchestrator doctor --cwd .
./scripts/install_research_skill.sh --target all --project-dir /path/to/project --doctor
```

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

For consistency, ask users for `paper_type + task_id` when using `/paper`.

## How Task-ID Routing Works

When a user requests a specific task (e.g. "I need to do A1_5 hypothesis generation"), use the `/paper` workflow as the master router:

1. Read `.agent/workflows/paper.md` → it maps **every task ID** (A1–K4) to the correct sub-workflow or skill card
2. Follow the routing. Examples:
   - `A1` → use `question-refiner` skill → output `RESEARCH/[topic]/framing/research_question.md`
   - `A3` → `/build-framework` workflow
   - `B1` → `/lit-review` workflow
   - `F3` → `/paper-write` workflow
   - `H1` → `/submission-prep` workflow
   - `K1` → `/academic-present` workflow
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
Use `skills-core.md` for the consolidated skill reference (~10KB). Contains core purpose, process, and output format for each skill.

### Detailed Mode (Full Reference)
Load full skill files from `skills/[stage]/[skill-name].md` only when:
- First encounter with complex edge cases
- Need detailed output format templates
- Error recovery requiring fallback strategies

### Invocation Pattern
When a workflow says "Use the **skill-name** skill":
1. Check `skills-core.md` for the skill's core process
2. If sufficient: execute using core reference
3. If need detail: load `skills/[stage]/[skill-name].md` for full templates

### Skill Directory Structure

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
├── presentation/            # Slide deck spec, slides.md / slides.tex
├── bibliography.bib         # BibTeX references
└── ...
```

**Path Convention:** `[topic]` should be normalized: lowercase, hyphens for spaces (e.g., "AI Ethics" → `ai-ethics`).

## Multi-Model Collaboration

The `bridges/` directory provides multi-model collaboration via Codex and Gemini CLIs.

### Prerequisites

```bash
export OPENAI_API_KEY="..."
export ANTHROPIC_API_KEY="..."
export GOOGLE_API_KEY="..."
```

### Execution Modes

| Mode | Purpose | Unit of work |
|------|---------|--------------|
| `parallel` | Same prompt → multiple agents analyze → synthesis | Open-ended prompt |
| `task-run` | Single Task ID → serial draft → review → triad | One research task |
| `team-run` | Single Task ID → fanout workers → merge → review | Multiple work units (MVP: `B1`, `H3`) |
| `code-build` | Strict Stage-I academic code flow (`I5`→`I6`→`I7`→`I8`) | Method implementation |

### Usage Examples

```bash
# Preflight
python3 -m bridges.orchestrator doctor --cwd .

# Task-run with capability-map agent routing
python3 -m bridges.orchestrator task-run \
  --task-id F3 --paper-type empirical --topic ai-in-education --cwd . --triad

# Task-run with depth controls
python3 -m bridges.orchestrator task-run \
  --task-id F3 --paper-type empirical --topic ai-in-education --cwd . \
  --focus-output manuscript/manuscript.md \
  --research-depth deep --max-rounds 4

# Team-run parallel execution
python3 -m bridges.orchestrator team-run \
  --task-id B1 --paper-type systematic-review --topic ai-in-education --cwd .

# Strict Stage-I academic code flow
python3 -m bridges.orchestrator code-build \
  --method "Staggered DID" --topic policy-effects --domain econ --focus full --cwd .

# Targeted follow-up for Stage-I
python3 -m bridges.orchestrator code-build \
  --method "Transformer Fine-Tuning" --topic llm-bias --domain cs --focus full \
  --only-target I5:decision-1 --only-target I8:P1-01 --cwd .
```

### Key task-run Controls

- `--focus-output <path>`: restrict this run to specific contract output paths
- `--output-budget <n>`: cap how many contract outputs are active
- `--research-depth deep`: evidence-expansion, contradiction-check, narrow-claim pressure
- `--max-rounds <n>`: increases revision depth after review blocks
- `--only-target <id>`: for Stage-I, reload existing artifact and rerun only selected targets
- `--mcp-strict`: block execution when required MCP providers are unavailable
- `--skills-strict`: block execution when required skill spec files are missing
- `--triad`: request a third independent audit
- Built-in profiles: `focused-delivery`, `deep-research`, `strict-review`, `rapid-draft`, `default`

### External MCP Connector

- Configure command bridges via env vars such as `RESEARCH_MCP_SCHOLARLY_SEARCH_CMD`
- `doctor` checks runtime CLIs, API keys, standards files, and MCP command bindings
- Runtime execution defaults to non-interactive mode with hard timeout controls

## Development Notes

### When Adding New Skills
1. Create skill file in the appropriate stage sub-directory inside `skills/`
2. Register in `skills/registry.yaml` — this is the **single canonical skill version source**
3. Add a summary entry to `skills-core.md`
4. Update `CLAUDE.md` skill directory listing (this file)
5. Update `.gemini/research-skills.md` skill directory listing
6. Run `python3 scripts/validate_research_standard.py --strict` to verify

### When Modifying Workflows
1. Update workflow in `.agent/workflows/`
2. Verify cross-platform consistency: workflows must be discoverable across Claude, Codex, and Gemini
3. Do NOT use `argument-hint` in workflow YAML frontmatter (breaks Gemini compatibility)
4. Test with a sample research topic

### Cross-Platform Consistency

Three guide files must stay synchronized:
- `CLAUDE.md` (this file) — Claude Code reads when working in the repo
- `.gemini/research-skills.md` — Gemini reads when working in the repo
- `templates/CLAUDE.project.md` — installed to user projects as `CLAUDE.md`

All three must list the same workflow commands and skill directory structure.

### Key Architecture References
- Layer model and dependency direction: `docs/architecture.md`
- Edit rules and decision boundaries: `docs/conventions.md`
- Runtime cooperation: `docs/advanced/agent-skill-collaboration.md`
- Extend / contribute: `docs/advanced/extend-research-skills.md`
- Release automation: `scripts/release_automation.sh`
- Version sync: `scripts/sync_versions.py`
