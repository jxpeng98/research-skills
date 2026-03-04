# CLAUDE.md - Academic Deep Research Skills

## Project Overview

This is an **Academic Deep Research Skills** system designed for Claude Code, providing systematic tools for literature review, paper analysis, gap identification, and academic writing.

Canonical standard (cross-model):
- `standards/research-workflow-contract.yaml`
- `standards/mcp-agent-capability-map.yaml`
- Task IDs: `A1` ... `I8`
- Artifact root: `RESEARCH/[topic]/`
- Collaboration guide: `guides/advanced/agent-skill-collaboration.md`
- Multi-client install guide: `guides/basic/install-multi-client.md`
- Full reference: `README.md`

Local validator:
- `python3 scripts/validate_research_standard.py`
- Use `--strict` to fail on warnings.
- `python3 -m unittest tests.test_orchestrator_workflows -v`
- Multi-client installer:
  - `./scripts/install_research_skill.sh --target all --project-dir /path/to/project --doctor`
- CI workflow: `.github/workflows/ci.yml`
- Release automation scripts:
  - `./scripts/release_automation.sh pre --tag v0.1.0-beta.2`
  - `./scripts/release_automation.sh post --tag v0.1.0-beta.2`
  - `./scripts/generate_release_notes.sh --tag v0.1.0-beta.3` (manual draft generation, optional)
  - `pre --tag` auto-generates and auto-fills validation evidence in `release/<tag>.md`
- Beta release docs: `release/v0.1.0-beta.2.md`, `release/v0.1.0-beta.1.md`, `release/rollback.md`

## Quick Commands

```bash
/paper [topic] [venue]                # Choose-your-path paper workflow (menu)
/lit-review [topic] [year range]     # Systematic literature review (PRISMA)
/paper-read [URL or DOI]             # Deep paper analysis
/find-gap [research area]            # Identify research gaps
/build-framework [theory/concept]    # Build theoretical framework
/academic-write [section] [topic]    # Academic writing assistance
/synthesize [topic] [outcome_id]     # Evidence synthesis / meta-analysis
/paper-write [topic] [type] [venue]  # Full manuscript drafting (outline → draft)
/study-design [topic]                # Empirical study design (protocol + analysis plan)
/ethics-check [topic]                # Ethics / IRB pack (human data)
/submission-prep [topic] [venue]     # Submission package (checklists + cover letter)
/rebuttal [topic]                    # Rebuttal / response to reviewers
/code-build [method] --domain ...    # Build academic research code
```

For consistency, ask users for `paper_type + task_id` when using `/paper`.

## Architecture

### Skills System

| Skill | Purpose | When to Use |
|-------|---------|-------------|
| `question-refiner` | Convert topic to structured RQ using PICO/PEO | Starting any research |
| `academic-searcher` | Search Semantic Scholar, arXiv APIs | Literature search phase |
| `paper-screener` | Apply inclusion/exclusion criteria | After initial search |
| `paper-extractor` | Extract structured data from papers | Deep reading phase |
| `study-designer` | Turn RQ into study design artifacts | Study design phase |
| `ethics-irb-helper` | Ethics/IRB documentation bundle | Before data collection |
| `evidence-synthesizer` | Narrative/qualitative/meta-analysis synthesis | Synthesis phase |
| `manuscript-architect` | Outline → full draft + integrity passes | Paper writing phase |
| `gap-analyzer` | Identify 5 types of research gaps | Gap analysis phase |
| `theory-mapper` | Map concept relationships | Framework building |
| `citation-formatter` | Format citations (APA/MLA/BibTeX) | Writing phase |
| `quality-assessor` | Apply A-E evidence rating | Quality assessment |
| `reporting-checker` | Reporting guideline completeness check | Pre-submission QA |
| `submission-packager` | Cover letter + submission checklist | Submission prep |
| `rebuttal-assistant` | Response matrix + response letter | Revision cycles |

### Cross-Model Package

- Portable Codex skill: `research-paper-workflow/SKILL.md`
- Codex UI metadata: `research-paper-workflow/agents/openai.yaml`
- Cross-model routing reference: `research-paper-workflow/references/platform-routing.md`

### PRISMA Workflow

```
Phase 1: Scoping
├── Define research question (PICO/PEO)
├── Set inclusion/exclusion criteria
└── Draft search strategy

Phase 2: Searching
├── Execute multi-database search
├── Semantic Scholar API
├── arXiv API
└── Google Scholar (web search)

Phase 3: Screening
├── Title/Abstract screening
├── Full-text screening
└── Generate PRISMA flowchart

Phase 4: Extraction
├── Extract structured data
├── Quality assessment (A-E rating)
└── Create extraction table

Phase 5: Synthesis
├── Thematic analysis
├── Narrative synthesis
├── Meta-analysis (when feasible)
└── Generate review report
```

### Empirical Study Workflow (Design → Submission)

```
Phase 1: Design
├── Define RQs/hypotheses + constraints
├── Study design + instruments
└── Analysis plan + DMP (+ prereg optional)

Phase 2: Ethics
└── IRB/ethics pack (consent, recruitment, data security)

Phase 3: Execution
├── Data collection + quality checks
└── Analysis code + robustness checks

Phase 4: Publication
├── Reporting checklist QA
├── Submission package + cover letter
└── Rebuttal / revision response
```

### Research Question Frameworks

**PICO** (Intervention studies):
- **P**opulation - Who is being studied?
- **I**ntervention - What intervention/exposure?
- **C**omparison - What is the comparison?
- **O**utcome - What outcomes are measured?

**PEO** (Non-intervention studies):
- **P**opulation - Who is being studied?
- **E**xposure - What exposure/phenomenon?
- **O**utcome - What outcomes are of interest?

**FINER** (Quality criteria):
- **F**easible - Can it be done?
- **I**nteresting - Is it worth doing?
- **N**ovel - Does it add new knowledge?
- **E**thical - Is it ethical?
- **R**elevant - Does it matter?

## Evidence Quality Rating (A-E)

| Grade | Evidence Type | Examples |
|-------|--------------|----------|
| **A** | Systematic review, Meta-analysis, RCT | Cochrane reviews, Nature RCTs |
| **B** | Cohort studies, High-IF journal papers | Longitudinal studies, Science/Cell papers |
| **C** | Case studies, Expert opinion, Conference | CHI papers, Expert commentaries |
| **D** | Preprints, Working papers | arXiv preprints, SSRN papers |
| **E** | Anecdotal, Theoretical speculation | Blog posts, Opinions |

## Output Structure

```
RESEARCH/[topic]/
├── protocol.md              # Research protocol
├── search_strategy.md       # Database-specific queries
├── search_log.md            # Reproducible search records
├── screening/
│   ├── title_abstract.md    # Initial screening log
│   ├── full_text.md         # Full-text screening log
│   └── prisma_flow.md       # PRISMA 2020 flowchart
├── notes/
│   ├── [citekey].md         # Individual paper notes
│   └── ...
├── extraction_table.md      # Data extraction table
├── quality_table.md         # Quality assessment
├── synthesis_matrix.md      # Theme × Paper matrix
├── synthesis.md             # Final synthesis report
└── bibliography.bib         # BibTeX references

Optional (empirical / publication):
├── study_design.md
├── analysis_plan.md
├── data_management_plan.md
├── ethics_irb.md
├── manuscript/
├── instruments/
├── submission/
└── revision/
```

**Path Convention:** All workflows use `RESEARCH/[topic]/...` structure. The `[topic]` should be normalized: lowercase, hyphens for spaces (e.g., "AI Ethics" → `ai-ethics`).

## Skill Loading Strategy

To optimize token usage, skills are organized in two tiers:

### Default Mode (Token-Efficient)

Use `skills-core.md` for the consolidated skill reference. This ~8KB file contains:
- Core purpose and process for each skill
- Essential output formats
- API quick reference

**When to use:** Most workflow executions, standard research tasks

### Detailed Mode (Full Reference)

Load full skill files from `skills/*/*.md` only when:
- First encounter with complex edge cases
- Need detailed output format templates
- Error recovery requiring fallback strategies
- User explicitly requests verbose output

### Invocation Pattern

When a workflow says "Use the **skill-name** skill":

1. **First:** Check `skills-core.md` for the skill's core process
2. **If sufficient:** Execute using core reference
3. **If need detail:** Load `skills/*/[skill-name].md` for full templates

**Example:**
```
Workflow: "Use the **paper-extractor** skill"
Action: 
  1. Check skills-core.md → paper-extractor section
  2. If need detailed extraction template → load skills/B_literature/paper-extractor.md
```

## API Integrations

### Semantic Scholar
- Endpoint: `https://api.semanticscholar.org/graph/v1`
- Features: Paper search, citations, references, author info
- Rate limit: 100 requests/5 minutes (public)

### arXiv
- Endpoint: `http://export.arxiv.org/api/query`
- Features: Full-text search, category filtering
- Rate limit: Reasonable use policy

## Multi-Model Collaboration

The `bridges/` directory provides multi-model collaboration via Codex and Gemini CLIs.

### Prerequisites

```bash
# Install CLIs
npm install -g @openai/codex
npm install -g @google/gemini-cli

# Set API keys
export OPENAI_API_KEY="..."
export GOOGLE_API_KEY="..."
```

### Usage

```bash
# Preflight checks
python -m bridges.orchestrator doctor --cwd "/path"

# Parallel analysis
python -m bridges.orchestrator parallel --prompt "..." --cwd "/path" --summarizer claude

# Parallel with run-level profile customization
python -m bridges.orchestrator parallel --prompt "..." --cwd "/path" \
  --profile-file ./standards/agent-profiles.example.json \
  --profile strict-review \
  --summarizer-profile strict-review

# Chain verification (generator -> independent verifier)
python -m bridges.orchestrator chain --prompt "..." --cwd "/path" --generator claude

# Role-based division
python -m bridges.orchestrator role --cwd "/path" \
  --codex-task "Implement algorithm" \
  --claude-task "Draft methods and critique assumptions" \
  --gemini-task "Generate documentation"

# Task-run (Task-ID standard orchestration)
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd "/path" \
  --mcp-strict \
  --skills-strict \
  --triad

# Task-run with stage-level profile overrides (draft/review/triad)
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd "/path" \
  --profile-file ./standards/agent-profiles.example.json \
  --draft-profile rapid-draft \
  --review-profile strict-review \
  --triad-profile strict-review

# Code Build (Standard / Tier 1)
python -m bridges.orchestrator code-build \
  --method "DID" --domain econ --tier standard --cwd "/path"

# Code Build (Advanced / Tier 2)
python -m bridges.orchestrator code-build \
  --method "Custom GMM" --tier advanced --cwd "/path"
```

### Collaboration Modes

| Mode | Description |
|------|-------------|
| `parallel` | Triad concurrent analysis + synthesis (auto fallback to dual/single) |
| `chain` | One generates, other verifies (iterative refinement) |
| `role` | Task division across Codex/Claude/Gemini |
| `single` | Single model execution |
| `task-run` | Task-ID orchestration using capability map |
| `code-build` | Specialized academic code generation (Standard/Advanced) |

External MCP connector contract:
- Configure command bridges via env vars such as `RESEARCH_MCP_SCHOLARLY_SEARCH_CMD`.
- `task-run --mcp-strict` blocks execution when required MCP providers are unavailable.
- `task-run` injects `required_skill_cards` from `skill_catalog` into draft/review prompts.
- `task-run --skills-strict` blocks execution when required skill spec files are unavailable.
- `task-run --triad` runs the third runtime agent for independent audit.
- Runtime agents now include `codex`, `claude`, and `gemini`.
- `parallel --summarizer` picks the post-analysis synthesis model.
- `doctor` runs preflight checks for runtime CLIs, key env vars, standards files, and external MCP command bindings.
- Runtime execution defaults to non-interactive mode with hard timeout controls to prevent hanging sessions.
- `--profile-file` + profile flags allow per-run persona/style/permission control (without global setting changes).
- Profile template file: `standards/agent-profiles.example.json`.

### When to Use

- 论文包含算法需要实现
- 代码复现任务
- 数据处理管道设计 (Polars/R)
- 复杂计量模型构建 (Tier 2 Custom Estimators)

## Development Notes

### When Adding New Skills
1. Create skill file in the appropriate stage sub-directory inside `skills/` (e.g., `skills/E_synthesis/`)
2. Add to `skill_registry` in `standards/mcp-agent-capability-map.yaml` format
3. Document in this CLAUDE.md

### When Modifying Workflows
1. Update workflow in `.agent/workflows/`
2. Ensure PRISMA compliance for lit-review
3. Test with sample research topic
