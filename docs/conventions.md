# Research Skills Framework — Conventions

## Terminology

### Portable Skills vs Internal Skill Specs

- `research-paper-workflow/` is a portable skill package distributed to Codex / Claude / Gemini clients.
- `skills/` contains repo-internal skill specifications referenced by `standards/`, `pipelines/`, and validators.
- Do not assume every markdown file under `skills/` is installable as a standalone client skill package.
- When a change affects end-user client entry behavior, check `research-paper-workflow/` first.
- When a change affects reusable execution behavior inside this repo, check `skills/` first.

### Functional Agents vs Runtime Agents

- Functional agents are the research responsibility layer: literature, methods, writing, compliance, and similar ownership boundaries.
- Today, that layer is represented primarily by `roles/` and pipeline ownership patterns.
- Runtime agents are the actual model executors: `codex`, `claude`, and `gemini`.
- Keep these concepts separate in docs, schemas, and routing logic.

## System Layers

| Layer | Directory / File | Purpose |
|---|---|---|
| **Contract** | `standards/research-workflow-contract.yaml` | Canonical task IDs, artifact paths, quality gates |
| **Capability Map** | `standards/mcp-agent-capability-map.yaml` | Skill/MCP/runtime routing |
| **Functional Agents** | `roles/` | Research responsibility and quality ownership |
| **Internal Skill Specs** | `skills/` | Reusable execution specs used by tasks and pipelines |
| **Pipelines** | `pipelines/` | Abstract DAGs defining dependencies and handoffs |
| **Workflows** | `.agent/workflows/` | Client-facing command entrypoints |
| **Bridges** | `bridges/` | Runtime adapters and orchestration |
| **Portable Skill Package** | `research-paper-workflow/` | Cross-client installable entry skill |

## Dependency Direction

Treat the architecture as a one-way dependency graph:

1. `standards/research-workflow-contract.yaml`
2. `standards/mcp-agent-capability-map.yaml`
3. `roles/` and `skills/`
4. `pipelines/` and `.agent/workflows/`
5. `bridges/`
6. `research-paper-workflow/` as the distribution surface

Practical implications:

- Do not redefine artifact paths or quality gates outside `standards/research-workflow-contract.yaml`.
- Do not create a second routing truth inside `pipelines/`, `bridges/`, or client workflow files.
- Do not let a portable client skill package become the hidden source of internal execution truth.
- If two layers disagree, fix the upstream layer first rather than patching downstream symptoms.

## Top-Level Skill Admission Rules

Create a new internal top-level skill only when all four conditions hold:

1. It consumes typed inputs and produces typed outputs.
2. It owns at least one stable artifact path under `RESEARCH/[topic]/`.
3. It is worth direct pipeline or task-level dependency wiring.
4. It carries distinct failure modes, review expectations, or quality-gate value.

Default demotion rules:

- Keep **provider adapters** in `mcp_registry` / bridge code, not in `skills/`.
- Keep **database-specific or API-specific calls** as MCP/provider abilities, not top-level skills.
- Keep **micro-steps inside an existing artifact-producing skill** as embedded subflows.
- Keep **field-level extraction variants** as structured slots inside an existing extraction template.

Common examples that should **not** become top-level skills:

- `query-builder`
- `keyword-expander`
- `semantic-scholar-search`
- `crossref-search`
- `database-connector`
- `methodology-extractor`
- `dataset-extractor`
- `theory-extractor`
- `limitation-extractor`

Examples that can remain top-level because they own contract artifacts:

- `academic-searcher`
- `paper-extractor`
- `study-designer`
- `manuscript-architect`
- `citation-formatter`
- `table-generator`
- `figure-specifier`

## When to Extend an Existing Skill Instead

Prefer expanding an existing skill spec when the change is one of these:

- A new subsection, checklist, or review rule inside an existing deliverable.
- A new extraction slot inside `paper-extractor`.
- A new search heuristic, provider order, or query tactic inside `academic-searcher`.
- A new manuscript subsection pattern inside `manuscript-architect`.
- A new robustness or diagnostic variant that still lands in the same parent artifact.

Good signals that the change belongs inside an existing skill:

1. The artifact path does not change.
2. The parent skill already owns the review or failure mode.
3. Pipelines do not need to branch directly on the sub-capability.
4. The change mostly affects output shape, prompts, or checklist content.

Default files to touch in that case:

- `skills/*/*.md`
- `templates/*.md`
- `research-paper-workflow/references/stage-*.md`

## When to Sink a Capability to MCP, Script, or Template

Use this routing table before creating a new skill:

| If the change is mainly... | Put it here | Why |
|---|---|---|
| External API / provider / database access | `mcp_registry` + `bridges/` | Provider wiring should stay below the skill layer |
| Reusable text/table structure | `templates/` | Structure reuse is not the same as a new orchestration unit |
| Validation / transformation / local automation | `scripts/` | Script logic should not inflate the skill taxonomy |
| Role ownership / thresholds / tone | `roles/` | This is functional-agent behavior, not skill behavior |
| Task routing / runtime selection | `standards/mcp-agent-capability-map.yaml` | Routing belongs in the capability map |

Concrete examples:

- Add a Crossref fallback: update MCP/provider logic, not `skills/`.
- Add a new extraction column: update `templates/extraction-table.md` and `paper-extractor.md`.
- Add a release or validation helper: add or update a script under `scripts/`.
- Add a different owner for `C4`: update `task_functional_routing`, not the skill file.

## Skill Naming

- `kebab-case` for all skill file names (e.g., `gap-analyzer.md`)
- Skill ID matches filename without `.md` extension
- Stage prefix in directory path (`A_framing/`, `B_literature/`, etc.)

## YAML Frontmatter Contract

Every skill `.md` file MUST begin with YAML frontmatter containing:

```yaml
---
id: skill-name
stage: A_framing          # Stage directory name
version: "1.0.0"          # Semver
description: "One-line description"
inputs:
  - type: ArtifactType    # From schemas/artifact-types.yaml
    description: "..."
    required: false        # Optional, defaults to true
outputs:
  - type: ArtifactType
    artifact: "path/file.md"
constraints:
  - "Must ..."
failure_modes:
  - "When ..."
tools: [filesystem, scholarly-search]
tags: [stage, topic, method]
domain_aware: false
---
```

## Artifact Types

All typed artifact names are defined in `schemas/artifact-types.yaml`. Use these types consistently in frontmatter `inputs` and `outputs`, registry entries, and pipeline step definitions.

## Pipeline DAGs vs Workflow Slash-Commands

| Layer | Directory | Purpose |
|---|---|---|
| **Pipelines** | `pipelines/` | Abstract DAGs defining step sequence + dependencies |
| **Workflows** | `.agent/workflows/` | Claude Code slash-command execution layer |

Pipelines reference skill IDs; workflows call skills directly.

## Edit Order

When a change spans multiple layers, apply it in this order:

1. `standards/` for contract or routing truth
2. `roles/` and `skills/` for responsibility or execution details
3. `templates/` for stable structured outputs
4. `pipelines/` and `.agent/workflows/` for sequencing or entry behavior
5. `bridges/` only if execution logic must change
6. `research-paper-workflow/` only if the portable client package must reflect the change

## Roles

Role configs in `roles/` define the current functional-agent layer: preferred skills, quality thresholds, and tone for different research responsibilities. Pass `--role pi` to adjust behavior.

## Domain Profiles

Domain profiles in `skills/domain-profiles/` customize skill behavior for specific disciplines. Each profile includes:
- Library recommendations per language
- Method templates with checklists
- Statistical diagnostics
- Reporting guidelines
- Default databases and query syntax
- Methodology priors
- Venue norms

## Adding a New Internal Skill Spec

1. Create `skills/{STAGE}/{skill-name}.md` with full YAML frontmatter
2. Add entry to `skills/registry.yaml`
3. Add new artifact types to `schemas/artifact-types.yaml` if needed
4. Add to relevant pipelines in `pipelines/`
5. Run `python3 scripts/validate_research_standard.py` to verify

If you need a portable client-facing skill package instead of an internal execution spec, follow the `research-paper-workflow/` style and keep it separate from `skills/`.

## Adding a New Domain Profile

1. Copy `skills/domain-profiles/custom-template.yaml`
2. Fill in all sections following the schema in `schemas/domain-profile.schema.json`
3. Run validator to confirm
