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
- 🧑‍⚖️ **Multi-Persona Peer Review** - Parallel, independent cross-reviews (Methodologist, Domain Expert, "Reviewer 2")
- 🚀 **CCG Code Execution** - Strict Spec -> Plan -> Execute -> Review isolation for code reliability
- 🛡️ **Iterative Critique Loop (Red Teaming)** - AI self-review and Socratic questioning to continuously narrow down and refine outputs
- 🤖 **Multi-Model Collaboration** - Codex + Claude + Gemini coordination across research stages
- 🧱 **Cross-Model Standard Contract** - Shared Task IDs + artifact paths for Codex/Claude/Gemini
- ⚡ **Token Optimized** - Layered skills architecture (~90% reduction)

## Standardization Layer

Use this project with a single canonical workflow contract:
- `standards/research-workflow-contract.yaml` (source of truth)
- `standards/mcp-agent-capability-map.yaml` (Task-ID-level MCP + agent orchestration)
- Task IDs: `A1` ... `I8`
- Artifact root: `RESEARCH/[topic]/`

Portable Codex skill package:
- `research-paper-workflow/SKILL.md`

Local consistency validator:

```bash
python3 scripts/validate_research_standard.py
python3 -m unittest tests.test_orchestrator_workflows -v

# Project artifact validator (run inside your project)
python3 scripts/validate_project_artifacts.py --cwd ./project --topic ai-in-education --task-id H1 --strict
```

Multi-client installer:

```bash
./scripts/install_research_skill.sh --target all --project-dir /path/to/project --doctor
```

Upgrade / auto-upgrade:
- Guide: `guides/basic/upgrade-research-skills.md`
- CLI aliases (after pipx install): `rs` / `rsw` (same as `research-skills`)
- Optional default upstream (omit `--repo`): set `RESEARCH_SKILLS_REPO=<owner>/<repo>`, or add `research-skills.toml` in your project root
- Check updates: `rs check --repo <owner>/<repo>` (or `rs check` if `RESEARCH_SKILLS_REPO` is set; or `python3 scripts/research_skill_update.py check ...`)
- Upgrade (no fork / no git clone required): `rs upgrade --repo <owner>/<repo> --project-dir /path/to/project --target all` (or omit `--repo` if `RESEARCH_SKILLS_REPO` is set; or `python3 scripts/research_skill_update.py upgrade ...`)

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
- `guides/advanced/agent-skill-collaboration.md`
- `guides/basic/install-multi-client.md`
- `guides/advanced/cli-reference.md` (CLI command reference)
- `guides/advanced/extend-research-skills.md` (how to extend/modify parts safely)
- `guides/advanced/mcp-zotero-integration.md` (Connecting local citation managers)

## 0 → 1 Navigation (New Users)

If you're new to this repo, this is the fastest way to understand and run it:

1. **Learn the contract (source of truth)**:
   - `standards/research-workflow-contract.yaml` (Task IDs, required outputs, quality gates, dependencies)
2. **Learn the routing (who does what)**:
   - `standards/mcp-agent-capability-map.yaml` (required skills/MCP + primary/review/fallback agents per Task ID)
3. **Install into your clients/project**:
   - Script: `./scripts/install_research_skill.sh --target all --project-dir <project> --doctor`
   - Or pipx + upgrade: `pipx install research-skills-installer` then `rs upgrade --project-dir <project> --target all --doctor`
4. **Run a workflow**:
   - In Claude Code: use `/paper` or any `.agent/workflows/*.md` command in your project
   - CLI: `python3 -m bridges.orchestrator task-run --task-id F3 --paper-type empirical --topic <topic> --cwd <project> --triad`
5. **Validate outputs**:
   - `python3 scripts/validate_project_artifacts.py --cwd <project> --topic <topic> --task-id <task> --strict`

Where to customize:
- Personas/runtime options: `standards/agent-profiles.example.json` (used by `parallel` / `task-run`)
- Stage playbooks (DoD/checklists): `research-paper-workflow/references/stage-*.md`
- Project upstream defaults: `research-skills.toml` (or `RESEARCH_SKILLS_REPO`)

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
| `/code-build` | CCG-driven Research code execution | `/code-build \"Staggered DID\" --domain econ` |

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
| `task-plan` | Render dependency-based task plan (contract dependency_catalog) |
| `task-run` | Task-ID orchestration using `mcp-agent-capability-map.yaml` |
| `doctor` | Environment preflight checks before orchestration |

Runtime note:
- `doctor` checks CLI availability, key env vars, standards files, and external MCP command bindings.
- `parallel` runs `codex + claude + gemini` concurrently, then uses `--summarizer` for final synthesis.
- If triad is unavailable in `parallel`, it degrades automatically to dual or single-agent analysis.
- `parallel --profile-file/--profile/--summarizer-profile` lets users customize persona/style/permission profile per run.
- Runtime now defaults to non-interactive execution (`CI=1`, `TERM=dumb`) with hard timeouts to avoid hanging sessions.
- `task-plan` renders prerequisites from `dependency_catalog` and checks which outputs exist under `RESEARCH/[topic]/`.
- `task-run` now supports runtime execution for `codex`, `claude`, and `gemini` directly.
- If a mapped runtime is unavailable, routing automatically falls back to available agents based on the capability map.
- `task-run` auto-injects `required_skills` from `task_skill_mapping` into draft/review prompts.
- `task-run` auto-injects `required_skill_cards` from `skill_catalog` (focus, category, default outputs, skill spec path).
- `task-run` auto-injects `task_plan` (dependency + completion status) into task packets and prompts.
- `task-run --profile-file` + `--draft-profile/--review-profile/--triad-profile` customizes each stage without touching global defaults.
- `task-run --skills-strict` blocks execution when required skill spec files are missing.
- `task-run --triad` adds a third independent audit so non-code stages can also run full 3-agent collaboration.
- External MCP providers can be wired by env var commands, e.g. `RESEARCH_MCP_SCHOLARLY_SEARCH_CMD`.

Profile file template:
- `standards/agent-profiles.example.json`
  - Defines personas, agent runtimes, and an optional `output_language` (e.g., `"zh-CN"`). Using `output_language` enforces localized output while keeping core instructions in English, ensuring maximum AI reasoning stability.

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
├── standards/                # Canonical workflow contract + capability map (Task IDs, outputs, routing)
├── research-paper-workflow/  # Portable skill package installed to Codex/Claude/Gemini
├── .agent/workflows/         # Claude Code slash-commands (project workflows)
├── bridges/                  # Multi-model orchestration (Codex/Claude/Gemini bridges + orchestrator)
├── skills/                   # Skill specs referenced by capability map (skill cards)
├── skills-core.md            # Token-optimized consolidated reference for skills
├── templates/                # Output templates (PRISMA, rebuttal, DMP, etc.)
├── guides/
│   ├── basic/                # Basic usage, installation, upgrades
│   └── advanced/             # Advanced features, CLI reference, MCP integration
├── scripts/                  # Install/upgrade/release automation + validators
├── research_skills/          # pipx CLI package (entrypoints: research-skills / rs / rsw)
│   └── project.toml          # Packaged default upstream (CI-injected; overrideable)
├── release/                  # Release notes + acceptance receipts + templates
├── tests/                    # Orchestrator workflow unit tests (mock bridges)
├── .github/workflows/        # GitHub Actions CI + release automation
├── RESEARCH/                 # Example / generated research artifacts (contract output root)
├── BETA_TODO.md              # Beta readiness checklist
├── TODO_ROADMAP.md           # Longer-term roadmap
├── CLAUDE.md                 # Claude Code quick reference (installed into projects)
├── pyproject.toml            # Packaging (console scripts + metadata)
├── README.md                 # English docs
└── README_CN.md              # 中文文档
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
