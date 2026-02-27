# Academic Deep Research Skills

A systematic research skills system designed for Claude Code, providing tools for literature review, paper analysis, gap identification, and academic writing.

## Features

- 📚 **Systematic Literature Review** - PRISMA 2020 compliant methodology
- 📖 **Deep Paper Reading** - Structured notes + BibTeX
- 🧪 **Evidence Synthesis & Meta-analysis** - Narrative / qualitative / quantitative pooling (PRISMA-aligned)
- 📝 **Full Manuscript Drafting** - Outline → draft → claim-evidence integrity → figures/tables
- 🧩 **Study Design → Publication** - Study design, ethics/IRB pack, submission prep, rebuttal workflow
- 🔍 **Research Gap Identification** - 5 types of academic gap analysis
- 🧠 **Theoretical Framework Building** - Concept relationship mapping
- ✍️ **Academic Writing Assistance** - Standard-compliant formatting
- 🤖 **Multi-Model Collaboration** - Codex + Claude + Gemini coordination across research stages
- 🧱 **Cross-Model Standard Contract** - Shared Task IDs + artifact paths for Codex/Claude/Gemini
- ⚡ **Token Optimized** - Layered skills architecture (~90% reduction)

## Standardization Layer

Use this project with a single canonical workflow contract:
- `standards/research-workflow-contract.yaml` (source of truth)
- `standards/mcp-agent-capability-map.yaml` (Task-ID-level MCP + agent orchestration)
- Task IDs: `A1` ... `I3`
- Artifact root: `RESEARCH/[topic]/`

Portable Codex skill package:
- `research-paper-workflow/SKILL.md`

Local consistency validator:

```bash
python3 scripts/validate_research_standard.py
python3 -m unittest tests.test_orchestrator_workflows -v
```

Multi-client installer:

```bash
./scripts/install_research_skill.sh --target all --project-dir /path/to/project --doctor
```

CI pipeline:
- `.github/workflows/ci.yml` (runs `py_compile`, strict validator, and unit tests on PR/push)

Beta release docs:
- `release/v0.1.0-beta.2.md`
- `release/v0.1.0-beta.1.md`
- `release/rollback.md`
- `release/automation.md`
- `release/templates/beta-acceptance-template.md`

Use `--strict` to treat warnings as failures.

Release automation:

```bash
./scripts/release_automation.sh pre --tag v0.1.0-beta.2
./scripts/release_automation.sh post --tag v0.1.0-beta.2
```

`pre --tag` auto-generates `release/<tag>.md` draft when missing.
`pre --tag` also auto-fills validator/unittest/smoke evidence lines after checks pass.
Manual draft generation: `./scripts/generate_release_notes.sh --tag v0.1.0-beta.3 --from-tag v0.1.0-beta.2`.

Collaboration rule:
- Skill = workflow router (`task_id`, output paths, quality gates)
- MCP = evidence/tools layer
- Agents = drafting/review layer (primary/reviewer/fallback from capability map)

Collaboration playbook:
- `guides/agent-skill-collaboration.md`
- `guides/install-multi-client.md`

## Skills + Agents Flow (ASCII)

```text
User Goal / Prompt
        |
        v
Skill Router (Task ID + paper_type)
  - standards/research-workflow-contract.yaml
  - standards/mcp-agent-capability-map.yaml
        |
        +--------------------------+
        |                          |
        v                          v
MCP Evidence Collection      Agent Runtime Routing
(search/extraction/stats)    (codex / claude / gemini)
        |                          |
        +------------+-------------+
                     v
              Draft Generation
                     |
                     v
              Review / Critique
                     |
         +-----------+-----------+
         |                       |
         v                       v
   Triad Audit (optional)   Dual/Single Fallback
                     \       /
                      v     v
            Synthesis (summarizer)
                     |
                     v
     Quality Gates + Artifact Output Write
         -> RESEARCH/[topic]/...
```

## Quick Start

### Installation

Clone this repository into your project. Claude Code will automatically recognize commands in `.agent/workflows/`.

```bash
git clone <repository-url> research-skills
```

Install to Codex + Claude Code + Gemini:

```bash
cd research-skills
./scripts/install_research_skill.sh --target all --project-dir /path/to/project --doctor
```

Installer notes:
- `--target codex|claude|gemini|all` selects install target.
- `--mode copy|link` controls whether files are copied or symlinked.
- `--overwrite` replaces existing installs.
- `--dry-run` previews the installation plan.

### Commands

| Command | Purpose | Example |
|---------|---------|---------|
| `/paper` | Choose-your-path paper workflow | `/paper ai-in-education CHI` |
| `/lit-review` | Systematic literature review | `/lit-review transformer architecture 2020-2024` |
| `/paper-read` | Deep paper analysis | `/paper-read https://arxiv.org/abs/2303.08774` |
| `/find-gap` | Identify research gaps | `/find-gap LLM in education` |
| `/build-framework` | Build theoretical framework | `/build-framework technology acceptance` |
| `/academic-write` | Academic writing assistance | `/academic-write introduction AI ethics` |
| `/paper-write` | Full paper drafting | `/paper-write ai-in-education empirical CHI` |
| `/synthesize` | Evidence synthesis / meta-analysis | `/synthesize ai-in-education` |
| `/study-design` | Empirical study design | `/study-design ai-in-education` |
| `/ethics-check` | Ethics / IRB pack | `/ethics-check ai-in-education` |
| `/submission-prep` | Submission package | `/submission-prep ai-in-education CHI` |
| `/rebuttal` | Rebuttal / revision response | `/rebuttal ai-in-education` |
| `/code-build` | Build research code | `/code-build \"Staggered DID\" --domain econ` |

Task ID recommendation:
- Ask users for both `paper_type` and `task_id` (for example `systematic-review + E3`).
- Keep task IDs and output paths aligned with `standards/research-workflow-contract.yaml`.

## Multi-Model Collaboration

Coordinate Codex, Claude, and Gemini for cross-stage research tasks.

### Prerequisites

```bash
npm install -g @openai/codex
npm install -g @anthropic-ai/claude-code
npm install -g @google/gemini-cli
export OPENAI_API_KEY="..."
export ANTHROPIC_API_KEY="..."
export GOOGLE_API_KEY="..."
```

### Usage

```bash
# Preflight check - verify local CLIs, API keys, and MCP command wiring
python -m bridges.orchestrator doctor --cwd ./project

# Parallel analysis - triad concurrent analysis + synthesis
python -m bridges.orchestrator parallel \
  --prompt "分析代码安全性" --cwd ./project --summarizer claude

# Optional: per-run profile (persona/style/tool permissions)
python -m bridges.orchestrator parallel \
  --prompt "审查该研究方案的证据风险" \
  --cwd ./project \
  --summarizer claude \
  --profile-file ./standards/agent-profiles.example.json \
  --profile strict-review \
  --summarizer-profile strict-review

# Chain verification - one generates, other verifies
python -m bridges.orchestrator chain \
  --prompt "实现论文中的算法" --cwd ./project --generator claude

# Role-based - task division by specialty (3-agent)
python -m bridges.orchestrator role --cwd ./project \
  --codex-task "实现数据管道" \
  --claude-task "起草方法与结果叙述" \
  --gemini-task "生成文档"

# Task-run - execute canonical Task ID with capability-map agent routing
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project \
  --context "Draft full manuscript with claim-evidence alignment"

# Optional: enforce required MCP availability
python -m bridges.orchestrator task-run \
  --task-id B1 \
  --paper-type systematic-review \
  --topic ai-in-education \
  --cwd ./project \
  --mcp-strict

# Optional: enforce skill spec availability
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project \
  --skills-strict

# Optional: force third-agent audit (Codex + Claude + Gemini)
python -m bridges.orchestrator task-run \
  --task-id G3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project \
  --triad

# Optional: stage-level profile overrides (not global)
python -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd ./project \
  --profile-file ./standards/agent-profiles.example.json \
  --profile default \
  --draft-profile rapid-draft \
  --review-profile strict-review \
  --triad-profile strict-review
```

| Mode | Description |
|------|-------------|
| `parallel` | Triad concurrent analysis + synthesis (auto fallback to dual/single) |
| `chain` | One generates, other verifies (iterative refinement) |
| `role` | Task division across Codex/Claude/Gemini |
| `single` | Single model execution |
| `task-run` | Task-ID orchestration using `mcp-agent-capability-map.yaml` |
| `doctor` | Environment preflight checks before orchestration |

Runtime note:
- `doctor` checks CLI availability, key env vars, standards files, and external MCP command bindings.
- `parallel` runs `codex + claude + gemini` concurrently, then uses `--summarizer` for final synthesis.
- If triad is unavailable in `parallel`, it degrades automatically to dual or single-agent analysis.
- `parallel --profile-file/--profile/--summarizer-profile` lets users customize persona/style/permission profile per run.
- Runtime now defaults to non-interactive execution (`CI=1`, `TERM=dumb`) with hard timeouts to avoid hanging sessions.
- `task-run` now supports runtime execution for `codex`, `claude`, and `gemini` directly.
- If a mapped runtime is unavailable, routing automatically falls back to available agents based on the capability map.
- `task-run` auto-injects `required_skills` from `task_skill_mapping` into draft/review prompts.
- `task-run` auto-injects `required_skill_cards` from `skill_catalog` (focus, category, default outputs, skill spec path).
- `task-run --profile-file` + `--draft-profile/--review-profile/--triad-profile` customizes each stage without touching global defaults.
- `task-run --skills-strict` blocks execution when required skill spec files are missing.
- `task-run --triad` adds a third independent audit so non-code stages can also run full 3-agent collaboration.
- External MCP providers can be wired by env var commands, e.g. `RESEARCH_MCP_SCHOLARLY_SEARCH_CMD`.

Profile file template:
- `standards/agent-profiles.example.json`

## Core Workflows

### 1. Systematic Literature Review `/lit-review`

Follows PRISMA 2020 methodology:

```
Research Question Definition (PICO/PEO)
       ↓
Multi-database Search (Semantic Scholar, arXiv, OpenAlex)
       ↓
Citation Snowballing + Grey Literature
       ↓
Title/Abstract Screening → Full-text Screening
       ↓
Data Extraction + Quality Assessment (RoB, GRADE)
       ↓
Synthesis Report + PRISMA Flow Diagram
```

### 2. Deep Paper Reading `/paper-read`

Extracts: RQs, Theoretical Framework, Methodology, Key Findings, Contributions & Limitations, Future Work.

Output: Markdown notes + BibTeX citation

### 3. Research Gap Identification `/find-gap`

Five types: Theoretical, Methodological, Empirical, Knowledge, Population gaps.

### 4. Theoretical Framework Building `/build-framework`

Theory review, concept mapping (Mermaid diagrams), hypothesis derivation.

## Evidence Quality Rating (A-E)

| Grade | Evidence Type |
|-------|--------------|
| **A** | Systematic reviews, Meta-analyses, Large RCTs |
| **B** | Cohort studies, High-IF journal papers |
| **C** | Case studies, Expert opinion, Conference papers |
| **D** | Preprints, Working papers |
| **E** | Anecdotal, Theoretical speculation |

## Project Structure

```
research-skills/
├── .agent/workflows/     # User commands
│   ├── paper.md
│   ├── lit-review.md
│   ├── paper-read.md
│   ├── find-gap.md
│   ├── build-framework.md
│   ├── academic-write.md
│   ├── paper-write.md
│   ├── code-build.md
│   ├── synthesize.md
│   ├── study-design.md
│   ├── ethics-check.md
│   ├── submission-prep.md
│   └── rebuttal.md
├── skills/               # Detailed skill modules
│   ├── question-refiner.md
│   ├── academic-searcher.md
│   ├── paper-extractor.md
│   ├── quality-assessor.md
│   ├── model-collaborator.md   # Multi-model collaboration
│   └── ... (22 skills)
├── skills-core.md        # Consolidated reference (~8KB vs 71KB)
├── bridges/              # Multi-model CLI bridges
│   ├── base_bridge.py
│   ├── codex_bridge.py
│   ├── gemini_bridge.py
│   └── orchestrator.py   # Single entry point
├── guides/               # Collaboration playbooks
├── scripts/              # Local validators and utility scripts
├── templates/            # Output templates
├── standards/            # Canonical workflow contract (Task IDs + outputs)
├── research-paper-workflow/  # Portable Codex skill package
├── RESEARCH/             # Research output directory
├── CLAUDE.md             # Claude Code quick reference
└── README.md
```

## Token Optimization

The system uses a layered architecture for token efficiency:

- **Default**: Use `skills-core.md` (~8KB consolidated reference)
- **Detail**: Load full `skills/*.md` only when needed

Result: ~90% token reduction for skill references.

## Supported APIs

| Source | Purpose | Coverage |
|--------|---------|----------|
| Semantic Scholar | Primary search | 200M+ papers |
| arXiv | CS/AI/Physics preprints | Full coverage |
| OpenAlex | Bibliometrics | 250M+ works |
| Crossref | Metadata verification | 140M+ DOIs |
| Unpaywall | OA full-text access | DOI-based |

## Reference Manager Integration

Supports: **Zotero** (BibTeX, CSL-JSON), **Mendeley** (BibTeX, RIS), **EndNote** (RIS)

## License

MIT
